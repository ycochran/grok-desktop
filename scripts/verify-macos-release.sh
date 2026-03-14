#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Grok Desktop for macOS"
VERSION="$(node -e 'const fs = require("fs"); const pkg = JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(pkg.version);' "$ROOT_DIR/package.json")"
ARTIFACT_DIR="$ROOT_DIR/release-artifacts/v${VERSION}"
APP_PATH="$ARTIFACT_DIR/app/${APP_NAME}.app"
PLIST_PATH="$APP_PATH/Contents/Info.plist"
ICNS_PATH="$APP_PATH/Contents/Resources/icon.icns"
DMG_PATH="$ARTIFACT_DIR/Grok-Desktop-for-macOS-v${VERSION}.dmg"
MOUNT_DIR="$ARTIFACT_DIR/dmg-mount"

if [[ ! -d "$APP_PATH" ]]; then
  echo "Missing app bundle: $APP_PATH" >&2
  exit 1
fi

if [[ ! -f "$DMG_PATH" ]]; then
  echo "Missing DMG artifact: $DMG_PATH" >&2
  exit 1
fi

echo "Bundle metadata"
plutil -extract CFBundleDisplayName raw -o - "$PLIST_PATH"
plutil -extract CFBundleName raw -o - "$PLIST_PATH"
plutil -extract CFBundleIdentifier raw -o - "$PLIST_PATH"
plutil -extract CFBundleShortVersionString raw -o - "$PLIST_PATH"
plutil -extract LSMinimumSystemVersion raw -o - "$PLIST_PATH"

echo
echo "Bundle icon"
test -f "$ICNS_PATH"
ls -l "$ICNS_PATH"

echo
echo "Tray assets"
sips -g pixelWidth -g pixelHeight "$ROOT_DIR/src-tauri/icons/tray-idle@2x.png" "$ROOT_DIR/src-tauri/icons/tray-attention@2x.png" >/dev/null
file "$ROOT_DIR/src-tauri/icons/tray-idle@2x.png" "$ROOT_DIR/src-tauri/icons/tray-attention@2x.png"

rm -rf "$MOUNT_DIR"
mkdir -p "$MOUNT_DIR"

hdiutil attach -readonly -nobrowse -mountpoint "$MOUNT_DIR" "$DMG_PATH" >/dev/null

echo
echo "Mounted DMG contents"
find "$MOUNT_DIR" -maxdepth 1 -mindepth 1 -print | sort
test -d "$MOUNT_DIR/${APP_NAME}.app"
test -L "$MOUNT_DIR/Applications"

hdiutil detach "$MOUNT_DIR" >/dev/null
rm -rf "$MOUNT_DIR"

echo
echo "Smoke verification completed for v${VERSION}"
