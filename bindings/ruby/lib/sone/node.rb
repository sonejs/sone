# frozen_string_literal: true

require_relative "ir"
require_relative "properties"

module Sone
  # A node in the document tree.
  #
  # A node is also its own builder: a configuration block is evaluated against
  # it, so setting a property and adding a child are the same kind of call.
  class Node
    attr_reader :type, :props, :children, :inline

    def initialize(type)
      @type = type
      @props = {}
      @children = []
      @inline = []
    end

    # Evaluate a configuration block.
    #
    # A block that takes an argument is yielded the node, so `self` stays the
    # caller's — which is the only way `@ivars` and helper methods from the
    # enclosing object keep resolving. A block that takes none is
    # `instance_eval`'d, which is what makes the bare `gap 20` form work.
    def configure(&block)
      return self unless block

      if block.arity.positive?
        block.call(self)
      else
        instance_eval(&block)
      end
      self
    end

    # A name for this node, echoed back by `layout` and `metadata`.
    def tag(*args)
      return @props["tag"] if args.empty?

      @props["tag"] = args.first.to_s
      self
    end

    # Set raw IR properties, for anything this API does not cover yet.
    def apply(values)
      values.each { |key, value| @props[Ir.camelize(key)] = Ir.value(value) }
      self
    end

    # The IR document fragment for this node.
    def to_ir
      out = { "type" => type }
      unless props.empty?
        out["props"] = props.each_with_object({}) { |(key, value), acc| acc[key] = Ir.encode(value) }
      end
      out["children"] = children.map(&:to_ir) unless children.empty?
      unless inline.empty?
        out["inline"] = inline.map { |item| item.is_a?(Node) ? item.to_ir : item }
      end
      out
    end
    alias to_h to_ir

    def inspect
      label = @props["tag"] ? " #{@props['tag'].inspect}" : ""
      "#<sone:#{type}#{label} props=#{props.size} children=#{children.size}>"
    end
  end

  # Flexbox, sizing, spacing and the visual box properties.
  module LayoutProps
    extend Macros

    prop :align_content
    prop :align_items
    prop :align_self
    prop :aspect_ratio
    prop :box_sizing
    prop :direction
    prop :display
    prop :flex
    prop :basis, "flexBasis"
    prop :flex_direction
    prop :grow, "flexGrow"
    prop :shrink, "flexShrink"
    prop :wrap, "flexWrap"
    prop :justify_content
    prop :overflow
    prop :position

    prop :gap
    prop :row_gap
    prop :column_gap

    prop :width
    prop :height
    prop :min_width
    prop :min_height
    prop :max_width
    prop :max_height

    prop :border_color
    prop :margin_top
    prop :margin_right
    prop :margin_bottom
    prop :margin_left
    prop :padding_top
    prop :padding_right
    prop :padding_bottom
    prop :padding_left

    prop :top
    prop :right
    prop :bottom
    prop :left
    prop :start
    # `end` is a Ruby keyword, so the bare-callable name is `inset_end`. The
    # keyword spelling still works with an explicit receiver: `self.end 8`.
    prop :inset_end, "end"
    alias_method :end, :inset_end
    alias_method :inset_start, :start
    prop :inset

    prop :opacity
    prop :corner_smoothing
    alias_method :border_smoothing, :corner_smoothing
    prop :corner
    prop :rotate, "rotation"
    prop :translate_x
    prop :translate_y
    prop :page_break

    list_prop :bg, "background"
    list_prop :background
    list_prop :shadow, "shadows"

    tuple_prop :corner_radius
    alias_method :rounded, :corner_radius
    alias_method :border_radius, :corner_radius

    BORDER_KEYS = %w[borderWidth borderTopWidth borderRightWidth borderBottomWidth borderLeftWidth].freeze
    MARGIN_KEYS = %w[margin marginTop marginRight marginBottom marginLeft].freeze
    PADDING_KEYS = %w[padding paddingTop paddingRight paddingBottom paddingLeft].freeze

    # Width and height. One argument makes a square.
    def size(*args)
      return [@props["width"], @props["height"]] if args.empty?

      width, height = args
      @props["width"] = Ir.value(width)
      @props["height"] = Ir.value(height.nil? ? width : height)
      self
    end

    # CSS 1-4 value shorthand, positional or by side. An omitted side follows
    # CSS: right defaults to top, bottom to top, left to right.
    def padding(*values, **sides)
      box(PADDING_KEYS, values, sides)
    end

    def margin(*values, **sides)
      box(MARGIN_KEYS, values, sides)
    end

    def border_width(*values, **sides)
      box(BORDER_KEYS, values, sides)
    end

    # Scale. One argument scales both axes.
    def scale(x, y = nil)
      @props["scale"] = [x, y.nil? ? x : y]
      self
    end

    def grid_column(start_line, span = nil)
      @props["gridColumnStart"] = start_line
      @props["gridColumnSpan"] = span unless span.nil?
      self
    end

    def grid_row(start_line, span = nil)
      @props["gridRowStart"] = start_line
      @props["gridRowSpan"] = span unless span.nil?
      self
    end

    # CSS filters, applied in the order they are added.
    def blur(radius)
      filter("blur(#{radius}px)")
    end

    def hue_rotate(degrees)
      filter("hue-rotate(#{degrees})")
    end

    %i[brightness contrast grayscale invert saturate sepia].each do |name|
      define_method(name) { |amount| filter("#{name}(#{amount})") }
    end

    def filter(css)
      (@props["filters"] ||= []) << css
      self
    end

    private

    def box(keys, values, sides)
      if values.empty? && sides.empty?
        return @props[keys[0]]
      end

      if sides.empty? && values.length == 1
        @props[keys[0]] = Ir.value(values.first)
        return self
      end

      top    = sides.fetch(:top) { values[0] }
      right  = sides.fetch(:right) { values[1].nil? ? top : values[1] }
      bottom = sides.fetch(:bottom) { values[2].nil? ? top : values[2] }
      left   = sides.fetch(:left) { values[3].nil? ? right : values[3] }

      @props[keys[1]] = Ir.value(top)
      @props[keys[2]] = Ir.value(right)
      @props[keys[3]] = Ir.value(bottom)
      @props[keys[4]] = Ir.value(left)
      self
    end
  end

  # Span-level text styling.
  module SpanStyleProps
    extend Macros

    prop :color
    prop :size
    prop :style
    prop :weight
    prop :letter_spacing
    prop :word_spacing
    prop :stroke_color
    prop :stroke_width
    prop :offset_y
    prop :text_dir

    tuple_prop :font
    list_prop :drop_shadow, "dropShadows"

    # Decorations turn on with a bare call; pass an explicit nil to a colour to
    # mean "use the text colour", which the engine reads differently from the
    # property being absent.
    flag :underline, 1.0
    flag :overline, 1.0
    flag :line_through, 1.0

    prop :underline_color
    prop :overline_color
    prop :line_through_color
    prop :highlight, "highlightColor"
  end

  # Paragraph-level properties.
  module TextBlockProps
    extend Macros

    prop :max_lines
    prop :line_break
    prop :text_overflow
    prop :line_height
    prop :align
    prop :indent, "indentSize"
    prop :hanging_indent, "hangingIndentSize"
    prop :tab_leader
    prop :orientation
    prop :clip_image
    prop :base_dir
    prop :text_wrap

    flag :nowrap
    flag :autofit

    tuple_prop :tab_stops

    # Whether the paragraph wraps. Not the flexbox `wrap`.
    def wrap(value = true)
      @props["nowrap"] = !value
      self
    end
  end
end
