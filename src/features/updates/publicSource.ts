export interface PublicSourceStatus {
  enabled: boolean;
  url: string | null;
  intervalMinutes: number;
  lastCheckedAt: string | null;
  title: string | null;
  error: string | null;
}

export function normalizedPublicSourceUrl(value: string): string | null {
  const trimmed = value.trim();
  if (trimmed.length === 0 || trimmed.length > 2_048) return null;
  try {
    const url = new URL(trimmed);
    const allowed =
      url.protocol === "https:" &&
      url.username === "" &&
      url.password === "" &&
      url.port === "" &&
      url.hash === "";
    return allowed ? url.toString() : null;
  } catch {
    return null;
  }
}
