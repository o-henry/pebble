import { describe, expect, it } from "vitest";
import {
  MAX_REGION_QUESTION_LENGTH,
  defaultAiModelLabel,
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

  it("labels the configured balanced models", () => {
    expect(defaultAiModelLabel("openAi")).toBe("GPT-5.6-TERRA");
    expect(defaultAiModelLabel("claude")).toBe("CLAUDE SONNET 5");
  });
});
