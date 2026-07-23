# Security Policy

## Supported Versions

Pebble is pre-alpha. Security fixes are applied to the latest commit on `main`
and included in the next tagged release. Older commits and prereleases are not
maintained separately.

## Report A Vulnerability

Please use
[GitHub private vulnerability reporting](https://github.com/o-henry/pebble/security/advisories/new).
Do not open a public issue for suspected credential exposure, arbitrary code
execution, screen-capture boundary bypasses, or other vulnerabilities that
could put users at risk.

Include:

- The affected commit or release.
- Reproduction steps and the expected security boundary.
- Whether the issue requires Screen Recording permission, AI sign-in, or an
  Anthropic API key.
- Any logs or proof with credentials and private screen content removed.

You should receive an acknowledgement within seven days. Please allow time for
a fix and coordinated disclosure before publishing details.

## Security Boundaries

Pebble is designed to:

- Capture only regions explicitly selected by the user.
- Read pixels without clicking, typing, scrolling, or controlling other apps.
- Keep captured frames and OCR output ephemeral.
- Require explicit opt-in before sending selected crops to an AI provider.
- Store optional Anthropic API keys only in macOS Keychain.

See [Security And Privacy](docs/SECURITY_AND_PRIVACY.md) for the complete data
flow and local storage policy.
