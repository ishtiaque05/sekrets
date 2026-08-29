#!/bin/sh
set -e

# Builds an AppImage for sekrets-gui. Requires appimagetool on PATH:
# https://github.com/AppImage/AppImageKit/releases

APP_NAME="sekrets-gui"
VERSION="${1:-0.1.0}"
APPDIR="target/appimage/${APP_NAME}.AppDir"

cargo build --release -p sekrets-gui

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
cp "target/release/${APP_NAME}" "$APPDIR/usr/bin/"

cat > "$APPDIR/${APP_NAME}.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Sekrets
Exec=${APP_NAME}
Icon=${APP_NAME}
Categories=Utility;Security;
EOF

# Placeholder-free minimal icon requirement: appimagetool needs an icon file
# to exist even if it's simple. Generate a flat-color PNG with ImageMagick
# rather than shipping a binary asset in this script.
if command -v convert >/dev/null 2>&1; then
    convert -size 256x256 xc:'#2b2b2b' "$APPDIR/${APP_NAME}.png"
else
    echo "ImageMagick 'convert' not found — install it or manually place ${APPDIR}/${APP_NAME}.png before continuing" >&2
    exit 1
fi

ln -sf "${APP_NAME}.png" "$APPDIR/.DirIcon" 2>/dev/null || true

cd target/appimage
appimagetool "${APP_NAME}.AppDir" "${APP_NAME}-${VERSION}-x86_64.AppImage"

echo "Built target/appimage/${APP_NAME}-${VERSION}-x86_64.AppImage"
