import type { LiveTileMode } from "../features/live-tile/liveTile";

export function liveFrameState(
  mode: LiveTileMode,
  hasFrame: boolean,
  error: string | null
) {
  if (error) return "NEEDS ATTENTION";
  if (mode === "blanked") return "HIDDEN";
  if (mode === "paused") return "PAUSED";
  return hasFrame ? "LIVE" : "STARTING";
}

export function captureErrorMessage(message: string) {
  return /permission/i.test(message)
    ? "Screen Recording permission is off. Enable pebble in macOS System Settings, then resume."
    : message;
}
