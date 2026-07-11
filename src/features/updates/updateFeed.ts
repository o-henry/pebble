export type UpdateKind = "watch" | "source";

export interface UpdateEntry {
  id: number;
  kind: UpdateKind;
  summary: string;
  sourceUrl: string | null;
  occurredAt: string;
  saved: boolean;
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
