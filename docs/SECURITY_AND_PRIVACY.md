# Security And Privacy

ScreenPebble handles screen pixels. That makes trust the central product
requirement.

## User Trust Model

Users should never wonder whether ScreenPebble is secretly watching more than
they selected.

The app must make capture visible:

- Every active region has a visible tile or visible status.
- Every tile shows whether it is live, paused, hidden, blanked, or AI-enabled.
- Privacy blank is always reachable.
- AI features are off by default and visible when enabled.

## Local-First Default

Default behavior:

- Capture selected regions locally.
- Run diff locally.
- Run OCR locally when OCR exists.
- Notify locally.
- Store config locally.

No captured content leaves the machine by default.

## AI Handoff Policy

AI handoff must be explicit in configuration, narrow in scope, and cheap by
design.

Allowed future behavior:

- Per-region AI enablement.
- Text-first handoff from local OCR.
- Image handoff only for explicitly allowed regions.
- Cooldown and deduplication before AI calls.
- Visible indicator during AI handoff.

Disallowed behavior:

- AI watching the whole screen.
- Continuous image streaming to AI.
- Hidden background AI calls.
- Browser cookie scraping.
- ChatGPT web automation using a user's logged-in session.
- API key theft, token reuse, or reading unrelated app credentials.

## Low-Usage Monitoring Design

Most monitoring should happen without AI:

```text
small crop -> local diff -> local OCR if changed -> dedupe -> optional AI text handoff
```

AI should receive compact text or a small image only when local checks show a
meaningful change and the user has enabled that region.

## Permission Rules

Request the smallest permissions needed for the current phase.

Do not add:

- Shell execution permissions.
- Broad filesystem permissions.
- Network permissions for core screen monitoring.
- Camera or microphone permissions.
- Browser history or URL permissions.
- Clipboard monitoring.

Any permission addition requires a decision note explaining why it is necessary
and how it is bounded.

## Continuous Security Review

Every change must be reviewed for security-sensitive drift, even when the task is
not explicitly a security task.

Check for:

- New permissions.
- New persistence.
- New network paths.
- New shell or filesystem access.
- Capture continuing in inactive states.
- Full-monitor frames crossing process or UI boundaries.
- AI handoff becoming enabled by default.
- Logs, errors, tests, fixtures, or examples containing private screen content,
  OCR output, secrets, tokens, cookies, or local account data.

If a change introduces any of those surfaces, add tests, narrow the permission,
or document the decision before committing.

## Release Blockers

Do not release if any of these are true:

- Frames are written to disk by default.
- Hidden, paused, or blanked regions keep capturing.
- Full monitor frames are sent to the UI or AI connector.
- AI handoff is enabled by default.
- Telemetry or analytics exist.
- Permission denied crashes the app.
- The user cannot see or stop active capture.
