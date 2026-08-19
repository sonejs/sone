# frozen_string_literal: true

require "json"

require_relative "engine"
require_relative "ir"

module Sone
  # A node plus its render configuration, with one method per output format.
  class Rendering
    CONFIG_KEYS = {
      width: "width",
      height: "height",
      background: "background",
      density: "density",
      page_height: "pageHeight",
      margin: "margin",
      last_page_height: "lastPageHeight",
      header: "header",
      footer: "footer"
    }.freeze

    FORMAT_BY_EXTENSION = {
      ".png" => :png, ".jpg" => :jpeg, ".jpeg" => :jpeg, ".webp" => :webp,
      ".pdf" => :pdf, ".svg" => :svg, ".raw" => :raw, ".rgba" => :raw
    }.freeze

    def initialize(root, config, engine)
      @root = root
      @config = config
      @engine = engine
    end

    def engine
      @engine || Engine.default
    end

    # ── the document ────────────────────────────────────────────────────────

    # The IR document as a Hash.
    def to_ir
      document = { "sone" => Ir::VERSION }
      fonts = @config[:fonts]
      unless fonts.nil? || fonts.empty?
        document["fonts"] = fonts.map { |font| font_entry(font) }
      end
      config = resolved_config
      document["config"] = config unless config.empty?
      document["root"] = @root.to_ir
      document
    end
    alias to_h to_ir

    # The IR document as JSON.
    def to_json(pretty: false)
      pretty ? JSON.pretty_generate(to_ir) : JSON.generate(to_ir)
    end

    # ── outputs ─────────────────────────────────────────────────────────────

    def png(density: nil)
      engine.render(to_json, format: :png, density: density)
    end

    def jpeg(quality: 1.0, density: nil)
      engine.render(to_json, format: :jpeg, density: density, quality: quality)
    end
    alias jpg jpeg

    def webp(quality: 1.0, density: nil)
      engine.render(to_json, format: :webp, density: density, quality: quality)
    end

    # Raw RGBA pixels, row-major, unpremultiplied.
    def raw(density: nil)
      engine.render(to_json, format: :raw, density: density)
    end

    # A PDF. With `page_height` set, one page per break and selectable text.
    def pdf
      engine.render(to_json, format: :pdf)
    end

    def svg
      engine.render(to_json, format: :svg)
    end

    # One raster image per page. Requires `page_height`.
    def pages(format: :png, density: nil, quality: 1.0)
      engine.render_pages(to_json, format: format, density: density, quality: quality)
    end

    # Render and write to `path`, inferring the format from its extension.
    def save(path, **options)
      format = format_for(path)
      bytes =
        case format
        when :png then png(density: options[:density])
        when :jpeg then jpeg(**options.select { |key, _| %i[quality density].include?(key) })
        when :webp then webp(**options.select { |key, _| %i[quality density].include?(key) })
        when :raw then raw(density: options[:density])
        when :pdf then pdf
        when :svg then svg
        end
      File.binwrite(path, bytes)
      path
    end

    # Write `name-1.png`, `name-2.png`, ... next to `path`.
    def save_pages(path, **options)
      extension = File.extname(path)
      stem = path[0...(path.length - extension.length)]
      format = FORMAT_BY_EXTENSION.fetch(extension.downcase, :png)
      pages(format: format, **options).each_with_index.map do |bytes, index|
        name = "#{stem}-#{index + 1}#{extension}"
        File.binwrite(name, bytes)
        name
      end
    end

    # ── introspection ───────────────────────────────────────────────────────

    # The computed layout tree.
    def layout
      JSON.parse(engine.dump_layout(to_json))
    end

    # Dataset-style boxes at `:node`, `:line` or `:word` granularity.
    def metadata(granularity = :node)
      JSON.parse(engine.dump_metadata(to_json, granularity))
    end

    private

    def resolved_config
      @config.each_with_object({}) do |(key, value), out|
        next if value.nil?

        name = CONFIG_KEYS[key]
        next if name.nil? # `fonts` and `engine` live outside config

        out[name] = value.is_a?(Node) ? value.to_ir : Ir.value(value)
      end
    end

    def font_entry(font)
      case font
      when Hash then { "name" => font[:name].to_s, "src" => font[:src].to_s }
      when Array then { "name" => font[0].to_s, "src" => font[1].to_s }
      else raise ArgumentError, "a font is a {name:, src:} hash or a [name, src] pair"
      end
    end

    def format_for(path)
      FORMAT_BY_EXTENSION.fetch(File.extname(path).downcase) do
        raise ArgumentError, "cannot infer an output format from #{path.inspect}"
      end
    end
  end

  # Wrap a node with render configuration.
  #
  #   Sone.render(root, density: 2).save("card.png")
  def self.render(root, engine: nil, **config)
    unknown = config.keys - Rendering::CONFIG_KEYS.keys - [:fonts]
    raise ArgumentError, "unknown render option #{unknown.first.inspect}" unless unknown.empty?

    Rendering.new(root, config, engine)
  end
end
