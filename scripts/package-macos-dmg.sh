#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Grok Desktop for macOS"
VERSION="$(node -e 'const fs = require("fs"); const pkg = JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(pkg.version);' "$ROOT_DIR/package.json")"
ARTIFACT_DIR="$ROOT_DIR/release-artifacts/v${VERSION}"
APP_SOURCE_PATH="$ARTIFACT_DIR/app/${APP_NAME}.app"
DMG_BASENAME="Grok-Desktop-for-macOS-v${VERSION}"
STAGING_DIR="$ARTIFACT_DIR/dmg-staging"
TEMP_DMG="$ARTIFACT_DIR/${DMG_BASENAME}.tmp.dmg"
FINAL_DMG="$ARTIFACT_DIR/${DMG_BASENAME}.dmg"

if [[ ! -d "$APP_SOURCE_PATH" ]]; then
  echo "App bundle is missing. Run scripts/build-macos-release.sh first." >&2
  exit 1
fi

rm -rf "$STAGING_DIR" "$TEMP_DMG" "$FINAL_DMG"
mkdir -p "$STAGING_DIR"

ditto "$APP_SOURCE_PATH" "$STAGING_DIR/${APP_NAME}.app"
ln -s /Applications "$STAGING_DIR/Applications"

hdiutil create \
  -fs HFS+ \
  -volname "$APP_NAME" \
  -srcfolder "$STAGING_DIR" \
  -ov \
  -format UDRW \
  "$TEMP_DMG" >/dev/null

hdiutil convert \
  -ov \
  -format UDZO \
  -imagekey zlib-level=9 \
  -o "$FINAL_DMG" \
  "$TEMP_DMG" >/dev/null

hdiutil verify "$FINAL_DMG" >/dev/null
rm -rf "$STAGING_DIR" "$TEMP_DMG"

echo "DMG created at: $FINAL_DMG"
shasum -a 256 "$FINAL_DMG"
