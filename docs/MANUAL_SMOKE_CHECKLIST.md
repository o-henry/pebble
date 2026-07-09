# Manual Smoke Checklist

Run this before tagging a public demo or recording a new demo GIF.

## Environment

- macOS desktop target.
- Fresh checkout.
- No private `.env` files.
- No captured screen content committed.
- Network disabled or monitored when validating local-only behavior.

## Build Checks

```bash
pnpm install
pnpm test
pnpm typecheck
pnpm lint
pnpm build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cd ..
pnpm tauri:build
```

## Manual App Checks

- Idle CPU with no tile.
- One 600x300 tile at 1 FPS for 60 seconds.
- Three 600x300 tiles at 1 FPS for 60 seconds.
- Pause/resume 20 times.
- Privacy blank 20 times.
- Create/delete tile 20 times.
- Permission denied flow.
- Quit and verify no capture task remains.

## Privacy Checks

- No frame files written to the repo.
- No screenshots, OCR output, prompts, tokens, cookies, or local account data in
  logs, docs, fixtures, or tests.
- Captured frame payloads remain cropped and memory-only.
- Paused, hidden, blanked, closed, and deleted states do not capture.
- AI handoff remains disabled by default.
- OCR remains disabled by default.
- Core monitoring adds no network requirement.

## Demo GIF Checks

- GIF is under 15 seconds.
- GIF shows visible user control, not hidden monitoring.
- GIF does not reveal private screen content.
- GIF shows local-first language or privacy blank.
- GIF is committed only after review.
