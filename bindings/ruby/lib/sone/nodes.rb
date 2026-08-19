# frozen_string_literal: true

require_relative "node"

module Sone
  class ColumnNode < Node
    include LayoutProps
    def initialize
      super("column")
    end
  end

  class RowNode < Node
    include LayoutProps
    def initialize
      super("row")
    end
  end

  class GridNode < Node
    include LayoutProps
    extend Macros

    tuple_prop :columns
    tuple_prop :rows
    tuple_prop :auto_rows
    tuple_prop :auto_columns

    def initialize
      super("grid")
    end
  end

  class SpanNode < Node
    include SpanStyleProps
    def initialize
      super("span")
    end
  end

  # Both a box and a paragraph, so it carries all three property sets.
  class TextNode < Node
    include LayoutProps
    include SpanStyleProps
    include TextBlockProps

    def initialize
      super("text")
    end

    # Declared here rather than left to module ancestry, because two of the
    # three included modules define each of these.

    # The font size, not the box size — matching the TypeScript API, where
    # `TextPropsBuilder` omits the layout `size`.
    def size(*args)
      return @props["size"] if args.empty?

      @props["size"] = Ir.value(args.first)
      self
    end

    # Whether the paragraph wraps. Not the flexbox `wrap`.
    def wrap(value = true)
      @props["nowrap"] = !value
      self
    end
  end

  # Cascades text styling onto its descendants without drawing a box.
  class TextDefaultNode < Node
    include SpanStyleProps
    include TextBlockProps
    def initialize
      super("text-default")
    end
  end

  class PhotoNode < Node
    include LayoutProps
    extend Macros

    prop :src
    prop :fill
    prop :clip_path
    flag :preserve_aspect_ratio
    flag :flip_horizontal
    flag :flip_vertical

    ALIGNMENTS = { start: 0.0, center: 0.5, end: 1.0 }.freeze

    def initialize
      super("photo")
    end

    # How the image fills its box. The alignment is 0..1, or one of
    # `:start`, `:center`, `:end`.
    def scale_type(value = nil, alignment = nil)
      return @props["scaleType"] if value.nil?

      @props["scaleType"] = Ir.value(value)
      unless alignment.nil?
        resolved = alignment.is_a?(Symbol) ? ALIGNMENTS.fetch(alignment) : alignment
        @props["scaleAlignment"] = resolved
      end
      self
    end
    alias scaleType scale_type
  end

  # An SVG path. `path` reads fine in Ruby, unlike the JVM and .NET bindings
  # where the name is taken.
  class PathNode < Node
    include LayoutProps
    extend Macros

    prop :d
    prop :stroke
    prop :stroke_width
    prop :stroke_line_cap
    prop :stroke_line_join
    prop :stroke_miter_limit
    prop :stroke_dash_offset
    prop :fill
    prop :fill_opacity
    prop :fill_rule
    prop :scale_path

    tuple_prop :stroke_dash_array

    def initialize
      super("path")
    end
  end

  class TableNode < Node
    include LayoutProps

    def initialize
      super("table")
    end

    # Row and column spacing. One argument sets both.
    def spacing(*args)
      return @props["spacing"] if args.empty?

      row, column = args
      @props["spacing"] = [row, column.nil? ? row : column]
      self
    end
  end

  class TableRowNode < Node
    include LayoutProps
    def initialize
      super("table-row")
    end
  end

  class TableCellNode < Node
    include LayoutProps
    extend Macros

    prop :colspan
    prop :rowspan

    def initialize
      super("table-cell")
    end
  end

  class ListNode < Node
    include LayoutProps
    extend Macros

    prop :list_style
    prop :marker_gap
    prop :marker_offset
    prop :start_index

    def initialize
      super("list")
    end
  end

  class ListItemNode < Node
    include LayoutProps
    extend Macros

    # Override the list's marker for this item alone.
    prop :marker

    def initialize
      super("list-item")
    end
  end

  class ClipGroupNode < Node
    include LayoutProps
    extend Macros

    prop :clip_path

    def initialize
      super("clip-group")
    end
  end
end
