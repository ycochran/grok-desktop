# Release Status

Current review date: 2026-03-14

## Summary

Status:
- local macOS release packaging is ready for `v0.1.0`
- public GitHub Release and Homebrew Cask metadata are prepared
- final external distribution is still blocked on repo slug finalization plus Apple signing/notarization

## Version and Identity

Aligned:
- [`package.json`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/package.json): `0.1.0`
- [`src-tauri/tauri.conf.json`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/src-tauri/tauri.conf.json): `0.1.0`
- [`src/lib/constants/app.ts`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/src/lib/constants/app.ts): About/version surface reads from `package.json`

Verified visible identity:
- `Grok Desktop for macOS`

## Build and Artifact Results

Validation completed:
- `npm run check`: passed
- `npm run build`: passed
- `cargo check`: passed, followed only by the known shell-wrapper `Broken pipe`

Release artifacts created:
- App: `release-artifacts/v0.1.0/app/Grok Desktop for macOS.app`
- DMG: `release-artifacts/v0.1.0/Grok-Desktop-for-macOS-v0.1.0.dmg`
- SHA256: `9088dbc2a09cba259c0edb6e63d67daccb6fb41f79194579be3227da113a9da8`

Prepared release metadata:
- GitHub Release notes: [`release/github/v0.1.0.md`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/release/github/v0.1.0.md)
- Homebrew cask: [`homebrew-grok-desktop/Casks/grok-desktop.rb`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/homebrew-grok-desktop/Casks/grok-desktop.rb)

## Metadata and Smoke Verification

Verified:
- bundle name, display name, identifier, version, and minimum macOS version
- bundle app icon present at `Contents/Resources/icon.icns`
- tray assets present and sized as expected
- built app launched directly from the generated bundle path
- DMG mounted successfully and exposed the app plus `Applications` symlink
- no obvious debug/log/temp/session/diagnostic files were found in the generated release artifacts

## Repository State

Repository initialized:
- `git init -b main` completed successfully

Ignored state verified via `git status --short --ignored`:
- `.claude/`
- `dist/`
- `node_modules/`
- `release-artifacts/`
- `src-tauri/gen/`
- `src-tauri/target/`

## Remaining Manual Steps

1. Set the final GitHub repo slug and rerun `GITHUB_REPOSITORY=OWNER/REPO bash scripts/render-release-metadata.sh`.
2. Sign the `.app` with Developer ID Application.
3. Notarize and staple the signed artifact.
4. Upload the final `.dmg` to the `v0.1.0` GitHub Release.
5. Publish the Homebrew tap repository `homebrew-grok-desktop` containing `Casks/grok-desktop.rb`.
6. Run final visual smoke testing on a clean macOS session after signing.
