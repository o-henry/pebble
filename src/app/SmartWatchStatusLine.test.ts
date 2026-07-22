import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { SmartWatchStatus } from "../features/ai/smartWatch";
import { SmartWatchStatusLine } from "./SmartWatchStatusLine";

const status: SmartWatchStatus = {
  enabled: true,
  targetCount: 1,
  targets: [{
    id: "watch-1",
    name: "REGION 1",
    watchingFor: "TELL ME WHEN A PRICE REVERSES",
    current: true,
    analysesCompleted: 2,
    localMatchesCompleted: 0,
    suppressedEvents: 0,
    analysisIntervalMinutes: 1,
    provider: "openAi",
    model: "gpt-5.6-terra",
    aiFallbackEnabled: true,
    evaluationMode: "ai",
    localEngine: null,
    ruleSummary: "AI SEMANTIC MATCH"
  }],
  analysesCompleted: 2,
  localMatchesCompleted: 0,
  suppressedEvents: 0,
  analysisIntervalMinutes: 1,
  provider: "openAi",
  model: "gpt-5.6-terra",
  aiFallbackEnabled: true,
  customIntent: true,
  watchingFor: "TELL ME WHEN A PRICE REVERSES",
  evaluationMode: "ai",
  localEngine: null,
  ruleSummary: "AI SEMANTIC MATCH",
  captureScope: "selectedRegionOnly",
  storagePolicy: "memoryOnly",
  imagesSaved: false,
  ocrSaved: false
};

describe("smart watch status line", () => {
  it("keeps the active intent, AI path, cadence, and privacy scope visible", () => {
    const markup = renderToStaticMarkup(createElement(SmartWatchStatusLine, {
      status,
      disabled: false,
      onRemove: () => undefined
    }));

    expect(markup).toContain("TELL ME WHEN A PRICE REVERSES");
    expect(markup).toContain("OPENAI · GPT-5.6-TERRA · AI MAX 1 MIN");
    expect(markup).toContain("FRAMES STAY IN MEMORY");
    expect(markup).toContain("JOURNAL DETAILS REDACTED");
  });
});
