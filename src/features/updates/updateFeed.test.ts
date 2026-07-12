import { describe, expect, it } from "vitest";
import { mergeUpdateEntry, type UpdateFeedSnapshot } from "./updateFeed";

describe("update feed", () => {
  it("keeps the newest unique entry first", () => {
    const snapshot: UpdateFeedSnapshot = {
      entries: [{
        id: 1,
        kind: "watch",
        summary: "OLD",
        occurredAt: "2026-07-11T10:00:00Z",
        saved: true
      }]
    };
    const next = mergeUpdateEntry(snapshot, {
      ...snapshot.entries[0],
      summary: "NEW"
    });
    expect(next.entries).toHaveLength(1);
    expect(next.entries[0].summary).toBe("NEW");
  });
});
