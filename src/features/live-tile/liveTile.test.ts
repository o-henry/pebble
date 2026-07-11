import { describe, expect, it } from "vitest";
import {
  INITIAL_LIVE_TILE_STATE,
  LIVE_TILE_FRAME_EVENT,
  applyFrameEvent,
  clampLiveTileFps,
  createLiveTileState,
  createLiveTileRequestScope,
  frameByteLength,
  liveTileReducer,
  liveTileRequest,
  shouldAcceptLiveTileFrame,
  shouldAcceptLiveTileResponse,
  scopedLiveTileRequestId,
  type LiveTileFrameEvent
} from "./liveTile";

describe("live tile state", () => {
  it("scopes request ids to one tile component instance", () => {
    const firstScope = createLiveTileRequestScope("tile", "instance-a");
    const secondScope = createLiveTileRequestScope("tile", "instance-b");

    expect(scopedLiveTileRequestId(firstScope, 1)).toBe("tile-instance-a-1");
    expect(scopedLiveTileRequestId(secondScope, 1)).toBe("tile-instance-b-1");
    expect(firstScope).not.toBe(secondScope);
  });

  it("starts live on the user-selected region", () => {
    const region = {
      monitorId: "display-2",
      x: 80,
      y: 120,
      width: 420,
      height: 220
    };
    const initial = createLiveTileState(region);
    const changed = liveTileReducer(initial, {
      type: "watchRegion",
      region: { ...region, x: 180 }
    });

    expect(initial).toMatchObject({ region, mode: "live" });
    expect(changed).toMatchObject({ region: { ...region, x: 180 }, mode: "live" });
    expect(changed.latestFrame).toBeNull();
  });

  it("keeps only the latest frame for rendering", () => {
    const live = liveTileReducer(INITIAL_LIVE_TILE_STATE, { type: "resume" });
    const first = applyFrameEvent(
      live,
      frameEvent(1, 2, 2)
    );
    const second = applyFrameEvent(first, frameEvent(2, 3, 2));
    const duplicate = applyFrameEvent(second, frameEvent(2, 4, 4));

    expect(first.renderCount).toBe(1);
    expect(second.renderCount).toBe(2);
    expect(duplicate).toBe(second);
    expect(frameByteLength(second.latestFrame)).toBe(3 * 2 * 4);
  });

  it("pause and close stop active capture intent", () => {
    const live = liveTileReducer(INITIAL_LIVE_TILE_STATE, { type: "resume" });
    const paused = liveTileReducer(live, { type: "pause" });
    const closed = liveTileReducer(
      applyFrameEvent(paused, frameEvent(1, 2, 2)),
      { type: "close" }
    );

    expect(liveTileRequest(live).mode).toBe("live");
    expect(liveTileRequest(paused).mode).toBe("paused");
    expect(liveTileRequest(closed).mode).toBe("closed");
    expect(closed.latestFrame).toBeNull();
    expect(closed.renderCount).toBe(0);
  });

  it("clamps fps in UI state", () => {
    const low = liveTileReducer(INITIAL_LIVE_TILE_STATE, {
      type: "setFps",
      fps: 0
    });
    const high = liveTileReducer(INITIAL_LIVE_TILE_STATE, {
      type: "setFps",
      fps: 99
    });

    expect(clampLiveTileFps(Number.NaN)).toBe(1);
    expect(low.fps).toBe(1);
    expect(high.fps).toBe(5);
  });

  it("applies backend effective fps", () => {
    const state = liveTileReducer(INITIAL_LIVE_TILE_STATE, {
      type: "backendSettled",
      response: {
        requestId: "request-1",
        blankGeneration: 0,
        tileId: INITIAL_LIVE_TILE_STATE.tileId,
        mode: "live",
        effectiveFps: 5,
        captureActive: true,
        frameSequence: null
      }
    });

    expect(state.effectiveFps).toBe(5);
    expect(state.mode).toBe("paused");
  });

  it("ignores stale frames after pause", () => {
    const live = liveTileReducer(INITIAL_LIVE_TILE_STATE, { type: "resume" });
    const paused = liveTileReducer(live, { type: "pause" });
    const stale = applyFrameEvent(paused, frameEvent(1, 2, 2));

    expect(stale).toBe(paused);
    expect(stale.latestFrame).toBeNull();
  });

  it("privacy blank clears latest frame without losing preblank mode", () => {
    const live = liveTileReducer(INITIAL_LIVE_TILE_STATE, { type: "resume" });
    const withFrame = applyFrameEvent(live, frameEvent(1, 2, 2));
    const blanked = liveTileReducer(withFrame, { type: "privacyBlank" });
    const request = liveTileRequest(blanked, "request-blank", "blanked");

    expect(blanked.mode).toBe("live");
    expect(blanked.latestFrame).toBeNull();
    expect(blanked.blankGeneration).toBe(1);
    expect(request.mode).toBe("blanked");
    expect(request.blankGeneration).toBe(1);
  });

  it("hidden windows discard pixels while preserving live intent", () => {
    const live = liveTileReducer(INITIAL_LIVE_TILE_STATE, { type: "resume" });
    const withFrame = applyFrameEvent(live, frameEvent(1, 2, 2));
    const hidden = liveTileReducer(withFrame, { type: "windowHidden" });

    expect(hidden.mode).toBe("live");
    expect(hidden.latestFrame).toBeNull();
    expect(hidden.renderCount).toBe(0);
  });

  it("adopts the backend privacy generation before capture resumes", () => {
    const blanked = liveTileReducer(INITIAL_LIVE_TILE_STATE, {
      type: "backendSettled",
      response: {
        requestId: "request-blank",
        blankGeneration: 4,
        tileId: INITIAL_LIVE_TILE_STATE.tileId,
        mode: "blanked",
        effectiveFps: 1,
        captureActive: false,
        frameSequence: null
      }
    });

    expect(blanked.blankGeneration).toBe(4);
    expect(liveTileRequest(blanked, "request-resume", "live").blankGeneration).toBe(4);
  });

  it("rejects stale backend responses and privacy blank frames", () => {
    const response = {
      requestId: "request-1",
      blankGeneration: 1,
      tileId: INITIAL_LIVE_TILE_STATE.tileId,
      mode: "blanked" as const,
      effectiveFps: 1,
      captureActive: false,
      frameSequence: null
    };
    const event = frameEvent(1, 2, 2);

    expect(shouldAcceptLiveTileResponse("request-2", response)).toBe(false);
    expect(shouldAcceptLiveTileResponse("request-1", response)).toBe(true);
    expect(shouldAcceptLiveTileFrame(event.requestId, true, event)).toBe(false);
    expect(shouldAcceptLiveTileFrame(event.requestId, false, event)).toBe(true);
  });
});

function frameEvent(sequence: number, width: number, height: number): LiveTileFrameEvent {
  return {
    eventName: LIVE_TILE_FRAME_EVENT,
    requestId: `request-${sequence}`,
    tileId: INITIAL_LIVE_TILE_STATE.tileId,
    sequence,
    frame: {
      monitorId: "main",
      region: {
        monitorId: "main",
        x: 10,
        y: 20,
        width,
        height
      },
      width,
      height,
      pixelFormat: "rgba8",
      bytesPerPixel: 4,
      storagePolicy: "memoryOnly",
      bytes: Array.from({ length: width * height * 4 }, (_, index) => index % 255)
    }
  };
}
