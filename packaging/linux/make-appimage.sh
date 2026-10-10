#!/bin/sh
# Build a self-contained AppImage (Linux). Requires network access to fetch
# appimagetool and linuxdeploy, or set APPIMAGETOOL/LINUXDEPLOY to local paths.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
ARCH="$(uname -m)"
APPDIR="$ROOT/dist/Microscope.AppDir"

cargo build --release --manifest-path "$ROOT/Cargo.toml"

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/scalable/apps" "$APPDIR/usr/share/metainfo"

install -m755 "$ROOT/target/release/microscope-rust" "$APPDIR/usr/bin/microscope-rust"
install -m644 "$ROOT/packaging/linux/microscope-rust.desktop" "$APPDIR/usr/share/applications/"
install -m644 "$ROOT/packaging/linux/microscope-rust.svg" \
    "$APPDIR/usr/share/icons/hicolor/scalable/apps/"
install -m644 "$ROOT/packaging/linux/microscope-rust.metainfo.xml" "$APPDIR/usr/share/metainfo/"

ln -sf usr/share/applications/microscope-rust.desktop "$APPDIR/microscope-rust.desktop"
ln -sf usr/share/icons/hicolor/scalable/apps/microscope-rust.svg "$APPDIR/microscope-rust.svg"

TOOLS_DIR="$ROOT/build/tools"
mkdir -p "$TOOLS_DIR" "$ROOT/dist"
APPIMAGETOOL="${APPIMAGETOOL:-$TOOLS_DIR/appimagetool-$ARCH.AppImage}"
if [ ! -x "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool..."
    curl -L -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-$ARCH.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

ARCH="$ARCH" APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGETOOL" "$APPDIR" "$ROOT/dist/Microscope-$VERSION-$ARCH.AppImage"
echo "Created $ROOT/dist/Microscope-$VERSION-$ARCH.AppImage"
