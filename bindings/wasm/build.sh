#!/usr/bin/env bash
#
# Build the WebAssembly engine.
#
#   ./build.sh              release build into dist/
#   SONE_WASM_DEBUG=1 ...   keep names and assertions
#
# Needs emscripten. Point EMSDK at an emsdk checkout, or install emscripten from
# Homebrew and let the shim below stand in for the layout skia-bindings expects.
#
# Skia is compiled from source here, which takes roughly an hour the first time.
# That is not a choice: the published emscripten prebuilt was linked with
# emscripten's legacy JavaScript exception handling, current Rust emits
# `-fwasm-exceptions` for this target, and the escape hatch that used to let
# Rust opt out (`-Zemscripten-wasm-eh=false`) has been removed. The two object
# formats cannot be linked together, so Skia has to be built with the same
# `-fwasm-exceptions` the Rust side uses. `target/` caches the result.
set -euo pipefail

cd "$(dirname "$0")"

# ── toolchain ────────────────────────────────────────────────────────────────

if [[ -z "${EMSDK:-}" ]]; then
  if [[ -d /opt/homebrew/opt/emscripten/libexec ]]; then
    # Homebrew installs a flat layout; skia-bindings wants an emsdk one and
    # reads `$EMSDK/upstream/emscripten/...`. One symlink bridges the two.
    mkdir -p .emsdk/upstream
    ln -sfn /opt/homebrew/opt/emscripten/libexec .emsdk/upstream/emscripten
    EMSDK="$PWD/.emsdk"
  else
    echo "error: set EMSDK to your emscripten installation" >&2
    exit 1
  fi
fi
export EMSDK

command -v emcc >/dev/null || { echo "error: emcc is not on PATH" >&2; exit 1; }

# ── flags ────────────────────────────────────────────────────────────────────

# Applies to every emcc invocation, Skia's included — which is the point, since
# every object has to agree on the exception model.
export EMCC_CFLAGS="-fwasm-exceptions -sERROR_ON_UNDEFINED_SYMBOLS=0"
export FORCE_SKIA_BUILD=1

# Link-only settings. They go through `-Clink-arg` rather than EMCC_CFLAGS so
# they are not repeated across thousands of compile steps, and they deliberately
# leave EXPORTED_FUNCTIONS alone — rustc generates that list from the
# `#[no_mangle]` symbols, and overriding it would mean maintaining it by hand.
LINK_ARGS=(
  -sEXPORT_ES6=1
  -sMODULARIZE=1
  -sEXPORT_NAME=createSoneEngine
  -sENVIRONMENT=web,worker,node
  -sALLOW_MEMORY_GROWTH=1
  -sMAXIMUM_MEMORY=4GB
  # Skia's SkSL and path code recurse deeply; the 64 KB default overflows.
  -sSTACK_SIZE=4MB
  -sEXPORTED_RUNTIME_METHODS=HEAPU8
  # `main` is empty and there is nothing to run at startup.
  -sINVOKE_RUN=0
)
if [[ -n "${SONE_WASM_DEBUG:-}" ]]; then
  LINK_ARGS+=(-sASSERTIONS=1 -g2)
else
  LINK_ARGS+=(-sASSERTIONS=0)
fi

RUSTFLAGS=""
for arg in "${LINK_ARGS[@]}"; do RUSTFLAGS+=" -Clink-arg=$arg"; done
export RUSTFLAGS

# ── build ────────────────────────────────────────────────────────────────────

echo "==> cargo build --target wasm32-unknown-emscripten"
cargo build --release --target wasm32-unknown-emscripten

OUT=target/wasm32-unknown-emscripten/release
mkdir -p dist
cp "$OUT/sone.js" dist/sone.js
cp "$OUT/sone.wasm" dist/sone.wasm

echo "==> tsdown (glue)"
npm run --silent build:ts

ls -l dist
