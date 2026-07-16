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
    <div className="smart-watch-status" role="status" aria-live="polite">
      <div className="smart-watch-status__summary">
        <span>{status.targetCount} REGIONS WATCHING</span>
        <span>SELECTED REGIONS ONLY · MEMORY ONLY · NOTHING SAVED</span>
      </div>
      {status.targets.map((target) => (
        <div className="smart-watch-target" key={target.id}>
          {smartWatchTargetSegments(target).map((segment, index) => (
            <span
              key={segment}
              className={index === 0 ? "smart-watch-status__intent" : undefined}
            >
              {segment}
            </span>
          ))}
          <button
            type="button"
            disabled={disabled}
            aria-label={`STOP ${target.name}`}
            onClick={() => onRemove(target.id)}
          >
            STOP
          </button>
        </div>
      ))}
    </div>
  );
}
