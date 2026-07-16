import { memo, useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import sendIcon from "../assets/icons/up-arrow.svg";
import {
  MAX_REGION_QUESTION_LENGTH,
  aiAccessLabel,
  defaultAiModelId,
  rememberAiModel,
  selectedAiModel,
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
import {
  AiConnectionPrompt,
  type AiConnectionState
} from "./AiConnectionPrompt";
import { AiResponseArea } from "./AiResponseArea";
import { AiPanelHeader } from "./AiPanelHeader";
import { ClaudeCredentialControl } from "./ClaudeCredentialControl";
import { AiModelSwitch } from "./AiModelSwitch";

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
  const [connection, setConnection] = useState<AiConnectionState>(
    browserPreview ? "disconnected" : "checking"
  );
  const [status, setStatus] = useState<AiConnectionStatus | null>(null);
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<AiAnswer | null>(null);
  const [panelError, setPanelError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const [asking, setAsking] = useState(false);
  const [credentialRevision, setCredentialRevision] = useState(0);
  const [model, setModel] = useState(() => defaultAiModelId("openAi"));
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
        setModel(selectedAiModel(provider, nextStatus.models, globalThis.localStorage));
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
  }, [browserPreview, credentialRevision, provider]);

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
        await askSelectedRegion(provider, model, normalizedQuestion, navigator.language)
      );
    } catch (reason) {
      setPanelError(errorMessage(reason, "THE SELECTED REGION COULD NOT BE ANALYZED."));
    } finally {
      setAsking(false);
      onBusyChange(false);
    }
  }, [model, normalizedQuestion, onBusyChange, provider]);

  const models = status?.models ?? [];
  const accessLabel = aiAccessLabel(status?.connectionMode);

  const selectModel = useCallback((nextModel: string) => {
    setModel(nextModel);
    rememberAiModel(provider, nextModel, globalThis.localStorage);
  }, [provider]);
  return (
    <section className="region-question" aria-label="AI">
      <AiPanelHeader
        browserPreview={browserPreview}
        connection={connection}
        provider={provider}
        model={model}
        watchIntent={normalizedQuestion ?? ""}
        disabled={disabled || asking || connecting}
        privacyBlankActive={privacyBlankActive}
        onProviderChange={setProvider}
        onBusyChange={onBusyChange}
        onError={setPanelError}
      />

      {provider === "claude" && !browserPreview ? (
        <ClaudeCredentialControl
          disabled={disabled || asking || connecting}
          onChanged={() => setCredentialRevision((revision) => revision + 1)}
          onBusyChange={onBusyChange}
          onError={setPanelError}
        />
      ) : null}

      <AiConnectionPrompt
        connection={connection}
        provider={provider}
        browserPreview={browserPreview}
        disabled={disabled}
        connecting={connecting}
        onConnect={() => void connect()}
      />

      {connection === "connected" ? (
        <form className="region-question__form" onSubmit={(event) => void ask(event)}>
          <textarea
            aria-label="QUESTION ABOUT THE SELECTED REGION"
            value={question}
            maxLength={MAX_REGION_QUESTION_LENGTH}
            rows={3}
            placeholder="ASK OR TELL PEBBLE WHAT TO WATCH FOR"
            autoFocus
            disabled={disabled || asking || privacyBlankActive}
            onChange={(event) => setQuestion(event.currentTarget.value)}
          />
          <div className="region-question__composer-footer">
            <div className="region-question__model-choice">
              <AiModelSwitch
                models={models}
                selectedModel={model}
                disabled={disabled || asking}
                onChange={selectModel}
              />
              {accessLabel ? <span>{accessLabel}</span> : null}
            </div>
            <button
              type="submit"
              className="region-question__send"
              aria-label={asking ? "SENDING" : "SEND"}
              title={asking ? "SENDING" : "SEND"}
              disabled={
                disabled || asking || privacyBlankActive || normalizedQuestion === null
              }
            >
              <span
                className="region-question__send-icon"
                aria-hidden="true"
                style={{
                  maskImage: `url("${sendIcon}")`,
                  WebkitMaskImage: `url("${sendIcon}")`
                }}
              />
            </button>
          </div>
        </form>
      ) : null}

      {panelError ? <p className="region-question__error" role="alert">{panelError}</p> : null}
      <AiResponseArea answer={answer} />
    </section>
  );
});
