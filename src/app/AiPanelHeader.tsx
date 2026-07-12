import type { AiProvider } from "../features/ai/regionQuestion";
import { AiProviderSwitch } from "./AiProviderSwitch";
import type { AiConnectionState } from "./AiConnectionPrompt";
import { SmartWatchControl } from "./SmartWatchControl";

export function AiPanelHeader({
  browserPreview,
  connection,
  provider,
  disabled,
  privacyBlankActive,
  onProviderChange,
  onBusyChange,
  onError
}: {
  browserPreview: boolean;
  connection: AiConnectionState;
  provider: AiProvider;
  disabled: boolean;
  privacyBlankActive: boolean;
  onProviderChange: (provider: AiProvider) => void;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  return (
    <div className="region-question__header">
      <h3>AI</h3>
      <div className="region-question__actions">
        {!browserPreview ? (
          <SmartWatchControl
            provider={provider}
            disabled={disabled || connection !== "connected"}
            privacyBlankActive={privacyBlankActive}
            onBusyChange={onBusyChange}
            onError={onError}
          />
        ) : null}
        <AiProviderSwitch
          provider={provider}
          disabled={disabled || connection === "checking"}
          onChange={onProviderChange}
        />
      </div>
    </div>
  );
}
