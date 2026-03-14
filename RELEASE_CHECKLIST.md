# Release Checklist

Release target: public macOS GitHub Releases plus Homebrew Cask distribution

Release version:
- `0.1.0`

App identity:
- Name: `Grok Desktop for macOS`
- Bundle identifier: `ai.x.grok.desktop`
- Minimum macOS version: `13.0`

## 1. Version Alignment

Verify these stay aligned before every release:
- [`package.json`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/package.json): `0.1.0`
- [`src-tauri/tauri.conf.json`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/src-tauri/tauri.conf.json): `0.1.0`
- [`src/lib/constants/app.ts`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/src/lib/constants/app.ts): About/version surface reads `package.json`

Visible identity must remain:
- `productName = Grok Desktop for macOS`
- main window title = `Grok Desktop for macOS`
- About surface title = `Grok Desktop for macOS`
- Dock/app bundle name = `Grok Desktop for macOS`

## 2. Local Validation

Run:

```bash
npm run check
npm run build
cd src-tauri && cargo check
```

Current result on 2026-03-14:
- `npm run check`: passed with `0 errors and 0 warnings`
- `npm run build`: passed and produced `dist/`
- `cargo check`: passed; shell-wrapper `Broken pipe` panic occurred only after the successful `Finished` line

## 3. Release Build Workflow

Preferred commands:

```bash
npm run release:macos:app
npm run release:macos:dmg
npm run release:macos:metadata
npm run release:macos:verify
```

One-shot wrapper:

```bash
npm run release:macos
```

Generated local artifacts:
- Ignored artifact root: `release-artifacts/`
- App bundle: `release-artifacts/v0.1.0/app/Grok Desktop for macOS.app`
- DMG: `release-artifacts/v0.1.0/Grok-Desktop-for-macOS-v0.1.0.dmg`

## 4. Bundle and Icon Verification

Verified against the built app on 2026-03-14:
- `CFBundleDisplayName = Grok Desktop for macOS`
- `CFBundleName = Grok Desktop for macOS`
- `CFBundleIdentifier = ai.x.grok.desktop`
- `CFBundleShortVersionString = 0.1.0`
- `LSMinimumSystemVersion = 13.0`
- bundle icon present at `Contents/Resources/icon.icns`
- tray assets present:
  - `src-tauri/icons/tray-template.png` (`18x18`)
  - `src-tauri/icons/tray-template.svg`
  - `src-tauri/icons/tray-idle@2x.png` (`44x44`)
  - `src-tauri/icons/tray-attention@2x.png` (`44x44`)

## 5. Focused Smoke Tests

Completed on 2026-03-14:
- built app bundle launched directly from `release-artifacts/v0.1.0/app/...`
- mounted DMG contained:
  - `Grok Desktop for macOS.app`
  - `Applications` symlink
- DMG integrity verification passed via `hdiutil verify`
- no obvious debug/log/temp/session/diagnostic files were present inside the built `.app` or `.dmg`

Still manual before public distribution:
- authenticated sign-in verification
- authenticated history visibility verification
- active-display fallback visual verification on a multi-display setup
- visual menu-bar tray icon inspection on a clean desktop session
- click-through smoke test of command palette, scratchpad, prompt library, workspaces, window restore, and auth recovery
- Gatekeeper verification after signing/notarization

## 6. GitHub Release Metadata

Prepared files:
- Release notes: [`release/github/v0.1.0.md`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/release/github/v0.1.0.md)
- DMG SHA256: `9088dbc2a09cba259c0edb6e63d67daccb6fb41f79194579be3227da113a9da8`

Before publishing:
- replace `OWNER/REPO` in generated release metadata if the final GitHub repo slug differs
- upload `Grok-Desktop-for-macOS-v0.1.0.dmg` to the `v0.1.0` GitHub Release
- keep the raw `.app` as internal/testing output only

## 7. Homebrew Cask

Prepared files:
- Cask: [`homebrew-grok-desktop/Casks/grok-desktop.rb`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/homebrew-grok-desktop/Casks/grok-desktop.rb)
- Tap notes: [`homebrew-grok-desktop/README.md`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/homebrew-grok-desktop/README.md)

Before publishing:
- use `homebrew-grok-desktop/` as the tap repo root
- rerun `GITHUB_REPOSITORY=OWNER/REPO bash scripts/render-release-metadata.sh` if the GitHub slug changes
- validate install from the published release URL with `brew install --cask grok-desktop`

## 8. Signing / Notarization / Publish

Still manual:
- sign the `.app` with a Developer ID Application certificate
- notarize the signed artifact with Apple
- staple the notarization ticket
- repackage or reverify the final signed `.dmg`
- publish the GitHub Release
- publish the Homebrew tap update
