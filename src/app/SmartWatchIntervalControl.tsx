import type { KeyboardEvent } from "react";
import {
  smartWatchIntervalAtOffset,
  smartWatchIntervalLabel,
  SMART_WATCH_INTERVAL_OPTIONS,
  type SmartWatchIntervalMinutes
} from "../features/ai/smartWatch";

export function SmartWatchIntervalControl({
  value,
  disabled,
  onChange
}: {
  value: SmartWatchIntervalMinutes;
  disabled: boolean;
  onChange: (value: SmartWatchIntervalMinutes) => void;
}) {
  const next = smartWatchIntervalAtOffset(value, 1);

  function selectOffset(offset: number) {
    onChange(smartWatchIntervalAtOffset(value, offset));
  }

  function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    switch (event.key) {
      case "ArrowRight":
      case "ArrowDown":
        event.preventDefault();
        selectOffset(1);
        break;
      case "ArrowLeft":
      case "ArrowUp":
        event.preventDefault();
        selectOffset(-1);
        break;
      case "Home":
        event.preventDefault();
        onChange(SMART_WATCH_INTERVAL_OPTIONS[0]);
        break;
      case "End":
        event.preventDefault();
        onChange(SMART_WATCH_INTERVAL_OPTIONS.at(-1) ?? value);
        break;
    }
  }

  return (
    <button
      type="button"
      className="smart-watch-interval-control"
      aria-label={`WATCH AI ANALYSIS INTERVAL ${smartWatchIntervalLabel(value)}. SELECT ${smartWatchIntervalLabel(next)}.`}
      title={`WATCH INTERVAL · ${smartWatchIntervalLabel(value)} · NEXT ${smartWatchIntervalLabel(next)}`}
      disabled={disabled}
      onClick={() => onChange(next)}
      onKeyDown={handleKeyDown}
    >
      {smartWatchIntervalLabel(value)}
    </button>
  );
}
