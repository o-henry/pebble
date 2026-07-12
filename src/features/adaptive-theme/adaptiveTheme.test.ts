import { describe, expect, it } from "vitest";
import { deriveAdaptiveTheme } from "./adaptiveTheme";

describe("deriveAdaptiveTheme", () => {
  it("uses a dark readable surface behind a dark window location", () => {
    const theme = deriveAdaptiveTheme({ red: 8, green: 8, blue: 8 });

    expect(theme?.mode).toBe("dark");
    expect(theme?.variables["--surface"]).toBe("rgb(8 8 8)");
    expect(theme?.variables["--ink-strong"]).toBe("rgb(255 255 255)");
  });

  it("uses a light readable surface behind a light window location", () => {
    const theme = deriveAdaptiveTheme({ red: 248, green: 248, blue: 248 });

    expect(theme?.mode).toBe("light");
    expect(theme?.variables["--surface"]).toBe("rgb(248 248 248)");
    expect(theme?.variables["--ink-strong"]).toBe("rgb(17 20 22)");
  });

  it("preserves the sampled backdrop hue", () => {
    const theme = deriveAdaptiveTheme({ red: 32, green: 104, blue: 152 });

    expect(theme?.variables["--surface"]).toBe("rgb(32 104 152)");
  });

  it("rejects malformed color channels", () => {
    expect(deriveAdaptiveTheme({ red: -1, green: 0, blue: 0 })).toBeNull();
    expect(deriveAdaptiveTheme({ red: 0.5, green: 0, blue: 0 })).toBeNull();
    expect(deriveAdaptiveTheme({ red: 0, green: 0, blue: 256 })).toBeNull();
  });
});
