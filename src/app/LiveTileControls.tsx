import type { LiveTileMode } from "../features/live-tile/liveTile";

export function LiveTileControls({
  mode,
  onLive,
  onPause,
  onClose
}: {
  mode: LiveTileMode;
  onLive: () => void;
  onPause: () => void;
  onClose: () => void;
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
      <button type="button" className="close-action" onClick={onClose}>
        Close
      </button>
    </div>
  );
}
