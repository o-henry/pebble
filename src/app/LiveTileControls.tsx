import { clampLiveTileFps, frameByteLength } from "../features/live-tile/liveTile";
import type { CroppedFramePayload } from "../features/capture/captureFrame";
import type { LiveTileState } from "../features/live-tile/liveTile";

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
        <dt>FPS</dt>
        <dd>{tile.effectiveFps}</dd>
      </div>
      <div>
        <dt>Frame</dt>
        <dd>{frameByteLength(frame)} bytes</dd>
      </div>
    </dl>
  );
}

export function LiveTileControls({
  fps,
  onLive,
  onPause,
  onClose,
  onFpsChange
}: {
  fps: number;
  onLive: () => void;
  onPause: () => void;
  onClose: () => void;
  onFpsChange: (fps: number) => void;
}) {
  return (
    <div className="live-tile-controls">
      <button type="button" onClick={onLive}>
        Live
      </button>
      <button type="button" onClick={onPause}>
        Pause
      </button>
      <button type="button" onClick={onClose}>
        Close
      </button>
      <label>
        <span>FPS</span>
        <input
          type="number"
          min={1}
          max={5}
          value={fps}
          onChange={(event) =>
            onFpsChange(clampLiveTileFps(event.currentTarget.valueAsNumber))
          }
        />
      </label>
    </div>
  );
}
