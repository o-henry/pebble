import { PERFORMANCE_LIMITS } from "../performance/performanceLimits";
import type { PhysicalRegion } from "../region-selector/regionSelection";

export const PEBBLE_SESSION_UPDATED_EVENT = "pebble://session-updated";
export const PEBBLE_TILE_LABEL = "screenpebble-tile";
export const BROWSER_SESSION_STORAGE_KEY = "screenpebble.browser-session";

export interface PebbleSessionSnapshot {
  region: PhysicalRegion | null;
  windowOpen: boolean;
  privacyBlankActive: boolean;
  revision: number;
}

export const EMPTY_PEBBLE_SESSION: PebbleSessionSnapshot = {
  region: null,
  windowOpen: false,
  privacyBlankActive: false,
  revision: 0
};

export function isPebbleSessionSnapshot(
  value: unknown
): value is PebbleSessionSnapshot {
  if (!isRecord(value)) {
    return false;
  }

  return (
    (value.region === null || isPhysicalRegion(value.region)) &&
    typeof value.windowOpen === "boolean" &&
    typeof value.privacyBlankActive === "boolean" &&
    isSafeInteger(value.revision) &&
    value.revision >= 0
  );
}

export function advanceBrowserSession(
  current: PebbleSessionSnapshot,
  update: Partial<Omit<PebbleSessionSnapshot, "revision">>
): PebbleSessionSnapshot {
  return {
    ...current,
    ...update,
    revision: current.revision + 1
  };
}

export function newestSession(
  current: PebbleSessionSnapshot,
  incoming: PebbleSessionSnapshot
): PebbleSessionSnapshot {
  return incoming.revision >= current.revision ? incoming : current;
}

export function regionKey(region: PhysicalRegion): string {
  return [
    region.monitorId,
    region.x,
    region.y,
    region.width,
    region.height
  ].join(":");
}

export function browserSessionFromStorage(
  storage: Pick<Storage, "getItem">
): PebbleSessionSnapshot {
  const serialized = storage.getItem(BROWSER_SESSION_STORAGE_KEY);

  if (!serialized) {
    return EMPTY_PEBBLE_SESSION;
  }

  try {
    const parsed: unknown = JSON.parse(serialized);
    return isPebbleSessionSnapshot(parsed) ? parsed : EMPTY_PEBBLE_SESSION;
  } catch {
    return EMPTY_PEBBLE_SESSION;
  }
}

export function storeBrowserSession(
  storage: Pick<Storage, "setItem">,
  snapshot: PebbleSessionSnapshot
) {
  storage.setItem(BROWSER_SESSION_STORAGE_KEY, JSON.stringify(snapshot));
}

function isPhysicalRegion(value: unknown): value is PhysicalRegion {
  if (!isRecord(value)) {
    return false;
  }

  return (
    typeof value.monitorId === "string" &&
    value.monitorId.length > 0 &&
    value.monitorId.length <= 256 &&
    isSafeInteger(value.x) &&
    isSafeInteger(value.y) &&
    isSafeInteger(value.width) &&
    isSafeInteger(value.height) &&
    value.width >= 24 &&
    value.height >= 24 &&
    value.width <= PERFORMANCE_LIMITS.maxRegion.width &&
    value.height <= PERFORMANCE_LIMITS.maxRegion.height
  );
}

function isSafeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
