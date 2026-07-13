export const SMART_WATCH_CONSENT_VERSION = 5;
export const SMART_WATCH_CONSENT_KEY =
  "pebble.smart-watch-consent-version";
export const SMART_WATCH_INTERVAL_KEY = "pebble.smart-watch-interval-minutes";
export const SMART_WATCH_INTERVAL_OPTIONS = [1, 5, 30, 60] as const;
export const DEFAULT_SMART_WATCH_INTERVAL = 5;

export type SmartWatchIntervalMinutes =
  (typeof SMART_WATCH_INTERVAL_OPTIONS)[number];

export interface SmartWatchStatus {
  enabled: boolean;
  analysesCompleted: number;
  analysisIntervalMinutes: SmartWatchIntervalMinutes;
}

interface ConsentStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function hasSmartWatchConsent(storage: ConsentStorage): boolean {
  return (
    storage.getItem(SMART_WATCH_CONSENT_KEY) ===
    String(SMART_WATCH_CONSENT_VERSION)
  );
}

export function rememberSmartWatchConsent(storage: ConsentStorage): void {
  storage.setItem(
    SMART_WATCH_CONSENT_KEY,
    String(SMART_WATCH_CONSENT_VERSION)
  );
}

export function smartWatchInterval(storage: ConsentStorage): SmartWatchIntervalMinutes {
  const value = Number(storage.getItem(SMART_WATCH_INTERVAL_KEY));
  return isSmartWatchInterval(value) ? value : DEFAULT_SMART_WATCH_INTERVAL;
}

export function rememberSmartWatchInterval(
  storage: ConsentStorage,
  minutes: SmartWatchIntervalMinutes
): void {
  storage.setItem(SMART_WATCH_INTERVAL_KEY, String(minutes));
}

export function isSmartWatchInterval(
  value: number
): value is SmartWatchIntervalMinutes {
  return SMART_WATCH_INTERVAL_OPTIONS.some((minutes) => minutes === value);
}

export function smartWatchIntervalLabel(minutes: SmartWatchIntervalMinutes): string {
  return minutes === 60 ? "1 HOUR" : `${minutes} MIN`;
}

export function smartWatchTitle(status: SmartWatchStatus | null): string {
  if (!status) return "SEMANTIC SMART WATCH";
  return status.enabled
    ? `SEMANTIC SMART WATCH ON · AI MAX ONCE EVERY ${smartWatchIntervalLabel(status.analysisIntervalMinutes)}`
    : "SEMANTIC SMART WATCH OFF";
}
