# Real Capture Premortem

Phase 9 introduces one real platform capture adapter. The adapter is deliberately
small because screen pixels are the most sensitive surface in Pebble.

## Failure Modes

Permission denied:

- The macOS adapter checks screen recording access before capture.
- Denial returns `permissionDenied` as a recoverable typed error.
- The adapter does not silently fall back to fake data.

Full-frame temporary memory:

- Pebble requests a `CGRect` matching the selected physical region.
- The app does not request, retain, serialize, or emit a full-monitor frame.
- macOS may perform internal compositing, but the app boundary receives only the
  selected crop image.

Crop-before-encode:

- The platform request uses the selected crop rectangle.
- Pixel conversion copies only selected rows into an RGBA memory buffer.
- The adapter returns `CroppedFramePayload` with `memoryOnly` storage.
- Real capture is not exposed as a direct frontend command in Phase 9; live use
  must go through scheduler and lifecycle policy.

Cleanup on close and blank:

- Existing lifecycle rules stop capture for paused, hidden, blanked, closed, and
  deleted tiles.
- The real adapter owns no background task and writes no file.
- Any captured bytes are owned by the caller and dropped with the latest frame.

## Minimum Verification

- Unit tests keep fake capture as the primary deterministic backend.
- Platform tests cover invalid-region mapping before any OS capture attempt.
- macOS tests cover crop rect shape and row-copy behavior.
- Source checks reject file-backed capture helpers such as `screencapture` and
  temporary file writes.
- Manual smoke verifies permission denied and permission allowed behavior.
