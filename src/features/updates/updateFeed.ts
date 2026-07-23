export type UpdateKind = "watch";

export type WatchSignalKind =
  | "match"
  | "stuck"
  | "conflict"
  | "noFollowThrough"
  | "loop"
  | "waiting"
  | "analysisSkipped";

export type WatchSignalEngine =
  | "system"
  | "localOcr"
  | "localVisual"
  | "localCrossCheck"
  | "localFollowThrough"
  | "localVisualLoop"
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

export interface ChangeStory {
  id: string;
  entries: UpdateEntry[];
  regions: string[];
  startedAt: string;
  endedAt: string;
}

export type UpdateFeedItem =
  | { type: "entry"; entry: UpdateEntry }
  | { type: "story"; story: ChangeStory };

const CHANGE_STORY_WINDOW_MS = 5 * 60 * 1_000;
const CHANGE_STORY_ENTRY_LIMIT = 8;
const STORY_SIGNAL_KINDS = new Set<WatchSignalKind>([
  "match",
  "stuck",
  "conflict",
  "noFollowThrough",
  "loop"
]);

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

export function buildChangeStoryItems(
  entries: UpdateEntry[]
): UpdateFeedItem[] {
  const items: UpdateFeedItem[] = [];
  let candidate: UpdateEntry[] = [];

  const flush = () => {
    if (candidate.length >= 2) {
      const chronological = [...candidate].reverse();
      const oldest = chronological[0];
      const newest = chronological[chronological.length - 1];
      items.push({
        type: "story",
        story: {
          id: `story-${newest.id}-${oldest.id}`,
          entries: chronological,
          regions: storyRegions(chronological),
          startedAt: oldest.occurredAt,
          endedAt: newest.occurredAt
        }
      });
    } else if (candidate.length === 1) {
      items.push({ type: "entry", entry: candidate[0] });
    }
    candidate = [];
  };

  for (const entry of entries) {
    if (!isStoryEntry(entry)) {
      flush();
      items.push({ type: "entry", entry });
      continue;
    }
    const previous = candidate[candidate.length - 1];
    const gap = previous ? entryTime(previous) - entryTime(entry) : 0;
    const withinWindow = gap >= 0 && gap <= CHANGE_STORY_WINDOW_MS;
    if (!withinWindow || candidate.length >= CHANGE_STORY_ENTRY_LIMIT) {
      flush();
    }
    candidate.push(entry);
  }
  flush();
  return items;
}

export function changeStoryLabel(story: ChangeStory): string {
  const eventLabel = story.entries.length === 1 ? "EVENT" : "EVENTS";
  const regions = story.regions.length > 0 ? ` · ${story.regions.join(" + ")}` : "";
  return `CHANGE STORY · ${story.entries.length} ${eventLabel}${regions}`;
}

function isStoryEntry(entry: UpdateEntry): boolean {
  return Boolean(
    entry.signal &&
      STORY_SIGNAL_KINDS.has(entry.signal.kind) &&
      Number.isFinite(entryTime(entry))
  );
}

function entryTime(entry: UpdateEntry): number {
  return new Date(entry.occurredAt).getTime();
}

function storyRegions(entries: UpdateEntry[]): string[] {
  const regions: string[] = [];
  for (const entry of entries) {
    if (!entry.signal) continue;
    for (const region of [
      entry.signal.region,
      ...(entry.signal.relatedRegions ?? [])
    ]) {
      if (!regions.includes(region)) regions.push(region);
    }
  }
  return regions;
}

export function updateSignalLabel(signal: WatchSignal): string {
  const kind: Record<WatchSignalKind, string> = {
    match: "MATCH",
    stuck: "STUCK",
    conflict: "CONFLICT",
    noFollowThrough: "NO FOLLOW-THROUGH",
    loop: "LOOP",
    waiting: "WAITING",
    analysisSkipped: "ANALYSIS SKIPPED"
  };
  const engine: Record<WatchSignalEngine, string> = {
    system: "SYSTEM",
    localOcr: "LOCAL OCR",
    localVisual: "LOCAL VISUAL",
    localCrossCheck: "LOCAL CROSS-CHECK",
    localFollowThrough: "LOCAL FOLLOW-THROUGH",
    localVisualLoop: "LOCAL VISUAL LOOP",
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

export function isAttentionEntry(entry: UpdateEntry): boolean {
  return Boolean(
    entry.signal &&
      ["match", "stuck", "conflict", "noFollowThrough", "loop"].includes(
        entry.signal.kind
      )
  );
}
