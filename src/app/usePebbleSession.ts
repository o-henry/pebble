import { useCallback, useEffect, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  EMPTY_PEBBLE_SESSION,
  browserSessionFromStorage,
  isPebbleSessionSnapshot,
  newestSession,
  storeBrowserSession,
  type PebbleSessionSnapshot
} from "../features/pebble-session/pebbleSession";
import { listenToPebbleSession } from "../lib/events";
import { getPebbleSession } from "../lib/invoke";
import { isTauriRuntime } from "../lib/runtime";

export function usePebbleSession() {
  const browserPreview = !isTauriRuntime();
  const [session, setSession] = useState<PebbleSessionSnapshot>(() =>
    browserPreview
      ? browserSessionFromStorage(globalThis.sessionStorage)
      : EMPTY_PEBBLE_SESSION
  );
  const [loading, setLoading] = useState(!browserPreview);
  const [error, setError] = useState<string | null>(null);

  const updateSession = useCallback(
    (next: PebbleSessionSnapshot) => {
      if (!isPebbleSessionSnapshot(next)) {
        return;
      }

      setSession((current) => newestSession(current, next));
      if (browserPreview) {
        storeBrowserSession(globalThis.sessionStorage, next);
      }
    },
    [browserPreview]
  );

  useEffect(() => {
    if (browserPreview) {
      return;
    }

    let active = true;
    let unlisten: UnlistenFn = () => undefined;

    listenToPebbleSession((next) => {
      if (active) {
        updateSession(next);
      }
    })
      .then((nextUnlisten) => {
        if (!active) {
          nextUnlisten();
          return null;
        }

        unlisten = nextUnlisten;
        return getPebbleSession();
      })
      .then((snapshot) => {
        if (active && snapshot) {
          updateSession(snapshot);
          setError(null);
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(errorMessage(reason, "Pebble session could not be loaded."));
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
      });

    return () => {
      active = false;
      unlisten();
    };
  }, [browserPreview, updateSession]);

  return {
    session,
    loading,
    error,
    browserPreview,
    updateSession,
    setError
  };
}

export function errorMessage(reason: unknown, fallback: string): string {
  if (reason instanceof Error) {
    return reason.message;
  }

  if (
    typeof reason === "object" &&
    reason !== null &&
    "message" in reason &&
    typeof reason.message === "string"
  ) {
    return reason.message;
  }

  return fallback;
}
