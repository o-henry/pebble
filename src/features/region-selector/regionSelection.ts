import {
  REGION_SELECTION_LIMITS,
  type LogicalPoint,
  type LogicalSize,
  type PhysicalPoint,
  type PhysicalRegion,
  type RegionSelectionIssue,
  type RegionSelectionLimits,
  type RegionSelectionRequest,
  type RegionSelectionResult
} from "./regionSelectionTypes";
import { isRegionSelectionIssue, issue } from "./regionSelectionIssues";
import { mapLogicalSelectionToPhysical } from "./regionSelectionMapper";

export {
  REGION_SELECTION_LIMITS,
  type LogicalPoint,
  type LogicalSize,
  type MonitorGeometry,
  type PhysicalPoint,
  type PhysicalRegion,
  type RegionSelection,
  type RegionSelectionIssue,
  type RegionSelectionIssueCode,
  type RegionSelectionLimits,
  type RegionSelectionRequest,
  type RegionSelectionResult
} from "./regionSelectionTypes";

export function selectRegion(
  request: RegionSelectionRequest,
  limits: RegionSelectionLimits = REGION_SELECTION_LIMITS
): RegionSelectionResult {
  const geometryError = validateGeometry(request);

  if (geometryError) {
    return { ok: false, error: geometryError };
  }

  const mappedRegion = mapLogicalSelectionToPhysical(request);

  if (isRegionSelectionIssue(mappedRegion)) {
    return { ok: false, error: mappedRegion };
  }

  const region = mappedRegion;
  const validationError =
    validateMinimumRegion(region, limits) ?? validateHardLimits(region, limits);

  if (validationError) {
    return { ok: false, error: validationError };
  }

  return {
    ok: true,
    selection: {
      region,
      warnings: recommendedWarnings(region, limits),
      limits
    }
  };
}

function validateGeometry(
  request: RegionSelectionRequest
): RegionSelectionIssue | null {
  const points = [
    request.monitor.logicalOrigin,
    request.monitor.physicalOrigin,
    request.start,
    request.end
  ];

  if (points.some((point) => !isFinitePoint(point))) {
    return issue("invalidCoordinate", 0, 0);
  }

  if (
    !Number.isFinite(request.monitor.scaleFactor) ||
    request.monitor.scaleFactor <= 0
  ) {
    return issue("invalidScaleFactor", 0, request.monitor.scaleFactor);
  }

  if (!isFinitePositiveSize(request.monitor.logicalSize)) {
    return issue("invalidCoordinate", 0, 0);
  }

  if (!selectionIsInsideMonitor(request)) {
    return issue("selectionOutsideMonitor", 0, 0);
  }

  return null;
}

function selectionIsInsideMonitor(request: RegionSelectionRequest): boolean {
  const left = request.monitor.logicalOrigin.x;
  const top = request.monitor.logicalOrigin.y;
  const right = left + request.monitor.logicalSize.width;
  const bottom = top + request.monitor.logicalSize.height;

  return [request.start, request.end].every(
    (point) =>
      point.x >= left &&
      point.x <= right &&
      point.y >= top &&
      point.y <= bottom
  );
}

function validateMinimumRegion(
  region: PhysicalRegion,
  limits: RegionSelectionLimits
): RegionSelectionIssue | null {
  if (region.width < limits.minimumRegion.width) {
    return issue("regionTooNarrow", limits.minimumRegion.width, region.width);
  }

  if (region.height < limits.minimumRegion.height) {
    return issue("regionTooShort", limits.minimumRegion.height, region.height);
  }

  return null;
}

function validateHardLimits(
  region: PhysicalRegion,
  limits: RegionSelectionLimits
): RegionSelectionIssue | null {
  if (region.width > limits.maxRegion.width) {
    return issue("regionWidthTooLarge", limits.maxRegion.width, region.width);
  }

  if (region.height > limits.maxRegion.height) {
    return issue("regionHeightTooLarge", limits.maxRegion.height, region.height);
  }

  return null;
}

function recommendedWarnings(
  region: PhysicalRegion,
  limits: RegionSelectionLimits
): RegionSelectionIssue[] {
  const warnings: RegionSelectionIssue[] = [];

  if (region.width > limits.recommendedRegion.width) {
    warnings.push(
      issue(
        "regionWidthAboveRecommended",
        limits.recommendedRegion.width,
        region.width
      )
    );
  }

  if (region.height > limits.recommendedRegion.height) {
    warnings.push(
      issue(
        "regionHeightAboveRecommended",
        limits.recommendedRegion.height,
        region.height
      )
    );
  }

  return warnings;
}

function isFinitePoint(point: LogicalPoint | PhysicalPoint): boolean {
  return Number.isFinite(point.x) && Number.isFinite(point.y);
}

function isFinitePositiveSize(size: LogicalSize): boolean {
  return (
    Number.isFinite(size.width) &&
    Number.isFinite(size.height) &&
    size.width > 0 &&
    size.height > 0
  );
}
