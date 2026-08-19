# frozen_string_literal: true

require_relative "nodes"

module Sone
  # Child factories, available inside any container's block.
  #
  # A factory appends a child and returns it, so `column { ... }` reads as part
  # of the parent's block rather than as an assignment.
  module Children
    CONTAINERS = {
      column: ColumnNode,
      row: RowNode,
      grid: GridNode,
      text_default: TextDefaultNode,
      table: TableNode,
      table_row: TableRowNode,
      table_cell: TableCellNode,
      list: ListNode,
      list_item: ListItemNode
    }.freeze

    CONTAINERS.each do |name, klass|
      define_method(name) { |&block| adopt(klass.new, &block) }
    end

    # A paragraph. Positional strings become its content.
    def text(*content, &block)
      node = TextNode.new
      content.each { |item| node.inline << item.to_s }
      adopt(node, &block)
    end

    def photo(src, &block)
      adopt(PhotoNode.new.src(src.to_s), &block)
    end

    def path(d, &block)
      adopt(PathNode.new.d(d.to_s), &block)
    end
    alias svg_path path

    def clip_group(clip_path, &block)
      adopt(ClipGroupNode.new.clip_path(clip_path.to_s), &block)
    end

    # An explicit page break. Named with a bang because `page_break` is already
    # the property that decides where breaks may fall.
    def page_break!
      adopt(ColumnNode.new.height(0).page_break(:before))
    end

    # Append an already-built node — the hook for helper methods that return a
    # subtree.
    def append(child)
      children << child
      child
    end
    alias << append

    private

    def adopt(child, &block)
      child.configure(&block)
      children << child
      child
    end
  end

  # Paragraph content, for `text` and `span`.
  module InlineContent
    # Append raw text.
    def content(*strings)
      strings.each { |string| inline << string.to_s }
      self
    end

    # Append a styled run.
    def span(text = nil, &block)
      node = SpanNode.new
      node.inline << text.to_s unless text.nil?
      node.configure(&block)
      inline << node
      node
    end
  end

  [ColumnNode, RowNode, GridNode, TextDefaultNode, TableNode, TableRowNode,
   TableCellNode, ListNode, ListItemNode, ClipGroupNode].each do |klass|
    klass.include(Children)
  end

  [TextNode, SpanNode].each { |klass| klass.include(InlineContent) }

  # ── top-level factories ───────────────────────────────────────────────────
  #
  # `Sone.column do ... end` builds a root. Nothing is defined on Object, so
  # there is no import ceremony and nothing to collide with.

  Children::CONTAINERS.each do |name, klass|
    define_singleton_method(name) { |&block| klass.new.configure(&block) }
  end

  def self.text(*content, &block)
    node = TextNode.new
    content.each { |item| node.inline << item.to_s }
    node.configure(&block)
  end

  def self.span(text = nil, &block)
    node = SpanNode.new
    node.inline << text.to_s unless text.nil?
    node.configure(&block)
  end

  def self.photo(src, &block)
    PhotoNode.new.src(src.to_s).configure(&block)
  end

  def self.path(d, &block)
    PathNode.new.d(d.to_s).configure(&block)
  end

  class << self
    alias svg_path path
  end

  def self.clip_group(clip_path, &block)
    ClipGroupNode.new.clip_path(clip_path.to_s).configure(&block)
  end

  def self.page_break
    ColumnNode.new.height(0).page_break(:before)
  end
end
