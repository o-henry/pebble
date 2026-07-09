import { describe, expect, it } from "vitest";
import {
  PERFORMANCE_LIMITS,
  validateActiveTileCount,
  validateFps,
  validatePerformanceRequest,
  validateRegionSize
} from "./performanceLimits";

describe("performance limits", () => {
  it("matches the product performance contract", () => {
    expect(PERFORMANCE_LIMITS).toEqual({
      defaultFps: 1,
      maxFps: 5,
      maxActiveTiles: 3,
      recommendedRegion: {
        width: 600,
        height: 300
      },
      maxRegion: {
        width: 800,
        height: 600
      }
    });
  });

  it("accepts values inside the hard limits", () => {
    expect(
      validatePerformanceRequest({
        fps: 1,
        activeTileCount: 3,
        region: {
          width: 800,
          height: 600
        }
      })
    ).toEqual({ valid: true });
  });

  it("rejects fps values outside the allowed range with typed errors", () => {
    expect(validateFps(-1)).toEqual({
      valid: false,
      error: {
        code: "fpsTooLow",
        limit: 1,
        actual: -1
      }
    });

    expect(validateFps(0)).toEqual({
      valid: false,
      error: {
        code: "fpsTooLow",
        limit: 1,
        actual: 0
      }
    });

    expect(validateFps(6)).toEqual({
      valid: false,
      error: {
        code: "fpsTooHigh",
        limit: 5,
        actual: 6
      }
    });
  });

  it("rejects active tile counts above the maximum", () => {
    expect(validateActiveTileCount(-1)).toEqual({
      valid: false,
      error: {
        code: "activeTileCountTooLow",
        limit: 0,
        actual: -1
      }
    });

    expect(validateActiveTileCount(4)).toEqual({
      valid: false,
      error: {
        code: "activeTileLimitExceeded",
        limit: 3,
        actual: 4
      }
    });
  });

  it("rejects empty and oversized regions with typed errors", () => {
    expect(validateRegionSize({ width: 0, height: 300 })).toEqual({
      valid: false,
      error: {
        code: "regionEmpty",
        limit: 1,
        actual: 0
      }
    });

    expect(validateRegionSize({ width: 800, height: -1 })).toEqual({
      valid: false,
      error: {
        code: "regionEmpty",
        limit: 1,
        actual: -1
      }
    });

    expect(validateRegionSize({ width: 801, height: 600 })).toEqual({
      valid: false,
      error: {
        code: "regionWidthTooLarge",
        limit: 800,
        actual: 801
      }
    });

    expect(validateRegionSize({ width: 800, height: 601 })).toEqual({
      valid: false,
      error: {
        code: "regionHeightTooLarge",
        limit: 600,
        actual: 601
      }
    });
  });
});
