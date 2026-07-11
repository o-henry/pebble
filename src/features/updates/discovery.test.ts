import { describe, expect, it } from "vitest";
import {
  filterDiscoveryItems,
  formatDiscoveryMeta,
  type DiscoveryItem
} from "./discovery";

const items: DiscoveryItem[] = [
  {
    id: "news:1",
    category: "news",
    title: "NEWS",
    source: "BBC WORLD",
    url: "https://example.com/news",
    score: null,
    comments: null
  },
  {
    id: "hn:1",
    category: "community",
    title: "COMMUNITY",
    source: "HACKER NEWS",
    url: "https://news.ycombinator.com/item?id=1",
    score: 42,
    comments: 8
  }
];

describe("discovery", () => {
  it("filters locally without changing the fetched collection", () => {
    expect(filterDiscoveryItems(items, "news")).toEqual([items[0]]);
    expect(filterDiscoveryItems(items, "community")).toEqual([items[1]]);
    expect(filterDiscoveryItems(items, "x")).toEqual([]);
    expect(items).toHaveLength(2);
  });

  it("formats only available engagement metadata", () => {
    expect(formatDiscoveryMeta(items[0])).toBe("BBC WORLD");
    expect(formatDiscoveryMeta(items[1])).toBe(
      "HACKER NEWS · 42 POINTS · 8 COMMENTS"
    );
  });
});
