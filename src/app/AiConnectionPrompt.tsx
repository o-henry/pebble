import type { AiProvider } from "../features/ai/regionQuestion";

export type AiConnectionState =
  | "checking"
  | "connected"
  | "disconnected"
  | "unavailable";

export function AiConnectionPrompt({
  connection,
  provider,
  browserPreview,
  disabled,
  connecting,
  onConnect
}: {
  connection: AiConnectionState;
  provider: AiProvider;
  browserPreview: boolean;
  disabled: boolean;
  connecting: boolean;
  onConnect: () => void;
}) {
  if (connection === "connected") return null;
  if (connection === "checking") {
    return (
      <p className="region-question__quiet" aria-live="polite">
        CHECKING CONNECTION
      </p>
    );
  }

  const label = browserPreview
    ? "DESKTOP APP REQUIRED"
    : connecting
      ? "FINISH SIGN-IN"
      : connection === "unavailable"
        ? "INSTALL"
        : `CONNECT ${provider === "openAi" ? "OPENAI" : "CLAUDE"}`;

  return (
    <div className="region-question__connect">
      <button
        type="button"
        className="secondary-action"
        disabled={disabled || connecting || browserPreview}
        onClick={onConnect}
      >
        {label}
      </button>
      <span>NO API KEY</span>
    </div>
  );
}
