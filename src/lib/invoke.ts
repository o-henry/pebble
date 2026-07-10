import { invoke } from "@tauri-apps/api/core";
import type { AppStatus } from "../app/appContent";
import type {
  AiAnswer,
  AiConnectionStatus
} from "../features/ai/regionQuestion";
import type { CaptureError } from "../features/capture/captureFrame";
import type {
  LiveTileCaptureRequest,
  LiveTileCaptureResponse,
  LiveTileCaptureResult
} from "../features/live-tile/liveTile";
import type { PebbleSessionSnapshot } from "../features/pebble-session/pebbleSession";
import type {
  PerformanceLimitRequest,
  PerformanceLimits,
  PerformanceValidationResult
} from "../features/performance/performanceLimits";
import type {
  RegionSelection,
  RegionSelectionIssue,
  RegionSelectionRequest
} from "../features/region-selector/regionSelection";
import type { RegionSelectorWindowShell } from "../features/region-selector/regionSelectorShell";

export interface BackendCommandMap {
  get_app_status: {
    result: AppStatus;
  };
  get_performance_limits: {
    result: PerformanceLimits;
  };
  validate_performance_request: {
    args: { request: PerformanceLimitRequest };
    result: PerformanceValidationResult;
  };
  resolve_region_selection: {
    args: { request: RegionSelectionRequest };
    result: RegionSelection;
  };
  open_region_selector_window: {
    result: RegionSelectorWindowShell;
  };
  get_region_selector_monitor: {
    result: RegionSelectionRequest["monitor"];
  };
  close_region_selector_window: {
    result: void;
  };
  get_pebble_session: {
    result: PebbleSessionSnapshot;
  };
  confirm_pebble_region: {
    args: { request: RegionSelectionRequest };
    result: PebbleSessionSnapshot;
  };
  show_pebble_window: {
    result: PebbleSessionSnapshot;
  };
  set_pebble_privacy_blank: {
    args: { active: boolean };
    result: PebbleSessionSnapshot;
  };
  remove_pebble: {
    result: PebbleSessionSnapshot;
  };
  close_pebble_window: {
    result: PebbleSessionSnapshot;
  };
  request_screen_capture_access: {
    result: boolean;
  };
  get_ai_connection_status: {
    result: AiConnectionStatus;
  };
  connect_chatgpt: {
    result: AiConnectionStatus;
  };
  ask_selected_region: {
    args: { question: string };
    result: AiAnswer;
  };
  capture_live_tile_once: {
    args: { request: LiveTileCaptureRequest };
    result: LiveTileCaptureResponse;
  };
}

type BackendCommandResult<K extends keyof BackendCommandMap> =
  BackendCommandMap[K]["result"];

type BackendCommandArgs<K extends keyof BackendCommandMap> =
  BackendCommandMap[K] extends { args: infer Args } ? Args : never;

type BackendCommandsWithArgs = {
  [K in keyof BackendCommandMap]: BackendCommandMap[K] extends { args: unknown }
    ? K
    : never;
}[keyof BackendCommandMap];

type BackendCommandsWithoutArgs = Exclude<
  keyof BackendCommandMap,
  BackendCommandsWithArgs
>;

export function invokeBackend<K extends BackendCommandsWithoutArgs>(
  command: K
): Promise<BackendCommandResult<K>>;
export function invokeBackend<K extends BackendCommandsWithArgs>(
  command: K,
  args: BackendCommandArgs<K>
): Promise<BackendCommandResult<K>>;
export function invokeBackend<K extends keyof BackendCommandMap>(
  command: K,
  args?: unknown
): Promise<BackendCommandResult<K>> {
  return invoke<BackendCommandResult<K>>(
    command,
    args as Record<string, unknown> | undefined
  );
}

export function getAppStatus(): Promise<AppStatus> {
  return invokeBackend("get_app_status");
}

