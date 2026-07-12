import { useCallback, useEffect, useState } from "react";
import {
  rememberSmartWatchConsent,
  smartWatchTitle,
  type SmartWatchStatus
} from "../features/ai/smartWatch";
import {
  getSmartWatchStatus,
  setSmartWatch
} from "../lib/invoke";
import { listenToSmartWatchStatus } from "../lib/events";
import { errorMessage } from "./usePebbleSession";
import type { AiProvider } from "../features/ai/regionQuestion";

export function SmartWatchControl({
  provider,
  disabled,
  privacyBlankActive,
  onBusyChange,
  onError
}: {
  provider: AiProvider;
  disabled: boolean;
  privacyBlankActive: boolean;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  const [status, setStatus] = useState<SmartWatchStatus | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let active = true;
    let unlisten: () => void = () => undefined;
    getSmartWatchStatus()
      .then((next) => active && setStatus(next))
      .catch((reason: unknown) => {
        if (active) onError(errorMessage(reason, "SMART WATCH IS UNAVAILABLE."));
      });
    void listenToSmartWatchStatus((next) => active && setStatus(next)).then(
      (nextUnlisten) => {
        if (active) unlisten = nextUnlisten;
        else nextUnlisten();
      }
    );
    return () => {
      active = false;
      unlisten();
    };
  }, [onError]);

  const update = useCallback(async (enabled: boolean) => {
    try {
      setBusy(true);
      onBusyChange(true);
      onError(null);
      setStatus(await setSmartWatch(enabled, provider, globalThis.navigator.language));
    } catch (reason) {
      onError(errorMessage(reason, "SMART WATCH COULD NOT BE UPDATED."));
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }, [onBusyChange, onError, provider]);

  function toggle() {
    if (status?.enabled) {
      void update(false);
      return;
    }
    rememberSmartWatchConsent(globalThis.localStorage);
    void update(true);
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
    </div>
  );
}
