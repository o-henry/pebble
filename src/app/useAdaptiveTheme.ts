import { useEffect } from "react";
import type { CroppedFramePayload } from "../features/capture/captureFrame";
import { deriveAdaptiveTheme } from "../features/adaptive-theme/adaptiveTheme";

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

export function useAdaptiveTheme(frame: CroppedFramePayload | null) {
  useEffect(() => {
    const root = document.documentElement;
    const theme = frame ? deriveAdaptiveTheme(frame) : null;

    resetAdaptiveTheme(root);
    if (!theme) {
      return;
    }

    root.dataset.adaptiveTheme = theme.mode;
    for (const [property, value] of Object.entries(theme.variables)) {
      root.style.setProperty(property, value);
    }
  }, [frame]);

  useEffect(
    () => () => {
      resetAdaptiveTheme(document.documentElement);
    },
    []
  );
}

function resetAdaptiveTheme(root: HTMLElement) {
  delete root.dataset.adaptiveTheme;
  for (const property of ADAPTIVE_PROPERTIES) {
    root.style.removeProperty(property);
  }
}
