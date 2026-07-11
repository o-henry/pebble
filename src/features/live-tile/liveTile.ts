import type {
  CaptureError,
  CroppedFramePayload
} from "../capture/captureFrame";
import { PERFORMANCE_LIMITS } from "../performance/performanceLimits";
import type { PhysicalRegion } from "../region-selector/regionSelection";

export const LIVE_TILE_FRAME_EVENT = "pebble://frame-updated";

export type LiveTileMode = "live" | "paused" | "blanked" | "closed";

export interface LiveTileFrameEvent {
  eventName: typeof LIVE_TILE_FRAME_EVENT;
  requestId: string;
  tileId: string;
  sequence: number;
  frame: CroppedFramePayload;
}

export interface LiveTileCaptureRequest {
  requestId: string;
  blankGeneration: number;
  tileId: string;
  region: PhysicalRegion;
  fps: number;
  mode: LiveTileMode;
}

export interface LiveTileCaptureResponse {
  requestId: string;
  blankGeneration: number;
  tileId: string;
  mode: LiveTileMode;
  effectiveFps: number;
  captureActive: boolean;
  frameSequence: number | null;
}

export type LiveTileCaptureResult =
  | { ok: true; response: LiveTileCaptureResponse }
  | { ok: false; error: CaptureError };

export interface LiveTileState {
  tileId: string;
  title: string;
  region: PhysicalRegion;
  mode: LiveTileMode;
  fps: number;
  effectiveFps: number;
  latestFrame: CroppedFramePayload | null;
  latestSequence: number;
  renderCount: number;
  blankGeneration: number;
}

export type LiveTileAction =
  | { type: "resume" }
  | { type: "pause" }
  | { type: "windowHidden" }
  | { type: "privacyBlank" }
  | { type: "close" }
  | { type: "watchRegion"; region: PhysicalRegion }
  | { type: "setFps"; fps: number }
  | { type: "backendSettled"; response: LiveTileCaptureResponse }
  | { type: "frameReceived"; event: LiveTileFrameEvent };

export const DEFAULT_LIVE_TILE_REGION: PhysicalRegion = {
  monitorId: "main",
  x: 10,
  y: 20,
  width: 300,
  height: 180
};

export const INITIAL_LIVE_TILE_STATE: LiveTileState = {
  tileId: "main-live-tile",
  title: "Pebble tile",
  region: DEFAULT_LIVE_TILE_REGION,
  mode: "paused",
  fps: PERFORMANCE_LIMITS.defaultFps,
  effectiveFps: PERFORMANCE_LIMITS.defaultFps,
  latestFrame: null,
  latestSequence: 0,
  renderCount: 0,
  blankGeneration: 0
};

export function createLiveTileState(region: PhysicalRegion): LiveTileState {
  return {
    ...INITIAL_LIVE_TILE_STATE,
    region,
    mode: "live"
  };
}

export function liveTileReducer(
  state: LiveTileState,
  action: LiveTileAction
): LiveTileState {
  switch (action.type) {
    case "resume":
      return { ...state, mode: "live" };
    case "pause":
      return { ...state, mode: "paused" };
    case "windowHidden":
      return clearLatestFrame(state);
    case "privacyBlank":
      return clearLatestFrame({
        ...state,
        blankGeneration: state.blankGeneration + 1
      });
    case "close":
      return clearLatestFrame({ ...state, mode: "closed" });
    case "watchRegion":
      return clearLatestFrame({
        ...state,
        region: action.region,
        mode: "live"
      });
    case "setFps":
      return {
        ...state,
        fps: clampLiveTileFps(action.fps)
      };
    case "backendSettled":
      return action.response.mode === "blanked"
        ? clearLatestFrame({
            ...state,
            blankGeneration: action.response.blankGeneration,
            effectiveFps: action.response.effectiveFps
          })
        : {
            ...state,
            blankGeneration: action.response.blankGeneration,
            effectiveFps: action.response.effectiveFps
          };
    case "frameReceived":
      return applyFrameEvent(state, action.event);
  }
}

export function applyFrameEvent(
  state: LiveTileState,
  event: LiveTileFrameEvent
): LiveTileState {
  if (event.tileId !== state.tileId || event.sequence <= state.latestSequence) {
    return state;
  }

  if (state.mode !== "live") {
    return state;
  }

  return {
    ...state,
    latestFrame: event.frame,
    latestSequence: event.sequence,
    renderCount: state.renderCount + 1
  };
}

export function liveTileRequest(
  state: LiveTileState,
  requestId = "local-preview-request",
  mode: LiveTileMode = state.mode
): LiveTileCaptureRequest {
  return {
    requestId,
    blankGeneration: state.blankGeneration,
    tileId: state.tileId,
    region: state.region,
    fps: state.fps,
    mode
  };
}

export function createLiveTileRequestScope(
  tileId: string,
  randomId = randomScopeId()
): string {
  return `${tileId}-${randomId}`;
}

export function scopedLiveTileRequestId(scope: string, sequence: number): string {
  return `${scope}-${sequence}`;
}

export function clampLiveTileFps(fps: number): number {
  const rounded = Math.round(Number.isFinite(fps) ? fps : 1);

  return Math.min(Math.max(rounded, 1), PERFORMANCE_LIMITS.maxFps);
}

export function frameByteLength(frame: CroppedFramePayload | null): number {
  return frame?.bytes.length ?? 0;
}

export function shouldAcceptLiveTileResponse(
  activeRequestId: string | null,
  response: LiveTileCaptureResponse
): boolean {
  return activeRequestId === response.requestId;
}

export function shouldAcceptLiveTileFrame(
  activeRequestId: string | null,
  privacyBlankActive: boolean,
  event: LiveTileFrameEvent
): boolean {
  return !privacyBlankActive && activeRequestId === event.requestId;
}

function clearLatestFrame(state: LiveTileState): LiveTileState {
  return {
    ...state,
    latestFrame: null,
    latestSequence: 0,
    renderCount: 0
  };
}

let fallbackScopeSequence = 0;

function randomScopeId(): string {
  if (typeof globalThis.crypto?.randomUUID === "function") {
    return globalThis.crypto.randomUUID();
  }

  fallbackScopeSequence += 1;
  return `${Date.now()}-${fallbackScopeSequence}`;
}
