#!/usr/bin/env bash
# Build the Bevy game for the web and stage a GitHub Pages-ready directory.
#
# Usage: tools/build_web.sh [output-dir]   (default: web/dist)
#
# Needs: rustup target add wasm32-unknown-unknown
#        wasm-bindgen-cli matching Cargo.lock (0.2.129)
# Optional: wasm-opt (Binaryen) for a smaller bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${1:-$ROOT/web/dist}"
WASM_BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
PROFILE_DIR="$ROOT/target/wasm32-unknown-unknown/wasm-release"
WASM="$PROFILE_DIR/elevator-action-bevy.wasm"

cd "$ROOT"

echo "==> building wasm (profile: wasm-release)"
cargo build --profile wasm-release --target wasm32-unknown-unknown --bin elevator-action-bevy

if [[ ! -f "$WASM" ]]; then
  echo "wasm artifact not found at $WASM" >&2
  exit 1
fi

rm -rf "$DIST"
mkdir -p "$DIST/assets/art"

echo "==> wasm-bindgen"
"$WASM_BINDGEN" \
  --target web \
  --no-typescript \
  --out-name elevator_action \
  --out-dir "$DIST" \
  "$WASM"

if command -v wasm-opt >/dev/null 2>&1; then
  echo "==> wasm-opt -Oz"
  wasm-opt -Oz --output "$DIST/elevator_action_bg.opt.wasm" "$DIST/elevator_action_bg.wasm"
  mv "$DIST/elevator_action_bg.opt.wasm" "$DIST/elevator_action_bg.wasm"
else
  echo "==> wasm-opt not found; shipping the unoptimised wasm"
fi

echo "==> staging static shell and runtime assets"
cp "$ROOT/web/index.html" "$ROOT/web/style.css" "$ROOT/web/favicon.svg" "$DIST/"
cp -R "$ROOT/assets/sprites" "$ROOT/assets/backgrounds" "$ROOT/assets/audio" "$ROOT/assets/fonts" "$DIST/assets/"
cp "$ROOT/assets/art/banner.png" "$DIST/assets/art/banner.png"
touch "$DIST/.nojekyll"

echo
echo "built: $DIST"
du -sh "$DIST"
find "$DIST" -maxdepth 1 -type f -printf '  %-28f %10s bytes\n' | sort
