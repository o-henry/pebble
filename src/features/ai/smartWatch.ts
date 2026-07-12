export const SMART_WATCH_CONSENT_VERSION = 3;
export const SMART_WATCH_CONSENT_KEY =
  "pebble.smart-watch-consent-version";

export interface SmartWatchStatus {
  enabled: boolean;
  notificationsSent: number;
  sessionLimit: number;
  remaining: number;
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

export function smartWatchTitle(status: SmartWatchStatus | null): string {
  if (!status) return "SEMANTIC SMART WATCH";
  return status.enabled
    ? `SEMANTIC SMART WATCH ON · ${status.remaining}/${status.sessionLimit} ANALYSES LEFT`
    : "SEMANTIC SMART WATCH OFF";
}
