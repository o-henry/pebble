import { describe, expect, it } from "vitest";
import {
  MAX_REGION_QUESTION_LENGTH,
  normalizedRegionQuestion
} from "./regionQuestion";

describe("region questions", () => {
  it("trims a valid explicit question", () => {
    expect(normalizedRegionQuestion("  What changed?  ")).toBe("What changed?");
  });

  it("rejects empty, oversized, and unsafe control input", () => {
    expect(normalizedRegionQuestion("   ")).toBeNull();
    expect(
      normalizedRegionQuestion("x".repeat(MAX_REGION_QUESTION_LENGTH + 1))
    ).toBeNull();
    expect(normalizedRegionQuestion("unsafe\u0000question")).toBeNull();
  });

  it("counts Unicode code points rather than UTF-16 units", () => {
    expect(normalizedRegionQuestion("한".repeat(MAX_REGION_QUESTION_LENGTH))).not.toBeNull();
  });
});
