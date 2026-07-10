import { useEffect, useState, type FormEvent } from "react";
import {
  MAX_REGION_QUESTION_LENGTH,
  normalizedRegionQuestion,
  type AiConnectionStatus
} from "../features/ai/regionQuestion";
import {
  askSelectedRegion,
  connectChatGPT,
  getAiConnectionStatus
} from "../lib/invoke";
import { errorMessage } from "./usePebbleSession";

type ConnectionState = "checking" | "connected" | "disconnected";

export function RegionQuestionPanel({
  browserPreview,
  disabled,
  privacyBlankActive,
  onBusyChange
}: {
  browserPreview: boolean;
  disabled: boolean;
  privacyBlankActive: boolean;
  onBusyChange: (busy: boolean) => void;
}) {
  const [connection, setConnection] = useState<ConnectionState>(() =>
    browserPreview ? "disconnected" : "checking"
  );
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<string | null>(null);
  const [panelError, setPanelError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [asking, setAsking] = useState(false);

  useEffect(() => {
    if (browserPreview) {
      return;
    }

    let active = true;
    getAiConnectionStatus()
      .then((status) => {
        if (active) {
          setConnection(status.connected ? "connected" : "disconnected");
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          setConnection("disconnected");
          setPanelError(
            errorMessage(reason, "ChatGPT connection could not be checked.")
          );
        }
      });

    return () => {
      active = false;
    };
  }, [browserPreview]);

  async function connect() {
    try {
      setConnecting(true);
      onBusyChange(true);
      setPanelError(null);
      const status: AiConnectionStatus = await connectChatGPT();
      setConnection(status.connected ? "connected" : "disconnected");
    } catch (reason) {
      setPanelError(errorMessage(reason, "ChatGPT sign-in was not completed."));
    } finally {
      setConnecting(false);
      onBusyChange(false);
    }
  }

  async function ask(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalized = normalizedRegionQuestion(question);
    if (!normalized) {
      setPanelError("Enter a question between 1 and 1,000 characters.");
      return;
    }

    try {
      setAsking(true);
      onBusyChange(true);
      setPanelError(null);
      setAnswer(null);
      const response = await askSelectedRegion(normalized);
      setAnswer(response.answer);
    } catch (reason) {
      setPanelError(
        errorMessage(reason, "The selected region could not be analyzed.")
      );
    } finally {
      setAsking(false);
      onBusyChange(false);
    }
  }

  return (
    <section className="region-question" aria-labelledby="region-question-title">
      <div className="region-question__header">
        <div>
          <p className="section-label">ChatGPT</p>
          <h3 id="region-question-title">Ask about this region</h3>
        </div>
        {connection === "connected" ? (
          <span className="region-question__status">Connected</span>
        ) : null}
      </div>

      {connection === "checking" ? (
        <p className="region-question__quiet" aria-live="polite">
          Checking connection
        </p>
      ) : connection === "disconnected" ? (
        <div className="region-question__connect">
          <button
            type="button"
            className="secondary-action"
            disabled={disabled || connecting || browserPreview}
            onClick={() => void connect()}
          >
            {browserPreview
              ? "Desktop app required"
              : connecting
                ? "Finish sign-in in browser"
                : "Connect ChatGPT"}
          </button>
          <span>No API key</span>
        </div>
      ) : (
        <form className="region-question__form" onSubmit={(event) => void ask(event)}>
          <textarea
            aria-label="Question about the selected region"
            value={question}
            maxLength={MAX_REGION_QUESTION_LENGTH}
            rows={3}
            placeholder="What should I notice here?"
            disabled={disabled || asking || privacyBlankActive}
            onChange={(event) => setQuestion(event.currentTarget.value)}
          />
          <div className="region-question__submit">
            <span>{privacyBlankActive ? "Preview hidden" : "Selected crop only"}</span>
            <button
              type="submit"
              className="primary-action"
              disabled={
                disabled ||
                asking ||
                privacyBlankActive ||
                normalizedRegionQuestion(question) === null
              }
            >
              {asking ? "Looking" : "Ask"}
            </button>
          </div>
        </form>
      )}

      {panelError ? (
        <p className="region-question__error" role="alert">
          {panelError}
        </p>
      ) : null}
      {answer ? (
        <p className="region-question__answer" aria-live="polite">
          {answer}
        </p>
      ) : null}
    </section>
  );
}
