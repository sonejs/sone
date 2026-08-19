# frozen_string_literal: true

require "json"

require_relative "errors"
require_relative "native"

module Sone
  # Owns the font registry and the decoded-image cache.
  #
  # Skia's font collection is shared inside an engine, so one engine renders one
  # document at a time and every call here takes the lock. Give each thread its
  # own engine for real parallelism rather than sharing one.
  class Engine
    STATUSES = {
      Native::INVALID_ARGUMENT => ArgumentError,
      Native::IR_ERROR => IrError,
      Native::ASSET_ERROR => AssetError,
      Native::RENDER_ERROR => RenderError
    }.freeze

    # @param base_dir [String] the directory relative asset paths resolve against
    def initialize(base_dir = Dir.pwd)
      @pointer = Native.sone_engine_new(base_dir)
      raise Error, "could not create a sone engine" if @pointer.null?

      @lock = Mutex.new
      @closed = false
      ObjectSpace.define_finalizer(self, self.class.reaper(@pointer))
    end

    # The process-wide engine, used when no explicit one is passed.
    def self.default
      @default ||= new
    end

    def self.version
      Native.sone_version.read_string
    end

    # Frees the handle without capturing the engine, which would keep it alive.
    def self.reaper(pointer)
      proc { Native.sone_engine_free(pointer) }
    end

    def close
      @lock.synchronize do
        next if @closed

        ObjectSpace.undefine_finalizer(self)
        Native.sone_engine_free(@pointer)
        @closed = true
      end
      nil
    end

    def closed?
      @closed
    end

    # ── fonts and assets ────────────────────────────────────────────────────

    # Register a font family from raw TTF/OTF bytes.
    def register_font(name, data)
      with_bytes(data) do |pointer, length|
        call { Native.sone_register_font(handle, name.to_s, pointer, length) }
      end
    end

    # Register a font family from a file.
    def register_font_file(name, path)
      call { Native.sone_register_font_file(handle, name.to_s, path.to_s) }
    end

    # Make bytes available to documents as `asset:<name>`.
    def register_image(name, data)
      with_bytes(data) do |pointer, length|
        call { Native.sone_register_image(handle, name.to_s, pointer, length) }
      end
    end

    def font?(name)
      @lock.synchronize { Native.sone_has_font(handle, name.to_s) }
    end

    def font_families
      JSON.parse(buffer { |out| Native.sone_font_families(handle, out) })
    end

    def reset_fonts
      @lock.synchronize { Native.sone_reset_fonts(handle) }
      nil
    end

    # ── rendering ───────────────────────────────────────────────────────────

    # Render an IR document to bytes.
    def render(document, format: :png, density: nil, quality: 1.0, strict: false)
      options = options_for(format, density, quality, strict)
      buffer { |out| Native.sone_render_json(handle, document, options.pointer, out) }
    end

    # One raster image per page. Requires `pageHeight` in the document config.
    def render_pages(document, format: :png, density: nil, quality: 1.0, strict: false)
      options = options_for(format, density, quality, strict)
      list = Native::BufferList.new
      @lock.synchronize do
        begin
          check(Native.sone_render_pages(handle, document, options.pointer, list))
          list.pages
        ensure
          Native.sone_buffer_list_free(list)
        end
      end
    end

    # The computed layout tree, as a JSON string.
    def dump_layout(document)
      buffer { |out| Native.sone_dump_layout(handle, document, out) }.force_encoding(Encoding::UTF_8)
    end

    # Dataset-style metadata, as a JSON string.
    def dump_metadata(document, granularity = :node)
      buffer { |out| Native.sone_dump_metadata(handle, document, granularity.to_s, out) }
        .force_encoding(Encoding::UTF_8)
    end

    def inspect
      "#<Sone::Engine#{closed? ? " closed" : ""}>"
    end

    private

    def handle
      raise Error, "this engine has been closed" if @closed

      @pointer
    end

    def options_for(format, density, quality, strict)
      code = Native::FORMATS[format.to_s.downcase.to_sym]
      raise ArgumentError, "unknown output format #{format.inspect}" unless code

      options = Native::RenderOptions.new
      options[:format] = code
      # Zero tells the engine to fall back to the document's own config.
      options[:density] = density ? density.to_f : 0.0
      options[:quality] = quality.to_f
      options[:strict] = strict ? 1 : 0
      options
    end

    # Runs a call that fills a buffer, and always releases it.
    def buffer
      out = Native::Buffer.new
      @lock.synchronize do
        begin
          check(yield(out))
          out.bytes
        ensure
          Native.sone_buffer_free(out)
        end
      end
    end

    def call
      @lock.synchronize { check(yield) }
      nil
    end

    def with_bytes(data)
      bytes = data.to_s.b
      FFI::MemoryPointer.new(:uint8, bytes.bytesize) do |pointer|
        pointer.put_bytes(0, bytes)
        return yield(pointer, bytes.bytesize)
      end
    end

    def check(status)
      return status if status == Native::OK

      pointer = Native.sone_engine_last_error(@pointer)
      message = pointer.null? ? "sone failed with status #{status}" : pointer.read_string
      raise (STATUSES[status] || Error), message.force_encoding(Encoding::UTF_8)
    end
  end

  # Font registration on the process-wide engine, for scripts that do not want
  # to own one. Skia carries no system fonts, so at least one family must be
  # registered before any text renders.
  module Font
    module_function

    def load(name, source)
      engine = Engine.default
      if source.respond_to?(:to_path) || (source.is_a?(String) && File.exist?(source))
        engine.register_font_file(name, source.to_s)
      else
        engine.register_font(name, source)
      end
    end

    def has?(name)
      Engine.default.font?(name)
    end

    def families
      Engine.default.font_families
    end

    def reset
      Engine.default.reset_fonts
    end
  end
end
