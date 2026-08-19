#!/usr/bin/env bash
# Builds sone as an XCFramework for macOS, iOS and the iOS simulator.
#
#     tools/build-apple.sh [outdir]        # default: bindings/swift/Sone.xcframework
#
# As on Android, the feature set is chosen to name a prebuilt that rust-skia
# actually publishes. For 0.99.0 the Apple assets carrying svg and webp are
#
#     …-aarch64-apple-darwin-jpegd-jpege-pdf-svg-textlayout-webpd-webpe
#     …-aarch64-apple-ios-jpegd-jpege-pdf-svg-textlayout-webpd-webpe
#     …-aarch64-apple-ios-sim-jpegd-jpege-pdf-svg-textlayout-webpd-webpe
#
# which is exactly the default set minus `embed-freetype`, so every slice is
# built with --no-default-features. Leaving the default on asks for a key nobody
# publishes and falls back to compiling Skia from source, which on this project
# fails in ninja after ten minutes rather than merely being slow.
#
# The consequence is worth knowing: without embed-freetype, Skia rasterizes
# glyphs through the platform's font engine rather than its bundled FreeType.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/bindings/swift/Sone.xcframework}"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

TARGETS=(
  "aarch64-apple-darwin"
  "aarch64-apple-ios"
  "aarch64-apple-ios-sim"
)

for triple in "${TARGETS[@]}"; do
  echo "==> $triple"
  ( cd "$ROOT" && cargo build -p sone-ffi --release --target "$triple" --no-default-features )
done

# One headers directory, shared by every slice.
mkdir -p "$STAGE/include"
cp "$ROOT/include/sone.h" "$STAGE/include/sone.h"
cat > "$STAGE/include/module.modulemap" <<'MODULE'
module CSone {
    header "sone.h"
    export *
}
MODULE

rm -rf "$OUT"
xcodebuild -create-xcframework \
  -library "$ROOT/target/aarch64-apple-darwin/release/libsone.a" -headers "$STAGE/include" \
  -library "$ROOT/target/aarch64-apple-ios/release/libsone.a" -headers "$STAGE/include" \
  -library "$ROOT/target/aarch64-apple-ios-sim/release/libsone.a" -headers "$STAGE/include" \
  -output "$OUT"

echo
echo "wrote $OUT"
du -sh "$OUT"
