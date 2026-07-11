import { describe, expect, it } from "vitest";
import { captureErrorMessage, liveFrameState } from "./liveTilePresentation";

describe("live tile presentation", () => {
  it("reports the visible capture state", () => {
    expect(liveFrameState("live", false, null)).toBe("STARTING");
    expect(liveFrameState("live", true, null)).toBe("LIVE");
    expect(liveFrameState("paused", true, null)).toBe("PAUSED");
    expect(liveFrameState("blanked", false, null)).toBe("HIDDEN");
    expect(liveFrameState("live", true, "failed")).toBe("NEEDS ATTENTION");
  });

  it("turns permission failures into a recoverable instruction", () => {
    expect(captureErrorMessage("permission denied")).toContain("System Settings");
    expect(captureErrorMessage("other failure")).toBe("other failure");
  });
});
