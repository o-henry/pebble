import {
  smartWatchTargetSegments,
  type SmartWatchStatus
} from "../features/ai/smartWatch";

export function SmartWatchStatusLine({
  status,
  disabled,
  onRemove
}: {
  status: SmartWatchStatus | null;
  disabled: boolean;
  onRemove: (targetId: string) => void;
}) {
  if (!status || status.targetCount === 0) return null;

  return (
    <div
      className="smart-watch-status"
      role="status"
      aria-live="polite"
      aria-label={`${status.targetCount} REGIONS WATCHING. SELECTED REGIONS ONLY. FRAMES STAY IN MEMORY. JOURNAL DETAILS ARE REDACTED.`}
    >
      {status.targets.map((target) => {
        const segments = smartWatchTargetSegments(target);
        return (
          <div
            className="smart-watch-target"
            key={target.id}
            title={segments.join(" · ")}
          >
            <span className="smart-watch-target__name">{target.name}</span>
            <span className="smart-watch-status__intent">
              {target.watchingFor}
            </span>
            <span className="smart-watch-status__engine">{segments[1]}</span>
            <button
              type="button"
              disabled={disabled}
              aria-label={`STOP ${target.name}`}
              onClick={() => onRemove(target.id)}
            >
              STOP
            </button>
          </div>
        );
      })}
      <div className="smart-watch-status__scope">
        SELECTED REGIONS ONLY · FRAMES STAY IN MEMORY · JOURNAL DETAILS REDACTED
      </div>
    </div>
  );
}
