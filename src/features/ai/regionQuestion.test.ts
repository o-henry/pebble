import { describe, expect, it } from "vitest";
import {
  MAX_REGION_QUESTION_LENGTH,
  aiAccessLabel,
  defaultAiModelLabel,
  selectedAiModel,
  rememberAiModel,
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

  it("makes the active billing path explicit", () => {
    expect(aiAccessLabel("apiKey")).toBe("API BILLING");
    expect(aiAccessLabel("subscription")).toBe("SUBSCRIPTION");
    expect(aiAccessLabel("account")).toBe("ACCOUNT");
    expect(aiAccessLabel(null)).toBe("");
  });

  it("remembers only an available model for each provider", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value)
    };
    const models = [
      { id: "gpt-5.6-sol", label: "SOL" },
      { id: "gpt-5.6-terra", label: "TERRA" }
    ];

    expect(selectedAiModel("openAi", models, storage)).toBe("gpt-5.6-terra");
    rememberAiModel("openAi", "gpt-5.6-sol", storage);
    expect(selectedAiModel("openAi", models, storage)).toBe("gpt-5.6-sol");
    rememberAiModel("openAi", "not-available", storage);
    expect(selectedAiModel("openAi", models, storage)).toBe("gpt-5.6-terra");
  });
});
