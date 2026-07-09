import { useState } from "react";
import { appStatus } from "./appContent";
import {
  Docs,
  PerformanceLimits,
  Principles,
  WindowShellControls
} from "./MainSections";
import {
  TEST_TILE_DEFAULT_STATE,
  WINDOW_SHELL_DEFAULT_SNAPSHOT,
  tileWindowReducer,
  type WindowShellSnapshot
} from "../features/window-shell/tileWindowState";
import {
  getWindowShellSnapshot,
  openTestTileWindow
} from "../lib/invoke";

export function MainView() {
  const [snapshot, setSnapshot] = useState<WindowShellSnapshot>(
    WINDOW_SHELL_DEFAULT_SNAPSHOT
  );

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

  return (
    <main className="app-shell">
      <section className="hero-section" aria-labelledby="screenpebble-title">
        <p className="status-line">
          {appStatus.phase} · window shell ready · capture off · AI off
        </p>
        <h1 id="screenpebble-title">ScreenPebble</h1>
        <p className="hero-copy">
          Pin a tiny part of your screen. Let local watchers notice what changed.
        </p>
        <p className="trust-copy">
          This build includes the desktop scaffold, hard performance limits, and
          a test tile window shell. There is no screen capture, OCR, AI
          connector, telemetry, or network feature in this build.
        </p>
      </section>

      <Principles />
      <PerformanceLimits />
      <WindowShellControls
        tile={snapshot.testTile}
        onOpen={openTile}
        onRefresh={refreshShell}
      />
      <Docs />
    </main>
  );
}
