import { invoke } from "@tauri-apps/api/core";
import type { AppStatus } from "../app/appContent";

export interface BackendCommandMap {
  get_app_status: {
    result: AppStatus;
  };
}

export async function invokeBackend<K extends keyof BackendCommandMap>(
  command: K
): Promise<BackendCommandMap[K]["result"]> {
  return invoke<BackendCommandMap[K]["result"]>(command);
}

export function getAppStatus(): Promise<AppStatus> {
  return invokeBackend("get_app_status");
}
