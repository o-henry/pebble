# Pebble

> Select anywhere on your desktop. Keep it visible. Get local change alerts.
> Ask AI only when you choose.

[![Status](https://img.shields.io/badge/status-pre--alpha-6b7280)](#status)
[![Privacy](https://img.shields.io/badge/privacy-local--first-0f766e)](#privacy)
[![AI](https://img.shields.io/badge/AI-explicit%20requests%20only-4338ca)](#ask-ai)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Pebble is a local-first desktop utility for the tiny parts of your screen you
keep checking: build logs, queues, upload progress, render jobs, dashboards,
timers, status rows, and other small visual states.

**Pebble is not a browser extension.** It works at the desktop layer, so the
region can come from a browser, terminal, IDE, native app, game, simulator,
remote desktop, or any other visible macOS surface. Browser AI helpers stop at
web content; Pebble starts with whatever the user can actually see on screen.

The product idea is intentionally small:

```text
select any visible desktop region -> keep it visible -> local alerts -> optionally ask AI
```

![Pebble demo](docs/assets/pebble-demo.gif)

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
- Capture lifecycle and scheduler states: live, paused, hidden, blanked,
  closed, deleted.
- Local visual diff engine with cooldown and one small in-memory sample per
  tile.
- Explicit **Watch** mode with local visual classification, a five-minute
  change cooldown, and at most 24 native alerts per app session.
- Local-only material-change alerts through the menu bar and native
  notifications; Watch never uploads frames.
- Collapsible Updates feed whose generic Watch summaries are appended to one
  local Markdown journal under Downloads after Watch is explicitly enabled.
- Privacy blank hotkey/state that stops capture.
- Low-FPS live tile path connected to the selected physical screen region.
- Config-only store for named regions and safe capture settings.
- Optional local OCR service boundary, disabled by default.
- API-key-free OpenAI account connection through the bundled Codex app-server.
- Optional Claude Pro/Max account connection through an installed official
  Claude CLI, without bundling another large runtime.
- Explicit selected-region image questions using balanced image models at
  medium reasoning effort, with model and generation-time metadata.

Not shipped yet:

- Signed installer or Homebrew formula.
- Production local OCR adapter.
- Telemetry, cloud sync, browser automation, or website session automation.

## Principles

| Principle | Behavior |
| --- | --- |
| Desktop-wide, region-scoped | Pebble can select any visible app, but captures only the region the user pins. |
| Visible by design | Active capture must have a visible tile or visible status. |
| Low FPS on purpose | Default refresh is 1 FPS; first public target caps at 5 FPS. |
| No frame history | Frames are not stored as a timeline, replay, or preview archive. |
| Local first | Diff runs locally now; future OCR and AI handoff must stay behind local gates. |
| Watch is opt-in | Startup explains the scope; every new region starts with Watch off. |
| AI is explicit | One selected crop is sent only after the user presses **Send**. |
| Instant privacy | Privacy blank stops capture loops, not just the UI. |

## Privacy

Pebble should be safe to explain in one sentence:

> It watches only the small regions you pin, locally, with no frame history and
> no upload unless you explicitly ask AI about the selected crop.

Never persisted:

- Captured frames.
- Screenshots or previews.
- OCR history.
- AI prompts derived from screen content.
- Browser URLs, cookies, tokens, API keys, or clipboard contents.

Generic Watch alert summaries are an explicit exception: after the versioned
Watch is explicitly enabled, they are appended to
`Downloads/Pebble/pebble-updates.md`. Captured pixels, OCR text, manual AI
questions, and AI answers are never written to that journal. Public Source
entries contain one title and link.

Persisted configuration is limited to safe settings such as named regions,
coordinates, and refresh configuration. See
[Security And Privacy](docs/SECURITY_AND_PRIVACY.md).

## Ask AI

OpenAI and Claude are outside the monitoring loop. Pebble makes no automatic
AI requests.

After selecting a region:

1. Toggle the **AI** button in the Pebble toolbar.
2. Choose **OpenAI** or **Claude**.
3. Connect the provider and complete its official account sign-in once.
4. Enter a question and press **Send**.
5. Pebble captures the backend-authorized crop once, encodes it in memory,
   and sends that single image with the question.
6. The ephemeral answer, model, and generation time are shown inside the same
   Pebble and are not persisted.

No API key is requested. OpenAI uses Pebble's isolated bundled Codex app-server
and the OS keychain. Claude uses the separately installed official
[Claude CLI](https://code.claude.com/docs/en/quickstart) and its Pro/Max account
sign-in. Pebble prefers `gpt-5.6-terra` at medium effort, permits
`gpt-5.6-luna` only as its OpenAI fallback, and uses Claude Sonnet 5 at medium
effort. It never falls back to mini or Haiku automatically.

Pebble does not read browser cookies, automate an AI website, reuse another
app's tokens, use MCP, or stream screen images continuously.

## Smart Watch

**Watch** is a local notification layer, not background cloud AI. On every app
launch, a native notification explains how to start it and that:

- Only the selected region is compared on the Mac.
- No frame is automatically sent to OpenAI or Claude.
- Notifications are capped at 24 per app session and material changes have a
  five-minute cooldown.
- Pause, Hide, privacy blank, close, and reselection stop monitoring; a newly
  selected region requires Watch to be enabled again.

Watch classifies broad visible changes such as marked brightness shifts or a
large increase in red, amber, or green content. It does not claim to understand
text or predict domain-specific outcomes. Production local OCR remains future
work.

## Adaptive Background

Pebble matches the color directly behind its own window, not the selected
region. While the window is visible, macOS samples a 96-pixel square beneath
the center of Pebble every 1.5 seconds. Rust reduces that temporary sample to
three quantized RGB values before returning anything to the webview. The sample
is never persisted, included in Updates, or sent to AI.

## Public Source Watch

The expanded Updates area can follow one user-entered public HTTPS RSS, Atom,
JSON, or web URL for the current app session. Pebble checks it every 15 minutes
and appends only the latest public title and source link when it changes.

Pebble never derives a search query from captured screen content. Source Watch
does not use cookies, browser sessions, credentials, proxies, redirects, custom
ports, local hosts, private IP ranges, or responses larger than 512 KB. Article
bodies are neither displayed nor saved.

## Use

1. Launch Pebble and click its macOS menu bar item to open the compact window.
2. Press **Select Region**.
3. Approve the macOS Screen Recording prompt. Pebble cannot capture before
   macOS grants this permission.
4. Drag over a small region in any visible browser or native desktop app.
5. Release the pointer. The always-on-top Pebble opens and starts at 1 FPS.
6. Use **Live**, **Pause**, **Select Region**, **AI**, and preview visibility.
7. Toggle **AI**, then press **Watch** for local change alerts. Watch never
   uploads the region.
8. Choose a provider, type a question, and press **Send** when cloud analysis is
   wanted. This sends one fresh crop only for that request.

Pebble captures only the selected crop and does not save frame history. Live
monitoring stays local; only a visible **Send** action sends one fresh crop.

## Install From Source

Requirements:

- macOS for the current desktop target.
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
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Before a public demo, also run the
[manual smoke checklist](docs/MANUAL_SMOKE_CHECKLIST.md).

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
- `OcrEngine`: optional local OCR boundary.
- `AiRuntime`: isolated AI auth, balanced-model selection, one-shot image
  questions, and response limits.

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
