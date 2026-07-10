export function isTauriRuntime(runtime: object = globalThis): boolean {
  return Object.prototype.hasOwnProperty.call(runtime, "__TAURI_INTERNALS__");
}
