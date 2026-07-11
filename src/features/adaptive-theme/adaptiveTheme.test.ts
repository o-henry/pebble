import { describe, expect, it } from "vitest";
import type { CroppedFramePayload } from "../capture/captureFrame";
import { deriveAdaptiveTheme } from "./adaptiveTheme";

describe("deriveAdaptiveTheme", () => {
  it("uses a dark readable surface for black regions", () => {
    const theme = deriveAdaptiveTheme(frameWithPixels([[0, 0, 0], [8, 8, 8]]));

    expect(theme?.mode).toBe("dark");
    expect(theme?.variables["--surface"]).toBe("rgb(8 8 8)");
    expect(theme?.variables["--ink-strong"]).toBe("rgb(255 255 255)");
  });

  it("uses a light readable surface for white regions", () => {
    const theme = deriveAdaptiveTheme(
      frameWithPixels([[255, 255, 255], [248, 248, 248]])
    );

    expect(theme?.mode).toBe("light");
    expect(theme?.variables["--surface"]).toBe("rgb(255 255 255)");
    expect(theme?.variables["--ink-strong"]).toBe("rgb(17 20 22)");
  });

  it("follows the dominant background instead of sparse foreground pixels", () => {
    const theme = deriveAdaptiveTheme(
      frameWithPixels([
        [20, 20, 20],
        [240, 240, 240],
        [240, 240, 240],
        [240, 240, 240]
      ])
    );

    expect(theme?.mode).toBe("light");
    expect(theme?.variables["--surface"]).toBe("rgb(240 240 240)");
  });

  it("rejects malformed frames", () => {
    const frame = frameWithPixels([[0, 0, 0]]);
    frame.bytes.pop();

    expect(deriveAdaptiveTheme(frame)).toBeNull();
  });
});

function frameWithPixels(pixels: number[][]): CroppedFramePayload {
  return {
    monitorId: "main",
    region: {
      monitorId: "main",
      x: 0,
      y: 0,
      width: pixels.length,
      height: 1
    },
    width: pixels.length,
    height: 1,
    pixelFormat: "rgba8",
    bytesPerPixel: 4,
    storagePolicy: "memoryOnly",
    bytes: pixels.flatMap(([red, green, blue]) => [red, green, blue, 255])
  };
}