export function getPerformanceLimits(): Promise<PerformanceLimits> {
  return invokeBackend("get_performance_limits");
}

export function validateBackendPerformanceRequest(
  request: PerformanceLimitRequest
): Promise<PerformanceValidationResult> {
  return invokeBackend("validate_performance_request", {
    request
  });
}

export function resolveBackendRegionSelection(
  request: RegionSelectionRequest
): Promise<
  | { ok: true; selection: RegionSelection }
  | { ok: false; error: RegionSelectionIssue }
> {
  return invokeBackend("resolve_region_selection", {
    request
  })
    .then((selection) => ({ ok: true as const, selection }))
    .catch((error: unknown) => {
      if (isRegionSelectionIssue(error)) {
        return { ok: false as const, error };
      }

      throw error;
    });
}

export function openRegionSelectorWindow(): Promise<RegionSelectorWindowShell> {
  return invokeBackend("open_region_selector_window");
}

export function getRegionSelectorMonitor(): Promise<RegionSelectionRequest["monitor"]> {
  return invokeBackend("get_region_selector_monitor");
}

export function closeRegionSelectorWindow(): Promise<void> {
  return invokeBackend("close_region_selector_window");
}

export function getPebbleSession(): Promise<PebbleSessionSnapshot> {
  return invokeBackend("get_pebble_session");
}

export function confirmPebbleRegion(
  request: RegionSelectionRequest
): Promise<PebbleSessionSnapshot> {
  return invokeBackend("confirm_pebble_region", { request });
}

export function showPebbleWindow(): Promise<PebbleSessionSnapshot> {
  return invokeBackend("show_pebble_window");
}

export function setPebblePrivacyBlank(
  active: boolean
): Promise<PebbleSessionSnapshot> {
  return invokeBackend("set_pebble_privacy_blank", { active });
}

export function removePebble(): Promise<PebbleSessionSnapshot> {
  return invokeBackend("remove_pebble");
}

export function closePebbleWindow(): Promise<PebbleSessionSnapshot> {
  return invokeBackend("close_pebble_window");
}

export function requestScreenCaptureAccess(): Promise<boolean> {
  return invokeBackend("request_screen_capture_access");
}

let pendingAiConnectionStatus: Promise<AiConnectionStatus> | null = null;

export function getAiConnectionStatus(): Promise<AiConnectionStatus> {
  if (!pendingAiConnectionStatus) {
    pendingAiConnectionStatus = invokeBackend("get_ai_connection_status").finally(
      () => {
        pendingAiConnectionStatus = null;
      }
    );
  }
  return pendingAiConnectionStatus;
}

export function connectChatGPT(): Promise<AiConnectionStatus> {
  return invokeBackend("connect_chatgpt");
}

export function askSelectedRegion(question: string): Promise<AiAnswer> {
  return invokeBackend("ask_selected_region", { question });
}

export function captureLiveTileOnce(
  request: LiveTileCaptureRequest
): Promise<LiveTileCaptureResult> {
  return recoverCaptureError(
    invokeBackend("capture_live_tile_once", {
      request
    }).then((response) => ({
      ok: true as const,
      response
    }))
  );
}

function isCaptureError(error: unknown): error is CaptureError {
  if (!isRecord(error)) {
    return false;
  }

  return (
    typeof error.code === "string" &&
    typeof error.monitorId === "string" &&
    typeof error.message === "string"
  );
}

function isRegionSelectionIssue(error: unknown): error is RegionSelectionIssue {
  if (!isRecord(error)) {
    return false;
  }

  return (
    typeof error.code === "string" &&
    typeof error.limit === "number" &&
    typeof error.actual === "number" &&
    typeof error.message === "string"
  );
}

function recoverCaptureError<T>(
  promise: Promise<T>
): Promise<T | { ok: false; error: CaptureError }> {
  return promise.catch((error: unknown) => {
    if (isCaptureError(error)) {
      return { ok: false as const, error };
    }

    throw error;
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
