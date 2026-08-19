# frozen_string_literal: true

module Sone
  # The base for every sone failure.
  class Error < StandardError; end

  # The IR document could not be parsed.
  class IrError < Error; end

  # A font or an image could not be loaded.
  class AssetError < Error; end

  # Layout or rasterization failed.
  class RenderError < Error; end
end
