# frozen_string_literal: true

require_relative "test_helper"

# The builder layer, which touches no native code — the point of the IR split is
# that this is testable without a rasterizer.
class DslTest < Minitest::Test
  include TestHelper

  def test_a_block_sets_properties_and_appends_children
    root = Sone.column do
      gap 20
      padding 20
      size 420, 300
      bg "khaki"
      corner_radius 28

      column { flex 1; corner_radius 20; corner_smoothing 0.7; bg "white" }

      row do
        gap 10
        column { bg "lightgreen"; size 50; border_radius 14 }
      end
    end

    ir = root.to_ir
    assert_equal 20, ir["props"]["gap"]
    assert_equal 420, ir["props"]["width"]
    assert_equal ["khaki"], ir["props"]["background"]
    assert_equal [28], ir["props"]["cornerRadius"]
    assert_equal %w[column row], ir["children"].map { |child| child["type"] }
    assert_equal 10, ir["children"][1]["props"]["gap"]
    assert_equal 1, ir["children"][1]["children"].length
  end

  def test_a_block_that_takes_an_argument_keeps_the_callers_self
    # instance_eval would move `self` and break @colour.
    assert_equal "salmon", Holder.new.build.to_ir["props"]["background"].first
  end

  class Holder
    def initialize
      @colour = "salmon"
    end

    def build
      Sone.column { |c| c.bg @colour }
    end
  end

  def test_a_no_argument_call_reads
    node = Sone.column { gap 12 }
    assert_equal 12, node.gap
    assert_nil node.padding
  end

  def test_a_block_can_branch_on_what_it_already_has
    node = Sone.column do
      gap 8
      padding(gap * 2)
    end
    assert_equal 16, props_of(node)["padding"]
  end

  def test_flags_set_rather_than_read
    # A bare `nowrap` that silently read would be a trap.
    assert_equal true, props_of(Sone.text("x") { nowrap })["nowrap"]
    assert_equal 1.0, props_of(Sone.text("x") { underline })["underline"]
    assert_equal 0.5, props_of(Sone.text("x") { underline 0.5 })["underline"]
  end

  def test_a_decoration_colour_can_be_explicitly_null
    props = props_of(Sone.text("x") { underline; underline_color nil })
    assert props.key?("underlineColor")
    assert_nil props["underlineColor"]
  end

  def test_symbols_become_keywords
    props = props_of(Sone.row { justify_content :space_between; align_items :center })
    assert_equal "space-between", props["justifyContent"]
    assert_equal "center", props["alignItems"]
  end

  def test_strings_and_numbers_pass_through_for_dims
    props = props_of(Sone.column { width 100; min_width "50%"; max_width :auto })
    assert_equal 100, props["width"]
    assert_equal "50%", props["minWidth"]
    assert_equal "auto", props["maxWidth"]
  end

  def test_box_shorthand_is_css
    assert_equal 12, props_of(Sone.column { margin 12 })["margin"]

    props = props_of(Sone.column { padding 10, 20 })
    assert_equal [10, 20, 10, 20],
                 %w[paddingTop paddingRight paddingBottom paddingLeft].map { |key| props[key] }
    refute props.key?("padding")
  end

  def test_box_shorthand_takes_sides_by_name
    props = props_of(Sone.column { padding top: 8, left: 4 })
    assert_equal [8, 8, 8, 4],
                 %w[paddingTop paddingRight paddingBottom paddingLeft].map { |key| props[key] }
  end

  def test_end_is_reachable_despite_being_a_keyword
    node = Sone.column { inset_end 8 }
    assert_equal 8, props_of(node)["end"]
    # The keyword spelling works with an explicit receiver.
    assert_equal 4, props_of(Sone.column { self.end 4 })["end"]
  end

  def test_a_local_can_be_worked_around_with_an_explicit_receiver
    node = Sone.column do
      size = 999 # rubocop:disable Lint/UselessAssignment — this is the point
      self.size 50
    end
    assert_equal 50, props_of(node)["width"]
  end

  def test_camel_case_aliases_exist_so_typescript_examples_transfer
    props = props_of(Sone.column { cornerRadius 8; alignItems :center })
    assert_equal [8], props["cornerRadius"]
    assert_equal "center", props["alignItems"]
  end

  def test_text_size_is_the_font_size_not_the_box_size
    props = props_of(Sone.text("Hello") { size 28 })
    assert_equal 28, props["size"]
    refute props.key?("width")
  end

  def test_text_wrap_is_the_paragraph_property_not_flex_wrap
    assert_equal true, props_of(Sone.text("x") { wrap false })["nowrap"]
    assert_equal "wrap", props_of(Sone.row { wrap :wrap })["flexWrap"]
  end

  def test_text_takes_content_and_spans
    node = Sone.text("Hello ") do
      font "Inter"
      span("world") { weight :bold; color "salmon" }
    end
    inline = node.to_ir["inline"]
    assert_equal "Hello ", inline[0]
    assert_equal "span", inline[1]["type"]
    assert_equal "bold", inline[1]["props"]["weight"]
    assert_equal ["Inter"], node.to_ir["props"]["font"]
  end

  def test_generated_children_are_just_ruby
    rows = [%w[a b], %w[c d]]
    table = Sone.table do
      rows.each do |cells|
        table_row do
          cells.each { |cell| table_cell { text cell } }
        end
      end
      table_row { table_cell { text "empty" } } if rows.empty?
    end

    assert_equal 2, table.to_ir["children"].length
    assert_equal "a", table.to_ir["children"][0]["children"][0]["children"][0]["inline"][0]
  end

  def test_a_subtree_can_be_appended_from_a_helper
    badge = Sone.column { bg "salmon" }
    root = Sone.column { append badge }
    assert_equal 1, root.to_ir["children"].length
  end

  def test_page_break_factory_and_property_coexist
    assert_equal "before", props_of(Sone.page_break)["pageBreak"]
    root = Sone.column { page_break! }
    assert_equal "before", root.to_ir["children"][0]["props"]["pageBreak"]
    assert_equal "avoid", props_of(Sone.column { page_break :avoid })["pageBreak"]
  end

  def test_filters_keep_the_order_they_were_added_in
    props = props_of(Sone.column { blur 4; grayscale 0.5 })
    assert_equal ["blur(4px)", "grayscale(0.5)"], props["filters"]
  end

  def test_photo_scale_type_takes_a_keyword_alignment
    props = props_of(Sone.photo("a.png") { scale_type :cover, :center })
    assert_equal "cover", props["scaleType"]
    assert_equal 0.5, props["scaleAlignment"]
  end

  def test_grid_tracks
    props = props_of(Sone.grid { columns "1fr", :auto, 120 })
    assert_equal ["1fr", "auto", 120], props["columns"]
  end

  def test_the_document_carries_the_schema_version
    document = Sone.render(Sone.column).to_ir
    assert_equal 1, document["sone"]
    assert_equal "column", document["root"]["type"]
    refute document.key?("config")
  end

  def test_config_is_written_when_set
    config = Sone.render(Sone.column, width: 420, page_height: 800, margin: 20).to_ir["config"]
    assert_equal 420, config["width"]
    assert_equal 800, config["pageHeight"]
    assert_equal 20, config["margin"]
  end

  def test_pagination_tokens_are_passed_through_untouched
    json = Sone.render(Sone.column, page_height: 800, header: Sone.text("Page {pageNumber}")).to_json
    assert_includes json, "{pageNumber}"
  end

  def test_an_unknown_render_option_is_rejected
    assert_raises(ArgumentError) { Sone.render(Sone.column, colour: "red") }
  end

  def test_to_h_exposes_the_ir
    assert_equal "column", Sone.column.to_h["type"]
  end
end
