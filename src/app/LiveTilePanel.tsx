import { useCallback, useEffect, useReducer, useState } from "react";
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
import { listenToMonitorInsights, type MonitorInsight } from "../lib/events";
import { useAdaptiveTheme } from "./useAdaptiveTheme";
import { captureErrorMessage, liveFrameState } from "./liveTilePresentation";

export function LiveTilePanel({
  region,
  browserPreview,
  privacyBlankActive,
  sessionError,
  onAiExpandedChange,
  onPrivacyBlankChange,
  onReselect
}: {
  region: PhysicalRegion;
  browserPreview: boolean;
  privacyBlankActive: boolean;
  sessionError: string | null;
  onAiExpandedChange: (expanded: boolean) => void | Promise<void>;
  onPrivacyBlankChange: (active: boolean) => void | Promise<void>;
  onReselect: () => void | Promise<void>;
}) {
  const [tile, dispatch] = useReducer(liveTileReducer, region, createLiveTileState);
  const [captureError, setCaptureError] = useState<string | null>(null);
  const [controlError, setControlError] = useState<string | null>(null);
  const [aiExpanded, setAiExpanded] = useState(false);
  const [aiBusy, setAiBusy] = useState(false);
  const [controlBusy, setControlBusy] = useState(false);
  const [monitorInsight, setMonitorInsight] = useState<MonitorInsight | null>({
    kind: "baseline",
    summary: "LOCAL MONITORING ACTIVE"
  });
  const requestMode: LiveTileMode = privacyBlankActive ? "blanked" : tile.mode;
  const visibleFrame = privacyBlankActive ? null : tile.latestFrame;
  const visibleMode = privacyBlankActive ? "blanked" : tile.mode;
  useAdaptiveTheme();
  const handleBackendError = useCallback((message: string | null) => {
    setCaptureError(message);
    if (message) {
      dispatch({ type: "privacyBlank" });
      dispatch({ type: "pause" });
    }
  }, []);
  useEffect(() => {
    let unlisten: () => void = () => undefined;
    let active = true;
    void listenToMonitorInsights(setMonitorInsight).then((nextUnlisten) => {
      if (active) unlisten = nextUnlisten;
      else nextUnlisten();
    });
    return () => {
      active = false;
      unlisten();
    };
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

  async function toggleAi() {
    const expanded = !aiExpanded;
    try {
      setControlBusy(true);
      setControlError(null);
      await onAiExpandedChange(expanded);
      setAiExpanded(expanded);
    } catch {
      setControlError("AI PANEL COULD NOT BE RESIZED.");
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
      aria-label="LIVE REGION"
    >
      <header className="live-tile-heading">
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
        />
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

      {monitorInsight ? (
        <div
          className={`monitor-insight is-${monitorInsight.kind}`}
          role={monitorInsight.kind === "change" ? "alert" : "status"}
        >
          <span className="status-dot" aria-hidden="true" />
          {monitorInsight.summary}
        </div>
      ) : null}

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
