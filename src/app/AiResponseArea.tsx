import type { AiAnswer } from "../features/ai/regionQuestion";
import { AiAnswerView } from "./AiAnswerView";
import { UpdateFeedPanel } from "./UpdateFeedPanel";

export function AiResponseArea({ answer }: { answer: AiAnswer | null }) {
  return (
    <>
      <AiAnswerView answer={answer} />
      <UpdateFeedPanel />
    </>
  );
}
