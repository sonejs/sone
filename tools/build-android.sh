#!/usr/bin/env bash
# Cross-compiles sone-ffi for Android.
#
#     tools/build-android.sh [--profile release] [outdir]
#
# The feature set is not the desktop one, and that is deliberate. skia-bindings
# picks a prebuilt tarball by hashing target + features, so the features have to
# name an asset that rust-skia actually publishes. For 0.99.0 the only Android
# asset carrying svg and webp is
#
#     …-<target>-jpegd-jpege-pdf-svg-textlayout-vulkan-webpd-webpe
#
# so the build drops `embed-freetype` and adds `gpu-vulkan` purely to make the
# cache key match. Neither changes what the engine does — nothing here renders
# through Vulkan. Get it wrong and the build silently compiles Skia from source,
# which takes about an hour per ABI.
#
# armeabi-v7a is not built: rust-skia publishes no armv7 Android binary, and
# Play Store has required 64-bit since 2019.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE="release"
OUT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile) PROFILE="$2"; shift 2 ;;
    *) OUT="$1"; shift ;;
  esac
done

: "${ANDROID_NDK_HOME:=${ANDROID_NDK:-}}"
if [[ -z "$ANDROID_NDK_HOME" ]]; then
  SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
  ANDROID_NDK_HOME="$(ls -d "$SDK"/ndk/* 2>/dev/null | sort -V | tail -1 || true)"
fi
if [[ ! -d "$ANDROID_NDK_HOME" ]]; then
  echo "no Android NDK found — set ANDROID_NDK_HOME" >&2
  exit 1
fi

case "$(uname -s)" in
  Darwin) HOST_TAG="darwin-x86_64" ;;
  Linux)  HOST_TAG="linux-x86_64" ;;
  *) echo "unsupported host $(uname -s)" >&2; exit 1 ;;
esac

BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"
# 21 is Flutter's own minSdk floor, and every published prebuilt targets it.
API=21

# rust triple : Android ABI directory name
TARGETS=(
  "aarch64-linux-android:arm64-v8a"
  "x86_64-linux-android:x86_64"
)

export ANDROID_NDK="$ANDROID_NDK_HOME"
export ANDROID_NDK_HOME
export AR="$BIN/llvm-ar"
export RANLIB="$BIN/llvm-ranlib"

for entry in "${TARGETS[@]}"; do
  TRIPLE="${entry%%:*}"
  ABI="${entry##*:}"
  UPPER="$(echo "$TRIPLE" | tr 'a-z-' 'A-Z_')"

  export "CARGO_TARGET_${UPPER}_LINKER=$BIN/${TRIPLE}${API}-clang"
  export "CC_${TRIPLE//-/_}=$BIN/${TRIPLE}${API}-clang"
  export "CXX_${TRIPLE//-/_}=$BIN/${TRIPLE}${API}-clang++"

  echo "==> $TRIPLE ($ABI)"
  ( cd "$ROOT" && cargo build -p sone-ffi \
      --target "$TRIPLE" \
      --profile "$PROFILE" \
      --no-default-features \
      --features gpu-vulkan )

  if [[ -n "$OUT" ]]; then
    mkdir -p "$OUT/$ABI"
    cp "$ROOT/target/$TRIPLE/$PROFILE/libsone.so" "$OUT/$ABI/libsone.so"
    echo "    -> $OUT/$ABI/libsone.so"
  fi
done
