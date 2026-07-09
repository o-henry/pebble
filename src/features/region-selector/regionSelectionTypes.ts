import {
  PERFORMANCE_LIMITS,
  type RegionSize
} from "../performance/performanceLimits";

const MIN_REGION_SIZE: RegionSize = {
  width: 24,
  height: 24
};

export interface LogicalPoint {
  x: number;
  y: number;
}

export interface PhysicalPoint {
  x: number;
  y: number;
}

export interface MonitorGeometry {
  id: string;
  logicalOrigin: LogicalPoint;
  physicalOrigin: PhysicalPoint;
  scaleFactor: number;
}

export interface RegionSelectionRequest {
  monitor: MonitorGeometry;
  start: LogicalPoint;
  end: LogicalPoint;
}

export interface PhysicalRegion {
  monitorId: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RegionSelectionLimits {
  minimumRegion: RegionSize;
  recommendedRegion: RegionSize;
  maxRegion: RegionSize;
}

export interface RegionSelection {
  region: PhysicalRegion;
  warnings: RegionSelectionIssue[];
  limits: RegionSelectionLimits;
}

export type RegionSelectionIssueCode =
  | "invalidCoordinate"
  | "invalidScaleFactor"
  | "regionTooNarrow"
  | "regionTooShort"
  | "regionCoordinateOutOfRange"
  | "regionWidthAboveRecommended"
  | "regionHeightAboveRecommended"
  | "regionWidthTooLarge"
  | "regionHeightTooLarge";

export interface RegionSelectionIssue {
  code: RegionSelectionIssueCode;
  limit: number;
  actual: number;
  message: string;
}

export type RegionSelectionResult =
  | { ok: true; selection: RegionSelection }
  | { ok: false; error: RegionSelectionIssue };

export const REGION_SELECTION_LIMITS: RegionSelectionLimits = {
  minimumRegion: MIN_REGION_SIZE,
  recommendedRegion: PERFORMANCE_LIMITS.recommendedRegion,
  maxRegion: PERFORMANCE_LIMITS.maxRegion
};

export const ISSUE_MESSAGES: Record<RegionSelectionIssueCode, string> = {
  invalidCoordinate: "Selection coordinates must be finite.",
  invalidScaleFactor: "Monitor scale factor must be greater than zero.",
  regionTooNarrow: "Selected region is too narrow.",
  regionTooShort: "Selected region is too short.",
  regionCoordinateOutOfRange:
    "Selected region is outside the supported coordinate range.",
  regionWidthAboveRecommended: "Selected region is wider than recommended.",
  regionHeightAboveRecommended: "Selected region is taller than recommended.",
  regionWidthTooLarge: "Selected region is wider than allowed.",
  regionHeightTooLarge: "Selected region is taller than allowed."
};
