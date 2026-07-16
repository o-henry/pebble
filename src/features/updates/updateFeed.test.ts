import { describe, expect, it } from "vitest";
import {
  buildChangeStoryItems,
  changeStoryLabel,
  mergeUpdateEntry,
  updateSignalLabel,
  type UpdateEntry,
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

  it("labels every region participating in a local conflict", () => {
    expect(updateSignalLabel({
      kind: "conflict",
      region: "REGION 1",
      relatedRegions: ["REGION 2"],
      engine: "localCrossCheck",
      confidence: "high"
    })).toBe("REGION 1 + REGION 2 · CONFLICT · LOCAL CROSS-CHECK · HIGH");
  });

  it("labels linked regions when a follow-through result is missing", () => {
    expect(updateSignalLabel({
      kind: "noFollowThrough",
      region: "REGION 1",
      relatedRegions: ["REGION 2"],
      engine: "localFollowThrough",
      confidence: "high"
    })).toBe(
      "REGION 1 + REGION 2 · NO FOLLOW-THROUGH · LOCAL FOLLOW-THROUGH · HIGH"
    );
  });

  it("labels a local visual loop without a provider model", () => {
    expect(updateSignalLabel({
      kind: "loop",
      region: "REGION 1",
      engine: "localVisualLoop",
      confidence: "high"
    })).toBe("REGION 1 · LOOP · LOCAL VISUAL LOOP · HIGH");
  });

  it("groups nearby meaningful signals into one chronological change story", () => {
    const entries = [
      storyEntry(3, "2026-07-16T10:04:00Z", "REGION 2", "conflict", ["REGION 1"]),
      storyEntry(2, "2026-07-16T10:02:00Z", "REGION 1", "match"),
      storyEntry(1, "2026-07-16T10:00:00Z", "REGION 1", "stuck")
    ];
    const items = buildChangeStoryItems(entries);

    expect(items).toHaveLength(1);
    expect(items[0].type).toBe("story");
    if (items[0].type !== "story") throw new Error("expected story");
    expect(items[0].story.entries.map((entry) => entry.id)).toEqual([1, 2, 3]);
    expect(items[0].story.regions).toEqual(["REGION 1", "REGION 2"]);
    expect(changeStoryLabel(items[0].story)).toBe(
      "CHANGE STORY · 3 EVENTS · REGION 1 + REGION 2"
    );
    expect(entries.map((entry) => entry.id)).toEqual([3, 2, 1]);
  });

  it("keeps operational messages and distant signals outside stories", () => {
    const items = buildChangeStoryItems([
      storyEntry(4, "2026-07-16T10:20:00Z", "REGION 1", "loop"),
      storyEntry(3, "2026-07-16T10:19:00Z", "REGION 1", "waiting"),
      storyEntry(2, "2026-07-16T10:04:00Z", "REGION 2", "match"),
      storyEntry(1, "2026-07-16T10:00:00Z", "REGION 2", "stuck")
    ]);

    expect(items.map((item) => item.type)).toEqual(["entry", "entry", "story"]);
  });

  it("bounds stories to eight events and rejects invalid or reversed time", () => {
    const entries = Array.from({ length: 9 }, (_, index) =>
      storyEntry(
        9 - index,
        `2026-07-16T10:${String(8 - index).padStart(2, "0")}:00Z`,
        "REGION 1",
        "match"
      )
    );
    const bounded = buildChangeStoryItems(entries);
    expect(bounded.map((item) => item.type)).toEqual(["story", "entry"]);
    if (bounded[0].type !== "story") throw new Error("expected bounded story");
    expect(bounded[0].story.entries).toHaveLength(8);

    const invalid = buildChangeStoryItems([
      storyEntry(2, "INVALID", "REGION 1", "match"),
      storyEntry(1, "2026-07-16T10:00:00Z", "REGION 1", "match")
    ]);
    expect(invalid.map((item) => item.type)).toEqual(["entry", "entry"]);

    const reversed = buildChangeStoryItems([
      storyEntry(1, "2026-07-16T10:00:00Z", "REGION 1", "match"),
      storyEntry(2, "2026-07-16T10:01:00Z", "REGION 1", "match")
    ]);
    expect(reversed.map((item) => item.type)).toEqual(["entry", "entry"]);
  });
});

function storyEntry(
  id: number,
  occurredAt: string,
  region: string,
  kind: NonNullable<UpdateEntry["signal"]>["kind"],
  relatedRegions?: string[]
): UpdateEntry {
  return {
    id,
    kind: "watch",
    summary: `SUMMARY ${id}`,
    occurredAt,
    saved: true,
    signal: {
      kind,
      region,
      relatedRegions,
      engine: kind === "waiting" ? "system" : "localVisual"
    }
  };
}
