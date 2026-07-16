export type UpdateKind = "watch";

export type WatchSignalKind =
  | "match"
  | "stuck"
  | "conflict"
  | "noFollowThrough"
  | "waiting"
  | "analysisSkipped";

export type WatchSignalEngine =
  | "system"
  | "localOcr"
  | "localVisual"
  | "localCrossCheck"
  | "localFollowThrough"
  | "openAi"
  | "claude";

export type WatchSignalConfidence = "low" | "medium" | "high";

export interface WatchSignal {
  kind: WatchSignalKind;
  region: string;
  relatedRegions?: string[];
  engine: WatchSignalEngine;
  model?: string;
  confidence?: WatchSignalConfidence;
  durationMs?: number;
}

export interface UpdateEntry {
  id: number;
  kind: UpdateKind;
  summary: string;
  occurredAt: string;
  saved: boolean;
  signal?: WatchSignal;
}

export interface UpdateFeedSnapshot {
  entries: UpdateEntry[];
}

export function mergeUpdateEntry(
  snapshot: UpdateFeedSnapshot,
  entry: UpdateEntry
): UpdateFeedSnapshot {
  return {
    ...snapshot,
    entries: [
      entry,
      ...snapshot.entries.filter((current) => current.id !== entry.id)
    ].slice(0, 100)
  };
}

export function formatUpdateTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "UNKNOWN";
  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit"
  }).format(date);
}

export function updateSignalLabel(signal: WatchSignal): string {
  const kind: Record<WatchSignalKind, string> = {
    match: "MATCH",
    stuck: "STUCK",
    conflict: "CONFLICT",
    noFollowThrough: "NO FOLLOW-THROUGH",
    waiting: "WAITING",
    analysisSkipped: "ANALYSIS SKIPPED"
  };
  const engine: Record<WatchSignalEngine, string> = {
    system: "SYSTEM",
    localOcr: "LOCAL OCR",
    localVisual: "LOCAL VISUAL",
    localCrossCheck: "LOCAL CROSS-CHECK",
    localFollowThrough: "LOCAL FOLLOW-THROUGH",
    openAi: "OPENAI",
    claude: "CLAUDE"
  };
  const source = signal.model?.toUpperCase() ?? engine[signal.engine];
  const regions = [signal.region, ...(signal.relatedRegions ?? [])].join(" + ");
  const segments = [regions, kind[signal.kind], source];
  if (signal.confidence) segments.push(signal.confidence.toUpperCase());
  if (signal.durationMs !== undefined) {
    segments.push(`${(signal.durationMs / 1_000).toFixed(1)}S`);
  }
  return segments.join(" · ");
}
