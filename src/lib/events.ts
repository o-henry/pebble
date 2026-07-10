import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  LIVE_TILE_FRAME_EVENT,
  type LiveTileFrameEvent
} from "../features/live-tile/liveTile";
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

function noop() {
  return undefined;
}
