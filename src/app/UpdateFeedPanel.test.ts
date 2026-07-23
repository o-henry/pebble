import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { UpdateEntry } from "../features/updates/updateFeed";
import { isAttentionEntry } from "../features/updates/updateFeed";
import { UpdateFeedList, UpdateFeedPanel } from "./UpdateFeedPanel";

describe("UpdateFeedPanel", () => {
  it("keeps the divider without rendering empty-state placeholders", () => {
    const markup = renderToStaticMarkup(createElement(UpdateFeedPanel));

    expect(markup).toContain("update-feed--empty");
    expect(markup).not.toContain("UPDATES 0");
    expect(markup).not.toContain("DOWNLOADS/PEBBLE");
    expect(markup).not.toContain("NO SAVED UPDATES YET");
  });

  it("renders nearby meaningful signals as one chronological change story", () => {
    const entries: UpdateEntry[] = [
      entry(3, "2026-07-16T10:04:00Z", "THIRD", "REGION 2"),
      entry(2, "2026-07-16T10:02:00Z", "SECOND", "REGION 1"),
      entry(1, "2026-07-16T10:00:00Z", "FIRST", "REGION 1")
    ];
    const markup = renderToStaticMarkup(
      createElement(UpdateFeedList, { entries })
    );

    expect(markup).toContain(
      "CHANGE STORY · 3 EVENTS · REGION 1 + REGION 2"
    );
    expect(markup).toContain('class="update-feed__timeline"');
    expect(markup.indexOf("FIRST")).toBeLessThan(markup.indexOf("SECOND"));
    expect(markup.indexOf("SECOND")).toBeLessThan(markup.indexOf("THIRD"));
  });

  it("keeps meaningful alerts visible without expanding routine status entries", () => {
    expect(isAttentionEntry(entry(1, "2026-07-16T10:00:00Z", "MATCH", "REGION 1")))
      .toBe(true);
    expect(isAttentionEntry({
      ...entry(2, "2026-07-16T10:01:00Z", "WAITING", "REGION 1"),
      signal: {
        kind: "waiting",
        region: "REGION 1",
        engine: "system"
      }
    })).toBe(false);
  });
});

function entry(
  id: number,
  occurredAt: string,
  summary: string,
  region: string
): UpdateEntry {
  return {
    id,
    kind: "watch",
    summary,
    occurredAt,
    saved: true,
    signal: {
      kind: "match",
      region,
      engine: "localOcr",
      confidence: "high"
    }
  };
}
