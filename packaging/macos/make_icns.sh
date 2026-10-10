#!/bin/sh
# Build dist/icon.icns from assets/icon.png (macOS only: uses sips + iconutil).
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SRC="$ROOT/assets/icon.png"
ICONSET="$ROOT/dist/icon.iconset"

rm -rf "$ICONSET"
mkdir -p "$ICONSET" "$ROOT/dist"

for size in 16 32 64 128 256 512 1024; do
    sips -z "$size" "$size" "$SRC" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
done
# Retina variants required by iconutil.
cp "$ICONSET/icon_32x32.png"    "$ICONSET/icon_16x16@2x.png"
cp "$ICONSET/icon_64x64.png"    "$ICONSET/icon_32x32@2x.png"
cp "$ICONSET/icon_256x256.png"  "$ICONSET/icon_128x128@2x.png"
cp "$ICONSET/icon_512x512.png"  "$ICONSET/icon_256x256@2x.png"
cp "$ICONSET/icon_1024x1024.png" "$ICONSET/icon_512x512@2x.png"

iconutil -c icns "$ICONSET" -o "$ROOT/dist/icon.icns"
echo "Created $ROOT/dist/icon.icns"
