#!/usr/bin/env bash
#
# Refresh the parity corpus from a checkout of the TypeScript engine
# (github.com/seanghay/sone), which is the behavioural reference.
#
#   tools/sync-fixtures.sh ../sone
#
# Regenerates the IR documents, the computed layout trees and the line-break
# corpus by running the dumpers in that checkout, then copies them here along
# with the golden renders, fonts and images.

set -euo pipefail

SONE_DIR="${1:-}"
if [[ -z "$SONE_DIR" || ! -f "$SONE_DIR/package.json" ]]; then
  echo "usage: tools/sync-fixtures.sh <path-to-seanghay/sone-checkout>" >&2
  exit 64
fi

SONE_DIR="$(cd "$SONE_DIR" && pwd)"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$HERE/fixtures"

echo "==> regenerating corpora in $SONE_DIR"
(cd "$SONE_DIR" && npx tsx test/visual/dump-ir.ts && npx tsx test/visual/dump-breaks.ts)

echo "==> copying fonts and images"
rsync -a --delete "$SONE_DIR/test/font/"  "$OUT/font/"
rsync -a --delete "$SONE_DIR/test/image/" "$OUT/image/"

echo "==> copying IR, layout and break corpora"
rsync -a --delete "$SONE_DIR/test/visual/ir/"     "$OUT/visual/ir/"
rsync -a --delete "$SONE_DIR/test/visual/layout/" "$OUT/visual/layout/"
cp "$SONE_DIR/test/visual/break-corpus.json" "$OUT/visual/break-corpus.json"

echo "==> copying golden renders"
find "$OUT/visual" -maxdepth 1 \( -name '*.jpg' -o -name '*.pdf' \) -delete
count=0
for ir in "$OUT/visual/ir/"*.json; do
  name="$(basename "$ir" .json)"
  for ext in jpg pdf; do
    if [[ -f "$SONE_DIR/test/visual/$name.$ext" ]]; then
      cp "$SONE_DIR/test/visual/$name.$ext" "$OUT/visual/$name.$ext"
      count=$((count + 1))
    fi
  done
done

echo "==> $count goldens, $(ls "$OUT/visual/ir" | wc -l | tr -d ' ') IR documents"
echo "    now run: cargo run -p sone-goldens --bin sone-goldens"
