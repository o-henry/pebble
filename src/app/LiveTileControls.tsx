import { clampLiveTileFps } from "../features/live-tile/liveTile";
import type { CroppedFramePayload } from "../features/capture/captureFrame";
import type { LiveTileMode, LiveTileState } from "../features/live-tile/liveTile";

export function LiveTileStats({
  tile,
  frame,
  privacyBlankActive
}: {
  tile: LiveTileState;
  frame: CroppedFramePayload | null;
  privacyBlankActive: boolean;
}) {
  return (
    <dl className="live-tile-stats">
      <div>
        <dt>Mode</dt>
        <dd>{privacyBlankActive ? "blanked" : tile.mode}</dd>
      </div>
      <div>
        <dt>Refresh</dt>
        <dd>{tile.effectiveFps} FPS</dd>
      </div>
      <div>
        <dt>Frame</dt>
        <dd>
          {frame
            ? String(frame.width) + " x " + String(frame.height)
            : "Empty"}
        </dd>
      </div>
    </dl>
  );
}

export function LiveTileControls({
  fps,
  mode,
  onLive,
  onPause,
  onClose,
  onFpsChange
}: {
  fps: number;
  mode: LiveTileMode;
  onLive: () => void;
  onPause: () => void;
  onClose: () => void;
  onFpsChange: (fps: number) => void;
}) {
  return (
    <div className="live-tile-controls" aria-label="Live tile controls">
      <div className="mode-controls" role="group" aria-label="Capture state">
        <button
          type="button"
          className={mode === "live" ? "is-active" : "secondary-action"}
          onClick={onLive}
        >
          Live
        </button>
        <button
          type="button"
          className={mode === "paused" ? "is-active" : "secondary-action"}
          onClick={onPause}
        >
          Pause
        </button>
      </div>
      <label className="fps-control">
        <span>Refresh</span>
        <span className="fps-input-wrap">
          <input
            type="number"
            min={1}
            max={5}
            value={fps}
            onChange={(event) =>
              onFpsChange(clampLiveTileFps(event.currentTarget.valueAsNumber))
            }
          />
          <span aria-hidden="true">FPS</span>
        </span>
      </label>
      <button type="button" className="close-action" onClick={onClose}>
        Close
      </button>
    </div>
  );
}
