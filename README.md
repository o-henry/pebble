# ScreenPebble

> Pin a tiny part of your screen. Let local watchers notice what changed.

[![Status](https://img.shields.io/badge/status-pre--alpha-6b7280)](#status)
[![Privacy](https://img.shields.io/badge/privacy-local--first-0f766e)](#privacy)
[![AI](https://img.shields.io/badge/AI-explicit%20requests%20only-4338ca)](#ask-chatgpt)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

ScreenPebble is a local-first desktop utility for the tiny parts of your screen
you keep checking: build logs, queues, upload progress, render jobs, dashboards,
timers, status rows, and other small visual states.

The product idea is intentionally small:

```text
select a region -> keep it visible -> optionally ask ChatGPT about that crop
```

![ScreenPebble demo](docs/assets/screenpebble-demo.gif)

## Why

Some work is not blocked by complexity. It is blocked by waiting.

ScreenPebble is for the status surfaces that do not have good webhooks, APIs, or
notifications. If you can see a small region, the app should help you keep an
eye on it without becoming a screen recorder, remote desktop app, or hidden
monitoring tool.

## Status

ScreenPebble is pre-alpha and not packaged for end users yet. The current macOS
build has a complete local region-to-floating-tile workflow for contributors
and early testers.

Implemented:

- Tauri 2 + React + TypeScript + Rust desktop scaffold.
- Hard performance limits: 1 FPS default, 5 FPS max, 3 active tiles, 800x600
  max region.
- One-drag region selection that opens the floating tile automatically.
- Always-on-top live tile with pause, resume, refresh, close, and privacy blank.
- Real macOS selected-region capture at runtime and a deterministic fake backend
  for tests.
- Capture lifecycle and scheduler states: live, paused, hidden, blanked,
  closed, deleted.
- Local visual diff engine with cooldown and one small in-memory sample per
  tile.
- Privacy blank hotkey/state that stops capture.
- Low-FPS live tile path connected to the selected physical screen region.
- Config-only store for named regions and safe capture settings.
- Optional local OCR service boundary, disabled by default.
- API-key-free ChatGPT account connection through the bundled official Codex
  app-server.
- Explicit selected-region image questions using a compact image model at low
  reasoning effort.

Not shipped yet:

- Signed installer or Homebrew formula.
- Production local OCR adapter.
- Telemetry, cloud sync, browser automation, or ChatGPT session automation.

## Principles

| Principle | Behavior |
| --- | --- |
| Selected regions only | ScreenPebble works on user-pinned regions, not the whole screen. |
| Visible by design | Active capture must have a visible tile or visible status. |
| Low FPS on purpose | Default refresh is 1 FPS; first public target caps at 5 FPS. |
| No frame history | Frames are not stored as a timeline, replay, or preview archive. |
| Local first | Diff runs locally now; future OCR and AI handoff must stay behind local gates. |
| AI is explicit | One selected crop is sent only after the user presses **Ask**. |
| Instant privacy | Privacy blank stops capture loops, not just the UI. |

## Privacy

ScreenPebble should be safe to explain in one sentence:

> It watches only the small regions you pin, locally, with no frame history and
> no upload unless you explicitly ask ChatGPT about the selected crop.

Never persisted:

- Captured frames.
- Screenshots or previews.
- OCR history.
- AI prompts derived from screen content.
- Browser URLs, cookies, tokens, API keys, or clipboard contents.

Persisted configuration is limited to safe settings such as named regions,
coordinates, and refresh configuration. See
[Security And Privacy](docs/SECURITY_AND_PRIVACY.md).

## Ask ChatGPT

ChatGPT is outside the monitoring loop. ScreenPebble makes no automatic AI
requests.

After selecting a region:

1. Press **Connect ChatGPT** and complete the official OpenAI sign-in once.
2. Enter a question and press **Ask**.
3. ScreenPebble captures the backend-authorized crop once, encodes it in memory,
   and sends that single image with the question.
4. The ephemeral answer is shown in the main window and is not persisted.

No OpenAI API key is requested. The app bundles the official
[Codex app-server](https://learn.chatgpt.com/docs/app-server), keeps its account
storage isolated from other apps, and stores credentials in the OS keychain.
It selects an image-capable `mini` model with low reasoning effort; if the
signed-in subscription does not offer a compatible compact model, the request
fails instead of silently selecting a larger model.

ScreenPebble does not read browser cookies, automate the ChatGPT website, reuse
another app's tokens, use MCP, or stream screen images continuously.

## Use

1. Launch ScreenPebble and select **Select a region**.
2. Approve the macOS Screen Recording prompt. ScreenPebble cannot capture before
   macOS grants this permission.
3. Drag over the small status or output area you want to keep visible.
4. Release the pointer. The always-on-top Pebble opens and starts at 1 FPS.
5. Use **Pause**, **Live**, **Hide preview**, or **Close** as needed. Closing the
   floating window keeps the region selected so it can be reopened from the main
   window.
6. To ask about the current crop, connect ChatGPT, type a question, and press
   **Ask**. This sends one fresh crop only for that request.

ScreenPebble captures only the selected crop. It does not save frame history or
send captured pixels over the network.

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
src-tauri/target/release/screenpebble
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

- `PerformanceLimits`: FPS, tile count, and region size contract.
- `CaptureBackend`: bounded selected-region capture with a test-only fake.
- `CaptureLifecycle`: capture state policy.
- `CaptureScheduler`: task/buffer ownership.
- `DiffEngine`: local visual change scoring.
- `PebbleStore`: config-only persistence.
- `OcrEngine`: optional local OCR boundary.
- `AiRuntime`: isolated ChatGPT auth, compact-model selection, one-shot image
  questions, and response limits.

## Contributing

ScreenPebble is still earning trust before expanding features. Good
contributions are narrow, tested, and privacy-preserving:

- Safer capture lifecycle behavior.
- Better region selection and multi-monitor handling.
- Lower resource usage.
- Clearer permission-denied flows.
- Better local diff/OCR quality.
- Better setup, packaging, and demo docs.

Avoid broad feature proposals that add cloud sync, hidden monitoring, telemetry,
browser automation, or always-on AI.

Read first:

- [AGENTS.md](AGENTS.md)
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Engineering Charter](docs/ENGINEERING_CHARTER.md)
- [Git And Security Policy](docs/GIT_AND_SECURITY_POLICY.md)
- [AI Handoff Design](docs/AI_HANDOFF_DESIGN.md)

## License

MIT. See [LICENSE](LICENSE).
