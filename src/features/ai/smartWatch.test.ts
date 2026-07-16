import { describe, expect, it } from "vitest";
import {
  SMART_WATCH_CONSENT_KEY,
  SMART_WATCH_INTERVAL_KEY,
  hasSmartWatchConsent,
  rememberSmartWatchConsent,
  rememberSmartWatchInterval,
  smartWatchInterval,
  smartWatchIntervalAtOffset,
  smartWatchStatusSegments,
  smartWatchTargetSegments,
  smartWatchTitle,
  type SmartWatchStatus
} from "./smartWatch";

function memoryStorage() {
  const values = new Map<string, string>();
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value)
  };
}

describe("smart watch consent", () => {
  it("requires the current notice before enabling", () => {
    const storage = memoryStorage();
    expect(hasSmartWatchConsent(storage)).toBe(false);
    rememberSmartWatchConsent(storage);
    expect(storage.getItem(SMART_WATCH_CONSENT_KEY)).toBe("7");
    expect(hasSmartWatchConsent(storage)).toBe(true);
  });

  it("stores only a supported watch interval", () => {
    const storage = memoryStorage();
    expect(smartWatchInterval(storage)).toBe(5);
    rememberSmartWatchInterval(storage, 30);
    expect(storage.getItem(SMART_WATCH_INTERVAL_KEY)).toBe("30");
    expect(smartWatchInterval(storage)).toBe(30);
    storage.setItem(SMART_WATCH_INTERVAL_KEY, "2");
    expect(smartWatchInterval(storage)).toBe(5);
  });

  it("reports the selected semantic analysis interval", () => {
    const status: SmartWatchStatus = {
        enabled: true,
        targetCount: 1,
        targets: [{
          id: "watch-1",
          name: "REGION 1",
          current: true,
          analysesCompleted: 12,
          localMatchesCompleted: 0,
          suppressedEvents: 0,
          analysisIntervalMinutes: 60,
          provider: "openAi",
          model: "gpt-5.6-terra",
          aiFallbackEnabled: true,
          evaluationMode: "ai",
          ruleSummary: "AI SEMANTIC MATCH"
        }],
        analysesCompleted: 12,
        localMatchesCompleted: 0,
        suppressedEvents: 0,
        analysisIntervalMinutes: 60,
        provider: "openAi",
        model: "gpt-5.6-terra",
        aiFallbackEnabled: true,
        customIntent: true,
        watchingFor: "NOTIFY ME ABOUT A MEANINGFUL CHANGE",
        evaluationMode: "ai",
        ruleSummary: "AI SEMANTIC MATCH",
        captureScope: "selectedRegionOnly",
        storagePolicy: "memoryOnly",
        imagesSaved: false,
        ocrSaved: false
      };
    expect(smartWatchTitle(status)).toContain("1 HOUR");
    expect(smartWatchStatusSegments(status)).toEqual([
      "WATCHING FOR · AI SEMANTIC MATCH",
      "OPENAI · GPT-5.6-TERRA · AI MAX 1 HOUR",
      "12 AI RUNS",
      "SELECTED REGION ONLY · MEMORY ONLY · NOTHING SAVED"
    ]);
    expect(smartWatchTargetSegments(status.targets[0])).toEqual([
      "REGION 1 · AI SEMANTIC MATCH",
      "OPENAI · GPT-5.6-TERRA · AI MAX 1 HOUR",
      "12 AI RUNS"
    ]);
  });

  it("makes zero-token local watch explicit", () => {
    expect(smartWatchTargetSegments({
      id: "watch-1",
      name: "REGION 1",
      current: true,
      analysesCompleted: 0,
      localMatchesCompleted: 3,
      suppressedEvents: 2,
      analysisIntervalMinutes: 5,
      provider: "openAi",
      model: "gpt-5.6-terra",
      aiFallbackEnabled: false,
      evaluationMode: "local",
      ruleSummary: "TEXT APPEARS: error"
    })).toEqual([
      "REGION 1 · TEXT APPEARS: error",
      "LOCAL OCR ONLY · NO AI USAGE",
      "3 MATCHES",
      "2 REPEATS HIDDEN"
    ]);
  });

  it("wraps interval keyboard navigation in both directions", () => {
    expect(smartWatchIntervalAtOffset(1, -1)).toBe(60);
    expect(smartWatchIntervalAtOffset(60, 1)).toBe(1);
    expect(smartWatchIntervalAtOffset(5, 2)).toBe(60);
  });
});
