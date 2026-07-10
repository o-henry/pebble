import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import type { PointerEvent } from "react";
import {
  INITIAL_REGION_SELECTOR_STATE,
  canBeginRegionDrag,
  createRegionSelectionRequest,
  createViewportMonitor,
  dragRect,
  regionSelectorReducer
} from "../features/region-selector/regionSelectorInteraction";
import type { MonitorGeometry } from "../features/region-selector/regionSelection";
import {
  closeRegionSelectorWindow,
  confirmPebbleRegion,
  getRegionSelectorMonitor
} from "../lib/invoke";
import { storeBrowserSession } from "../features/pebble-session/pebbleSession";
import { SelectionBox, SelectorHud, SelectorResult } from "./RegionSelectorParts";

export function RegionSelectorView() {
  const stageRef = useRef<HTMLElement | null>(null);
  const [state, dispatch] = useReducer(
    regionSelectorReducer,
    INITIAL_REGION_SELECTOR_STATE
  );
  const [monitor, setMonitor] = useState<MonitorGeometry | null>(null);
  const [commitError, setCommitError] = useState<string | null>(null);
  const [committing, setCommitting] = useState(false);
  const commitStartedRef = useRef(false);
  const rect = dragRect(state.start, state.current);
  const cancelSelector = useCallback(() => {
    dispatch({ type: "cancel" });
    if (isBrowserPreviewRuntime()) {
      globalThis.location.hash = "";
      return;
    }

    void Promise.resolve()
      .then(() => closeRegionSelectorWindow())
      .catch(() => undefined);
  }, []);

  useTransparentShellClass();
  useSelectorMonitor(setMonitor);
  useEscapeCancel(cancelSelector);

  useEffect(() => {
    if (
      state.status !== "ready" ||
      !state.result?.ok ||
      !state.start ||
      !state.current ||
      commitStartedRef.current
    ) {
      return;
    }

    commitStartedRef.current = true;
    setCommitting(true);
    setCommitError(null);
    const request = createRegionSelectionRequest(
      state.monitor,
      state.start,
      state.current
    );

    if (isBrowserPreviewRuntime()) {
      storeBrowserSession(globalThis.sessionStorage, {
        region: state.result.selection.region,
        windowOpen: false,
        privacyBlankActive: false,
        revision: 1
      });
      globalThis.location.hash = "";
      return;
    }

    confirmPebbleRegion(request).catch((reason: unknown) => {
      const message =
        typeof reason === "object" &&
        reason !== null &&
        "message" in reason &&
        typeof reason.message === "string"
          ? reason.message
          : "Pebble could not be started.";

      setCommitError(message);
      setCommitting(false);
    });
  }, [state]);

  function handlePointerDown(event: PointerEvent<HTMLElement>) {
    if (!canBeginRegionDrag(monitor)) {
      return;
    }

    const point = pointFromEvent(event);

    commitStartedRef.current = false;
    setCommitting(false);
    setCommitError(null);
    event.currentTarget.setPointerCapture(event.pointerId);
    dispatch({
      type: "begin",
      point,
      monitor
    });
  }

  function handlePointerMove(event: PointerEvent<HTMLElement>) {
    if (state.status !== "dragging") {
      return;
    }

    dispatch({
      type: "move",
      point: pointFromEvent(event)
    });
  }

  function handlePointerUp(event: PointerEvent<HTMLElement>) {
    if (state.status !== "dragging") {
      return;
    }

    dispatch({
      type: "finish",
      point: pointFromEvent(event)
    });
  }

  return (
    <main
      ref={stageRef}
      className="selector-shell"
      aria-label="Region selector"
      tabIndex={0}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
    >
      <SelectorHud
        status={state.status}
        onCancel={cancelSelector}
      />
      {rect ? <SelectionBox rect={rect} /> : null}
      <SelectorResult state={state} committing={committing} error={commitError} />
    </main>
  );
}

function pointFromEvent(event: PointerEvent<HTMLElement>) {
  const bounds = event.currentTarget.getBoundingClientRect();

  return {
    x: clamp(event.clientX - bounds.left, 0, bounds.width),
    y: clamp(event.clientY - bounds.top, 0, bounds.height)
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function useEscapeCancel(onCancel: () => void) {
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onCancel();
      }
    }

    globalThis.addEventListener("keydown", handleKeyDown);

    return () => globalThis.removeEventListener("keydown", handleKeyDown);
  }, [onCancel]);
}

function useSelectorMonitor(setMonitor: (monitor: MonitorGeometry) => void) {
  useEffect(() => {
    let active = true;

    Promise.resolve()
      .then(() => getRegionSelectorMonitor())
      .then((monitor) => {
        if (active) {
          setMonitor(monitor);
        }
      })
      .catch(() => {
        if (active && isBrowserPreviewRuntime()) {
          setMonitor(
            createViewportMonitor(
              globalThis.innerWidth,
              globalThis.innerHeight,
              globalThis.devicePixelRatio || 1
            )
          );
        }
      });

    return () => {
      active = false;
    };
  }, [setMonitor]);
}

function isBrowserPreviewRuntime() {
  return !Object.prototype.hasOwnProperty.call(
    globalThis,
    "__TAURI_INTERNALS__"
  );
}

function useTransparentShellClass() {
  useEffect(() => {
    document.documentElement.classList.add("selector-document");
    document.body.classList.add("selector-body");
    stageRefocus();

    return () => {
      document.documentElement.classList.remove("selector-document");
      document.body.classList.remove("selector-body");
    };
  }, []);
}

function stageRefocus() {
  requestAnimationFrame(() => {
    document.querySelector<HTMLElement>(".selector-shell")?.focus();
  });
}
