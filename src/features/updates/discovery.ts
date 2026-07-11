export type DiscoveryCategory = "news" | "community";
export type DiscoveryFilter = "all" | DiscoveryCategory | "x";

export interface DiscoveryItem {
  id: string;
  category: DiscoveryCategory;
  title: string;
  source: string;
  url: string;
  score: number | null;
  comments: number | null;
}

export interface DiscoveryStatus {
  enabled: boolean;
  intervalMinutes: number;
  lastCheckedAt: string | null;
  items: DiscoveryItem[];
  warnings: string[];
  error: string | null;
}

export const EMPTY_DISCOVERY_STATUS: DiscoveryStatus = {
  enabled: false,
  intervalMinutes: 30,
  lastCheckedAt: null,
  items: [],
  warnings: [],
  error: null
};

export const DISCOVERY_FILTERS: DiscoveryFilter[] = [
  "all",
  "news",
  "community",
  "x"
];

export function filterDiscoveryItems(
  items: DiscoveryItem[],
  filter: DiscoveryFilter
): DiscoveryItem[] {
  if (filter === "all") return items;
  if (filter === "x") return [];
  return items.filter((item) => item.category === filter);
}

export function formatDiscoveryMeta(item: DiscoveryItem): string {
  const metrics = [
    item.score === null ? null : `${item.score} POINTS`,
    item.comments === null ? null : `${item.comments} COMMENTS`
  ].filter(Boolean);
  return [item.source, ...metrics].join(" · ");
}
