import { useCallback, useEffect, useState } from "react";
import {
  isSmartWatchInterval,
  rememberSmartWatchConsent,
  rememberSmartWatchInterval,
  smartWatchInterval,
  smartWatchTitle,
  type SmartWatchIntervalMinutes,
  type SmartWatchStatus
} from "../features/ai/smartWatch";
import {
  getSmartWatchStatus,
  setSmartWatch,
  setSmartWatchInterval
} from "../lib/invoke";
import { listenToSmartWatchStatus } from "../lib/events";
import { errorMessage } from "./usePebbleSession";
import type { AiProvider } from "../features/ai/regionQuestion";
import { SmartWatchIntervalControl } from "./SmartWatchIntervalControl";

export function SmartWatchControl({
  provider,
  model,
  intent,
  disabled,
  privacyBlankActive,
  aiConnected,
  onStatusChange,
  onBusyChange,
  onError
}: {
  provider: AiProvider;
  model: string;
  intent: string;
  disabled: boolean;
  privacyBlankActive: boolean;
  aiConnected: boolean;
  onStatusChange: (status: SmartWatchStatus | null) => void;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  const [status, setStatus] = useState<SmartWatchStatus | null>(null);
  const [intervalMinutes, setIntervalMinutes] =
    useState<SmartWatchIntervalMinutes>(() =>
      smartWatchInterval(globalThis.localStorage)
    );
  const [busy, setBusy] = useState(false);

  const acceptStatus = useCallback((next: SmartWatchStatus) => {
    setStatus(next);
    onStatusChange(next);
    if (next.enabled) {
      setIntervalMinutes(next.analysisIntervalMinutes);
      rememberSmartWatchInterval(
        globalThis.localStorage,
        next.analysisIntervalMinutes
      );
    }
  }, [onStatusChange]);

  useEffect(() => () => onStatusChange(null), [onStatusChange]);

  useEffect(() => {
    let active = true;
    let unlisten: () => void = () => undefined;
    getSmartWatchStatus()
      .then((next) => active && acceptStatus(next))
      .catch((reason: unknown) => {
        if (active) onError(errorMessage(reason, "SMART WATCH IS UNAVAILABLE."));
      });
    void listenToSmartWatchStatus((next) => active && acceptStatus(next)).then(
      (nextUnlisten) => {
        if (active) unlisten = nextUnlisten;
        else nextUnlisten();
      }
    );
    return () => {
      active = false;
      unlisten();
    };
  }, [acceptStatus, onError]);

  const update = useCallback(async (enabled: boolean) => {
    try {
      setBusy(true);
      onBusyChange(true);
      onError(null);
      acceptStatus(await setSmartWatch(
        enabled,
        provider,
        model,
        intent,
        globalThis.navigator.language,
        intervalMinutes,
        aiConnected
      ));
    } catch (reason) {
      onError(errorMessage(reason, "SMART WATCH COULD NOT BE UPDATED."));
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }, [acceptStatus, aiConnected, intent, intervalMinutes, model, onBusyChange, onError, provider]);

  const updateInterval = useCallback(async (
    nextInterval: SmartWatchIntervalMinutes
  ) => {
    try {
      setBusy(true);
      onBusyChange(true);
      onError(null);
      acceptStatus(await setSmartWatchInterval(nextInterval));
    } catch (reason) {
      onError(errorMessage(reason, "WATCH INTERVAL COULD NOT BE UPDATED."));
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }, [acceptStatus, onBusyChange, onError]);

  function toggle() {
    if (status?.enabled) {
      void update(false);
      return;
    }
    rememberSmartWatchConsent(globalThis.localStorage);
    void update(true);
  }

  function selectInterval(value: string) {
    const nextInterval = Number(value);
    if (!isSmartWatchInterval(nextInterval)) return;
    setIntervalMinutes(nextInterval);
    rememberSmartWatchInterval(globalThis.localStorage, nextInterval);
    if (status?.enabled) void updateInterval(nextInterval);
  }

  return (
    <div className="smart-watch-control">
      <button
        type="button"
        className={status?.enabled ? "secondary-action is-active" : "secondary-action"}
        aria-pressed={status?.enabled ?? false}
        title={smartWatchTitle(status)}
        disabled={disabled || busy || privacyBlankActive}
        onClick={toggle}
      >
        {status?.enabled ? "WATCHING" : "WATCH"}
      </button>
      <SmartWatchIntervalControl
        value={intervalMinutes}
        disabled={busy}
        onChange={(minutes) => selectInterval(String(minutes))}
      />
    </div>
  );
}
