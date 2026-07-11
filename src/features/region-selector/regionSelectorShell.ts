export interface RegionSelectorWindowShell {
  label: string;
  title: string;
  visualOverlay: boolean;
  nativeTransparent: boolean;
  alwaysOnTop: boolean;
  captureActive: boolean;
}

export const REGION_SELECTOR_DEFAULT_SHELL: RegionSelectorWindowShell = {
  label: "pebble-region-selector",
  title: "Select Region",
  visualOverlay: true,
  nativeTransparent: true,
  alwaysOnTop: true,
  captureActive: false
};
