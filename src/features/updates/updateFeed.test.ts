import { describe, expect, it } from "vitest";
import {
  mergeUpdateEntry,
  updateSignalLabel,
  type UpdateFeedSnapshot
} from "./updateFeed";

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

  it("formats structured Watch metadata separately from its summary", () => {
    expect(updateSignalLabel({
      kind: "match",
      region: "REGION 1",
      engine: "openAi",
      model: "gpt-5.6-terra",
      confidence: "high",
      durationMs: 1_240
    })).toBe("REGION 1 · MATCH · GPT-5.6-TERRA · HIGH · 1.2S");
  });

  it("labels a zero-token stuck signal without inventing a model", () => {
    expect(updateSignalLabel({
      kind: "stuck",
      region: "REGION 2",
      engine: "localVisual",
      confidence: "high"
    })).toBe("REGION 2 · STUCK · LOCAL VISUAL · HIGH");
  });
});
