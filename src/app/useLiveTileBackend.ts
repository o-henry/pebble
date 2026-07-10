import { useCallback, useEffect, useLayoutEffect, useRef } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type {
  LiveTileAction,
  LiveTileCaptureRequest,
  LiveTileMode,
  LiveTileState
} from "../features/live-tile/liveTile";
import {
  createLiveTileRequestScope,
  liveTileRequest,
  scopedLiveTileRequestId,
  shouldAcceptLiveTileFrame,
  shouldAcceptLiveTileResponse
} from "../features/live-tile/liveTile";
import { listenToLiveTileFrames } from "../lib/events";
import { captureLiveTileOnce } from "../lib/invoke";
import { isTauriRuntime } from "../lib/runtime";

export function useLiveTileBackend({
  tile,
  requestMode,
  privacyBlankActive,
  onError,
  dispatch
}: {
  tile: LiveTileState;
  requestMode: LiveTileMode;
  privacyBlankActive: boolean;
  onError: (message: string | null) => void;
  dispatch: (action: LiveTileAction) => void;
}) {
  const backendEnabled = isTauriRuntime();
  const tileRef = useRef(tile);
  const privacyBlankRef = useRef(privacyBlankActive);
  const requestSequenceRef = useRef(0);
  const requestScopeRef = useRef<string | null>(null);
  requestScopeRef.current ??= createLiveTileRequestScope(tile.tileId);
  const activeRequestIdRef = useRef<string | null>(null);
  const nextRequest = useCallback((state: LiveTileState, mode: LiveTileMode) => {
    const requestId = scopedLiveTileRequestId(
      requestScopeRef.current ?? createLiveTileRequestScope(state.tileId),
      requestSequenceRef.current + 1
    );

    requestSequenceRef.current += 1;
    activeRequestIdRef.current = requestId;
    return liveTileRequest(state, requestId, mode);
  }, []);
  const settleBackend = useCallback(
    async (request: LiveTileCaptureRequest) => {
      const result = await captureLiveTileOnce(request);

      if (!result.ok) {
        if (request.requestId === activeRequestIdRef.current) {
          onError(result.error.message);
        }
        return;
      }

      if (shouldAcceptLiveTileResponse(activeRequestIdRef.current, result.response)) {
        onError(null);
        dispatch({ type: "backendSettled", response: result.response });
      }
    },
    [dispatch, onError]
  );

  useEffect(() => {
    tileRef.current = tile;
  }, [tile]);

  useLayoutEffect(() => {
    privacyBlankRef.current = privacyBlankActive;
    if (privacyBlankActive) {
      activeRequestIdRef.current = null;
    }
  }, [privacyBlankActive]);

  useEffect(() => {
    if (privacyBlankActive) {
      globalThis.queueMicrotask(() => dispatch({ type: "privacyBlank" }));
    }
  }, [dispatch, privacyBlankActive]);

  useEffect(() => {
    if (!backendEnabled) {
      return;
    }

    return subscribeFrames(
      tile.tileId,
      dispatch,
      privacyBlankRef,
      activeRequestIdRef
    );
  }, [backendEnabled, dispatch, tile.tileId]);

  useEffect(() => {
    if (
      !backendEnabled ||
      requestMode === "live" ||
      requestMode === "blanked"
    ) {
      return;
    }

    const timeout = globalThis.setTimeout(() => {
      void settleBackend(nextRequest(tileRef.current, requestMode));
    }, 0);

    return () => globalThis.clearTimeout(timeout);
  }, [
    backendEnabled,
    nextRequest,
    settleBackend,
    requestMode,
    tile.fps,
    tile.region,
    tile.tileId
  ]);

  useEffect(() => {
    if (!backendEnabled || requestMode !== "live") {
      return;
    }

    const stopPolling = startSequentialPolling(tile.fps, () =>
      settleBackend(nextRequest(tileRef.current, "live")).catch(
        (reason: unknown) => {
          onError(reason instanceof Error ? reason.message : "Live tile failed");
        }
      )
    );

    return () => {
      activeRequestIdRef.current = null;
      stopPolling();
    };
  }, [
    backendEnabled,
    nextRequest,
    onError,
    requestMode,
    settleBackend,
    tile.fps,
    tile.region,
    tile.tileId
  ]);

  return {
    clearActiveRequest: () => {
      activeRequestIdRef.current = null;
    }
  };
}

function subscribeFrames(
  tileId: string,
  dispatch: (action: LiveTileAction) => void,
  privacyBlankRef: { current: boolean },
  activeRequestIdRef: { current: string | null }
) {
  let unlisten: UnlistenFn = () => undefined;
  let active = true;

  listenToLiveTileFrames(tileId, (event) => {
    if (
      shouldAcceptLiveTileFrame(
        activeRequestIdRef.current,
        privacyBlankRef.current,
        event
      )
    ) {
      dispatch({ type: "frameReceived", event });
    }
  })
    .then((nextUnlisten) => {
      if (active) {
        unlisten = nextUnlisten;
      } else {
        nextUnlisten();
      }
    })
    .catch(() => undefined);

  return () => {
    active = false;
    unlisten();
  };
}

function startSequentialPolling(fps: number, tick: () => Promise<void>) {
  let timeout: number | undefined;
  let cancelled = false;
  const schedule = () => {
    if (!cancelled) {
      timeout = globalThis.setTimeout(run, 1000 / fps);
    }
  };
  const run = () => {
    if (cancelled) {
      return;
    }
    void tick().finally(schedule);
  };

  globalThis.queueMicrotask(run);
  return () => {
    cancelled = true;
    if (timeout !== undefined) {
      globalThis.clearTimeout(timeout);
    }
  };
}
