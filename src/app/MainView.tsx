import { useEffect, useReducer, useState } from "react";
import { appStatus } from "./appContent";
import {
  PerformanceLimits,
  Principles,
  WindowShellControls
} from "./MainSections";
import { LiveTilePanel } from "./LiveTilePanel";
import { PrivacyBanner } from "./PrivacyBanner";
import { RegionSelectorSection } from "./RegionSelectorSection";
import {
  PRIVACY_BLANK_INITIAL_STATE,
  privacyBlankReducer,
  privacyHotkeyAction
} from "../features/privacy/privacyBlank";
import {
  REGION_SELECTOR_DEFAULT_SHELL,
  type RegionSelectorWindowShell
} from "../features/region-selector/regionSelectorShell";
import {
  TEST_TILE_DEFAULT_STATE,
  WINDOW_SHELL_DEFAULT_SNAPSHOT,
  tileWindowReducer,
  type WindowShellSnapshot
} from "../features/window-shell/tileWindowState";
import {
  getWindowShellSnapshot,
  openRegionSelectorWindow,
  openTestTileWindow
} from "../lib/invoke";

export function MainView() {
  const [snapshot, setSnapshot] = useState<WindowShellSnapshot>(
    WINDOW_SHELL_DEFAULT_SNAPSHOT
  );
  const [selectorShell, setSelectorShell] = useState<RegionSelectorWindowShell>(
    REGION_SELECTOR_DEFAULT_SHELL
  );
  const [selectorError, setSelectorError] = useState<string | null>(null);
  const [privacy, dispatchPrivacy] = useReducer(
    privacyBlankReducer,
    PRIVACY_BLANK_INITIAL_STATE
  );

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const action = privacyHotkeyAction(event, privacy);

      if (action) {
        event.preventDefault();
        dispatchPrivacy(action);
      }
    }

    globalThis.addEventListener("keydown", handleKeyDown);

    return () => globalThis.removeEventListener("keydown", handleKeyDown);
  }, [privacy]);

  async function openTile() {
    try {
      const tile = await openTestTileWindow();
      setSnapshot((current) => ({ ...current, testTile: tile }));
    } catch (error) {
      setSnapshot((current) => ({
        ...current,
        testTile: tileWindowReducer(TEST_TILE_DEFAULT_STATE, {
          type: "errored",
          message: error instanceof Error ? error.message : "Tile shell failed"
        })
      }));
    }
  }

  async function refreshShell() {
    try {
      setSnapshot(await getWindowShellSnapshot());
    } catch {
      setSnapshot(WINDOW_SHELL_DEFAULT_SNAPSHOT);
    }
  }

  async function openSelector() {
    try {
      setSelectorShell(await openRegionSelectorWindow());
      setSelectorError(null);
    } catch (error) {
      setSelectorError(
        error instanceof Error ? error.message : "Selector overlay failed"
      );
    }
  }

  return (
    <main
      className={
        "app-shell " + (privacy.blankActive ? "is-privacy-blanked" : "")
      }
    >
      <header className="workspace-header">
        <div className="brand-lockup">
          <span className="brand-mark" aria-hidden="true">
            <span />
          </span>
          <span className="brand-name">ScreenPebble</span>
        </div>
        <div className="workspace-header__status" aria-label="Application status">
          <span className="status-dot" aria-hidden="true" />
          <span>Local session</span>
          <span className="status-divider" aria-hidden="true" />
          <span>{appStatus.phase}</span>
        </div>
      </header>

      <section className="command-deck" aria-labelledby="screenpebble-title">
        <div className="command-deck__title">
          <p className="section-label">Observer console</p>
          <h1 id="screenpebble-title">ScreenPebble</h1>
          <p className="command-deck__subtitle">
            Local screen signals, kept small.
          </p>
        </div>
        <div className="command-deck__actions">
          <button type="button" className="primary-action" onClick={openSelector}>
            New pebble
          </button>
          <button type="button" className="secondary-action" onClick={openTile}>
            Open test tile
          </button>
        </div>
        <dl className="command-deck__facts" aria-label="Session guarantees">
          <div>
            <dt>Scope</dt>
            <dd>Selected only</dd>
          </div>
          <div>
            <dt>Storage</dt>
            <dd>Memory only</dd>
          </div>
          <div>
            <dt>Network</dt>
            <dd>Off</dd>
          </div>
        </dl>
      </section>

      <PrivacyBanner
        state={privacy}
        onBlank={() => dispatchPrivacy({ type: "blank" })}
        onRestore={() => dispatchPrivacy({ type: "restore" })}
      />

      <div className="workspace-grid">
        <LiveTilePanel privacyBlankActive={privacy.blankActive} />
        <aside className="operations-rail" aria-label="Pebble controls">
          <RegionSelectorSection
            shell={selectorShell}
            error={selectorError}
            onOpen={openSelector}
          />
          <WindowShellControls
            tile={snapshot.testTile}
            onOpen={openTile}
            onRefresh={refreshShell}
          />
          <PerformanceLimits />
        </aside>
      </div>

      <Principles />
    </main>
  );
}
