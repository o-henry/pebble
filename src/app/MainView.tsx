import { useCallback, useEffect, useState } from "react";
import { privacyHotkeyAction } from "../features/privacy/privacyBlank";
import { advanceBrowserSession } from "../features/pebble-session/pebbleSession";
import {
  openRegionSelectorWindow,
  removePebble,
  requestScreenCaptureAccess,
  setPebblePrivacyBlank,
  showPebbleWindow
} from "../lib/invoke";
import { usePebbleSession, errorMessage } from "./usePebbleSession";
import { RegionQuestionPanel } from "./RegionQuestionPanel";

export function MainView() {
  const {
    session,
    loading,
    error,
    browserPreview,
    updateSession,
    setError
  } = usePebbleSession();
  const [busy, setBusy] = useState(false);
  const [aiBusy, setAiBusy] = useState(false);
  const hasRegion = session.region !== null;

  const setPrivacyBlank = useCallback(
    async (active: boolean) => {
      try {
        setBusy(true);
        if (browserPreview) {
          updateSession(
            advanceBrowserSession(session, { privacyBlankActive: active })
          );
        } else {
          updateSession(await setPebblePrivacyBlank(active));
        }
        setError(null);
      } catch (reason) {
        setError(errorMessage(reason, "Privacy control could not be updated."));
      } finally {
        setBusy(false);
      }
    },
    [browserPreview, session, setError, updateSession]
  );

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const action = privacyHotkeyAction(event, {
        blankActive: session.privacyBlankActive,
        lastAction: "none",
        hotkeyPermission: "notRequested"
      });

      if (action && hasRegion) {
        event.preventDefault();
        void setPrivacyBlank(action.type === "blank");
      }
    }

    globalThis.addEventListener("keydown", handleKeyDown);
    return () => globalThis.removeEventListener("keydown", handleKeyDown);
  }, [hasRegion, session.privacyBlankActive, setPrivacyBlank]);

  async function selectRegion() {
    try {
      setBusy(true);
      setError(null);

      if (browserPreview) {
        globalThis.location.hash = "#selector";
        return;
      }

      const permissionGranted = await requestScreenCaptureAccess();
      if (!permissionGranted) {
        setError(
          "Allow Screen Recording for ScreenPebble in macOS System Settings, then try again."
        );
        return;
      }

      await openRegionSelectorWindow();
    } catch (reason) {
      setError(errorMessage(reason, "Region selector could not be opened."));
    } finally {
      setBusy(false);
    }
  }

  async function showWindow() {
    try {
      setBusy(true);
      if (browserPreview) {
        updateSession(advanceBrowserSession(session, { windowOpen: true }));
        globalThis.location.hash = "#tile";
      } else {
        updateSession(await showPebbleWindow());
      }
      setError(null);
    } catch (reason) {
      setError(errorMessage(reason, "Pebble window could not be opened."));
    } finally {
      setBusy(false);
    }
  }

  async function stopWatching() {
    try {
      setBusy(true);
      if (browserPreview) {
        updateSession(
          advanceBrowserSession(session, {
            region: null,
            windowOpen: false,
            privacyBlankActive: false
          })
        );
      } else {
        updateSession(await removePebble());
      }
      setError(null);
    } catch (reason) {
      setError(errorMessage(reason, "Pebble could not be removed."));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main
      className={
        "app-shell " + (session.privacyBlankActive ? "is-privacy-blanked" : "")
      }
    >
      <header className="workspace-header">
        <div className="brand-lockup">
          <strong>ScreenPebble</strong>
        </div>
        <div className="workspace-header__actions">
          <span className="local-status">
            <span className="status-dot" aria-hidden="true" />
            {browserPreview ? "Preview mode" : "Local capture"}
          </span>
          {hasRegion && session.windowOpen ? (
            <button
              type="button"
              className="privacy-action"
              disabled={busy}
              onClick={() => void setPrivacyBlank(!session.privacyBlankActive)}
            >
              {session.privacyBlankActive ? "Show preview" : "Hide preview"}
            </button>
          ) : null}
        </div>
      </header>

      <section className="workspace-intro" aria-labelledby="screenpebble-title">
        <h1 id="screenpebble-title">Keep the part that matters in view.</h1>
        <p>
          Select any part of your screen. ScreenPebble keeps it visible while you
          work elsewhere.
        </p>
      </section>

      {error ? (
        <p className="workspace-error" role="alert">
          {error}
        </p>
      ) : null}

      {loading ? (
        <section className="workspace-loading" aria-live="polite">
          Loading local session
        </section>
      ) : hasRegion ? (
        <ActiveWorkspace
          session={session}
          busy={busy || aiBusy}
          browserPreview={browserPreview}
          onAiBusyChange={setAiBusy}
          onShow={showWindow}
          onReselect={selectRegion}
          onStop={stopWatching}
        />
      ) : (
        <EmptyWorkspace busy={busy} onSelect={selectRegion} />
      )}
    </main>
  );
}

function EmptyWorkspace({
  busy,
  onSelect
}: {
  busy: boolean;
  onSelect: () => void;
}) {
  return (
    <section className="empty-workspace" aria-label="Start ScreenPebble">
      <button
        type="button"
        className="primary-action"
        disabled={busy}
        onClick={onSelect}
      >
        {busy ? "Opening selector" : "Select a region"}
      </button>
    </section>
  );
}

function ActiveWorkspace({
  session,
  busy,
  browserPreview,
  onShow,
  onReselect,
  onStop,
  onAiBusyChange
}: {
  session: ReturnType<typeof usePebbleSession>["session"];
  busy: boolean;
  browserPreview: boolean;
  onShow: () => void;
  onReselect: () => void;
  onStop: () => void;
  onAiBusyChange: (busy: boolean) => void;
}) {
  const status = session.privacyBlankActive
    ? "Preview hidden"
    : session.windowOpen
      ? "Watching now"
      : "Window closed";
  const title = session.privacyBlankActive
    ? "Selected region is hidden"
    : session.windowOpen
      ? "Watching selected region"
      : "Selected region is ready";

  return (
    <section className="active-workspace" aria-labelledby="active-title">
      <div className="active-workspace__summary">
        <span
          className={
            "active-signal " + (session.windowOpen ? "" : "is-inactive")
          }
          aria-hidden="true"
        />
        <div>
          <p className="section-label">{status}</p>
          <h2 id="active-title">{title}</h2>
          <p>
            {browserPreview
              ? "Open the compact pebble to preview the selected area."
              : session.windowOpen
                ? "The selected region is visible above your other windows."
                : "Show the pebble whenever you need it again."}
          </p>
        </div>
      </div>

      <div className="active-workspace__actions">
        {!session.windowOpen || browserPreview ? (
          <button
            type="button"
            className="primary-action"
            disabled={busy}
            onClick={onShow}
          >
            {browserPreview ? "Open tile preview" : "Show pebble"}
          </button>
        ) : null}
        <button
          type="button"
          className="secondary-action"
          disabled={busy}
          onClick={onReselect}
        >
          Select another region
        </button>
        <button
          type="button"
          className="danger-action"
          disabled={busy}
          onClick={onStop}
        >
          Stop watching
        </button>
      </div>

      <RegionQuestionPanel
        key={`${session.region?.monitorId}:${session.region?.x}:${session.region?.y}:${session.region?.width}:${session.region?.height}`}
        browserPreview={browserPreview}
        disabled={busy}
        privacyBlankActive={session.privacyBlankActive}
        onBusyChange={onAiBusyChange}
      />
    </section>
  );
}
