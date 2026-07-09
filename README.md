# ScreenPebble

> Pin a tiny part of your screen. Let local watchers notice what changed.

[![Status](https://img.shields.io/badge/status-pre--alpha-6b7280)](#status)
[![Privacy](https://img.shields.io/badge/privacy-local--first-0f766e)](#privacy-model)
[![AI](https://img.shields.io/badge/AI-optional%20%26%20off%20by%20default-4338ca)](#ai-without-the-creepiness)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

ScreenPebble is being built to turn a small user-selected screen region into a
low-FPS always-on-top tile. It is for the parts of your screen you keep checking:
build logs, upload progress, render jobs, queue numbers, dashboards, timers,
status rows, and charts.

> Pre-alpha: this repository currently contains the product, security,
> architecture, workflow contracts, and Phase 0 app scaffold. Screen capture is
> not implemented yet.

The goal is simple: stop babysitting tiny UI states without giving an app
permission to watch your whole computer.

```text
select a region -> keep it visible -> detect meaningful changes locally
```

## Why

Some work is not blocked by complexity. It is blocked by waiting.

You keep glancing at the same small area:

- Did the build fail?
- Did the upload finish?
- Did the queue number change?
- Did the dashboard cell turn red?
- Did the chart row update?
- Did the long-running job finally complete?

ScreenPebble is designed for those small status surfaces. The product direction
does not require an API, browser extension, webhook, cloud account, or
app-specific integration. If you can see the region, you should be able to pin
it.

## What ScreenPebble Is

- A desktop utility for selected screen regions.
- A tiny always-on-top live tile.
- A low-FPS ambient monitor.
- A local-first change detector.
- A privacy-aware bridge between visual UI state and optional AI assistance.

## What ScreenPebble Is Not

- Not a screen recorder.
- Not a remote desktop app.
- Not a hidden monitoring tool.
- Not a cloud sync service.
- Not an AI agent framework.
- Not a stock, trading, brokerage, or financial advice app.
- Not a workplace-policy bypass tool.

## Core Principles

| Principle | Product behavior |
| --- | --- |
| User-selected only | ScreenPebble watches only regions the user pins. |
| Visible by design | Active capture must have visible tile or visible status. |
| Low FPS on purpose | Default refresh is 1 FPS; the first public release caps at 5 FPS. |
| No frame history | Captured pixels are not stored as a timeline, replay, or preview archive. |
| Local-first | Diff and future OCR should run locally before any AI handoff. |
| AI is optional | AI connector features are off by default and scoped per region. |
| Instant privacy | Privacy blank must stop capture loops, not merely hide the tile. |

## Planned User Experience

```text
1. Drag to select a small screen region.
2. Name it: build-log, upload, dashboard-cell, queue, chart-row.
3. Keep it as a small always-on-top tile.
4. Let ScreenPebble detect visual changes locally.
5. Optionally use local OCR to extract changed text.
6. Optionally hand compact text to an AI tool when the user enables it.
```

The important detail: AI should not continuously watch the screen. Local diff
and OCR should do the cheap work first. AI is for interpretation after a
meaningful change, not for constant image polling.

## AI Without The Creepiness

ScreenPebble's AI direction is deliberately narrow.

Default behavior:

- No AI connection.
- No cloud upload.
- No hidden calls.
- No browser cookie scraping.
- No ChatGPT web automation.
- No whole-screen access.

Future optional behavior:

- Per-region AI enablement.
- Text-first handoff from local OCR.
- Image handoff only for explicitly allowed regions.
- Cooldowns and deduplication to reduce usage.
- Visible indicator whenever AI handoff is active.

The target is to work well even with cheaper subscription plans: send compact
text when possible, send images only when necessary, and let the local app do
most monitoring work.

## Privacy Model

ScreenPebble should be safe to explain in one sentence:

> It watches only the small regions you pin, locally, with no frame history and
> no upload by default.

Privacy requirements:

- Store configuration only: region coordinates, tile names, tile position,
  refresh settings, and alert preferences.
- Never persist captured frames, screenshots, previews, OCR history, browser
  URLs, clipboard contents, or AI prompts derived from screen content.
- Stop capture when a tile is paused, hidden, blanked, closed, or deleted.
- Treat permission changes and AI connector changes as high-risk work requiring
  explicit review.

See [Security And Privacy](docs/SECURITY_AND_PRIVACY.md).

## Performance Contract

ScreenPebble is intentionally low-FPS.

| Limit | First public release target |
| --- | ---: |
| Default refresh | 1 FPS |
| Maximum refresh | 5 FPS |
| Active tiles | 3 |
| Recommended region | 600x300 or smaller |
| Hard maximum region | 800x600 |
| Stored frame history | 0 |

The app should prefer local, cheap checks:

```text
small crop -> local diff -> local OCR if changed -> dedupe -> optional AI text handoff
```

## Status

ScreenPebble is in pre-alpha. Phase 0 is implemented and pushed. Screen
capture, OCR, AI handoff, and persisted user configuration are not implemented
yet.

| Phase | State | Scope |
| --- | --- | --- |
| Phase 0 | Implemented | Tauri + React + TypeScript + Rust scaffold |
| Phase 1 | Remaining | Performance limit validation |
| Phase 2 | Remaining | Main window and tile window shell |
| Phase 3 | Remaining | Region selector model |
| Phase 4 | Remaining | Region selector overlay UI |
| Phase 5 | Remaining | Fake capture backend |
| Phase 6 | Remaining | Capture lifecycle and scheduler |
| Phase 7 | Remaining | Local diff engine |
| Phase 8 | Remaining | Privacy blank and hotkey shell |
| Phase 9 | Remaining | Real capture backend |
| Phase 10 | Remaining | Live tile |
| Phase 11 | Remaining | Config-only persistence |
| Phase 12 | Remaining | Optional local OCR |
| Phase 13 | Remaining | Optional AI handoff |
| Phase 14 | Remaining | Release readiness |

## Installation

Not released yet.

The first release should provide:

```bash
brew install screenpebble
```

Until then, development setup will be added with the scaffold commit.

## Development

Read these before changing code:

- [AGENTS.md](AGENTS.md)
- [Product Spec](docs/PRODUCT_SPEC.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [AI Handoff Design](docs/AI_HANDOFF_DESIGN.md)
- [Engineering Charter](docs/ENGINEERING_CHARTER.md)
- [Security And Privacy](docs/SECURITY_AND_PRIVACY.md)
- [Git And Security Policy](docs/GIT_AND_SECURITY_POLICY.md)
- [Development Workflow](docs/DEVELOPMENT_WORKFLOW.md)

Current repository state:

- Phase 0 app scaffold is present.
- Runtime dependencies are limited to the Tauri/React shell.
- Tests cover the pre-alpha shell content and Rust app status command.
- Every future feature is expected to include tests, review, an atomic commit,
  and a push.

## Architecture Direction

Planned layers:

```text
React UI
Typed frontend command wrappers
Tauri command boundary
Rust application services
OS adapters
```

Planned service boundaries:

- `PerformanceLimits`: FPS, tile, and region limits.
- `RegionMapper`: logical-to-physical coordinate conversion.
- `CaptureBackend`: OS capture adapter trait.
- `CaptureScheduler`: single owner of capture loops.
- `CaptureLifecycle`: live, paused, hidden, blanked, closed states.
- `DiffEngine`: local visual change scoring.
- `OcrEngine`: optional local OCR adapter.
- `PebbleStore`: config-only persistence.
- `AiConnector`: optional, explicit, permissioned handoff.

## Contributing

ScreenPebble is not accepting broad feature expansion yet. Early contributions
should strengthen the core contract:

- Smaller, clearer architecture.
- Better tests.
- Safer capture lifecycle.
- Better permission handling.
- Lower resource usage.
- Clearer user-facing privacy language.

Before opening a change, keep the scope narrow and make the behavior testable.

## Roadmap Philosophy

The product should earn trust before it earns features.

Good additions:

- Better local capture reliability.
- Better multi-monitor support.
- Better region selection.
- Better local diff/OCR.
- Better privacy indicators.
- Better setup and packaging.

Bad additions:

- Continuous AI screen watching.
- Hidden capture.
- Cloud sync by default.
- Frame history.
- High-FPS mirroring.
- Broad filesystem or browser access.
- Anything that makes the app hard to explain safely.

## Name

ScreenPebble is a small thing you keep on the side of your workspace. It is not a
wall-sized monitor, not a recorder, and not a second desktop. It is a pebble:
small, visible, local, and easy to put away.

## License

MIT. See [LICENSE](LICENSE).
