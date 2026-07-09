import { invoke } from "@tauri-apps/api/core";
import type { AppStatus } from "../app/appContent";
import type {
  PerformanceLimitRequest,
  PerformanceLimits,
  PerformanceValidationResult
} from "../features/performance/performanceLimits";

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
