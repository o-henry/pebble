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
    <div
      className="live-tile-controls"
      role="group"
      aria-label="CAPTURE CONTROLS"
    >
      <TextAction
        label={mode === "paused" ? "LIVE" : "PAUSE"}
        ariaLabel={mode === "paused" ? "RESUME LIVE CAPTURE" : "PAUSE LIVE CAPTURE"}
        disabled={disabled || mode === "blanked"}
        onClick={mode === "paused" ? onLive : onPause}
      />
      <TextAction
        label="SELECT REGION"
        ariaLabel="SELECT ANOTHER REGION"
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
        ariaLabel={privacyBlankActive ? "SHOW PREVIEW" : "HIDE PREVIEW"}
        active={privacyBlankActive}
        disabled={disabled}
        onClick={onTogglePrivacy}
      />
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
  const className = ["text-action", active ? "is-active" : ""]
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
