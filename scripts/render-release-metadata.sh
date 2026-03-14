#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Grok Desktop for macOS"
VERSION="$(node -e 'const fs = require("fs"); const pkg = JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(pkg.version);' "$ROOT_DIR/package.json")"
REPO_SLUG="${GITHUB_REPOSITORY:-OWNER/REPO}"
ARTIFACT_DIR="$ROOT_DIR/release-artifacts/v${VERSION}"
DMG_NAME="Grok-Desktop-for-macOS-v${VERSION}.dmg"
DMG_PATH="$ARTIFACT_DIR/${DMG_NAME}"
DMG_URL="https://github.com/${REPO_SLUG}/releases/download/v${VERSION}/${DMG_NAME}"
HOMEPAGE_URL="https://github.com/${REPO_SLUG}"

if [[ ! -f "$DMG_PATH" ]]; then
  echo "DMG artifact is missing. Run scripts/package-macos-dmg.sh first." >&2
  exit 1
fi

SHA256="$(shasum -a 256 "$DMG_PATH" | awk '{print $1}')"

mkdir -p "$ROOT_DIR/release/github" "$ROOT_DIR/homebrew-grok-desktop/Casks"

cat >"$ROOT_DIR/release/github/v${VERSION}.md" <<EOF
# Grok Desktop for macOS v${VERSION}

Public macOS release delivered as a drag-install \`.dmg\`.

## Artifact

- Primary download: \`${DMG_NAME}\`
- SHA256: \`${SHA256}\`
- Release URL: \`${DMG_URL}\`

## Installation

1. Download \`${DMG_NAME}\`.
2. Open the disk image.
3. Drag \`${APP_NAME}.app\` into \`Applications\`.
4. Launch the app from \`/Applications\`.

## Notes

- The raw \`${APP_NAME}.app\` remains a local internal/testing artifact and is not intended to be uploaded as the public release asset.
- Built locally with Tauri 2.9.5, Svelte, and Vite.

## Known Release Constraints

- This repo currently renders release metadata with \`${REPO_SLUG}\`. Replace \`OWNER/REPO\` before publishing if the public GitHub slug differs.
- Apple Developer ID signing, notarization, and stapling remain manual release steps.
EOF

cat >"$ROOT_DIR/homebrew-grok-desktop/Casks/grok-desktop.rb" <<EOF
cask "grok-desktop" do
  version "${VERSION}"
  sha256 "${SHA256}"

  url "${DMG_URL}"
  name "Grok Desktop for macOS"
  desc "Single-window macOS shell for Grok with local notes, prompt snippets, and workspaces"
  homepage "${HOMEPAGE_URL}"

  depends_on macos: ">= :ventura"

  app "${APP_NAME}.app"
end
EOF

cat >"$ROOT_DIR/homebrew-grok-desktop/README.md" <<EOF
# Homebrew Tap

This folder is ready to be used as a dedicated tap repository:

\`\`\`
homebrew-grok-desktop/
  Casks/
    grok-desktop.rb
\`\`\`

Current cask target:

- Version: \`${VERSION}\`
- URL: \`${DMG_URL}\`
- SHA256: \`${SHA256}\`
- App bundle: \`${APP_NAME}.app\`

If the public GitHub slug differs from \`${REPO_SLUG}\`, rerun:

\`\`\`bash
GITHUB_REPOSITORY=OWNER/REPO bash scripts/render-release-metadata.sh
\`\`\`
EOF

rm -f "$ROOT_DIR/homebrew-tap/Casks/grok-desktop-for-macos.rb" "$ROOT_DIR/homebrew-tap/README.md"

echo "Release notes written to: $ROOT_DIR/release/github/v${VERSION}.md"
echo "Homebrew cask written to: $ROOT_DIR/homebrew-grok-desktop/Casks/grok-desktop.rb"
