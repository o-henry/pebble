export type AdaptiveThemeMode = "light" | "dark";

export interface BackdropColor {
  red: number;
  green: number;
  blue: number;
}

export interface AdaptiveTheme {
  mode: AdaptiveThemeMode;
  variables: Readonly<Record<string, string>>;
}

export function deriveAdaptiveTheme(
  sample: BackdropColor
): AdaptiveTheme | null {
  if (!isValidColor(sample)) {
    return null;
  }

  const color = [sample.red, sample.green, sample.blue];
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

function isValidColor(color: BackdropColor) {
  return [color.red, color.green, color.blue].every(
    (channel) =>
      Number.isInteger(channel) && channel >= 0 && channel <= 255
  );
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
