import { describe, expect, it } from "vitest";
import {
  SMART_WATCH_CONSENT_KEY,
  SMART_WATCH_INTERVAL_KEY,
  hasSmartWatchConsent,
  rememberSmartWatchConsent,
  rememberSmartWatchInterval,
  smartWatchInterval,
  smartWatchIntervalAtOffset,
  smartWatchTitle
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
    expect(storage.getItem(SMART_WATCH_CONSENT_KEY)).toBe("5");
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
    expect(
      smartWatchTitle({
        enabled: true,
        analysesCompleted: 12,
        analysisIntervalMinutes: 60
      })
    ).toContain("1 HOUR");
  });

  it("wraps interval keyboard navigation in both directions", () => {
    expect(smartWatchIntervalAtOffset(1, -1)).toBe(60);
    expect(smartWatchIntervalAtOffset(60, 1)).toBe(1);
    expect(smartWatchIntervalAtOffset(5, 2)).toBe(60);
  });
});
