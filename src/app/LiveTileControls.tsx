import type { LiveTileMode } from "../features/live-tile/liveTile";

export function LiveTileControls({
  mode,
  aiExpanded,
  disabled,
  privacyBlankActive,
  onLive,
  onPause,
  onReselect,
  onToggleAi,
  onTogglePrivacy,
  onClose
}: {
  mode: LiveTileMode;
  aiExpanded: boolean;
  disabled: boolean;
  privacyBlankActive: boolean;
  onLive: () => void;
  onPause: () => void;
  onReselect: () => void;
  onToggleAi: () => void;
  onTogglePrivacy: () => void;
  onClose: () => void;
}) {
  return (
    <div className="live-tile-controls" aria-label="Pebble controls">
      <div className="mode-controls" role="group" aria-label="Capture state">
        <button
          type="button"
          className={mode === "live" ? "is-active" : "secondary-action"}
          disabled={disabled || mode === "blanked"}
          onClick={onLive}
        >
          LIVE
        </button>
        <button
          type="button"
          className={mode === "paused" ? "is-active" : "secondary-action"}
          disabled={disabled || mode === "blanked"}
          onClick={onPause}
        >
          PAUSE
        </button>
      </div>

      <div className="tile-tool-controls" role="group" aria-label="Pebble tools">
        <TextAction
          label="SELECT REGION"
          ariaLabel="Select another region"
          disabled={disabled}
          onClick={onReselect}
        />
        <TextAction
          label="CHATGPT"
          ariaLabel={aiExpanded ? "Hide ChatGPT" : "Ask ChatGPT"}
          active={aiExpanded}
          disabled={disabled}
          onClick={onToggleAi}
        />
        <TextAction
          label={privacyBlankActive ? "SHOW" : "HIDE"}
          ariaLabel={privacyBlankActive ? "Show preview" : "Hide preview"}
          active={privacyBlankActive}
          disabled={disabled}
          onClick={onTogglePrivacy}
        />
        <TextAction
          label="CLOSE"
          ariaLabel="Close pebble"
          disabled={disabled}
          danger
          onClick={onClose}
        />
      </div>
    </div>
  );
}

function TextAction({
  label,
  ariaLabel,
  active = false,
  danger = false,
  disabled,
  onClick
}: {
  label: string;
  ariaLabel: string;
  active?: boolean;
  danger?: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  const className = [
    "text-action",
    active ? "is-active" : "",
    danger ? "is-danger" : ""
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      type="button"
      className={className}
      aria-label={ariaLabel}
      title={ariaLabel}
      disabled={disabled}
      onClick={onClick}
    >
      {label}
    </button>
  );
}
