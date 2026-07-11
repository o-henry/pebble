import { useEffect, useLayoutEffect, type MutableRefObject } from "react";
import type { LiveTileAction } from "../features/live-tile/liveTile";

export function useCaptureVisibilityGuards({
  documentVisible,
  privacyBlankActive,
  activeRequestIdRef,
  privacyBlankRef,
  dispatch
}: {
  documentVisible: boolean;
  privacyBlankActive: boolean;
  activeRequestIdRef: MutableRefObject<string | null>;
  privacyBlankRef: MutableRefObject<boolean>;
  dispatch: (action: LiveTileAction) => void;
}) {
  useLayoutEffect(() => {
    privacyBlankRef.current = privacyBlankActive;
    if (privacyBlankActive) activeRequestIdRef.current = null;
  }, [activeRequestIdRef, privacyBlankActive, privacyBlankRef]);

  useEffect(() => {
    if (privacyBlankActive) {
      globalThis.queueMicrotask(() => dispatch({ type: "privacyBlank" }));
    }
  }, [dispatch, privacyBlankActive]);

  useLayoutEffect(() => {
    if (!documentVisible) {
      activeRequestIdRef.current = null;
      dispatch({ type: "windowHidden" });
    }
  }, [activeRequestIdRef, dispatch, documentVisible]);
}
