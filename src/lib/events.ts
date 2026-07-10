import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  LIVE_TILE_FRAME_EVENT,
  type LiveTileFrameEvent
} from "../features/live-tile/liveTile";
import {
  PEBBLE_SESSION_UPDATED_EVENT,
  isPebbleSessionSnapshot,
  type PebbleSessionSnapshot
} from "../features/pebble-session/pebbleSession";
import { isTauriRuntime } from "./runtime";

export function listenToLiveTileFrames(
  tileId: string,
  onFrame: (event: LiveTileFrameEvent) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(noop);
  }

  return listen<LiveTileFrameEvent>(LIVE_TILE_FRAME_EVENT, (event) => {
    if (event.payload.tileId === tileId) {
      onFrame(event.payload);
    }
  });
}

export function listenToPebbleSession(
  onSession: (snapshot: PebbleSessionSnapshot) => void
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return Promise.resolve(noop);
  }

  return listen<unknown>(PEBBLE_SESSION_UPDATED_EVENT, (event) => {
    if (isPebbleSessionSnapshot(event.payload)) {
      onSession(event.payload);
    }
  });
}

function noop() {
  return undefined;
}
