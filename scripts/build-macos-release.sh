#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Grok Desktop for macOS"
VERSION="$(node -e 'const fs = require("fs"); const pkg = JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(pkg.version);' "$ROOT_DIR/package.json")"
APP_BUNDLE_PATH="$ROOT_DIR/src-tauri/target/release/bundle/macos/${APP_NAME}.app"
ARTIFACT_DIR="$ROOT_DIR/release-artifacts/v${VERSION}"
APP_OUTPUT_DIR="$ARTIFACT_DIR/app"
APP_OUTPUT_PATH="$APP_OUTPUT_DIR/${APP_NAME}.app"

cd "$ROOT_DIR"

npm run tauri build -- --bundles app

if [[ ! -d "$APP_BUNDLE_PATH" ]]; then
  echo "Expected app bundle was not created: $APP_BUNDLE_PATH" >&2
  exit 1
fi

mkdir -p "$APP_OUTPUT_DIR"
rm -rf "$APP_OUTPUT_PATH"
ditto "$APP_BUNDLE_PATH" "$APP_OUTPUT_PATH"

echo "App bundle copied to: $APP_OUTPUT_PATH"
