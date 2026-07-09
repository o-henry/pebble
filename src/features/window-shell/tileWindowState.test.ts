import { describe, expect, it } from "vitest";
import {
  TEST_TILE_DEFAULT_STATE,
  isInactiveTileMode,
  shouldTileCapture,
  tileWindowReducer,
  type TileMode
} from "./tileWindowState";

describe("tile window state", () => {
  it("starts closed with always-on-top configured for the test tile", () => {
    expect(TEST_TILE_DEFAULT_STATE).toMatchObject({
      mode: "closed",
      captureActive: false,
      alwaysOnTop: true,
      label: "screenpebble-test-tile"
    });
  });

  it("moves through tile lifecycle modes", () => {
    const live = tileWindowReducer(TEST_TILE_DEFAULT_STATE, { type: "opened" });
    const paused = tileWindowReducer(live, { type: "paused" });
    const hidden = tileWindowReducer(paused, { type: "hidden" });
    const blanked = tileWindowReducer(hidden, { type: "blanked" });
    const errored = tileWindowReducer(blanked, {
      type: "errored",
      message: "Tile shell failed"
    });
    const closed = tileWindowReducer(errored, { type: "closed" });

    expect(live.mode).toBe("live");
    expect(live.captureActive).toBe(false);
    expect(paused.mode).toBe("paused");
    expect(paused.captureActive).toBe(false);
    expect(hidden.mode).toBe("hidden");
    expect(hidden.captureActive).toBe(false);
    expect(blanked.mode).toBe("blanked");
    expect(blanked.captureActive).toBe(false);
    expect(errored).toMatchObject({
      mode: "error",
      captureActive: false,
      placeholder: "Tile shell failed"
    });
    expect(closed.mode).toBe("closed");
    expect(closed.captureActive).toBe(false);
  });

  it("keeps capture disabled for every shell-only mode", () => {
    const modes: TileMode[] = [
      "live",
      "paused",
      "hidden",
      "blanked",
      "error",
      "closed"
    ];

    expect(modes.filter(shouldTileCapture)).toEqual([]);
    expect(modes.filter(isInactiveTileMode)).toEqual([
      "live",
      "paused",
      "hidden",
      "blanked",
      "error",
      "closed"
    ]);
  });
});
