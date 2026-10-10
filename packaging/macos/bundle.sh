#!/bin/sh
# Create dist/Microscope.app (a macOS application bundle) from the release binary.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
APP="$ROOT/dist/Microscope.app"

cargo build --release --manifest-path "$ROOT/Cargo.toml"
sh "$ROOT/packaging/macos/make_icns.sh"

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
install -m755 "$ROOT/target/release/microscope-rust" "$APP/Contents/MacOS/microscope-rust"
install -m644 "$ROOT/dist/icon.icns" "$APP/Contents/Resources/icon.icns"
sed "s/__VERSION__/$VERSION/g" "$ROOT/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"

echo "Created $APP"
