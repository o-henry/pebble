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
      <div>
        <p className="section-label">Live tile</p>
        <h2 id="live-tile-title">{tile.title}</h2>
      </div>
      <div className="live-tile-panel">
        <LiveFrameCanvas frame={visibleFrame} />
        <LiveTileStats
          tile={tile}
          frame={visibleFrame}
          privacyBlankActive={privacyBlankActive}
        />
        {error ? <p className="live-tile-error">{error}</p> : null}
        <LiveTileControls
          fps={tile.fps}
          onLive={() => dispatch({ type: "resume" })}
          onPause={pauseTile}
          onClose={closeTile}
          onFpsChange={(fps) => dispatch({ type: "setFps", fps })}
        />
      </div>
    </section>
  );
}
