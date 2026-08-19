# frozen_string_literal: true

require_relative "lib/sone/version"

Gem::Specification.new do |spec|
  spec.name = "sone"
  spec.version = Sone::VERSION
  spec.authors = ["Seanghay Yath"]
  spec.summary = "A declarative canvas layout engine with rich international text"
  spec.description = "Flexbox layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG " \
                     "output, driven by a block DSL."
  spec.homepage = "https://github.com/seanghay/sone"
  spec.license = "Apache-2.0"
  spec.required_ruby_version = ">= 2.6.0"

  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["documentation_uri"] = "#{spec.homepage}/blob/main/bindings/ruby/README.md"

  spec.files = Dir["lib/**/*.rb"] + ["README.md"]
  spec.require_paths = ["lib"]

  # FFI over include/sone.h, so installing needs no Rust toolchain and no build
  # step. The native library itself is not in this gem yet — see the README.
  spec.add_dependency "ffi", "~> 1.15"
end
