# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "sone"

module TestHelper
  ROOT = Sone::Native.checkout_root
  FONT = File.join(ROOT, "fixtures", "font", "GeistMono-Regular.ttf")
  FAMILY = "Geist Mono"

  def engine
    @engine ||= Sone::Engine.new(ROOT).tap { |e| e.register_font_file(FAMILY, FONT) }
  end

  def teardown
    @engine&.close
  end

  # The IR props of a root node, as the engine would see them.
  def props_of(node)
    node.to_ir["props"] || {}
  end
end
