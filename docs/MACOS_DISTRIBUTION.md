# macOS Distribution

Pebble's public macOS builds must use one stable Developer ID identity. This is
what lets macOS recognize an update as the same app and keep the user's Screen
Recording choice. Ad-hoc builds are for local development only.

## User Experience

A release user should need to:

1. Download the DMG for Apple Silicon (`arm64`) or Intel (`x64`).
2. Drag Pebble to Applications and open it.
3. Approve Screen Recording the first time a region is selected.

Updates signed with the same Developer ID and bundle identifier should not
require the user to remove and re-add Pebble. macOS can still ask again after a
permission reset or an operating-system security change.

An ad-hoc build is a different security identity on every rebuild because its
designated requirement is tied to a code hash. It must never replace a user's
installed Pebble. `npm run install:macos -- /path/to/Pebble.app` enforces this
boundary and also requires notarization, a stapled ticket, and Gatekeeper
acceptance before an atomic replacement.

## Release Requirements

- Apple Developer Program membership.
- A `Developer ID Application` certificate exported as a password-protected
  PKCS#12 file.
- An App Store Connect API key with Developer access for notarization.
- The fixed bundle identifier `com.ohenry.screenpebble`.
- GitHub Actions secrets configured before creating a release tag.

The workflow requires these secrets:

| Secret | Contents |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12` |
| `APPLE_API_ISSUER` | App Store Connect issuer ID |
| `APPLE_API_KEY` | App Store Connect key ID |
| `APPLE_API_KEY_CONTENT` | Base64-encoded `.p8` private key |
| `KEYCHAIN_PASSWORD` | Random password for the temporary CI keychain |

Never commit those values, certificate files, or private keys. Configure them
in GitHub repository secrets. The workflow writes credentials only into the
runner's temporary directory, limits the private key to mode `0600`, and
deletes the temporary keychain and files after every run.

## Release Flow

1. Synchronize the version in `package.json`, `package-lock.json`,
   `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and
   `src-tauri/tauri.conf.json`.
2. Update `docs/RELEASE_NOTES.md`.
3. Run the normal test, lint, build, and Clippy checks.
4. Run `npm run release:check`.
5. Push a new immutable version tag matching those files, such as `v0.2.1`.
   Never move or reuse a failed release tag.
6. Confirm both architecture jobs pass signing, notarization, stapling,
   Gatekeeper assessment, and artifact verification.
7. Confirm the two architecture signing reports contain one identical Team ID,
   bundle identifier, and Apple-anchored designated requirement with no `cdhash`.
8. Install the previous official build on clean Intel and Apple Silicon Macs,
   approve Screen Recording once, update to the candidate without resetting TCC,
   and confirm capture still works without another approval.

Only after both DMGs pass does the workflow create the GitHub prerelease. A
missing secret, ad-hoc identity, mismatched version, missing sidecar, failed
notarization, or failed Gatekeeper check stops publication.

The prerelease is not promoted as a stable public download until the permission
continuity test succeeds on both architectures. This manual operating-system
test cannot be replaced by a unit test or a headless GitHub runner.

The release check compares the npm lockfile version directly and reads Cargo
metadata with `--locked`, so stale lockfiles also stop the workflow.

## Source Builds

`npm run tauri:dev` and `npm run tauri:build` remain development commands. A
locally rebuilt app does not have the public Developer ID identity, so macOS may
treat it as a different app and request Screen Recording again. Do not publish
those outputs as official Pebble downloads or install them over an official
copy. The guarded installer rejects them.
