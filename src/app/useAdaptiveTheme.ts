import { useEffect } from "react";
import { deriveAdaptiveTheme } from "../features/adaptive-theme/adaptiveTheme";
import { getPebbleBackdropColor } from "../lib/invoke";
import { isTauriRuntime } from "../lib/runtime";

const SAMPLE_INTERVAL_MS = 1_500;
const ADAPTIVE_PROPERTIES = [
  "--canvas",
  "--surface",
  "--surface-soft",
  "--ink",
  "--ink-strong",
  "--ink-muted",
  "--line",
  "--line-strong",
  "--blue",
  "--blue-strong",
  "--blue-soft",
  "--green",
  "--amber",
  "--amber-soft",
  "--danger",
  "--danger-soft",
  "--frame-line"
] as const;

export function useAdaptiveTheme() {
  useEffect(() => {
    const root = document.documentElement;
    let active = true;
    let inFlight = false;
    let lastColor = "";

    if (!isTauriRuntime()) {
      resetAdaptiveTheme(root);
      return;
    }

    async function sampleBackdrop() {
      if (!active || inFlight || document.hidden) {
        return;
      }
      inFlight = true;
      try {
        const color = await getPebbleBackdropColor();
        if (!active || !color) {
          return;
        }
        const theme = deriveAdaptiveTheme(color);
        if (!theme) {
          return;
        }
        const colorKey = `${color.red}:${color.green}:${color.blue}`;
        if (colorKey === lastColor) {
          return;
        }
        lastColor = colorKey;
        resetAdaptiveTheme(root);
        root.dataset.adaptiveTheme = theme.mode;
        for (const [property, value] of Object.entries(theme.variables)) {
          root.style.setProperty(property, value);
        }
      } catch {
        // Keep the last valid color when a transient sample is unavailable.
      } finally {
        inFlight = false;
      }
    }

    const onVisibilityChange = () => void sampleBackdrop();
    void sampleBackdrop();
    const interval = window.setInterval(sampleBackdrop, SAMPLE_INTERVAL_MS);
    document.addEventListener("visibilitychange", onVisibilityChange);

    return () => {
      active = false;
      window.clearInterval(interval);
      document.removeEventListener("visibilitychange", onVisibilityChange);
      resetAdaptiveTheme(root);
    };
  }, []);
}

function resetAdaptiveTheme(root: HTMLElement) {
  delete root.dataset.adaptiveTheme;
  for (const property of ADAPTIVE_PROPERTIES) {
    root.style.removeProperty(property);
  }
}
