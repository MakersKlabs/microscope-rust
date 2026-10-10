#!/bin/sh
# Install microscope-rust for the current user (no root required).
set -eu

PREFIX="${PREFIX:-$HOME/.local}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/target/release/microscope-rust"

if [ ! -x "$BIN" ]; then
    echo "Binary not found. Build it first with: cargo build --release"
    exit 1
fi

install -Dm755 "$BIN" "$PREFIX/bin/microscope-rust"
install -Dm644 "$ROOT/packaging/linux/microscope-rust.desktop" \
    "$PREFIX/share/applications/microscope-rust.desktop"
install -Dm644 "$ROOT/packaging/linux/microscope-rust.svg" \
    "$PREFIX/share/icons/hicolor/scalable/apps/microscope-rust.svg"
install -Dm644 "$ROOT/packaging/linux/microscope-rust.metainfo.xml" \
    "$PREFIX/share/metainfo/microscope-rust.metainfo.xml"

command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$PREFIX/share/applications" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && \
    gtk-update-icon-cache -q -t -f "$PREFIX/share/icons/hicolor" >/dev/null 2>&1 || true

echo "Installed microscope-rust to $PREFIX"
echo "Run it from the application menu or: $PREFIX/bin/microscope-rust"
