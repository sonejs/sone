# frozen_string_literal: true

require "ffi"

module Sone
  # The C ABI from `include/sone.h`. Nothing above this module sees a pointer.
  #
  # The `ffi` gem rather than a compiled extension: the C ABI is the whole
  # contract, so `gem install sone` needs no Rust toolchain and no build step.
  module Native
    extend FFI::Library

    OK = 0
    INVALID_ARGUMENT = 1
    IR_ERROR = 2
    ASSET_ERROR = 3
    RENDER_ERROR = 4

    FORMATS = {
      png: 0, jpeg: 1, jpg: 1, webp: 2, raw: 3, rgba: 3, pdf: 4, svg: 5
    }.freeze

    # A full path to the library, or a directory holding it.
    PATH_VARIABLE = "SONE_NATIVE_LIBRARY"

    LIBRARY_NAME =
      case RbConfig::CONFIG["host_os"]
      when /mswin|mingw|cygwin/ then "sone.dll"
      when /darwin/ then "libsone.dylib"
      else "libsone.so"
      end

    # Where to look, in order: an explicit hint, a `cargo build` in a checkout,
    # then the loader's own search path — which is what a released gem uses.
    def self.candidates
      found = []
      hint = ENV[PATH_VARIABLE]
      if hint && !hint.empty?
        found << (File.directory?(hint) ? File.join(hint, LIBRARY_NAME) : hint)
      end
      root = checkout_root
      if root
        %w[release debug].each do |profile|
          found << File.join(root, "target", profile, LIBRARY_NAME)
        end
      end
      found.select { |path| File.exist?(path) } + [LIBRARY_NAME, "sone"]
    end

    # The repository root, when this gem is being used from a checkout.
    def self.checkout_root
      directory = __dir__
      loop do
        if File.exist?(File.join(directory, "Cargo.toml")) &&
           File.directory?(File.join(directory, "crates"))
          return directory
        end
        parent = File.dirname(directory)
        return nil if parent == directory

        directory = parent
      end
    end

    begin
      ffi_lib candidates
    rescue LoadError => e
      raise LoadError, "could not load the sone native library (#{LIBRARY_NAME}). " \
                       "Build it with `cargo build --release -p sone-ffi`, or set " \
                       "#{PATH_VARIABLE} to its path. (#{e.message})"
    end

    # An owned byte buffer. Release with `sone_buffer_free`.
    class Buffer < FFI::Struct
      layout :data, :pointer, :len, :size_t, :capacity, :size_t

      def bytes
        return "".b if self[:data].null? || self[:len].zero?

        self[:data].read_bytes(self[:len])
      end
    end

    # One buffer per page. Release the whole list with `sone_buffer_list_free`.
    class BufferList < FFI::Struct
      layout :items, :pointer, :len, :size_t, :capacity, :size_t

      def pages
        Array.new(self[:len]) do |index|
          Buffer.new(self[:items] + (index * Buffer.size)).bytes
        end
      end
    end

    class RenderOptions < FFI::Struct
      layout :format, :int, :density, :float, :quality, :float, :strict, :int
    end

    attach_function :sone_engine_new, [:string], :pointer
    attach_function :sone_engine_free, [:pointer], :void
    attach_function :sone_engine_last_error, [:pointer], :pointer
    attach_function :sone_register_font, %i[pointer string pointer size_t], :int
    attach_function :sone_register_font_file, %i[pointer string string], :int
    attach_function :sone_register_image, %i[pointer string pointer size_t], :int
    attach_function :sone_has_font, %i[pointer string], :bool
    attach_function :sone_font_families, %i[pointer pointer], :int
    attach_function :sone_reset_fonts, [:pointer], :void
    attach_function :sone_render_json, [:pointer, :string, RenderOptions.by_value, :pointer], :int
    attach_function :sone_render_pages, [:pointer, :string, RenderOptions.by_value, :pointer], :int
    attach_function :sone_dump_layout, %i[pointer string pointer], :int
    attach_function :sone_dump_metadata, %i[pointer string string pointer], :int
    attach_function :sone_buffer_free, [:pointer], :void
    attach_function :sone_buffer_list_free, [:pointer], :void
    attach_function :sone_version, [], :pointer
  end
end
