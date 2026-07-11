import { LiveTilePanel } from "./LiveTilePanel";
import {
  advanceBrowserSession,
  regionKey
} from "../features/pebble-session/pebbleSession";
import {
  closePebbleWindow,
  openRegionSelectorWindow,
  requestScreenCaptureAccess,
  setPebbleAiPanelExpanded,
  setPebblePrivacyBlank
} from "../lib/invoke";
import { errorMessage, usePebbleSession } from "./usePebbleSession";

export function TileView() {
  const {
    session,
    loading,
    error,
    browserPreview,
    updateSession,
    setError
  } = usePebbleSession();

  async function selectRegion() {
    try {
      setError(null);
      if (browserPreview) {
        globalThis.location.hash = "#selector";
        return;
      }

      if (!(await requestScreenCaptureAccess())) {
        setError(
          "Allow Screen Recording for Pebble in macOS System Settings, then try again."
        );
        return;
      }
      await openRegionSelectorWindow();
    } catch (reason) {
      setError(errorMessage(reason, "Region selector could not be opened."));
    }
  }

  async function setPrivacyBlank(active: boolean) {
    try {
      if (browserPreview) {
        updateSession(
          advanceBrowserSession(session, { privacyBlankActive: active })
        );
      } else {
        updateSession(await setPebblePrivacyBlank(active));
      }
      setError(null);
    } catch (reason) {
      setError(errorMessage(reason, "Preview visibility could not be updated."));
    }
  }

  async function setAiExpanded(expanded: boolean) {
    if (!browserPreview) {
      await setPebbleAiPanelExpanded(expanded);
    }
  }

  async function closeWindow() {
    try {
      if (browserPreview) {
        updateSession(advanceBrowserSession(session, { windowOpen: false }));
        return;
      }
      updateSession(await closePebbleWindow());
    } catch (reason) {
      setError(errorMessage(reason, "Pebble window could not be closed."));
    }
  }

  if (loading) {
    return (
      <main className="tile-shell tile-loading" aria-live="polite">
        Starting pebble
      </main>
    );
  }

  if (!session.region) {
    return (
      <main className="tile-shell tile-empty">
        <div className="tile-empty__brand">pebble</div>
        <div className="tile-empty__content">
          <p className="section-label">No selected region</p>
          <h1>Choose what stays in view.</h1>
          <button
            type="button"
            className="primary-action tile-empty__select"
            onClick={() => void selectRegion()}
          >
            SELECT REGION
          </button>
          {error ? (
            <p className="live-tile-error" role="alert">
              {error}
            </p>
          ) : null}
        </div>
      </main>
    );
  }

  return (
    <main
      className={
        "tile-shell " + (session.privacyBlankActive ? "is-privacy-blanked" : "")
      }
    >
      <LiveTilePanel
        key={regionKey(session.region)}
        region={session.region}
        browserPreview={browserPreview}
        privacyBlankActive={session.privacyBlankActive}
        sessionError={error}
        onAiExpandedChange={setAiExpanded}
        onClose={closeWindow}
        onPrivacyBlankChange={setPrivacyBlank}
        onReselect={selectRegion}
      />
    </main>
  );
}
