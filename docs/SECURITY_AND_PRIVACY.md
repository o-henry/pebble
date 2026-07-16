# Security And Privacy

Pebble handles screen pixels. That makes trust the central product
requirement.

## User Trust Model

Users should never wonder whether Pebble is secretly watching more than
they selected.

The app must make capture visible:

- Every active region has a visible status row when Pebble is open. Startup
  disclosure explains that authorized Watch regions remain active while the
  window is hidden; macOS owns the system capture indicator.
- Every tile shows whether it is live, paused, hidden, or blanked.
- Privacy blank is always reachable.
- App startup shows a native Watch scope notice before region selection.
- Watch starts off for every newly selected region, supports at most three
  active regions, and requires explicit per-region consent.
- The startup notice discloses local monitoring before Watch can be enabled.
- Manual AI runs after **Send**; Watch AI runs only after its local change gate.

## Local-First Default

Default behavior:

- Capture selected regions locally.
- Run diff locally.
- Run Apple Vision OCR locally after a stable Watch candidate. Cross Check
  additionally performs one disclosed baseline read for each region explicitly
  enrolled in that recipe.
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
- Visible preview and manual AI frames are matched to the current session
  revision; late frames are discarded after privacy blank, close, or
  reselection. Watch uses a separate revocable authorization for each bound
  region so reselection cannot retarget an existing Watch.
- On macOS 14 or later, a region selected inside an app window is bound to that
  source window and captured with a desktop-independent ScreenCaptureKit filter.
  Covering the source with another window never changes the capture target.
- The ephemeral source window ID and window-relative crop are kept in memory
  only and are excluded from serialized region data, logs, and journals.
- If a bound source window closes or becomes unavailable, capture fails closed
  instead of falling back to whatever pixels occupy the old screen coordinates.
- Hidden or minimized Pebble windows cannot request or receive live frames.
- The floating tile is positioned outside the selected source region when the
  display has room, preventing recursive self-capture.
- Tile content is capture-protected so Pebble does not ingest its own
  preview if a user later moves the tile over the source.
- Native close clears the visible preview scheduler and hides Pebble. Explicit
  per-region Watch stop, privacy blank, Pebble removal, and app quit revoke
  Watch authorization and drop associated in-memory state.
- Adaptive colors are sampled locally beneath the visible Pebble window, are
  not persisted, and reset as soon as the window is hidden or privacy blank is
  on.

Watch compares crops locally. Unchanged frames never leave the Mac. After the
user explicitly enables semantic Watch, a locally detected material change may
send only the previous and current selected-region crops to the chosen provider.
Manual AI sends one selected crop only after **Send**.

## AI Handoff Policy

AI access is explicit, narrow in scope, and cheap by design:

- OpenAI API keys are not requested or accepted.
- The bundled Codex app-server owns the OpenAI account flow.
- Claude API keys are optional, accepted only from the visible Pebble window,
  and stored only in macOS Keychain. The saved value is never returned to the
  webview or written to a file, log, config, test fixture, or update journal.
- Claude is optional and uses only an installed official Claude CLI at a fixed,
  validated executable path when no API key is configured; Pebble does not
  bundle or download it.
- Pebble uses a private `CODEX_HOME` under its 0700 app data directory;
  another Codex installation's login is not read.
- Credentials use the OS keychain. Browser cookies are never read.
- The legacy macOS bundle identifier remains unchanged so upgrades retain existing
  Screen Recording and keychain access; it is not user-facing branding.
- AI processes receive a cleared environment, so inherited API-key and proxy
  variables are not available to them.
- Each manual request uses one backend-authorized selected crop; each eligible
  Watch analysis uses one before-and-after pair. Both are encoded as in-memory
  PNG data URLs, and no image temp file is created.
- Manual requests recheck the selected region, session revision, display
  bounds, and display scale before capture and upload. Watch validates its
  bound display before capture and checks its revocable per-region
  authorization immediately before and after provider work.
- OpenAI threads are ephemeral, sandboxed read-only, use approval policy `never`, and
  have web search, MCP servers, and analytics disabled.
- Claude runs in print mode with safe mode, slash commands disabled, strict
  empty MCP configuration, all tools denied, and one turn maximum when using
  the subscription path.
- Claude API mode uses fixed `api.anthropic.com` HTTPS endpoints, refuses
  redirects, sets bounded timeouts and response sizes, defines no tools, and
  rejects any tool-use response block.
- A configured key takes precedence; authentication failures are shown and do
  not silently switch to a subscription or another model.
- Pebble lets the user choose an image-capable model reported by the connected
  account: OpenAI Sol, Terra, or Luna, and Claude Sonnet or Opus. Rust validates
  the selected model again before every request; there is no silent model
  fallback. All supported choices run at medium effort.
- Unexpected tool, shell, file, web, plugin, or MCP activity aborts the response.
- Questions are limited to 1,000 characters and answers to 4,000 characters.

Disallowed behavior:

- AI watching the whole screen.
- Continuous image streaming to AI.
- AI calls outside a manual **Send** or an explicitly enabled Watch session.
- Browser cookie scraping.
- AI website automation using a user's logged-in browser session.
- API key exposure, token reuse, or reading unrelated app credentials.

## Semantic Watch Design

Every frame is filtered locally before bounded AI analysis:

```text
selected crop -> local diff gate -> activity/stability timer -> local stuck event
              -> Loop baseline/stable change -> compact in-memory fingerprint
              -> three repeated cycles -> local loop event
              -> Follow Start stable change -> memory-only result deadline
              -> Follow Result stable change or local missed-result event
              -> Cross Check baseline/stable change -> local OCR state enum
              -> 10-second cross-region confirmation -> local conflict event
              -> stable candidate -> Apple Vision OCR -> deterministic rule
              -> local notification or bounded AI fallback -> deduplicated event
```

