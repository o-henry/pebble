import type { AiProvider } from "./regionQuestion";

export const SMART_WATCH_CONSENT_VERSION = 7;
export const SMART_WATCH_CONSENT_KEY =
  "pebble.smart-watch-consent-version";
export const SMART_WATCH_INTERVAL_KEY = "pebble.smart-watch-interval-minutes";
export const SMART_WATCH_INTERVAL_OPTIONS = [1, 5, 30, 60] as const;
export const DEFAULT_SMART_WATCH_INTERVAL = 5;

export type SmartWatchIntervalMinutes =
  (typeof SMART_WATCH_INTERVAL_OPTIONS)[number];

export interface SmartWatchStatus {
  enabled: boolean;
  targetCount: number;
  targets: SmartWatchTargetStatus[];
  analysesCompleted: number;
  localMatchesCompleted: number;
  suppressedEvents: number;
  analysisIntervalMinutes: SmartWatchIntervalMinutes;
  provider: AiProvider;
  model: string;
  customIntent: boolean;
  watchingFor: string | null;
  evaluationMode: "local" | "ai";
  ruleSummary: string;
  captureScope: "selectedRegionOnly";
  storagePolicy: "memoryOnly";
  imagesSaved: false;
  ocrSaved: false;
}

export interface SmartWatchTargetStatus {
  id: string;
  name: string;
  current: boolean;
  analysesCompleted: number;
  localMatchesCompleted: number;
  suppressedEvents: number;
  analysisIntervalMinutes: SmartWatchIntervalMinutes;
  provider: AiProvider;
  model: string;
  evaluationMode: "local" | "ai";
  ruleSummary: string;
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

export function smartWatchIntervalAtOffset(
  current: SmartWatchIntervalMinutes,
  offset: number
): SmartWatchIntervalMinutes {
  const currentIndex = SMART_WATCH_INTERVAL_OPTIONS.indexOf(current);
  const optionCount = SMART_WATCH_INTERVAL_OPTIONS.length;
  const nextIndex = (currentIndex + offset % optionCount + optionCount) % optionCount;
  return SMART_WATCH_INTERVAL_OPTIONS[nextIndex];
}

export function smartWatchTitle(status: SmartWatchStatus | null): string {
  if (!status) return "SMART WATCH";
  if (!status.enabled) {
    return status.targetCount > 0
      ? `${status.targetCount} WATCH REGIONS ACTIVE · CURRENT REGION OFF`
      : "SMART WATCH OFF";
  }
  return status.evaluationMode === "local"
    ? `LOCAL WATCH ON · ${status.ruleSummary}`
    : `AI WATCH ON · MAX ONCE EVERY ${smartWatchIntervalLabel(status.analysisIntervalMinutes)}`;
}

export function smartWatchTargetSegments(target: SmartWatchTargetStatus): string[] {
  const engine = target.evaluationMode === "local"
    ? `${target.provider === "openAi" ? "OPENAI" : "CLAUDE"} · ${target.model.toUpperCase()} · LOCAL OCR FIRST · AI ONLY WHEN OCR CANNOT DECIDE · MAX ${smartWatchIntervalLabel(target.analysisIntervalMinutes)}`
    : `${target.provider === "openAi" ? "OPENAI" : "CLAUDE"} · ${target.model.toUpperCase()} · AI MAX ${smartWatchIntervalLabel(target.analysisIntervalMinutes)}`;
  const completed = target.evaluationMode === "local"
    ? `${target.localMatchesCompleted} MATCHES`
    : `${target.analysesCompleted} AI RUNS`;
  return [
    `${target.name} · ${target.ruleSummary}`,
    engine,
    completed,
    ...(target.suppressedEvents > 0
      ? [`${target.suppressedEvents} REPEATS HIDDEN`]
      : [])
  ];
}

export function smartWatchStatusSegments(status: SmartWatchStatus): string[] {
  if (!status.enabled) return [];
  const engine = status.evaluationMode === "local"
    ? `${status.provider === "openAi" ? "OPENAI" : "CLAUDE"} · ${status.model.toUpperCase()} · LOCAL OCR FIRST · AI ONLY WHEN OCR CANNOT DECIDE · MAX ${smartWatchIntervalLabel(status.analysisIntervalMinutes)}`
    : `${status.provider === "openAi" ? "OPENAI" : "CLAUDE"} · ${status.model.toUpperCase()} · AI MAX ${smartWatchIntervalLabel(status.analysisIntervalMinutes)}`;
  const completed = status.evaluationMode === "local"
    ? `${status.localMatchesCompleted} MATCHES`
    : `${status.analysesCompleted} AI RUNS`;
  return [
    `WATCHING FOR · ${status.ruleSummary}`,
    engine,
    completed,
    ...(status.suppressedEvents > 0
      ? [`${status.suppressedEvents} REPEATS HIDDEN`]
      : []),
    "SELECTED REGION ONLY · MEMORY ONLY · NOTHING SAVED"
  ];
}
