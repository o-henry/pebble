# Pebble

> A free, open-source AI watch for anything visible on your Mac.
> Point at a region, say what matters, and let Pebble tell you when it happens.

[![Status](https://img.shields.io/badge/status-pre--alpha-6b7280)](#status)
[![Price](https://img.shields.io/badge/price-free%20forever-15803d)](#free-and-open)
[![Privacy](https://img.shields.io/badge/privacy-local--first-0f766e)](#privacy)
[![AI](https://img.shields.io/badge/AI-bounded%20and%20opt--in-4338ca)](#ask-ai)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Pebble is a local-first desktop utility for the tiny parts of your screen you
keep checking: build logs, queues, upload progress, render jobs, dashboards,
timers, status rows, and other small visual states.

## Free And Open

Pebble is free forever, MIT licensed, and has no paid tier, account, telemetry,
or Pebble cloud. The source is public so its capture, storage, and AI boundaries
can be inspected. Connected AI providers may count usage against their own
subscription or API billing; Pebble itself does not charge for that access.

**Pebble is not a browser extension.** It works at the desktop layer, so the
region can come from a browser, terminal, IDE, native app, game, simulator,
remote desktop, or any other visible macOS surface. Browser AI helpers stop at
web content; Pebble starts with whatever the user can actually see on screen.

The product idea is intentionally small:

```text
select any visible desktop region -> keep it visible -> local alerts -> optionally ask AI
```

## Why

Some work is not blocked by complexity. It is blocked by waiting.

Pebble is for the status surfaces that do not have good webhooks, APIs, or
notifications. If you can see a small region, the app should help you keep an
eye on it without becoming a screen recorder, remote desktop app, or hidden
monitoring tool.

That desktop-wide reach is the core distinction. Pebble does not depend on a
page DOM, browser tab, website integration, or extension permission. It treats
a user-selected visual region consistently across web and non-web software.

## Status

Pebble is pre-alpha and not packaged for end users yet. The current macOS
build has a complete local region-to-floating-tile workflow for contributors
and early testers.

Implemented:

- Tauri 2 + React + TypeScript + Rust desktop scaffold.
- Hard performance limits: 1 FPS default, 5 FPS max, and 3 active tiles.
- Any non-empty region inside the selected display can be captured.
- Native macOS menu bar control with no persistent management window.
- One-drag region selection that opens the floating tile automatically.
- Always-on-top live tile with pause, resume, reselect, AI, and privacy blank.
- Real macOS selected-region capture at runtime and a deterministic fake backend
  for tests.
- Window-backed regions stay attached to their source window when another app
  covers it or the source moves behind other windows; desktop-only selections
  remain display-coordinate captures.
- Capture lifecycle and scheduler states: live, paused, hidden, blanked,
  closed, deleted.
- Local visual diff engine with cooldown and one small in-memory sample per
  tile.
- Explicit **Watch** mode with local prefiltering and a user-selected maximum AI
  cadence of 1, 5, 30, or 60 minutes. There is no fixed session analysis cap.
- Intent Watch: text in the AI composer becomes the condition Watch evaluates.
  An empty composer enables automatic local error, progress, queue, stuck, and
  visual-loop detectors, with optional AI only for other meaningful changes.
- Deterministic text appearance, disappearance, text-change, single-number
  threshold, progress, and state-word rules run locally without an AI
  connection or provider tokens.
- Automatic **No Progress** detection notices when a region that was visibly active becomes stuck
  for 1, 5, 30, or 60 minutes. A static starting screen and one-poll noise do
  not alert; the rule uses local visual samples with no OCR, AI, or network use.
- **Cross Check** compares two or three explicitly enrolled regions across
  browser and native apps. It alerts only when positive states such as success
  or healthy and negative states such as error, failed, or offline remain
  opposed for 10 seconds, using local OCR and no AI.
- **Follow Through** links a trigger region to one or two result regions. A
  stable trigger change starts the selected 1, 5, 30, or 60 minute deadline;
  Pebble alerts only for result regions that do not change in time. It uses
  local visual state with no OCR, AI, network request, or input control.
- Automatic **Loop Detector** finds 2- to 4-step visual cycles repeated three times, such
  as retry, refresh, or redirect loops. It compares at most twelve compact
  64-byte color fingerprints in memory and never retains frames, runs OCR, or
  calls AI.
- Stable-candidate gating ignores transient animation, while semantic
  fingerprints suppress repeated alerts within the selected interval.
- Up to three independently bound Watch regions can stay active. Selecting a
  new region does not retarget an existing Watch, and every region can be
  stopped separately.
- The built-in multi-region recipes are limited to **Cross Check** and the
  **Follow Start/Result** roles that cannot be inferred safely. Saved custom
  recipes store only a name, intent, and recommended interval. They never store
  pixels, coordinates, OCR output, or credentials. **Clear** removes every
  saved custom recipe from the Mac.
- Production Apple Vision OCR runs only after a stable material-change candidate,
  except for the one baseline read explicitly required by each Cross Check
  region. OCR remains ephemeral in memory.
- Changed before/after crops are sent only to the provider selected when Watch
  is enabled; unchanged frames never trigger AI.
- Collapsible Updates feed that opens automatically for a meaningful alert and
  retains structured region, signal, engine or model,
  confidence, and duration metadata. Safe Watch lifecycle markers and redacted
  AI result markers are appended to one local Markdown journal under Downloads.
- **Change Story** groups two to eight meaningful signals separated by no more
  than five minutes into one oldest-to-newest timeline. Waiting and skipped
  analysis messages remain separate, and the original journal stays unchanged.
- Privacy blank hotkey/state that stops capture.
- Low-FPS live tile path connected to the selected physical screen region.
- Config-only store for named regions and safe capture settings.
- API-key-free OpenAI account connection through the bundled Codex app-server.
- Claude access through either an installed official Claude CLI subscription or
  an optional Anthropic API key stored only in macOS Keychain, without bundling
  another large runtime.
- User-selectable image models: OpenAI Sol, Terra, or Luna when available to the
  connected account; Claude Sonnet or Opus through the active access path.
- Explicit selected-region image questions at medium reasoning effort, with
  model and generation-time metadata.

Not shipped yet:

- Signed installer or Homebrew formula.
- Telemetry, cloud sync, browser automation, or website session automation.

## Principles

| Principle | Behavior |
| --- | --- |
| Desktop-wide, region-scoped | Pebble can select any visible app, but captures only the region the user pins. |
| Visible by design | Active capture must have a visible tile or visible status. |
| Low FPS on purpose | Default refresh is 1 FPS; first public target caps at 5 FPS. |
| No frame history | Frames are not stored as a timeline, replay, or preview archive. |
| Local first | Visual filtering and Apple Vision OCR run locally; AI handoff stays behind those gates. |
| Watch is opt-in | Every new region starts with Watch off and the AI panel keeps its active scope and Stop control visible. |
| AI is bounded | Manual AI uses **Send**; Watch AI requires opt-in and only runs after a local material-change gate. |
| Instant privacy | Privacy blank stops capture loops, not just the UI. |

## Privacy

Pebble should be safe to explain in one sentence:

> It watches only the small region you pin, filters changes locally, and sends
> before/after crops only after you explicitly enable Watch and a stable change
> reaches the selected analysis interval.

Never persisted:

- Captured frames.
- Screenshots or previews.
- OCR history.
- AI prompts derived from screen content.
- Browser URLs, cookies, subscription tokens, clipboard contents, or API keys in
  Pebble files, logs, or configuration.

An optional Anthropic API key is persisted only as a macOS Keychain generic
password. Pebble never returns the saved key to the webview or writes it to a
Pebble-managed file.

Safe Watch event markers and structured metadata such as region label, signal
type, engine or model name, confidence, and generation time are appended to
`Downloads/Pebble/pebble-updates.md`. Detailed AI-generated Watch summaries can
appear in Pebble and macOS notifications, but their screen-derived values are
omitted from the journal. macOS controls banner duration, so meaningful alerts
also expand the persistent Updates feed and keep the menu-bar attention marker
active until Pebble is opened. Captured pixels, capture coordinates, window IDs,
OCR text, manual AI questions, and manual AI answers are never written there.

Custom Watch recipes are written only after **Save** is pressed. Their
instruction text remains in the app's local browser storage until the recipe is
removed or **Clear** is pressed.

Persisted configuration is limited to safe settings such as named regions,
coordinates, and refresh configuration. See
[Security And Privacy](docs/SECURITY_AND_PRIVACY.md).

## Ask AI

Manual questions are separate from Watch. They run only after **Send**.

After selecting a region:

1. Toggle the **AI** button in the Pebble toolbar.
2. Choose **OpenAI** or **Claude**.
3. Choose an available model, then connect the provider. Claude uses its CLI subscription by default, or you can
   add an Anthropic API key from the Claude access row.
4. Enter a question and press **Send**.
5. Pebble captures the backend-authorized crop once, encodes it in memory,
   and sends that single image with the question.
6. The ephemeral answer, model, and generation time are shown inside the same
   Pebble and are not persisted.

OpenAI never accepts an API key: it uses Pebble's isolated bundled Codex
app-server and official account sign-in. Without a saved Anthropic API key,
Claude uses the separately installed official
[Claude CLI](https://code.claude.com/docs/en/quickstart) and its Pro/Max account
sign-in. Adding a key switches Claude to the direct Anthropic Messages API and
the UI labels the path **API Billing**; API usage is billed separately by
Anthropic. An invalid saved key fails visibly instead of silently switching to
the subscription. Removing it returns Claude to subscription mode.

Pebble defaults to `gpt-5.6-terra` and Claude Sonnet at medium effort. Users can
choose Sol, Terra, or Luna when the connected OpenAI account exposes them, and
Sonnet or Opus when the active Claude access path exposes them. Pebble never
falls back to mini or Haiku automatically.

Pebble does not read browser cookies, automate an AI website, reuse another
app's tokens, use MCP, or stream screen images continuously.

## Smart Watch

**Watch** checks only regions the user explicitly selected and enabled. Up to
three source-window-bound regions can run independently. Selecting a different
region, covering the source window, or hiding or closing the Pebble window does
not silently retarget or stop an existing Watch. The AI panel lists active
regions; each row has its own **Stop** action. The menu-bar indicator marks new
matched events rather than every local check.

Every five seconds, Rust performs a local visual check. With an empty composer,
  one press of **Watch** automatically enables common error, progress, queue,
stuck, and loop detectors without requiring recipe selection. A candidate must
settle before Apple Vision OCR runs. Common text, number, progress, and state
rules are resolved locally and can run with no AI connection or token use.
Ambiguous or semantic changes use a connected provider when available; only
then may one previous and current crop pair be sent, no more often than the
selected 1, 5, 30, or 60 minute interval. There is no fixed session count cap.

Automatic **No Progress** detection follows a separate zero-token path. It first
requires repeated visible activity or one confirmed stable change, then starts
the selected interval when the region becomes stable. It sends one **Stuck**
signal if that interval expires, suppresses repeats while the screen remains
unchanged, and rearms only after renewed activity. This path does not run OCR,
AI, browser access, tools, or network requests.

The built-in **Cross Check** recipe is enrolled separately on each region that
should participate. Pebble runs local Apple Vision OCR once on each enrolled
baseline and after each stable change, immediately reduces the text to a
positive, negative, or unknown state, and discards the text. A conflict requires
at least two Cross Check regions, opposite positive and negative states, and a
fixed 10-second confirmation. General warnings and in-progress states do not
trigger it. The signal names all participating region labels but stores no OCR
text, pixels, coordinates, or source-window IDs and never calls AI.

**Follow Through** is configured with the built-in **Follow Start** recipe on
the source region and **Follow Result** on each expected destination. A stable
material change in Follow Start opens a memory-only deadline. Each Follow Result
that changes before the deadline is removed from the pending set; if every
result responds, Pebble stays quiet. At expiry, one signal names only the
results still missing. A new trigger change rearms the check, while capture
failure, target removal, privacy blank, or app shutdown cancels pending state.
This path retains target IDs and ticks only and never runs OCR or AI.

Automatic **Loop Detector** seeds one compact fingerprint from the selected baseline and
adds another only after a stable material change. Each fingerprint contains an
8-by-8 grid of heavily quantized average RGB and local contrast values. Pebble keeps at most twelve
in memory, detects distinct 2- to 4-step patterns only after three complete
cycles, and emits one signal while that cycle continues. A nonmatching state
breaks the cycle and rearms detection. Fingerprints are never serialized,
persisted, included in Updates, sent to AI, or exposed to the webview.

**Change Story** makes bursts of Watch results readable without collecting more
screen data. The frontend groups only already-sanitized Match, Stuck, Conflict,
No Follow-Through, and Loop entries when adjacent events are no more than five
minutes apart. Each story is capped at eight events, preserves event metadata
and summaries, and displays them oldest first. Waiting or skipped-analysis
entries end a story. Grouping is derived in memory and never rewrites the local
Markdown journal.

Watch freezes the provider, model, intent, interval, source-window binding, and
AI-fallback choice for each region when it starts. The model returns a typed
match decision, compact summary, and low/medium/high confidence. Pebble notifies
only matched, deduplicated changes and labels the region and engine in its log.
Shell, files, browser and computer control, apps, plugins, MCP, agents, memories,
image generation, and web search remain disabled.

**Live** and **Pause** control the visible preview, not background Watch. A
region keeps watching while Pebble is hidden. Use that region's **Stop**,
privacy blank, **Remove Pebble**, or app quit to end capture. If the bound source
window or display becomes unavailable, Watch fails closed and records a waiting
state instead of capturing whatever happens to occupy the old coordinates.

## Adaptive Background

Pebble matches the color directly behind its own window, not the selected
region. While the window is visible, macOS samples a 96-pixel square beneath
the center of Pebble every 1.5 seconds. Rust reduces that temporary sample to
three quantized RGB values before returning anything to the webview. The sample
is never persisted, included in Updates, or sent to AI.

## Use

1. Launch Pebble and click its macOS menu bar item to open the compact window.
2. Press **Select Region**.
3. Approve the macOS Screen Recording prompt. Pebble cannot capture before
   macOS grants this permission.
4. Drag over a small region in any visible browser or native desktop app.
5. Release the pointer. The always-on-top Pebble opens and starts at 1 FPS.
6. Use the single **Live/Pause** toggle, **Select Region**, **AI**, and preview
   visibility controls.
7. Toggle **AI** and press **Watch** for automatic local error, progress,
   queue, stuck, and loop detection. Type a condition first only when you want a
   narrower rule. Open **Options** only when changing the interval, provider,
   model, a saved condition, or an explicit multi-region role.
8. Repeat selection and Watch activation for up to three independent regions.
   Their status rows remain visible in the AI panel and can be stopped one by
   one. To compare apps, choose **Cross Check** and enable Watch on at least two
   regions; the interval control is disabled because conflict confirmation is a
   fixed 10 seconds. To verify a downstream response, apply **Follow Start** to
   the source, choose its deadline, then apply **Follow Result** to one or two
   destination regions. Repeated retry or refresh loops are detected
   automatically after three complete cycles.
9. Type a question and press **Send** when one-shot analysis is wanted. Change
   the provider or model under **Options** when the default is not appropriate.
   This sends one fresh crop only for that request.

Pebble captures only explicitly selected crops and does not save frame history.
Manual AI sends only after **Send**; Watch AI sends only after per-region opt-in,
a stable local gate, and the selected minimum interval.

## Install

The recommended public download is a Developer ID-signed and Apple-notarized
DMG from GitHub Releases. Pebble publishes no DMG unless both the Apple Silicon
and Intel builds pass signing, notarization, stapling, and Gatekeeper checks.

No signed DMG has been published yet. Until the first verified release appears,
use the source-build instructions below rather than downloading an unofficial
app bundle.

For a verified release:

1. Download `arm64` on an Apple Silicon Mac or `x64` on an Intel Mac.
2. Drag Pebble to Applications and open it.
3. Approve Screen Recording when selecting a region for the first time.

That first approval is required by macOS. Official updates keep the same bundle
identifier and Developer ID, which lets macOS recognize them as the same app and
normally preserves the approval. macOS can still ask again after the user resets
Privacy & Security settings or after a major system security change.

Only DMGs published by the protected GitHub release workflow are official
installers. The workflow refuses ad-hoc signatures, missing notarization,
Gatekeeper failures, architecture builds with different Team IDs, and designated
requirements tied to a changing code hash.

## Install From Source

Requirements:

- macOS 14 or later for source-window capture.
- Node.js compatible with the repository lockfile.
- Rust stable.

```bash
git clone https://github.com/o-henry/pebble.git
cd pebble
npm install
npm run tauri:build
```

The unsigned development binary is built at:

```text
src-tauri/target/release/pebble
```

Source builds are for contributors. Their local signing identity can change
after a rebuild, so macOS may request Screen Recording again. Never redistribute
that binary as an official Pebble release or copy an ad-hoc build over an
installed Pebble. The guarded installer accepts only a Developer ID-signed,
notarized app and checks identity continuity before replacing an installation:

```bash
npm run install:macos -- /path/to/Pebble.app
```

Replacing an older ad-hoc development install with the first official build is
a deliberate one-time migration and requires `--replace-ad-hoc`; macOS will ask
for Screen Recording again after that transition.

For development:

```bash
npm run tauri:dev
```

## Verify

Run the automated checks:

```bash
npm test
npm run typecheck
npm run lint
npm run build
npm run release:check
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Before a public demo, also run the
[manual smoke checklist](docs/MANUAL_SMOKE_CHECKLIST.md). Maintainers should
follow the fail-closed [macOS distribution guide](docs/MACOS_DISTRIBUTION.md)
before creating a version tag.

## Repository Map

```text
src/                     React UI and typed frontend command wrappers
src-tauri/src/           Rust services, Tauri commands, and platform adapters
scripts/                 Reproducible build-time sidecar preparation
docs/                    Product, architecture, security, demo, and release docs
.github/ISSUE_TEMPLATE/  Bug and feature templates
```

Key Rust boundaries:

- `PerformanceLimits`: FPS, tile count, and non-empty region contract.
- `CaptureBackend`: display-bounded selected-region capture with a test-only fake.
- `CaptureLifecycle`: capture state policy.
- `CaptureScheduler`: task/buffer ownership.
- `DiffEngine`: local visual change scoring.
- `PebbleStore`: config-only persistence.
- `OcrEngine`: ephemeral Apple Vision text recognition behind the Watch gate.
- `AiRuntime`: isolated AI auth, account-validated model selection, one-shot
  image questions, intent matching, and response limits.

## Contributing

Pebble is still earning trust before expanding features. Good
contributions are narrow, tested, and privacy-preserving:

- Safer capture lifecycle behavior.
- Better region selection and multi-monitor handling.
- Lower resource usage.
- Clearer permission-denied flows.
- Better local diff/OCR quality.
- Better setup, packaging, and demo docs.

Avoid broad feature proposals that add cloud sync, hidden monitoring, telemetry,
browser automation, or always-on external AI.

Read first:

- [AGENTS.md](AGENTS.md)
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Engineering Charter](docs/ENGINEERING_CHARTER.md)
- [Git And Security Policy](docs/GIT_AND_SECURITY_POLICY.md)
- [AI Handoff Design](docs/AI_HANDOFF_DESIGN.md)

## License

MIT. See [LICENSE](LICENSE).
