import { describe, expect, it } from "vitest";
import {
  SMART_WATCH_CONSENT_KEY,
  hasSmartWatchConsent,
  rememberSmartWatchConsent,
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
    expect(storage.getItem(SMART_WATCH_CONSENT_KEY)).toBe("1");
    expect(hasSmartWatchConsent(storage)).toBe(true);
  });

  it("reports the bounded local alert budget", () => {
    expect(
      smartWatchTitle({
        enabled: true,
        notificationsSent: 3,
        sessionLimit: 24,
        remaining: 21
      })
    ).toContain("21/24");
  });
});