Watch is off by default. App startup displays its scope before region
selection, pressing Watch records explicit activation, and every new region
requires a fresh opt-in. At most three independent regions may be active. Local
visual checks run every five seconds. Apple Vision OCR runs only after a stable
material-change candidate except for the disclosed one-time baseline read on
each explicitly enrolled Cross Check region. Deterministic text, single-number threshold,
progress, and state rules can notify locally with AI fallback disabled. Semantic
analysis runs only when required, connected, explicitly allowed for that
region, and no more often than the user-selected interval of 1, 5, 30, or 60
minutes. There is no fixed per-session analysis cap. Watch keeps only the
comparison crops and OCR text needed for the current in-memory decision and
drops them on stop or reset; no crop or OCR text is written to disk.

The No Progress rule is local visual state only. It requires repeated activity
or a confirmed stable change before starting the selected 1, 5, 30, or 60
minute stability timer. A static initial region and one-poll transient cannot
alert. One stuck event is emitted per activity cycle and the rule rearms only
after renewed activity. It does not run Apple Vision OCR, AI, tools, browser
access, or network requests.

Cross Check is local OCR state comparison. Only regions explicitly configured
with that recipe participate, and at least two are required. Each participating
region runs ephemeral Apple Vision OCR on its baseline and after a stable
change. The recognized text is immediately reduced to positive, negative, or
unknown and discarded. Only opposing positive and negative states that remain
unchanged for 10 seconds emit one conflict event. Capture failure, target
removal, or an unclassified state clears the pending conflict. Pebble retains
only the enum state and generated region labels; it does not persist or send
OCR text, frames, coordinates, or source-window IDs, and it never calls AI for
Cross Check.

Follow Through is local visual relationship tracking. Only targets explicitly
configured as Follow Start or Follow Result participate. A stable Follow Start
change stores its target ID, the waiting result IDs, and a deadline tick in
memory. Stable Follow Result changes remove their IDs from that pending set. No
frame, visual fingerprint, coordinate, window ID, or OCR text enters the
relationship state. Capture failure, target removal, privacy blank, and app
shutdown cancel the pending check so an unavailable result cannot create a
false alert. Follow Through never calls AI, opens a URL, searches the web, or
controls mouse or keyboard input.

Loop Detector is local visual fingerprint comparison. On the baseline and only
after stable material changes, Pebble reduces the selected crop to an 8-by-8
grid of quantized RGB and local contrast values. At most twelve 64-byte fingerprints exist per
active loop target. Detection requires three complete repetitions of a distinct
2- to 4-step pattern. The detector resets on capture failure or target reset and
suppresses repeated alerts until the pattern breaks. Fingerprints have no
serialization path and are never persisted, returned to the webview, included
in Updates, sent to AI, or used to control input.

After activation, Pebble appends safe Watch lifecycle and result summaries plus
structured region label, signal type, engine or model name, confidence, and
duration metadata to one local Markdown document at
`Downloads/Pebble/pebble-updates.md`. It never writes captured pixels, capture
coordinates, source-window IDs, OCR text, manual AI questions, AI answers,
article bodies, credentials, or browser session data to that journal.
The journal directory is mode 0700, the file is mode 0600, symbolic-link
targets are rejected, and the document stops accepting entries at 25 MB.

Unchanged frames never reach AI. A locally detected material change sends the
previous and current selected-region crops to the provider chosen at Watch
activation. Ephemeral Apple Vision OCR may accompany those crops as untrusted
supporting evidence; screen or OCR instructions are never executed. The model
returns a typed match decision, short summary, and confidence, and unmatched
candidates do not notify or enter the activity journal. OpenAI and Claude run
with tools, MCP, shell, files, and web search disabled. Manual AI still sends
one fresh crop only after **Send**.

Adaptive window color is a separate local-only capture path. On macOS, Pebble
uses the system's below-window capture option to sample a 96-physical-pixel
square beneath the center of its own visible window every 1.5 seconds. Raw
sample pixels remain inside Rust only long enough to calculate a quantized
median RGB color. The webview receives three color channels, not an image. No
sample is taken while the window or document is hidden, and neither pixels nor
RGB history are persisted, journaled, logged, or sent over the network.

Automatic Watch has no URL, RSS, web-search, browser-session, or arbitrary
network-fetch path. Every active target retains the exact source-window binding,
display, crop, scale, provider, model, interval, intent, and AI-fallback choice
that was authorized for that region. Reselection creates a new current region
without changing existing targets. Hiding Pebble or pausing its live preview
does not stop a disclosed Watch target. Per-region stop, privacy blank, Pebble
removal, or app quit revokes it. Display reconfiguration or source-window loss
fails closed without falling back to screen coordinates.

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
- Automatic AI calls bypassing the local material-change or selected-interval gates.
- Watch bypassing its consent version, per-region opt-in, or selected-region boundary.
- Logs, errors, tests, fixtures, or examples containing private screen content,
  OCR output, secrets, tokens, cookies, or local account data.

If a change introduces any of those surfaces, add tests, narrow the permission,
or document the decision before committing.

## Release Blockers

Do not release if any of these are true:

- Frames are written to disk by default.
- Privacy-blanked, removed, stopped, or unauthorized regions keep capturing.
- Hidden background Watch capture is not disclosed at startup and by macOS's
  system capture indicator.
- Full monitor frames are sent to the UI or AI connector.
- AI sends data without a visible user request.
- Watch scope is not disclosed at app startup.
- Telemetry or analytics exist.
- Permission denied crashes the app.
- The user cannot see or stop active capture.
