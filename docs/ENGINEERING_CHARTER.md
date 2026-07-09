# Engineering Charter

ScreenPebble should feel simple because the code is simple in the right places.
This project favors explicit boundaries, narrow adapters, and testable behavior
over clever shortcuts.

## Quality Bar

Production code must be:

- Easy to read.
- Easy to delete.
- Easy to test.
- Easy to reason about under failure.
- Conservative with permissions and resources.

Avoid abstractions until two or more real call sites prove they remove
complexity. Prefer small, named functions over dense logic.

## Module Boundaries

Expected app layers:

```text
React UI
Typed frontend command wrappers
Tauri command boundary
Rust application services
OS adapters
```

Core services should not know about React. UI should not own capture loops.

Planned service boundaries:

- `PerformanceLimits`: source of truth for FPS, tile, and region limits.
- `RegionMapper`: logical-to-physical coordinate conversion.
- `CaptureBackend`: OS capture adapter trait.
- `CaptureScheduler`: single owner of capture loops.
- `CaptureLifecycle`: active, paused, hidden, blanked, closed state.
- `DiffEngine`: small-frame local change scoring.
- `OcrEngine`: local OCR adapter, optional and off until implemented.
- `PebbleStore`: config-only persistence.
- `AiConnector`: optional, explicit, permissioned region handoff.

## Data Rules

Persist only:

- Region coordinates.
- Tile window bounds.
- Tile names.
- FPS settings under the hard limit.
- Alert settings.
- User preferences.

Never persist:

- Captured frames.
- Screenshots.
- OCR history.
- Frame history.
- Browser URLs.
- Clipboard contents.
- AI prompts derived from screen content unless the user explicitly exports them.

## Error Handling

Recoverable failures must become typed user-facing errors.

Examples:

- Screen recording permission denied.
- Monitor unavailable.
- Region outside bounds.
- Region too large.
- Too many active tiles.
- Capture backend unavailable.
- OCR unavailable.
- AI connector not authorized for a region.

Rust code must not use `unwrap()` outside tests. TypeScript must not use `any`
except at a parser boundary with immediate validation.

## Test Strategy

Prefer unit tests around policy and lifecycle. Prefer fake adapters over real OS
capture in CI.

Use integration or manual smoke tests for:

- OS capture permission behavior.
- Multi-monitor capture.
- Retina or mixed-DPI coordinate mapping.
- Always-on-top tile behavior.
- Hotkeys.
- Packaging.

## Refactoring Policy

Refactor when it improves the current change:

- Extract duplicated constants.
- Split files approaching line limits.
- Move business logic out of UI.
- Rename ambiguous concepts.
- Add a seam for fake testing.

Do not perform broad cleanup unrelated to the active task.
