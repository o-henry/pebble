import { invoke } from "@tauri-apps/api/core";
import type { AppStatus } from "../app/appContent";
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
import type {
  TileWindowState,
  WindowShellSnapshot
} from "../features/window-shell/tileWindowState";

export interface BackendCommandMap {
  get_app_status: {
    result: AppStatus;
  };
  get_performance_limits: {
    result: PerformanceLimits;
  };
  validate_performance_request: {
    args: {
      request: PerformanceLimitRequest;
    };
    result: PerformanceValidationResult;
  };
  resolve_region_selection: {
    args: {
      request: RegionSelectionRequest;
    };
    result: RegionSelection;
  };
  get_window_shell_snapshot: {
    result: WindowShellSnapshot;
  };
  open_test_tile_window: {
    result: TileWindowState;
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

export function getWindowShellSnapshot(): Promise<WindowShellSnapshot> {
  return invokeBackend("get_window_shell_snapshot");
}

export function openTestTileWindow(): Promise<TileWindowState> {
  return invokeBackend("open_test_tile_window");
}

export function openRegionSelectorWindow(): Promise<RegionSelectorWindowShell> {
  return invokeBackend("open_region_selector_window");
}

export function getRegionSelectorMonitor(): Promise<
  RegionSelectionRequest["monitor"]
> {
  return invokeBackend("get_region_selector_monitor");
}

export function closeRegionSelectorWindow(): Promise<void> {
  return invokeBackend("close_region_selector_window");
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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
