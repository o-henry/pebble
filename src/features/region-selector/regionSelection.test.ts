import { describe, expect, it } from "vitest";
import {
  selectRegion,
  type LogicalPoint,
  type MonitorGeometry,
  type RegionSelectionRequest
} from "./regionSelection";

describe("region selection", () => {
  it("maps a normal logical drag to a physical region", () => {
    const result = selectRegion(request(point(10, 20), point(210, 170)));

    expect(result).toMatchObject({
      ok: true,
      selection: {
        region: {
          monitorId: "main",
          x: 10,
          y: 20,
          width: 200,
          height: 150
        },
        warnings: []
      }
    });
  });

  it("normalizes reversed drag direction", () => {
    const result = selectRegion(request(point(210, 170), point(10, 20)));

    expect(result).toMatchObject({
      ok: true,
      selection: {
        region: {
          x: 10,
          y: 20,
          width: 200,
          height: 150
        }
      }
    });
  });

  it("rejects selections below the minimum region size", () => {
    const result = selectRegion(request(point(10, 20), point(20, 42)));

    expect(result).toEqual({
      ok: false,
      error: {
        code: "regionTooNarrow",
        limit: 24,
        actual: 10,
        message: "Selected region is too narrow."
      }
    });
  });

  it("rejects regions beyond the hard maximum size", () => {
    const result = selectRegion(request(point(0, 0), point(801, 600)));

    expect(result).toMatchObject({
      ok: false,
      error: {
        code: "regionWidthTooLarge",
        limit: 800,
        actual: 801
      }
    });
  });

  it("rejects out-of-range physical coordinates before creating a region", () => {
    const result = selectRegion(
      request(point(0, 0), point(100, 100), {
        id: "extreme",
        logicalOrigin: point(0, 0),
        logicalSize: { width: 1000, height: 800 },
        physicalOrigin: { x: 2147483647, y: 0 },
        scaleFactor: 1
      })
    );

    expect(result).toMatchObject({
      ok: false,
      error: {
        code: "regionCoordinateOutOfRange",
        limit: 2147483647
      }
    });
  });

  it("normalizes non-finite invalid scale issue values", () => {
    const result = selectRegion(
      request(point(0, 0), point(100, 100), {
        id: "invalid-scale",
        logicalOrigin: point(0, 0),
        logicalSize: { width: 1000, height: 800 },
        physicalOrigin: { x: 0, y: 0 },
        scaleFactor: Number.NaN
      })
    );

    expect(result).toEqual({
      ok: false,
      error: {
        code: "invalidScaleFactor",
        limit: 0,
        actual: 0,
        message: "Monitor scale factor must be greater than zero."
      }
    });
  });

  it("converts logical coordinates with the monitor scale factor", () => {
    const result = selectRegion(
      request(point(100.5, 40.5), point(300.25, 190.25), {
        id: "retina",
        logicalOrigin: point(100, 40),
        logicalSize: { width: 1000, height: 800 },
        physicalOrigin: { x: 1200, y: 400 },
        scaleFactor: 2
      })
    );

    expect(result).toMatchObject({
      ok: true,
      selection: {
        region: {
          x: 1201,
          y: 401,
          width: 400,
          height: 300
        }
      }
    });
  });

  it("preserves multi-monitor physical offsets", () => {
    const result = selectRegion(
      request(point(-1180, 90), point(-980, 210), {
        id: "left-display",
        logicalOrigin: point(-1280, 40),
        logicalSize: { width: 1280, height: 540 },
        physicalOrigin: { x: -2560, y: 80 },
        scaleFactor: 2
      })
    );

    expect(result).toMatchObject({
      ok: true,
      selection: {
        region: {
          monitorId: "left-display",
          x: -2360,
          y: 180,
          width: 400,
          height: 240
        }
      }
    });
  });

  it("returns recommended-limit warnings without rejecting the selection", () => {
    const result = selectRegion(request(point(0, 0), point(650, 320)));

    expect(result).toMatchObject({
      ok: true,
      selection: {
        warnings: [
          {
            code: "regionWidthAboveRecommended",
            limit: 600,
            actual: 650
          },
          {
            code: "regionHeightAboveRecommended",
            limit: 300,
            actual: 320
          }
        ]
      }
    });
  });

  it("rejects a drag outside the active monitor bounds", () => {
    const result = selectRegion(request(point(10, 10), point(1930, 200)));

    expect(result).toMatchObject({
      ok: false,
      error: { code: "selectionOutsideMonitor" }
    });
  });
});

function request(
  start: LogicalPoint,
  end: LogicalPoint,
  monitor: MonitorGeometry = defaultMonitor()
): RegionSelectionRequest {
  return {
    monitor,
    start,
    end
  };
}

function defaultMonitor(): MonitorGeometry {
  return {
    id: "main",
    logicalOrigin: point(0, 0),
    logicalSize: { width: 1920, height: 1080 },
    physicalOrigin: { x: 0, y: 0 },
    scaleFactor: 1
  };
}

function point(x: number, y: number): LogicalPoint {
  return { x, y };
}
