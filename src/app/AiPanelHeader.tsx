import { useCallback, useState } from "react";
import type { AiProvider } from "../features/ai/regionQuestion";
import {
  rememberSmartWatchInterval,
  type SmartWatchIntervalMinutes,
  type SmartWatchStatus
} from "../features/ai/smartWatch";
import { AiProviderSwitch } from "./AiProviderSwitch";
import type { AiConnectionState } from "./AiConnectionPrompt";
import { SmartWatchControl } from "./SmartWatchControl";
import { SmartWatchStatusLine } from "./SmartWatchStatusLine";
import { removeSmartWatchTarget, setSmartWatchInterval } from "../lib/invoke";
import { errorMessage } from "./usePebbleSession";
import { SmartWatchIntervalControl } from "./SmartWatchIntervalControl";

export function AiPanelHeader({
  browserPreview,
  connection,
  provider,
  model,
  watchIntent,
  disabled,
  privacyBlankActive,
  optionsExpanded,
  intervalMinutes,
  onProviderChange,
  onToggleOptions,
  onIntervalChange,
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
  optionsExpanded: boolean;
  intervalMinutes: SmartWatchIntervalMinutes;
  onProviderChange: (provider: AiProvider) => void;
  onToggleOptions: () => void;
  onIntervalChange: (minutes: SmartWatchIntervalMinutes) => void;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  const [watchStatus, setWatchStatus] = useState<SmartWatchStatus | null>(null);
  const [removingWatch, setRemovingWatch] = useState(false);
  const acceptWatchStatus = useCallback((status: SmartWatchStatus | null) => {
    setWatchStatus(status);
    if (status?.enabled) {
      onIntervalChange(status.analysisIntervalMinutes);
      rememberSmartWatchInterval(
        globalThis.localStorage,
        status.analysisIntervalMinutes
      );
    }
  }, [onIntervalChange]);

  const removeWatch = useCallback(async (targetId: string) => {
    try {
      setRemovingWatch(true);
      onBusyChange(true);
      onError(null);
      setWatchStatus(await removeSmartWatchTarget(targetId));
    } catch (reason) {
      onError(errorMessage(reason, "WATCH REGION COULD NOT BE STOPPED."));
    } finally {
      setRemovingWatch(false);
      onBusyChange(false);
    }
  }, [onBusyChange, onError]);

  const updateInterval = useCallback(async (
    nextInterval: SmartWatchIntervalMinutes
  ) => {
    const previousInterval = intervalMinutes;
    onIntervalChange(nextInterval);
    rememberSmartWatchInterval(globalThis.localStorage, nextInterval);
    if (!watchStatus?.enabled) return;

    try {
      onBusyChange(true);
      onError(null);
      acceptWatchStatus(await setSmartWatchInterval(nextInterval));
    } catch (reason) {
      onIntervalChange(previousInterval);
      rememberSmartWatchInterval(globalThis.localStorage, previousInterval);
      onError(errorMessage(reason, "WATCH INTERVAL COULD NOT BE UPDATED."));
    } finally {
      onBusyChange(false);
    }
  }, [acceptWatchStatus, intervalMinutes, onBusyChange, onError, onIntervalChange, watchStatus?.enabled]);

  const intervalDisabled =
    disabled ||
    watchStatus?.localEngine === "crossRegionOcr" ||
    watchStatus?.localEngine === "followThroughResult" ||
    watchStatus?.localEngine === "visualLoop";

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
              intervalMinutes={intervalMinutes}
              disabled={disabled || connection === "checking"}
              privacyBlankActive={privacyBlankActive}
              aiConnected={connection === "connected"}
              onStatusChange={acceptWatchStatus}
              onBusyChange={onBusyChange}
              onError={onError}
            />
          ) : null}
          <button
            type="button"
            className="region-question__options-toggle"
            aria-expanded={optionsExpanded}
            onClick={onToggleOptions}
          >
            {optionsExpanded ? "DONE" : "OPTIONS"}
          </button>
        </div>
      </div>
      {optionsExpanded ? (
        <div className="region-question__advanced-controls">
          <span>INTERVAL</span>
          <SmartWatchIntervalControl
            value={intervalMinutes}
            disabled={intervalDisabled}
            onChange={(minutes) => void updateInterval(minutes)}
          />
          <span>PROVIDER</span>
          <AiProviderSwitch
            provider={provider}
            disabled={disabled || connection === "checking"}
            onChange={onProviderChange}
          />
        </div>
      ) : null}
      <SmartWatchStatusLine
        status={watchStatus}
        disabled={disabled || removingWatch}
        onRemove={(targetId) => void removeWatch(targetId)}
      />
    </div>
  );
}
