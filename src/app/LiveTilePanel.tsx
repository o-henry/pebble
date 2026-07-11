import { useCallback, useReducer, useState } from "react";
import type { PhysicalRegion } from "../features/region-selector/regionSelection";
import {
  createLiveTileState,
  liveTileReducer,
  type LiveTileMode
} from "../features/live-tile/liveTile";
import { LiveFrameCanvas } from "./LiveFrameCanvas";
import { LiveTileControls } from "./LiveTileControls";
import { RegionQuestionPanel } from "./RegionQuestionPanel";
import { useLiveTileBackend } from "./useLiveTileBackend";

export function LiveTilePanel({
  region,
  browserPreview,
  privacyBlankActive,
  sessionError,
  onAiExpandedChange,
  onClose,
  onPrivacyBlankChange,
  onReselect
}: {
  region: PhysicalRegion;
  browserPreview: boolean;
  privacyBlankActive: boolean;
  sessionError: string | null;
  onAiExpandedChange: (expanded: boolean) => void | Promise<void>;
  onClose: () => void | Promise<void>;
  onPrivacyBlankChange: (active: boolean) => void | Promise<void>;
  onReselect: () => void | Promise<void>;
}) {
  const [tile, dispatch] = useReducer(liveTileReducer, region, createLiveTileState);
  const [captureError, setCaptureError] = useState<string | null>(null);
  const [controlError, setControlError] = useState<string | null>(null);
  const [aiExpanded, setAiExpanded] = useState(false);
  const [aiBusy, setAiBusy] = useState(false);
  const [controlBusy, setControlBusy] = useState(false);
  const requestMode: LiveTileMode = privacyBlankActive ? "blanked" : tile.mode;
  const visibleFrame = privacyBlankActive ? null : tile.latestFrame;
  const visibleMode = privacyBlankActive ? "blanked" : tile.mode;
  const handleBackendError = useCallback((message: string | null) => {
    setCaptureError(message);
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
  const busy = aiBusy || controlBusy;
  const error = sessionError ?? controlError ?? captureError;

  function pauseTile() {
    backend.clearActiveRequest();
    dispatch({ type: "pause" });
  }

  async function closeTile() {
    backend.clearActiveRequest();
    dispatch({ type: "close" });
    await onClose();
  }

  async function toggleAi() {
    const expanded = !aiExpanded;
    try {
      setControlBusy(true);
      setControlError(null);
      await onAiExpandedChange(expanded);
      setAiExpanded(expanded);
    } catch {
      setControlError("ChatGPT panel could not be resized.");
    } finally {
      setControlBusy(false);
    }
  }

  async function togglePrivacy() {
    try {
      setControlBusy(true);
      setControlError(null);
      await onPrivacyBlankChange(!privacyBlankActive);
    } catch {
      setControlError("Preview visibility could not be updated.");
    } finally {
      setControlBusy(false);
    }
  }

  const frameState = liveFrameState(
    visibleMode,
    visibleFrame !== null,
    error
  );

  return (
    <section
      className={
        "live-tile-section " + (aiExpanded ? "has-ai-panel" : "")
      }
      aria-labelledby="live-tile-title"
    >
      <header className="live-tile-heading">
        <h1 id="live-tile-title">pebble</h1>
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
        aiExpanded={aiExpanded}
        disabled={busy}
        privacyBlankActive={privacyBlankActive}
        onLive={() => dispatch({ type: "resume" })}
        onPause={pauseTile}
        onReselect={() => void onReselect()}
        onToggleAi={() => void toggleAi()}
        onTogglePrivacy={() => void togglePrivacy()}
        onClose={() => void closeTile()}
      />

      {aiExpanded ? (
        <RegionQuestionPanel
          browserPreview={browserPreview}
          disabled={busy}
          privacyBlankActive={privacyBlankActive}
          onBusyChange={setAiBusy}
        />
      ) : null}
    </section>
  );
}

function liveFrameState(
  mode: LiveTileMode,
  hasFrame: boolean,
  error: string | null
) {
  if (error) {
    return "NEEDS ATTENTION";
  }
  if (mode === "blanked") {
    return "HIDDEN";
  }
  if (mode === "paused") {
    return "PAUSED";
  }
  return hasFrame ? "LIVE" : "STARTING";
}

function captureErrorMessage(message: string) {
  return /permission/i.test(message)
    ? "Screen Recording permission is off. Enable pebble in macOS System Settings, then resume."
    : message;
}
