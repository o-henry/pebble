import type { CroppedFramePayload } from "../capture/captureFrame";

export type AdaptiveThemeMode = "light" | "dark";

export interface AdaptiveTheme {
  mode: AdaptiveThemeMode;
  variables: Readonly<Record<string, string>>;
}

const MAX_SAMPLES = 1_024;

export function deriveAdaptiveTheme(
  frame: CroppedFramePayload
): AdaptiveTheme | null {
  if (!isValidRgbaFrame(frame)) {
    return null;
  }

  const color = representativeColor(frame);
  const mode = relativeLuminance(color) < 0.38 ? "dark" : "light";
  const ink = mode === "dark" ? [245, 247, 248] : [37, 41, 44];
  const inkStrong = mode === "dark" ? [255, 255, 255] : [17, 20, 22];
  const softDirection = mode === "dark" ? [255, 255, 255] : [0, 0, 0];

  return {
    mode,
    variables: {
      "--canvas": cssColor(color),
      "--surface": cssColor(color),
      "--surface-soft": cssColor(mix(color, softDirection, 0.07)),
      "--ink": cssColor(ink),
      "--ink-strong": cssColor(inkStrong),
      "--ink-muted": cssColor(mix(color, ink, 0.62)),
      "--line": cssColor(mix(color, ink, 0.18)),
      "--line-strong": cssColor(mix(color, ink, 0.34)),
      "--blue": mode === "dark" ? "#7ea2ff" : "#2457d6",
      "--blue-strong": mode === "dark" ? "#a4bcff" : "#1945b5",
      "--blue-soft": cssColor(mix(color, [72, 111, 214], 0.2)),
      "--green": mode === "dark" ? "#57c8a6" : "#147a5b",
      "--amber": mode === "dark" ? "#f2ad54" : "#a7690e",
      "--amber-soft": cssColor(mix(color, [242, 173, 84], 0.18)),
      "--danger": mode === "dark" ? "#ff8d83" : "#a94037",
      "--danger-soft": cssColor(mix(color, [220, 89, 78], 0.16)),
      "--frame-line": cssColor(mix(color, ink, 0.32))
    }
  };
}

function isValidRgbaFrame(frame: CroppedFramePayload) {
  return (
    frame.pixelFormat === "rgba8" &&
    frame.width > 0 &&
    frame.height > 0 &&
    frame.bytes.length === frame.width * frame.height * 4
  );
}

function representativeColor(frame: CroppedFramePayload): number[] {
  const pixelCount = frame.width * frame.height;
  const stride = Math.max(1, Math.ceil(Math.sqrt(pixelCount / MAX_SAMPLES)));
  const channels: [number[], number[], number[]] = [[], [], []];

  for (let y = 0; y < frame.height; y += stride) {
    for (let x = 0; x < frame.width; x += stride) {
      const offset = (y * frame.width + x) * 4;
      channels[0].push(frame.bytes[offset]);
      channels[1].push(frame.bytes[offset + 1]);
      channels[2].push(frame.bytes[offset + 2]);
    }
  }

  return channels.map((values) => quantize(median(values)));
}

function median(values: number[]) {
  values.sort((left, right) => left - right);
  return values[Math.floor(values.length / 2)] ?? 0;
}

function quantize(value: number) {
  return Math.min(255, Math.max(0, Math.round(value / 8) * 8));
}

function relativeLuminance(color: number[]) {
  const [red, green, blue] = color.map((channel) => {
    const value = channel / 255;
    return value <= 0.04045
      ? value / 12.92
      : Math.pow((value + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

function mix(base: number[], target: number[], amount: number) {
  return base.map((channel, index) =>
    Math.round(channel + (target[index] - channel) * amount)
  );
}

function cssColor(color: number[]) {
  return `rgb(${color[0]} ${color[1]} ${color[2]})`;
}
