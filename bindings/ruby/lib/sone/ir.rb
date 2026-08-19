# frozen_string_literal: true

module Sone
  # Turning Ruby values into IR values.
  module Ir
    VERSION = 1

    module_function

    # `:corner_radius` -> `"cornerRadius"`, the spelling the IR uses.
    def camelize(name)
      name.to_s.gsub(/_([a-z])/) { Regexp.last_match(1).upcase }
    end

    # Symbols are the idiomatic way to write a keyword, and every keyword the
    # engine takes is kebab-case: `:space_between` -> `"space-between"`.
    def value(raw)
      case raw
      when Symbol then raw.to_s.tr("_", "-")
      when Array then raw.map { |item| value(item) }
      else raw
      end
    end

    def encode(raw)
      case raw
      when Node then raw.to_ir
      when Array then raw.map { |item| encode(item) }
      when Hash then raw.each_with_object({}) { |(key, item), out| out[key.to_s] = encode(item) }
      else raw
      end
    end
  end
end
