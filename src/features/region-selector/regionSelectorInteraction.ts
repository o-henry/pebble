import {
  selectRegion,
  type LogicalPoint,
  type MonitorGeometry,
  type RegionSelectionRequest,
  type RegionSelectionResult
} from "./regionSelection";

export type RegionSelectorStatus = "idle" | "dragging" | "ready" | "cancelled";

export interface DragRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RegionSelectorState {
  status: RegionSelectorStatus;
  monitor: MonitorGeometry;
  start: LogicalPoint | null;
  current: LogicalPoint | null;
  result: RegionSelectionResult | null;
}

export type RegionSelectorAction =
  | { type: "begin"; point: LogicalPoint; monitor: MonitorGeometry }
  | { type: "move"; point: LogicalPoint }
  | { type: "finish"; point: LogicalPoint }
  | { type: "cancel" }
  | { type: "reset" };

export const DEFAULT_SELECTOR_MONITOR: MonitorGeometry = {
  id: "selector-overlay",
  logicalOrigin: { x: 0, y: 0 },
  logicalSize: { width: 960, height: 640 },
  physicalOrigin: { x: 0, y: 0 },
  scaleFactor: 1
};

export const INITIAL_REGION_SELECTOR_STATE: RegionSelectorState = {
  status: "idle",
  monitor: DEFAULT_SELECTOR_MONITOR,
  start: null,
  current: null,
  result: null
};

export function regionSelectorReducer(
  state: RegionSelectorState,
  action: RegionSelectorAction
): RegionSelectorState {
  switch (action.type) {
    case "begin":
      return {
        status: "dragging",
        monitor: action.monitor,
        start: action.point,
        current: action.point,
        result: null
      };
    case "move":
      if (state.status !== "dragging") {
        return state;
      }

      return {
        ...state,
        current: action.point
      };
    case "finish":
      if (!state.start || state.status !== "dragging") {
        return state;
      }

      return {
        ...state,
        status: "ready",
        current: action.point,
        result: selectRegion(
          createRegionSelectionRequest(state.monitor, state.start, action.point)
        )
      };
    case "cancel":
      return {
        ...INITIAL_REGION_SELECTOR_STATE,
        status: "cancelled"
      };
    case "reset":
      return INITIAL_REGION_SELECTOR_STATE;
  }
}

export function createRegionSelectionRequest(
  monitor: MonitorGeometry,
  start: LogicalPoint,
  end: LogicalPoint
): RegionSelectionRequest {
  return {
    monitor,
    start,
    end
  };
}

export function createViewportMonitor(
  width: number,
  height: number,
  scaleFactor = 1
): MonitorGeometry {
  return {
    ...DEFAULT_SELECTOR_MONITOR,
    id: `selector-overlay-${Math.round(width)}x${Math.round(height)}`,
    scaleFactor,
    physicalOrigin: { x: 0, y: 0 },
    logicalOrigin: { x: 0, y: 0 },
    logicalSize: { width, height }
  };
}

export function canBeginRegionDrag(
  monitor: MonitorGeometry | null
): monitor is MonitorGeometry {
  return monitor !== null;
}

export function dragRect(
  start: LogicalPoint | null,
  current: LogicalPoint | null
): DragRect | null {
  if (!start || !current) {
    return null;
  }

  return {
    x: Math.min(start.x, current.x),
    y: Math.min(start.y, current.y),
    width: Math.abs(current.x - start.x),
    height: Math.abs(current.y - start.y)
  };
}
