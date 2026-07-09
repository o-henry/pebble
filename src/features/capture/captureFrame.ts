import type { PhysicalRegion } from "../region-selector/regionSelection";

export type FramePixelFormat = "rgba8";
export type FrameStoragePolicy = "memoryOnly";

export interface CroppedFramePayload {
  monitorId: string;
  region: PhysicalRegion;
  width: number;
  height: number;
  pixelFormat: FramePixelFormat;
  bytesPerPixel: number;
  storagePolicy: FrameStoragePolicy;
  bytes: number[];
}

export type CaptureErrorCode =
  | "activeTileLimitExceeded"
  | "captureUnavailable"
  | "invalidRegion"
  | "monitorUnavailable"
  | "permissionDenied"
  | "platformUnavailable"
  | "regionTooLarge"
  | "regionOutOfBounds"
  | "unsupportedPixelFormat";

export interface CaptureError {
  code: CaptureErrorCode;
  monitorId: string;
  message: string;
}

export type CaptureRegionResult =
  | { ok: true; frame: CroppedFramePayload }
  | { ok: false; error: CaptureError };
