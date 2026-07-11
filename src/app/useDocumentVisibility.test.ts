import { describe, expect, it } from "vitest";
import { visibilityAllowsCapture } from "./useDocumentVisibility";

describe("visibilityAllowsCapture", () => {
  it("allows capture only while the Pebble document is visible", () => {
    expect(visibilityAllowsCapture("visible")).toBe(true);
    expect(visibilityAllowsCapture("hidden")).toBe(false);
  });
});
