import type { AiAnswer } from "../features/ai/regionQuestion";

export function AiAnswerView({ answer }: { answer: AiAnswer | null }) {
  if (!answer) return null;

  return (
    <div className="region-question__answer" aria-live="polite">
      <p>{answer.answer}</p>
      <span>
        {answer.model.toUpperCase()} · {formatDuration(answer.durationMs)}
      </span>
    </div>
  );
}

function formatDuration(durationMs: number) {
  return `${Math.max(0, durationMs / 1_000).toFixed(1)}S`;
}
