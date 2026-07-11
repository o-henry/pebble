# Security And Privacy

Pebble handles screen pixels. That makes trust the central product
requirement.

## User Trust Model

Users should never wonder whether Pebble is secretly watching more than
they selected.

The app must make capture visible:

- Every active region has a visible tile or visible status.
- Every tile shows whether it is live, paused, hidden, or blanked.
- Privacy blank is always reachable.
- The first **Watch** activation requires a visible scope notice and consent.
- Watch starts off for every newly selected region and remains local-only.
- AI runs only after a visible **Ask** action in the expanded Pebble drawer.

## Local-First Default

Default behavior:

- Capture selected regions locally.
- Run diff locally.
- Run OCR locally when OCR exists.
- Notify locally.
- Store config locally.

Current desktop safeguards:

- macOS owns the Screen Recording consent prompt; selection does not bypass it.
- The backend validates logical-to-physical selection bounds again before
  opening a live tile.
- Live capture commands and cropped-frame events are accepted only by the
  visible Pebble window, never the selector window.
- Webviews may listen for backend events but cannot emit authoritative session
  or frame events.
- The selected display identity, bounds, and scale are checked again immediately
  before capture and before a frame is delivered.
- Every captured frame is matched to the current session revision; frames that
  finish after privacy blank, close, or reselection are discarded.
- Hidden or minimized Pebble windows cannot request or receive live frames.
- The floating tile is positioned outside the selected source region when the
  display has room, preventing recursive self-capture.
- Tile content is capture-protected so Pebble does not ingest its own
  preview if a user later moves the tile over the source.
- Native close and stop both clear the scheduler task and latest in-memory
  frame.
- Adaptive colors are sampled locally from the existing selected crop, are not
  persisted, and reset as soon as the window is hidden or privacy blank is on.

No captured content leaves the machine during Watch monitoring. One selected
crop leaves the machine only after the user explicitly asks AI about it.

## AI Handoff Policy

AI access is explicit, narrow in scope, and cheap by design:

- No API key is requested or accepted by the UI.
- The bundled Codex app-server owns the OpenAI account flow.
- Claude is optional and uses only an installed official Claude CLI at a fixed,
  validated executable path; Pebble does not bundle or download it.
- Pebble uses a private `CODEX_HOME` under its 0700 app data directory;
  another Codex installation's login is not read.
- Credentials use the OS keychain. Browser cookies are never read.
- The legacy macOS bundle identifier remains unchanged so upgrades retain existing
  Screen Recording and keychain access; it is not user-facing branding.
- AI processes receive a cleared environment, so inherited API-key and proxy
  variables are not available to them.
- Each request uses one backend-authorized selected crop encoded as an in-memory
  PNG data URL. No image temp file is created.
- The selected region, session revision, display bounds, and display scale are
  checked before capture and again immediately before upload.
- OpenAI threads are ephemeral, sandboxed read-only, use approval policy `never`, and
  have web search, MCP servers, and analytics disabled.
- Claude runs in print mode with safe mode, slash commands disabled, strict
  empty MCP configuration, all tools denied, and one turn maximum.
- Pebble prefers `gpt-5.6-terra`, permits only `gpt-5.6-luna` as an OpenAI
  fallback, and uses Claude Sonnet 5. All run at medium effort; mini and Haiku
  are rejected as automatic fallbacks.
- Unexpected tool, shell, file, web, plugin, or MCP activity aborts the response.
- Questions are limited to 1,000 characters and answers to 4,000 characters.

Disallowed behavior:

- AI watching the whole screen.
- Continuous image streaming to AI.
- Automatic or hidden background AI calls.
- Browser cookie scraping.
- AI website automation using a user's logged-in browser session.
- API key theft, token reuse, or reading unrelated app credentials.

## Local Watch Design

All monitoring happens without AI:

```text
small crop -> local capture/diff -> broad local visual signal -> local notification
```

Watch is off by default. Its first activation requires a versioned local
consent receipt, and every new region requires a fresh opt-in. It is limited by
the diff engine's five-minute material-change cooldown and a maximum of 24
notifications per app session. It keeps only small in-memory statistics and never
stores frames, OCR, or notification content.

The current local classifier can report broad brightness and color-distribution
changes. It does not claim semantic understanding, text recognition, or
domain-specific prediction. The only network image path remains a fresh
selected crop sent after the user presses **Ask**.

## Permission Rules

Request the smallest permissions needed for the current phase.

Webviews do not have:

- Shell execution or opener permissions.
- Broad filesystem permissions.
- Network permissions.
- Camera or microphone permissions.
- Browser history or URL permissions.
- Clipboard monitoring.

Rust alone starts the fixed bundled OpenAI sidecar or a validated installed
Claude executable. It opens only an exact-host OpenAI sign-in URL or the fixed
official Claude installation page. The webview cannot launch arbitrary
commands or URLs. On macOS, AI processes receive the real `HOME` path only so
the system can locate the default login keychain; Pebble clears all other
inherited variables and keeps provider runtime state in private 0700 app-data
directories. Any permission addition requires a decision note explaining why
it is necessary and how it is bounded.

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
- AI calls becoming automatic or detached from the visible **Ask** action.
- Watch bypassing its consent version, per-region opt-in, or local-only boundary.
- Logs, errors, tests, fixtures, or examples containing private screen content,
  OCR output, secrets, tokens, cookies, or local account data.

If a change introduces any of those surfaces, add tests, narrow the permission,
or document the decision before committing.

## Release Blockers

Do not release if any of these are true:

- Frames are written to disk by default.
- Hidden, paused, or blanked regions keep capturing.
- Full monitor frames are sent to the UI or AI connector.
- AI sends data without a visible user request.
- Watch can be enabled without its visible scope notice.
- Telemetry or analytics exist.
- Permission denied crashes the app.
- The user cannot see or stop active capture.
