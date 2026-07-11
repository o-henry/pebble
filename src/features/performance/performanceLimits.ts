export interface RegionSize {
  width: number;
  height: number;
}

export interface PerformanceLimits {
  defaultFps: number;
  maxFps: number;
  maxActiveTiles: number;
  recommendedRegion: RegionSize;
  maxRegion: RegionSize;
}

export interface PerformanceLimitRequest {
  fps: number;
  activeTileCount: number;
  region: RegionSize;
}

export type PerformanceLimitErrorCode =
  | "fpsTooLow"
  | "fpsTooHigh"
  | "activeTileCountTooLow"
  | "activeTileLimitExceeded"
  | "regionWidthTooLarge"
  | "regionHeightTooLarge"
  | "regionEmpty";

export interface PerformanceLimitError {
  code: PerformanceLimitErrorCode;
  limit: number;
  actual: number;
}

export type PerformanceValidationResult =
  | { valid: true; error?: null | undefined }
  | { valid: false; error: PerformanceLimitError };

export const PERFORMANCE_LIMITS: PerformanceLimits = {
  defaultFps: 1,
  maxFps: 5,
  maxActiveTiles: 3,
  recommendedRegion: {
    width: 2147483647,
    height: 2147483647
  },
  maxRegion: {
    width: 2147483647,
    height: 2147483647
  }
};

export function validatePerformanceRequest(
  request: PerformanceLimitRequest,
  limits: PerformanceLimits = PERFORMANCE_LIMITS
): PerformanceValidationResult {
  const fpsResult = validateFps(request.fps, limits);

  if (!fpsResult.valid) {
    return fpsResult;
  }

  const activeTileResult = validateActiveTileCount(
    request.activeTileCount,
    limits
  );

  if (!activeTileResult.valid) {
    return activeTileResult;
  }

  return validateRegionSize(request.region, limits);
}

export function validateFps(
  fps: number,
  limits: PerformanceLimits = PERFORMANCE_LIMITS
): PerformanceValidationResult {
  if (fps < 1) {
    return invalid("fpsTooLow", 1, fps);
  }

  if (fps > limits.maxFps) {
    return invalid("fpsTooHigh", limits.maxFps, fps);
  }

  return valid();
}

export function validateActiveTileCount(
  activeTileCount: number,
  limits: PerformanceLimits = PERFORMANCE_LIMITS
): PerformanceValidationResult {
  if (activeTileCount < 0) {
    return invalid("activeTileCountTooLow", 0, activeTileCount);
  }

  if (activeTileCount > limits.maxActiveTiles) {
    return invalid(
      "activeTileLimitExceeded",
      limits.maxActiveTiles,
      activeTileCount
    );
  }

  return valid();
}

export function validateRegionSize(
  region: RegionSize,
  limits: PerformanceLimits = PERFORMANCE_LIMITS
): PerformanceValidationResult {
  if (region.width < 1) {
    return invalid("regionEmpty", 1, region.width);
  }

  if (region.height < 1) {
    return invalid("regionEmpty", 1, region.height);
  }

  if (region.width > limits.maxRegion.width) {
    return invalid("regionWidthTooLarge", limits.maxRegion.width, region.width);
  }

  if (region.height > limits.maxRegion.height) {
    return invalid(
      "regionHeightTooLarge",
      limits.maxRegion.height,
      region.height
    );
  }

  return valid();
}

function valid(): PerformanceValidationResult {
  return { valid: true };
}

function invalid(
  code: PerformanceLimitErrorCode,
  limit: number,
  actual: number
): PerformanceValidationResult {
  return {
    valid: false,
    error: {
      code,
      limit,
      actual
    }
  };
}
