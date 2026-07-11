import { useCallback, useEffect, useState } from "react";
import {
  hasSmartWatchConsent,
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

export function SmartWatchControl({
  disabled,
  privacyBlankActive,
  onBusyChange,
  onError
}: {
  disabled: boolean;
  privacyBlankActive: boolean;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  const [status, setStatus] = useState<SmartWatchStatus | null>(null);
  const [noticeOpen, setNoticeOpen] = useState(false);
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
      setStatus(await setSmartWatch(enabled));
      setNoticeOpen(false);
    } catch (reason) {
      onError(errorMessage(reason, "SMART WATCH COULD NOT BE UPDATED."));
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }, [onBusyChange, onError]);

  function toggle() {
    if (status?.enabled) {
      void update(false);
      return;
    }
    if (hasSmartWatchConsent(globalThis.localStorage)) {
      void update(true);
      return;
    }
    setNoticeOpen(true);
  }

  function allow() {
    rememberSmartWatchConsent(globalThis.localStorage);
    void update(true);
  }

  return (
    <div className="smart-watch-control">
      <button
        type="button"
        className={status?.enabled ? "secondary-action is-active" : "secondary-action"}
        aria-pressed={status?.enabled ?? false}
        aria-expanded={noticeOpen}
        title={smartWatchTitle(status)}
        disabled={disabled || busy || privacyBlankActive}
        onClick={toggle}
      >
        {status?.enabled ? "WATCHING" : "WATCH"}
      </button>
      {noticeOpen ? (
        <div className="smart-watch-notice" role="dialog" aria-label="SMART WATCH NOTICE">
          <strong>SMART WATCH NOTICE</strong>
          <p>THE SELECTED REGION IS COMPARED ONLY ON THIS MAC.</p>
          <p>NO AUTOMATIC OPENAI OR CLAUDE UPLOAD. CLOUD AI RUNS ONLY WHEN YOU PRESS ASK.</p>
          <p>GENERAL WATCH ALERTS ARE APPENDED TO DOWNLOADS/PEBBLE/PEBBLE-UPDATES.MD.</p>
          <p>ALERTS ARE LIMITED TO 24 PER APP SESSION AND STOP ON PAUSE, HIDE, PRIVACY, OR RESELECTION.</p>
          <div>
            <button type="button" className="secondary-action" onClick={() => setNoticeOpen(false)}>
              CANCEL
            </button>
            <button type="button" className="primary-action" onClick={allow}>
              ALLOW
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
