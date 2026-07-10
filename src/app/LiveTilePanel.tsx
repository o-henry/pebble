import { useCallback, useReducer, useState } from "react";
import type { PhysicalRegion } from "../features/region-selector/regionSelection";
import {
  createLiveTileState,
  liveTileReducer,
  type LiveTileMode
} from "../features/live-tile/liveTile";
import { LiveFrameCanvas } from "./LiveFrameCanvas";
import { LiveTileControls } from "./LiveTileControls";
import { useLiveTileBackend } from "./useLiveTileBackend";

export function LiveTilePanel({
  region,
  privacyBlankActive,
  onClose
}: {
  region: PhysicalRegion;
  privacyBlankActive: boolean;
  onClose: () => void | Promise<void>;
}) {
  const [tile, dispatch] = useReducer(liveTileReducer, region, createLiveTileState);
  const [error, setError] = useState<string | null>(null);
  const requestMode: LiveTileMode = privacyBlankActive ? "blanked" : tile.mode;
  const visibleFrame = privacyBlankActive ? null : tile.latestFrame;
  const visibleMode = privacyBlankActive ? "blanked" : tile.mode;
  const handleBackendError = useCallback((message: string | null) => {
    setError(message);
    if (message) {
      dispatch({ type: "privacyBlank" });
      dispatch({ type: "pause" });
    }
  }, []);
  const backend = useLiveTileBackend({
    tile,
    requestMode,
    privacyBlankActive,
    onError: handleBackendError,
    dispatch
  });

  function pauseTile() {
    backend.clearActiveRequest();
    dispatch({ type: "pause" });
  }

  async function closeTile() {
    backend.clearActiveRequest();
    dispatch({ type: "close" });
    await onClose();
  }

  const frameState = liveFrameState(visibleMode, visibleFrame !== null, error);

  return (
    <section className="live-tile-section" aria-labelledby="live-tile-title">
      <header className="live-tile-heading">
        <h1 id="live-tile-title">ScreenPebble</h1>
        <span className={"tile-status is-" + visibleMode} role="status">
          <span className="status-dot" aria-hidden="true" />
          {frameState}
        </span>
      </header>

      <div className="live-frame-stage">
        <LiveFrameCanvas frame={visibleFrame} fallbackRegion={region} />
      </div>

      {error ? (
        <p className="live-tile-error" role="alert">
          {captureErrorMessage(error)}
        </p>
      ) : null}

      <LiveTileControls
        mode={visibleMode}
        onLive={() => dispatch({ type: "resume" })}
        onPause={pauseTile}
        onClose={() => void closeTile()}
      />
    </section>
  );
}

function liveFrameState(
  mode: LiveTileMode,
  hasFrame: boolean,
  error: string | null
) {
  if (error) {
    return "Capture needs attention";
  }

  if (mode === "blanked") {
    return "Preview hidden";
  }

  if (mode === "paused") {
    return "Paused";
  }

  return hasFrame ? "Live" : "Starting";
}

function captureErrorMessage(message: string) {
  return /permission/i.test(message)
    ? "Screen Recording permission is off. Enable ScreenPebble in macOS System Settings, then resume."
    : message;
}
