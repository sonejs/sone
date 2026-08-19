# frozen_string_literal: true

# sone — a declarative canvas layout engine with rich international text.
#
#   require "sone"
#
#   Sone::Font.load("Inter", "fonts/Inter-Regular.ttf")
#
#   root = Sone.column do
#     gap 20
#     padding 20
#     bg "khaki"
#     corner_radius 28
#
#     text("Hello") { size 28; weight :bold }
#
#     row do
#       gap 10
#       column { bg "salmon"; size 50; corner_radius 14 }
#       column { bg "orange"; size 50; corner_radius 14 }
#     end
#   end
#
#   Sone.render(root, density: 2).save("card.png")
require_relative "sone/version"
require_relative "sone/errors"
require_relative "sone/ir"
require_relative "sone/node"
require_relative "sone/nodes"
require_relative "sone/dsl"
require_relative "sone/engine"
require_relative "sone/rendering"
