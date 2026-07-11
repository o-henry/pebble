import { memo, useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import {
  MAX_REGION_QUESTION_LENGTH,
  normalizedRegionQuestion,
  type AiAnswer,
  type AiConnectionStatus,
  type AiProvider
} from "../features/ai/regionQuestion";
import {
  askSelectedRegion,
  connectAiProvider,
  getAiConnectionStatus
} from "../lib/invoke";
import { errorMessage } from "./usePebbleSession";

type ConnectionState = "checking" | "connected" | "disconnected" | "unavailable";

const PROVIDERS: ReadonlyArray<{ id: AiProvider; label: string }> = [
  { id: "openAi", label: "OPENAI" },
  { id: "claude", label: "CLAUDE" }
];

export const RegionQuestionPanel = memo(function RegionQuestionPanel({
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
  const [provider, setProvider] = useState<AiProvider>("openAi");
  const [connection, setConnection] = useState<ConnectionState>(
    browserPreview ? "disconnected" : "checking"
  );
  const [status, setStatus] = useState<AiConnectionStatus | null>(null);
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<AiAnswer | null>(null);
  const [panelError, setPanelError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [asking, setAsking] = useState(false);
  const normalizedQuestion = useMemo(
    () => normalizedRegionQuestion(question),
    [question]
  );

  useEffect(() => {
    if (browserPreview) {
      setConnection("disconnected");
      setStatus(null);
      return;
    }

    let active = true;
    setConnection("checking");
    setPanelError(null);
    getAiConnectionStatus(provider)
      .then((nextStatus) => {
        if (!active) return;
        setStatus(nextStatus);
        setConnection(
          !nextStatus.available
            ? "unavailable"
            : nextStatus.connected
              ? "connected"
              : "disconnected"
        );
      })
      .catch((reason: unknown) => {
        if (!active) return;
        setConnection("disconnected");
        setPanelError(errorMessage(reason, "AI CONNECTION COULD NOT BE CHECKED."));
      });

    return () => {
      active = false;
    };
  }, [browserPreview, provider]);

  const connect = useCallback(async () => {
    try {
      setConnecting(true);
      onBusyChange(true);
      setPanelError(null);
      const nextStatus = await connectAiProvider(provider);
      setStatus(nextStatus);
      setConnection(
        !nextStatus.available
          ? "unavailable"
          : nextStatus.connected
            ? "connected"
            : "disconnected"
      );
    } catch (reason) {
      setPanelError(errorMessage(reason, "AI SIGN-IN WAS NOT COMPLETED."));
    } finally {
      setConnecting(false);
      onBusyChange(false);
    }
  }, [onBusyChange, provider]);

  const ask = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!normalizedQuestion) {
      setPanelError("ENTER A QUESTION BETWEEN 1 AND 1,000 CHARACTERS.");
      return;
    }

    try {
      setAsking(true);
      onBusyChange(true);
      setPanelError(null);
      setAnswer(null);
      setAnswer(
        await askSelectedRegion(provider, normalizedQuestion, navigator.language)
      );
    } catch (reason) {
      setPanelError(errorMessage(reason, "THE SELECTED REGION COULD NOT BE ANALYZED."));
    } finally {
      setAsking(false);
      onBusyChange(false);
    }
  }, [normalizedQuestion, onBusyChange, provider]);

  const modelLabel = status?.model ?? defaultModel(provider);

  return (
    <section className="region-question" aria-label="AI">
      <div className="region-question__header">
        <h3>AI</h3>
        <div className="provider-switch" role="group" aria-label="AI PROVIDER">
          {PROVIDERS.map((item) => (
            <button
              key={item.id}
              type="button"
              className={provider === item.id ? "is-active" : ""}
              aria-pressed={provider === item.id}
              disabled={disabled || asking || connecting}
              onClick={() => setProvider(item.id)}
            >
              {item.label}
            </button>
          ))}
        </div>
      </div>

      {connection === "checking" ? (
        <p className="region-question__quiet" aria-live="polite">CHECKING CONNECTION</p>
      ) : connection !== "connected" ? (
        <div className="region-question__connect">
          <button
            type="button"
            className="secondary-action"
            disabled={disabled || connecting || browserPreview}
            onClick={() => void connect()}
          >
            {browserPreview
              ? "DESKTOP APP REQUIRED"
              : connecting
                ? "FINISH SIGN-IN"
                : connection === "unavailable"
                  ? "INSTALL"
                  : `CONNECT ${provider === "openAi" ? "OPENAI" : "CLAUDE"}`}
          </button>
          <span>NO API KEY</span>
        </div>
      ) : (
        <form className="region-question__form" onSubmit={(event) => void ask(event)}>
          <textarea
            aria-label="QUESTION ABOUT THE SELECTED REGION"
            value={question}
            maxLength={MAX_REGION_QUESTION_LENGTH}
            rows={3}
            placeholder="ASK ABOUT THIS REGION"
            autoFocus
            disabled={disabled || asking || privacyBlankActive}
            onChange={(event) => setQuestion(event.currentTarget.value)}
          />
          <div className="region-question__composer-footer">
            <span className="region-question__model">{modelLabel} · MEDIUM</span>
            <button
              type="submit"
              className="primary-action"
              disabled={
                disabled || asking || privacyBlankActive || normalizedQuestion === null
              }
            >
              {asking ? "LOOKING" : "ASK"}
            </button>
          </div>
        </form>
      )}

      {panelError ? <p className="region-question__error" role="alert">{panelError}</p> : null}
      {answer ? (
        <div className="region-question__answer" aria-live="polite">
          <p>{answer.answer}</p>
          <span>{answer.model.toUpperCase()} · {formatDuration(answer.durationMs)}</span>
        </div>
      ) : null}
    </section>
  );
});

function defaultModel(provider: AiProvider) {
  return provider === "openAi" ? "GPT-5.6-TERRA" : "CLAUDE SONNET 5";
}

function formatDuration(durationMs: number) {
  return `${Math.max(0, durationMs / 1_000).toFixed(1)}S`;
}
