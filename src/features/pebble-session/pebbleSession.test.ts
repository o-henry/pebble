import { describe, expect, it } from "vitest";
import {
  BROWSER_SESSION_STORAGE_KEY,
  EMPTY_PEBBLE_SESSION,
  advanceBrowserSession,
  browserSessionFromStorage,
  isPebbleSessionSnapshot,
  newestSession,
  regionKey,
  storeBrowserSession,
  type PebbleSessionSnapshot
} from "./pebbleSession";

const ACTIVE_SESSION: PebbleSessionSnapshot = {
  region: {
    monitorId: "Built-in Retina Display",
    x: 120,
    y: 80,
    width: 420,
    height: 220
  },
  windowOpen: true,
  privacyBlankActive: false,
  revision: 1
};

describe("pebble session", () => {
  it("accepts a bounded selected region", () => {
    expect(isPebbleSessionSnapshot(ACTIVE_SESSION)).toBe(true);
    expect(regionKey(ACTIVE_SESSION.region!)).toBe(
      "Built-in Retina Display:120:80:420:220"
    );
  });

  it("rejects malformed and oversized event payloads", () => {
    expect(
      isPebbleSessionSnapshot({
        ...ACTIVE_SESSION,
        region: { ...ACTIVE_SESSION.region, width: 801 }
      })
    ).toBe(false);
    expect(
      isPebbleSessionSnapshot({
        ...ACTIVE_SESSION,
        privacyBlankActive: "false"
      })
    ).toBe(false);
  });

  it("stores only the compact browser preview session", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value)
    };

    storeBrowserSession(storage, ACTIVE_SESSION);

    expect(browserSessionFromStorage(storage)).toEqual(ACTIVE_SESSION);
    expect(values.get(BROWSER_SESSION_STORAGE_KEY)).not.toContain("frame");
  });

  it("falls back safely when browser preview storage is invalid", () => {
    const storage = { getItem: () => "{invalid" };

    expect(browserSessionFromStorage(storage)).toBe(EMPTY_PEBBLE_SESSION);
  });

  it("keeps a newer event when an older snapshot arrives later", () => {
    const newer = advanceBrowserSession(ACTIVE_SESSION, {
      privacyBlankActive: true
    });

    expect(newestSession(newer, ACTIVE_SESSION)).toBe(newer);
    expect(newer.revision).toBe(2);
  });
});
