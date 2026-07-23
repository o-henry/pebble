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
    expect(storage.getItem(SMART_WATCH_CONSENT_KEY)).toBe("11");
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
          watchingFor: "NOTIFY ME ABOUT A MEANINGFUL CHANGE",
          current: true,
          analysesCompleted: 12,
          localMatchesCompleted: 0,
          suppressedEvents: 0,
          analysisIntervalMinutes: 60,
          provider: "openAi",
          model: "gpt-5.6-terra",
          aiFallbackEnabled: true,
          evaluationMode: "ai",
          localEngine: null,
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
        localEngine: null,
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
      watchingFor: "TEXT APPEARS: error",
      current: true,
      analysesCompleted: 0,
      localMatchesCompleted: 3,
      suppressedEvents: 2,
      analysisIntervalMinutes: 5,
      provider: "openAi",
      model: "gpt-5.6-terra",
      aiFallbackEnabled: false,
      evaluationMode: "local",
      localEngine: "ocr",
      ruleSummary: "TEXT APPEARS: error"
    })).toEqual([
      "REGION 1 · TEXT APPEARS: error",
      "LOCAL OCR ONLY · NO AI USAGE",
      "3 MATCHES",
      "2 REPEATS HIDDEN"
    ]);
  });

  it("describes the default automatic detectors and optional AI fallback", () => {
    expect(smartWatchTargetSegments({
      id: "watch-1",
      name: "REGION 1",
      watchingFor: "NOTIFY ME ABOUT ANY MEANINGFUL CONTENT CHANGE",
      current: true,
      analysesCompleted: 2,
      localMatchesCompleted: 4,
      suppressedEvents: 0,
      analysisIntervalMinutes: 5,
      provider: "openAi",
      model: "gpt-5.6-terra",
      aiFallbackEnabled: true,
      evaluationMode: "local",
      localEngine: "ocr",
      ruleSummary: "AUTOMATIC WATCH"
    })).toEqual([
      "REGION 1 · AUTOMATIC WATCH",
      "AUTO LOCAL · ERROR + PROGRESS + QUEUE + STUCK + LOOP · OPENAI · GPT-5.6-TERRA · AI MAX 5 MIN",
      "4 LOCAL MATCHES · 2 AI RUNS"
    ]);
  });

  it("describes stuck detection as local visual work with no OCR or AI", () => {
    expect(smartWatchTargetSegments({
      id: "watch-1",
      name: "REGION 1",
      watchingFor: "TELL ME WHEN PROGRESS STOPS",
      current: true,
      analysesCompleted: 0,
      localMatchesCompleted: 1,
      suppressedEvents: 0,
      analysisIntervalMinutes: 5,
      provider: "openAi",
      model: "gpt-5.6-terra",
      aiFallbackEnabled: false,
      evaluationMode: "local",
      localEngine: "visualStability",
      ruleSummary: "NO PROGRESS AFTER ACTIVITY"
    })).toEqual([
      "REGION 1 · NO PROGRESS AFTER ACTIVITY",
      "LOCAL VISUAL ONLY · ALERT AFTER 5 MIN WITHOUT PROGRESS · NO OCR · NO AI USAGE",
      "1 MATCHES"
    ]);
  });

  it("describes cross-region checks as local OCR state comparison", () => {
    expect(smartWatchTargetSegments({
      id: "watch-1",
      name: "REGION 1",
      watchingFor: "TELL ME WHEN REGIONS DISAGREE",
      current: true,
      analysesCompleted: 0,
      localMatchesCompleted: 1,
      suppressedEvents: 0,
      analysisIntervalMinutes: 5,
      provider: "openAi",
      model: "gpt-5.6-terra",
      aiFallbackEnabled: false,
      evaluationMode: "local",
      localEngine: "crossRegionOcr",
      ruleSummary: "CROSS-REGION STATUS CONFLICT"
    })).toEqual([
      "REGION 1 · CROSS-REGION STATUS CONFLICT",
      "LOCAL CROSS-CHECK · USE ON 2+ REGIONS · OCR STATE ONLY · NO AI USAGE",
      "1 MATCHES"
    ]);
  });

  it("describes both follow-through roles without implying OCR or AI", () => {
    const base = {
      id: "watch-1",
      name: "REGION 1",
      watchingFor: "TELL ME WHEN FOLLOW-THROUGH IS MISSING",
      current: true,
      analysesCompleted: 0,
      localMatchesCompleted: 0,
      suppressedEvents: 0,
      analysisIntervalMinutes: 5 as const,
      provider: "openAi" as const,
      model: "gpt-5.6-terra",
      aiFallbackEnabled: false,
      evaluationMode: "local" as const
    };
    expect(smartWatchTargetSegments({
      ...base,
      localEngine: "followThroughTrigger",
      ruleSummary: "FOLLOW THROUGH TRIGGER"
    })).toEqual([
      "REGION 1 · FOLLOW THROUGH TRIGGER",
      "LOCAL FOLLOW START · EXPECT RESULT WITHIN 5 MIN · NO OCR · NO AI USAGE",
      "0 MATCHES"
    ]);
    expect(smartWatchTargetSegments({
      ...base,
      name: "REGION 2",
      localEngine: "followThroughResult",
      ruleSummary: "FOLLOW THROUGH RESULT"
    })).toEqual([
      "REGION 2 · FOLLOW THROUGH RESULT",
      "LOCAL FOLLOW RESULT · RESPONDS TO FOLLOW START · NO OCR · NO AI USAGE",
      "0 MATCHES"
    ]);
  });

  it("describes loop detection as fixed local fingerprint analysis", () => {
    expect(smartWatchTargetSegments({
      id: "watch-1",
      name: "REGION 1",
      watchingFor: "TELL ME WHEN A VISUAL LOOP REPEATS",
      current: true,
      analysesCompleted: 0,
      localMatchesCompleted: 1,
      suppressedEvents: 0,
      analysisIntervalMinutes: 5,
      provider: "openAi",
      model: "gpt-5.6-terra",
      aiFallbackEnabled: false,
      evaluationMode: "local",
      localEngine: "visualLoop",
      ruleSummary: "REPEATING VISUAL LOOP"
    })).toEqual([
      "REGION 1 · REPEATING VISUAL LOOP",
      "LOCAL LOOP DETECTOR · 2-4 STEPS · ALERT AFTER 3 CYCLES · NO OCR · NO AI USAGE",
      "1 MATCHES"
    ]);
  });

  it("wraps interval keyboard navigation in both directions", () => {
    expect(smartWatchIntervalAtOffset(1, -1)).toBe(60);
    expect(smartWatchIntervalAtOffset(60, 1)).toBe(1);
    expect(smartWatchIntervalAtOffset(5, 2)).toBe(60);
  });
});
