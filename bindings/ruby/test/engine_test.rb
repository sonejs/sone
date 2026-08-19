# frozen_string_literal: true

require "tmpdir"
require_relative "test_helper"

# Everything that crosses the C ABI.
class EngineTest < Minitest::Test
  include TestHelper

  def test_renders_a_png
    png = Sone.render(Sone.column { size 16; bg "red" }, engine: engine).png
    assert_equal "\x89PNG".b, png[0, 4]
  end

  def test_density_scales_the_raster
    node = Sone.column { size 10; bg "red" }
    # Raw is 4 bytes per pixel, so the byte count is the pixel count.
    assert_equal 10 * 10 * 4, Sone.render(node, engine: engine).raw.bytesize
    assert_equal 20 * 20 * 4, Sone.render(node, engine: engine).raw(density: 2).bytesize
  end

  def test_renders_every_format
    rendering = Sone.render(Sone.column { size 16; bg "teal" }, engine: engine)
    refute_empty rendering.jpeg(quality: 0.8)
    refute_empty rendering.webp
    assert_equal "%PDF".b, rendering.pdf[0, 4]
    assert_includes rendering.svg, "<svg"
  end

  def test_one_page_per_declared_break
    root = Sone.column do
      column { height 60; bg "red" }
      column { height 60; bg "green"; page_break :before }
      column { height 60; bg "blue"; page_break :before }
    end

    pages = Sone.render(root, engine: engine, width: 40, page_height: 200).pages
    assert_equal 3, pages.length
    pages.each { |page| assert_equal "\x89PNG".b, page[0, 4] }
  end

  def test_save_infers_the_format_from_the_extension
    Dir.mktmpdir("sone-ruby") do |directory|
      path = File.join(directory, "card.pdf")
      Sone.render(Sone.column { size 16; bg "red" }, engine: engine).save(path)
      assert_equal "%PDF".b, File.binread(path)[0, 4]
    end
  end

  def test_save_pages_numbers_the_files
    root = Sone.column do
      column { height 60; bg "red" }
      column { height 60; bg "blue"; page_break :before }
    end
    Dir.mktmpdir("sone-ruby") do |directory|
      written = Sone.render(root, engine: engine, width: 40, page_height: 200)
                    .save_pages(File.join(directory, "page.png"))
      assert_equal 2, written.length
      assert written.all? { |path| File.size(path).positive? }
    end
  end

  def test_the_font_registry_round_trips
    fresh = Sone::Engine.new(ROOT)
    refute fresh.font?(FAMILY)
    fresh.register_font_file(FAMILY, FONT)
    assert fresh.font?(FAMILY)
    assert_includes fresh.font_families, FAMILY

    fresh.reset_fonts
    refute fresh.font?(FAMILY)

    fresh.register_font(FAMILY, File.binread(FONT))
    assert fresh.font?(FAMILY)
  ensure
    fresh&.close
  end

  def test_registered_images_resolve_as_assets
    png = Sone.render(Sone.column { size 8; bg "red" }, engine: engine).png
    engine.register_image("logo", png)
    refute_empty Sone.render(Sone.photo("asset:logo") { size 8 }, engine: engine).png
  end

  def test_layout_comes_back_as_a_tree
    root = Sone.column do
      padding 5
      column { size 20; tag "inner" }
    end
    layout = Sone.render(root, engine: engine).layout
    assert_equal 30, layout["width"]
    assert_equal "inner", layout["children"][0]["tag"]
  end

  def test_metadata_honours_granularity
    rendering = Sone.render(Sone.text("hello world") { font FAMILY; size 12 }, engine: engine)
    assert_kind_of Hash, rendering.metadata
    assert_kind_of Hash, rendering.metadata(:word)
  end

  def test_a_bad_document_is_an_ir_error
    error = assert_raises(Sone::IrError) do
      engine.render('{"sone":99,"root":{"type":"column"}}')
    end
    assert_includes error.message, "unsupported IR version"
  end

  def test_a_missing_font_file_is_an_asset_error
    assert_raises(Sone::AssetError) { engine.register_font_file("Nope", "does/not/exist.ttf") }
  end

  def test_using_a_closed_engine_raises_rather_than_crashing
    closed = Sone::Engine.new(ROOT)
    closed.close
    closed.close
    assert_raises(Sone::Error) { closed.font?(FAMILY) }
  end

  def test_an_unknown_format_is_rejected_before_it_reaches_the_engine
    assert_raises(ArgumentError) { engine.render("{}", format: :tiff) }
  end

  # The gate every binding owes: the same document must come out of this binding
  # byte for byte the way it comes out of `sone-cli`.
  def test_matches_the_cli_byte_for_byte
    root = Sone.column do
      gap 20
      padding 20
      size 420, 200
      bg "khaki"
      corner_radius 28

      text("Hello ") do
        font "Geist Mono"
        size 24
        line_height 1.4
        span("world") { weight :bold; color "#c0392b" }
      end

      row do
        gap 10
        column { bg "lightgreen"; size 50; border_radius 14 }
        column { bg "salmon"; height 50; border_radius 14; flex 1 }
      end
    end

    # An absolute src, because the CLI resolves a document's assets against the
    # document's own directory and the engine resolves them against its base
    # directory — the two only agree when the path is absolute.
    rendering = Sone.render(root, engine: engine, density: 2,
                                  fonts: [{ name: FAMILY, src: FONT }])

    Dir.mktmpdir("sone-parity") do |directory|
      document = File.join(directory, "doc.json")
      File.write(document, rendering.to_json(pretty: true))
      from_cli = File.join(directory, "cli.png")

      command = ["cargo", "run", "-q", "-p", "sone-cli", "--", "render", document,
                 "--density", "2", "-o", from_cli]
      Dir.chdir(ROOT) { assert system(*command, out: File::NULL), "sone-cli failed" }

      assert_equal File.binread(from_cli), rendering.png
    end
  end
end
