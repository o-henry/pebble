import { useEffect, useReducer, useState } from "react";
import { appStatus } from "./appContent";
import {
  Docs,
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
    <main className="app-shell">
      <section className="hero-section" aria-labelledby="screenpebble-title">
        <p className="status-line">
          {appStatus.phase} · live tile ready · real capture gated · AI off
        </p>
        <h1 id="screenpebble-title">ScreenPebble</h1>
        <p className="hero-copy">
          Pin a tiny part of your screen. Let local watchers notice what changed.
        </p>
        <p className="trust-copy">
          This build includes the desktop scaffold, hard performance limits, and
          a low-FPS live tile backed by memory-only cropped frames. There is no
          OCR, AI connector, telemetry, or network feature in this build.
        </p>
      </section>

      <PrivacyBanner
        state={privacy}
        onBlank={() => dispatchPrivacy({ type: "blank" })}
        onRestore={() => dispatchPrivacy({ type: "restore" })}
      />
      <Principles />
      <PerformanceLimits />
      <RegionSelectorSection
        shell={selectorShell}
        error={selectorError}
        onOpen={openSelector}
      />
      <LiveTilePanel privacyBlankActive={privacy.blankActive} />
      <WindowShellControls
        tile={snapshot.testTile}
        onOpen={openTile}
        onRefresh={refreshShell}
      />
      <Docs />
    </main>
  );
}
