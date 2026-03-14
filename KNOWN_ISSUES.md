# Known Issues

## Non-blocking

### Shell-wrapper `Broken pipe` after successful `cargo` commands

Observed on 2026-03-14:
- `cargo check` completed successfully and printed `Finished`
- the shell wrapper then emitted a `clap_complete` `Broken pipe` panic

Current interpretation:
- this does not indicate a Rust compile failure for this repo
- treat it as non-blocking only when Cargo has already completed successfully

### GitHub Release URLs are still templated with `OWNER/REPO`

Current state:
- [`release/github/v0.1.0.md`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/release/github/v0.1.0.md) and [`homebrew-grok-desktop/Casks/grok-desktop.rb`](/Users/yale.cochran/workspace/Tauri/Projects/grok-desktop-v2/homebrew-grok-desktop/Casks/grok-desktop.rb) use the standard GitHub Releases URL pattern
- the repo slug is intentionally left as `OWNER/REPO` until the public GitHub repository is finalized

Practical guidance:
- rerun `GITHUB_REPOSITORY=OWNER/REPO bash scripts/render-release-metadata.sh` once the final repo slug exists

## Blocking For External Signed Distribution

### Signing and notarization are not automated yet

Current state:
- local unsigned `.app` and `.dmg` artifacts build successfully
- Apple Developer ID signing, notarization, stapling, and final Gatekeeper validation remain manual

Impact:
- local release packaging is ready
- public distribution should not be treated as complete until the signed/notarized path is executed and verified
