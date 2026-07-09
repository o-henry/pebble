# ScreenPebble

Pin a small part of your screen.

ScreenPebble is a local-first desktop utility for turning a user-selected screen
region into a tiny low-FPS always-on-top tile. It is designed for progress bars,
build logs, queue numbers, timers, dashboards, and other small status areas that
you do not want to keep open full-size.

The long-term direction is optional AI assistance: ScreenPebble can locally watch
for meaningful changes and, only when explicitly enabled, help an AI read the
selected region through text-first, low-usage handoff paths.

## Current Status

This repository is at the pre-implementation stage. The first commits define the
product, security, architecture, testing, and workflow contracts before app code
is added.

## Product Principles

- User-selected regions only.
- Low FPS by design.
- Local-first by default.
- No hidden capture.
- No telemetry.
- No frame history.
- No captured frame persistence.
- AI connector features are optional and off by default.

## Intended Uses

- Watch a build or test log while working elsewhere.
- Keep an upload, export, or render progress bar visible.
- Monitor a ticket queue, waiting room number, or dashboard cell.
- Pin a small chart row, timer, or status panel.
- Let local diff/OCR detect meaningful changes before asking AI for help.

## Non-Goals

ScreenPebble is not a screen recorder, remote desktop app, hidden monitoring
tool, cloud sync product, AI agent framework, or workplace-policy bypass tool.

## Development

Read these first:

- [AGENTS.md](AGENTS.md)
- [Engineering Charter](docs/ENGINEERING_CHARTER.md)
- [Security And Privacy](docs/SECURITY_AND_PRIVACY.md)
- [Development Workflow](docs/DEVELOPMENT_WORKFLOW.md)

Implementation has not started yet. Tooling and commands will be added with the
scaffold commit.
