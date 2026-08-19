# frozen_string_literal: true

require_relative "ir"

module Sone
  # The macros every property module is built from.
  #
  # A property call with no argument reads, and with an argument writes — so a
  # block can branch on what it has already been given. The exceptions are
  # flags and decorations, which have a natural "on" value: `nowrap` and
  # `underline` set rather than read, because a bare call that silently did
  # nothing would be a trap.
  module Macros
    def prop(name, key = nil)
      key ||= Ir.camelize(name)
      define_method(name) do |*args|
        return @props[key] if args.empty?

        @props[key] = Ir.value(args.length == 1 ? args.first : args)
        self
      end
      alias_camel(name, key)
    end

    # A property whose bare call means "turn this on".
    def flag(name, default = true, key = nil)
      key ||= Ir.camelize(name)
      define_method(name) do |*args|
        @props[key] = args.empty? ? default : Ir.value(args.first)
        self
      end
      alias_camel(name, key)
    end

    # A list-valued property that accumulates across calls.
    def list_prop(name, key = nil)
      key ||= Ir.camelize(name)
      define_method(name) do |*values|
        return @props[key] if values.empty?

        (@props[key] ||= []).concat(values.map { |item| Ir.value(item) })
        self
      end
      alias_camel(name, key)
    end

    # A property set from every argument at once, replacing what was there.
    def tuple_prop(name, key = nil)
      key ||= Ir.camelize(name)
      define_method(name) do |*values|
        return @props[key] if values.empty?

        @props[key] = values.map { |item| Ir.value(item) }
        self
      end
      alias_camel(name, key)
    end

    # camelCase alias, so a TypeScript example transfers with no edits.
    def alias_camel(name, key)
      alias_method(key, name) if key != name.to_s && !method_defined?(key)
    end
  end
end
