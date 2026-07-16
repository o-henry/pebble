import { useCallback, useState } from "react";
import type { AiProvider } from "../features/ai/regionQuestion";
import type { SmartWatchStatus } from "../features/ai/smartWatch";
import { AiProviderSwitch } from "./AiProviderSwitch";
import type { AiConnectionState } from "./AiConnectionPrompt";
import { SmartWatchControl } from "./SmartWatchControl";
import { SmartWatchStatusLine } from "./SmartWatchStatusLine";

export function AiPanelHeader({
  browserPreview,
  connection,
  provider,
  model,
  watchIntent,
  disabled,
  privacyBlankActive,
  onProviderChange,
  onBusyChange,
  onError
}: {
  browserPreview: boolean;
  connection: AiConnectionState;
  provider: AiProvider;
  model: string;
  watchIntent: string;
  disabled: boolean;
  privacyBlankActive: boolean;
  onProviderChange: (provider: AiProvider) => void;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  const [watchStatus, setWatchStatus] = useState<SmartWatchStatus | null>(null);
  const acceptWatchStatus = useCallback((status: SmartWatchStatus | null) => {
    setWatchStatus(status);
  }, []);

  return (
    <div className="region-question__header-group">
      <div className="region-question__header">
        <h3>AI</h3>
        <div className="region-question__actions">
          {!browserPreview ? (
            <SmartWatchControl
              provider={provider}
              model={model}
              intent={watchIntent}
              disabled={disabled || connection !== "connected"}
              privacyBlankActive={privacyBlankActive}
              onStatusChange={acceptWatchStatus}
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
      <SmartWatchStatusLine status={watchStatus} />
    </div>
  );
}
