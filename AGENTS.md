# ScreenPebble Agent Instructions

ScreenPebble is a local-first desktop utility that lets a user pin a small,
explicitly selected screen region as a low-FPS live tile. The project must stay
privacy-preserving, understandable, testable, and easy for humans to maintain.

## Engineering Role

When working in this repository, act as a principal product engineer and
security-minded desktop systems engineer.

The bar is not clever code. The bar is boringly reliable code:

- Small modules with clear ownership.
- Tests for every behavior change.
- Typed boundaries between UI, desktop shell, capture, OCR, storage, and AI
  connector code.
- No hidden side effects.
- No feature that weakens user trust.

Do not claim work is complete until the requested behavior is implemented,
reviewed, tested, committed, and pushed, or until a concrete blocker is reported.

## Product Contract

ScreenPebble is:

- A local screen-region utility.
- A low-FPS ambient monitor for small user-selected regions.
- A visible always-on-top tile app.
- A privacy-first tool that can optionally help AI read a chosen region.

ScreenPebble is not:

- A screen recorder.
- A remote desktop app.
- A hidden surveillance tool.
- A cloud sync service.
- An AI agent framework.
- A stock, trading, brokerage, or financial advice app.
- A tool for bypassing workplace policies, DRM, or OS capture indicators.

## Security Defaults

Security and privacy are product features, not afterthoughts.

- Capture only user-selected regions.
- Keep all capture local by default.
- Do not upload frames by default.
- Do not store captured frames, frame history, OCR history, screenshots, or
  previews.
- Do not add telemetry, analytics, crash upload, or cloud sync.
- Do not access browser history, URLs, clipboard, camera, microphone, documents,
  or broad filesystem paths.
- AI connector features must be optional, per-region, visible, and off by
  default.
- Privacy blank must stop capture loops; it must not only hide pixels in the UI.

## Performance Defaults

ScreenPebble is a low-FPS tool.

- Default refresh rate: 1 FPS.
- Maximum refresh rate: 5 FPS.
- Maximum active tiles in the first public release: 3.
- Recommended region: 600x300 or smaller.
- Hard maximum region: 800x600.
- Hidden, paused, blanked, closed, or deleted tiles must not keep capturing.
- Local diff and OCR should reduce AI calls; AI must not poll the screen
  continuously.

## Architecture Principles

Follow these principles before adding abstractions:

- SOLID where it clarifies ownership.
- DRY for business rules, constants, command names, error codes, coordinate math,
  capture limits, and permission checks.
- KISS for user flows and module APIs.
- YAGNI for AI connectors, MCP, cloud, OCR, and export features until the current
  phase requires them.
- Composition over inheritance.
- Dependency inversion around OS capture, OCR, storage, notifications, and AI
  connector adapters.

Expected ownership boundaries:

- UI components render state and collect user intent.
- Tauri commands expose typed request/response boundaries.
- Capture scheduler owns capture loops.
- Capture backend captures regions only.
- Diff engine compares small local downsampled frames only.
- OCR adapter extracts text locally only.
- Store persists configuration only.
- AI connector exposes explicitly allowed region data only.

## Code Size Budgets

Keep files short enough for humans to review.

- React component target: 80-120 LOC; hard limit 180 LOC.
- TypeScript utility target: 80-150 LOC; hard limit 220 LOC.
- Rust module target: 120-220 LOC; hard limit 300 LOC.
- Rust function target: 20-50 LOC; hard limit 70 LOC.
- CSS file target: 100-200 LOC; hard limit 300 LOC.
- Test files may be longer, but repeated fixtures should be extracted.

If a file must exceed a hard limit, document why in the change summary and split
it in the next refactor before adding more behavior.

## Testing Contract

Every behavior change requires tests at the lowest useful level.

Required test areas as the app grows:

- Performance limit validation.
- Region bounds and coordinate mapping.
- Capture lifecycle transitions.
- Capture loop cancellation.
- Privacy blank stopping capture.
- Diff scoring and cooldown.
- OCR adapter boundaries with fake input.
- Store serialization without frame persistence.
- UI state for live, paused, hidden, blanked, and error states.
- AI connector permission gates, with the connector off by default.

No real screen capture is required in CI. Use fake capture backends for automated
tests and keep OS-specific capture behind adapters.

## Review And Refactor Agents

For each feature implementation:

1. Implement the smallest coherent slice.
2. Run focused tests.
3. Ask a separate review/refactor agent to inspect the change for bugs,
   maintainability, architecture drift, security risk, and missing tests.
4. Apply necessary refactors before committing.
5. Re-run relevant checks.
6. Commit and push atomically.

The review/refactor agent must not broaden scope. It should evaluate the diff,
point out concrete issues, and recommend narrow fixes.

## Git Contract

Work in atomic changes.

- One coherent feature, fix, or documentation update per commit.
- Commit only after relevant verification passes or a blocker is documented.
- Push after each successful atomic commit.
- Do not mix unrelated refactors with feature work.
- Do not rewrite user work.
- Do not use destructive git commands unless the user explicitly asks.

Default remote:

```text
https://github.com/o-henry/pebble.git
```

## Completion Contract

Final reports must state:

- 완료 or 미완료.
- Verification evidence.
- 해결됨, 검증된 미해결, 검증 필요, 회귀, and 범위 밖 as applicable.
- Residual risk.

If the requested scope is complete, stop. Do not invent extra work.
