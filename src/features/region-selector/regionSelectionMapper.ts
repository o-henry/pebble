import { isRegionSelectionIssue, issue } from "./regionSelectionIssues";
import type {
  PhysicalRegion,
  RegionSelectionIssue,
  RegionSelectionRequest
} from "./regionSelectionTypes";

const I32_MIN = -2147483648;
const I32_MAX = 2147483647;

export function mapLogicalSelectionToPhysical(
  request: RegionSelectionRequest
): PhysicalRegion | RegionSelectionIssue {
  const left = Math.min(request.start.x, request.end.x);
  const right = Math.max(request.start.x, request.end.x);
  const top = Math.min(request.start.y, request.end.y);
  const bottom = Math.max(request.start.y, request.end.y);
  const xAxis = mapAxis(
    left,
    right,
    request.monitor.logicalOrigin.x,
    request.monitor.physicalOrigin.x,
    request.monitor.scaleFactor
  );

  if (isRegionSelectionIssue(xAxis)) {
    return xAxis;
  }

  const yAxis = mapAxis(
    top,
    bottom,
    request.monitor.logicalOrigin.y,
    request.monitor.physicalOrigin.y,
    request.monitor.scaleFactor
  );

  if (isRegionSelectionIssue(yAxis)) {
    return yAxis;
  }

  const [x, width] = xAxis;
  const [y, height] = yAxis;

  return {
    monitorId: request.monitor.id,
    x,
    y,
    width,
    height
  };
}

function mapAxis(
  min: number,
  max: number,
  logicalOrigin: number,
  physicalOrigin: number,
  scale: number
): [number, number] | RegionSelectionIssue {
  const start = toI32(
    Math.floor((min - logicalOrigin) * scale) + physicalOrigin
  );
  const end = toI32(Math.ceil((max - logicalOrigin) * scale) + physicalOrigin);

  if (isRegionSelectionIssue(start)) {
    return start;
  }

  if (isRegionSelectionIssue(end)) {
    return end;
  }

  const width = end - start;

  if (width < 0 || width > I32_MAX) {
    return issue("regionCoordinateOutOfRange", I32_MAX, width);
  }

  return [start, width];
}

function toI32(value: number): number | RegionSelectionIssue {
  if (!Number.isFinite(value) || value < I32_MIN || value > I32_MAX) {
    return issue("regionCoordinateOutOfRange", I32_MAX, value);
  }

  return value;
}
