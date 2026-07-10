import { useReducer, useState } from "react";
import { LiveTileControls, LiveTileStats } from "./LiveTileControls";
import { LiveFrameCanvas } from "./LiveFrameCanvas";
import {
  INITIAL_LIVE_TILE_STATE,
  liveTileReducer,
  type LiveTileMode
} from "../features/live-tile/liveTile";
import { useLiveTileBackend } from "./useLiveTileBackend";

export function LiveTilePanel({
  privacyBlankActive
}: {
  privacyBlankActive: boolean;
}) {
  const [tile, dispatch] = useReducer(
    liveTileReducer,
    INITIAL_LIVE_TILE_STATE
  );
  const [error, setError] = useState<string | null>(null);
  const mode = tile.mode;
  const requestMode: LiveTileMode = privacyBlankActive ? "blanked" : mode;
  const visibleFrame = privacyBlankActive ? null : tile.latestFrame;
  const visibleMode = privacyBlankActive ? "blanked" : tile.mode;
  const frameState = visibleFrame
    ? String(visibleFrame.width) + " x " + String(visibleFrame.height) + " frame"
    : visibleMode === "live"
      ? "Waiting for frame"
      : "No active frame";
  const backend = useLiveTileBackend({
    tile,
    requestMode,
    privacyBlankActive,
    onError: setError,
    dispatch
  });

  function pauseTile() {
    backend.clearActiveRequest();
    dispatch({ type: "pause" });
  }

  function closeTile() {
    backend.clearActiveRequest();
    dispatch({ type: "close" });
  }

  return (
    <section className="live-tile-section" aria-labelledby="live-tile-title">
      <header className="panel-heading">
        <div>
          <p className="section-label">Primary observer</p>
          <h2 id="live-tile-title">{tile.title}</h2>
        </div>
        <span className={"mode-badge is-" + visibleMode}>{visibleMode}</span>
      </header>
      <div className="live-tile-panel">
        <div
          className={
            "live-frame-stage " + (visibleFrame ? "has-frame" : "is-empty")
          }
        >
          <LiveFrameCanvas frame={visibleFrame} />
          <div className="frame-chrome" aria-hidden="true">
            <span>{visibleMode}</span>
            <span>
              {tile.region.width} x {tile.region.height}
            </span>
          </div>
          <p className="frame-state" role="status">
            {frameState}
          </p>
        </div>
        <LiveTileStats
          tile={tile}
          frame={visibleFrame}
          privacyBlankActive={privacyBlankActive}
        />
        {error ? <p className="live-tile-error">{error}</p> : null}
        <LiveTileControls
          fps={tile.fps}
          mode={visibleMode}
          onLive={() => dispatch({ type: "resume" })}
          onPause={pauseTile}
          onClose={closeTile}
          onFpsChange={(fps) => dispatch({ type: "setFps", fps })}
        />
      </div>
    </section>
  );
}
