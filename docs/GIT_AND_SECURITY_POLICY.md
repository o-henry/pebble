# Git And Security Policy

This document defines what may be committed, what must never be committed, and
which security checks must happen before every commit and push.

## Safe To Commit

Commit these when they are relevant to the current atomic change:

- Source code.
- Tests and fixtures that contain no secrets or captured user content.
- Documentation.
- Architecture decision records.
- Build scripts.
- Package manifests and lockfiles.
- Static assets intentionally created for the product, such as logos, icons, and
  public demo media.
- Redacted sample config such as `.env.example`.

Public demo media must be created from synthetic or non-sensitive content. Never
use real private screen captures as demo assets.

## Never Commit

Do not commit:

- `.env` files or local config containing secrets.
- API keys, access tokens, refresh tokens, session cookies, SSH keys, signing
  keys, certificates, provisioning profiles, or passwords.
- Captured frames, screenshots, OCR output, frame history, previews, or debug
  captures from a user's screen.
- Browser history, URLs collected from the user's browser, clipboard contents,
  local document contents, or AI prompts derived from screen content.
- Crash dumps or logs containing private file paths, screen text, tokens, or
  user data.
- Build artifacts unless the release process explicitly requires them.
- Broad permission grants added only to make a prototype work.

If a file is useful but contains sensitive data, replace it with a redacted
sample and commit only the sample.

## Required Security Questions For Every Change

Before committing, answer these questions mentally and inspect the diff for
evidence:

1. Did this change add any new permission, network path, filesystem access, shell
   access, capture behavior, OCR behavior, AI handoff, or persistence?
2. Could this change cause capture to continue while paused, hidden, blanked,
   closed, or deleted?
3. Could any full-monitor frame, captured region, screenshot, OCR result, or AI
   payload be written to disk or committed later?
4. Could any secret, token, cookie, or local account credential be read, logged,
   stored, or sent?
5. Could this change make AI handoff happen by default or outside the selected
   region?
6. Could this change make the app harder for the user to understand, stop, or
   trust?

If the answer to any question is yes or uncertain, stop and either add a test,
add a guard, narrow the change, or document the risk before committing.

## Commit-Time Checklist

Before every commit:

1. Run `git status --short --branch`.
2. Review the unstaged and staged diffs.
3. Run `git diff --check` or `git diff --cached --check` as appropriate.
4. Search changed files for obvious secret markers such as `api_key`, `token`,
   `secret`, `password`, `cookie`, `BEGIN PRIVATE KEY`, and `Authorization`.
5. Run the relevant tests/checks for the touched code.
6. Confirm no generated private capture artifacts are staged.
7. Run the review/refactor agent for feature or behavior changes.

## Push-Time Checklist

Before pushing a completed feature:

1. Re-run relevant tests after review fixes.
2. Re-check `git status --short --branch`.
3. Inspect the final staged or committed diff.
4. Confirm the latest commit contains one coherent change.
5. Confirm no sensitive files or private artifacts are included.
6. Push only after the feature slice is complete and verified.

If push fails because of auth or remote state, report that plainly and do not
rewrite history unless the user explicitly asks.

## Extra Rules For Screen Capture Work

Screen capture work is high risk. Before committing capture-related code, verify:

- Full monitor frames are never emitted to the frontend.
- Cropped frames are not written to disk.
- Debug capture files are disabled or impossible in normal builds.
- Hidden, paused, blanked, closed, and deleted tiles cannot keep capturing.
- Permission-denied paths are recoverable.
- Tests use fake capture unless manual OS capture is explicitly required.
