import {
  smartWatchStatusSegments,
  type SmartWatchStatus
} from "../features/ai/smartWatch";

export function SmartWatchStatusLine({
  status
}: {
  status: SmartWatchStatus | null;
}) {
  if (!status?.enabled) return null;

  const segments = smartWatchStatusSegments(status);

  return (
    <div className="smart-watch-status" role="status" aria-live="polite">
      {segments.map((segment, index) => (
        <span
          key={segment}
          className={index === 0 ? "smart-watch-status__intent" : undefined}
        >
          {segment}
        </span>
      ))}
    </div>
  );
}
