export type TileMode = "live" | "paused" | "hidden" | "blanked" | "error" | "closed";

export interface TileWindowState {
  id: string;
  label: string;
  title: string;
  mode: TileMode;
  alwaysOnTop: boolean;
  captureActive: boolean;
  placeholder: string;
}

export interface WindowShellSnapshot {
  testTile: TileWindowState;
  supportedModes: TileMode[];
}

export interface WindowShellError {
  code: "tileWindowUnavailable";
  message: string;
}

export type TileWindowAction =
  | { type: "opened" }
  | { type: "paused" }
  | { type: "resumed" }
  | { type: "hidden" }
  | { type: "blanked" }
  | { type: "errored"; message: string }
  | { type: "closed" };

export const TEST_TILE_LABEL = "screenpebble-test-tile";

export const TEST_TILE_DEFAULT_STATE: TileWindowState = {
  id: "test-tile",
  label: TEST_TILE_LABEL,
  title: "Test Pebble",
  mode: "closed",
  alwaysOnTop: true,
  captureActive: false,
  placeholder: "Fake tile placeholder. Capture is not implemented."
};

export const WINDOW_SHELL_DEFAULT_SNAPSHOT: WindowShellSnapshot = {
  testTile: TEST_TILE_DEFAULT_STATE,
  supportedModes: ["live", "paused", "hidden", "blanked", "error", "closed"]
};

export function tileWindowReducer(
  state: TileWindowState,
  action: TileWindowAction
): TileWindowState {
  switch (action.type) {
    case "opened":
    case "resumed":
      return {
        ...state,
        mode: "live",
        captureActive: false
      };
    case "paused":
      return {
        ...state,
        mode: "paused",
        captureActive: false
      };
    case "hidden":
      return {
        ...state,
        mode: "hidden",
        captureActive: false
      };
    case "blanked":
      return {
        ...state,
        mode: "blanked",
        captureActive: false
      };
    case "errored":
      return {
        ...state,
        mode: "error",
        captureActive: false,
        placeholder: action.message
      };
    case "closed":
      return {
        ...state,
        mode: "closed",
        captureActive: false
      };
  }
}

export function shouldTileCapture(mode: TileMode): boolean {
  void mode;

  return false;
}

export function isInactiveTileMode(mode: TileMode): boolean {
  return !shouldTileCapture(mode);
}
