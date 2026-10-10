#!/bin/sh
# Remove the user-local installation created by install.sh.
set -eu

PREFIX="${PREFIX:-$HOME/.local}"

rm -f "$PREFIX/bin/microscope-rust"
rm -f "$PREFIX/share/applications/microscope-rust.desktop"
rm -f "$PREFIX/share/icons/hicolor/scalable/apps/microscope-rust.svg"
rm -f "$PREFIX/share/metainfo/microscope-rust.metainfo.xml"

command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$PREFIX/share/applications" >/dev/null 2>&1 || true

echo "Removed microscope-rust from $PREFIX"
