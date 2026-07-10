import { describe, expect, it } from "vitest";
import { isTauriRuntime } from "./runtime";

describe("isTauriRuntime", () => {
  it("keeps browser previews outside the desktop runtime boundary", () => {
    expect(isTauriRuntime({})).toBe(false);
  });

  it("recognizes the Tauri runtime marker", () => {
    expect(isTauriRuntime({ __TAURI_INTERNALS__: {} })).toBe(true);
  });
});
