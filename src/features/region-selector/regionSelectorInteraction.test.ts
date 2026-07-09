import { describe, expect, it } from "vitest";
import {
  DEFAULT_SELECTOR_MONITOR,
  INITIAL_REGION_SELECTOR_STATE,
  canBeginRegionDrag,
  dimensionLabel,
  dragRect,
  regionSelectorReducer
} from "./regionSelectorInteraction";

describe("region selector interaction", () => {
  it("creates a validated region from a normal drag", () => {
    const dragging = regionSelectorReducer(INITIAL_REGION_SELECTOR_STATE, {
      type: "begin",
      point: { x: 10, y: 20 },
      monitor: DEFAULT_SELECTOR_MONITOR
    });
    const ready = regionSelectorReducer(dragging, {
      type: "finish",
      point: { x: 210, y: 170 }
    });

    expect(ready).toMatchObject({
      status: "ready",
      result: {
        ok: true,
        selection: {
          region: {
            x: 10,
            y: 20,
            width: 200,
            height: 150
          }
        }
      }
    });
  });

  it("cancels the drag when escape is modeled as cancel", () => {
    const dragging = regionSelectorReducer(INITIAL_REGION_SELECTOR_STATE, {
      type: "begin",
      point: { x: 10, y: 20 },
      monitor: DEFAULT_SELECTOR_MONITOR
    });
    const cancelled = regionSelectorReducer(dragging, { type: "cancel" });

    expect(cancelled).toMatchObject({
      status: "cancelled",
      start: null,
      current: null,
      result: null
    });
  });

  it("blocks drag start until monitor geometry is ready", () => {
    expect(canBeginRegionDrag(null)).toBe(false);
    expect(canBeginRegionDrag(DEFAULT_SELECTOR_MONITOR)).toBe(true);
  });

  it("returns a warning for large but valid selections", () => {
    const dragging = regionSelectorReducer(INITIAL_REGION_SELECTOR_STATE, {
      type: "begin",
      point: { x: 0, y: 0 },
      monitor: DEFAULT_SELECTOR_MONITOR
    });
    const ready = regionSelectorReducer(dragging, {
      type: "finish",
      point: { x: 650, y: 320 }
    });

    expect(ready.result).toMatchObject({
      ok: true,
      selection: {
        warnings: [
          { code: "regionWidthAboveRecommended" },
          { code: "regionHeightAboveRecommended" }
        ]
      }
    });
  });

  it("uses monitor physical origin and scale for final selection", () => {
    const dragging = regionSelectorReducer(INITIAL_REGION_SELECTOR_STATE, {
      type: "begin",
      point: { x: 10, y: 20 },
      monitor: {
        id: "main-display",
        logicalOrigin: { x: 0, y: 0 },
        physicalOrigin: { x: 1000, y: 500 },
        scaleFactor: 2
      }
    });
    const ready = regionSelectorReducer(dragging, {
      type: "finish",
      point: { x: 110, y: 120 }
    });

    expect(ready.result).toMatchObject({
      ok: true,
      selection: {
        region: {
          monitorId: "main-display",
          x: 1020,
          y: 540,
          width: 200,
          height: 200
        }
      }
    });
  });

  it("keeps draft dimensions independent from drag direction", () => {
    const rect = dragRect({ x: 210, y: 170 }, { x: 10, y: 20 });

    expect(rect).toEqual({
      x: 10,
      y: 20,
      width: 200,
      height: 150
    });
    expect(dimensionLabel(rect)).toBe("200 x 150");
  });
});
