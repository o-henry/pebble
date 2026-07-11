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
  onTogglePrivacy
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
}) {
  return (
    <div className="live-tile-controls" aria-label="CAPTURE CONTROLS">
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

      <div className="tile-tool-controls" role="group" aria-label="CAPTURE TOOLS">
        <TextAction
          label="SELECT REGION"
          ariaLabel="Select another region"
          disabled={disabled}
          onClick={onReselect}
        />
        <TextAction
          label="AI"
          ariaLabel={aiExpanded ? "HIDE AI" : "ASK AI"}
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
      </div>
    </div>
  );
}

function TextAction({
  label,
  ariaLabel,
  active = false,
  disabled,
  onClick
}: {
  label: string;
  ariaLabel: string;
  active?: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  const className = [
    "text-action",
    active ? "is-active" : "",
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
