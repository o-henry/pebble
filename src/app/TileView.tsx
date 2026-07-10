import { LiveTilePanel } from "./LiveTilePanel";
import {
  advanceBrowserSession,
  regionKey
} from "../features/pebble-session/pebbleSession";
import { closePebbleWindow } from "../lib/invoke";
import { usePebbleSession } from "./usePebbleSession";

export function TileView() {
  const {
    session,
    loading,
    browserPreview,
    updateSession,
    setError
  } = usePebbleSession();

  async function closeWindow() {
    try {
      if (browserPreview) {
        updateSession(advanceBrowserSession(session, { windowOpen: false }));
        globalThis.location.hash = "";
        return;
      }

      updateSession(await closePebbleWindow());
    } catch {
      setError("Pebble window could not be closed.");
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
        <p className="section-label">No selected region</p>
        <h1>Select a region in ScreenPebble</h1>
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
        privacyBlankActive={session.privacyBlankActive}
        onClose={closeWindow}
      />
    </main>
  );
}
