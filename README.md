# Pebble

> A free, open-source AI watch for anything visible on your Mac.
> Point at a region, say what matters, and let Pebble tell you when it happens.

[![Status](https://img.shields.io/badge/status-pre--alpha-6b7280)](#status)
[![Price](https://img.shields.io/badge/price-free%20forever-15803d)](#free-and-open)
[![Privacy](https://img.shields.io/badge/privacy-local--first-0f766e)](#privacy)
[![AI](https://img.shields.io/badge/AI-explicit%20requests%20only-4338ca)](#ask-ai)
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
- Intent Watch: text in the AI composer becomes the condition Watch evaluates;
  an empty composer uses a general meaningful-change intent.
- Production Apple Vision OCR runs only after a stable material-change candidate
  and remains ephemeral in memory.
- Changed before/after crops are sent only to the provider selected when Watch
  is enabled; unchanged frames never trigger AI.
- Collapsible Updates feed whose semantic Watch summaries are appended to one
  local Markdown journal under Downloads after Watch is explicitly enabled.
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
| Watch is opt-in | Startup explains the scope; every new region starts with Watch off. |
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

Semantic Watch summaries, model names, and generation times are appended to
`Downloads/Pebble/pebble-updates.md`. Captured pixels, OCR text, manual AI
questions and manual AI answers are never written to that journal.

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

**Watch** uses a local change gate before bounded background AI. On every app
launch, a native notification discloses that:

- Only the selected region is compared on the Mac.
- Watch cannot follow URLs, browser sessions, other windows, or the full screen.
- A material change sends only the previous and current selected-region crops
  to the chosen provider.
- Apple Vision OCR reads text locally only after a stable material change. OCR
  output is never written to disk.
- AI runs no more often than the selected 1, 5, 30, or 60 minute interval. There
  is no fixed session count cap.
- Pause, Hide, privacy blank, close, and reselection stop monitoring; a newly
  selected region requires Watch to be enabled again.

Watch freezes the selected provider, model, and current composer text when it
starts. The model returns a typed match decision, compact summary, and
low/medium/high confidence. Pebble notifies only matched changes. Tools, MCP,
shell, files, and web search remain disabled. Watch uses the same explicit
Claude API-key or subscription path shown in the UI.

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
6. Use **Live**, **Pause**, **Select Region**, **AI**, and preview visibility.
7. Toggle **AI**, type what matters, choose a model and interval, then press
   **Watch**. Stable candidate changes may send one before/after pair to the
   selected provider; unchanged frames stay local.
8. Choose a provider, type a question, and press **Send** when one-shot analysis is
   wanted. This sends one fresh crop only for that request.

Pebble captures only the selected crop and does not save frame history. Live
monitoring stays local; only a visible **Send** action sends one fresh crop.

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
