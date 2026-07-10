import { useLayoutEffect, useRef } from "react";
import { INITIAL_LIVE_TILE_STATE } from "../features/live-tile/liveTile";
import type { LiveTileState } from "../features/live-tile/liveTile";

export function LiveFrameCanvas({
  frame
}: {
  frame: LiveTileState["latestFrame"];
}) {
  const ref = useRef<HTMLCanvasElement | null>(null);
  const width = frame?.width ?? INITIAL_LIVE_TILE_STATE.region.width;
  const height = frame?.height ?? INITIAL_LIVE_TILE_STATE.region.height;

  useLayoutEffect(() => {
    const canvas = ref.current;
    const context = canvas?.getContext("2d");

    if (!canvas || !context) {
      return;
    }

    context.clearRect(0, 0, canvas.width, canvas.height);
    if (frame && frame.bytes.length === frame.width * frame.height * 4) {
      context.putImageData(
        new ImageData(new Uint8ClampedArray(frame.bytes), frame.width),
        0,
        0
      );
    }
  }, [frame]);

  return (
    <canvas
      ref={ref}
      className={"live-frame-canvas " + (frame ? "has-frame" : "is-empty")}
      width={width}
      height={height}
      aria-label={frame ? "Latest live tile frame" : "No live tile frame"}
    />
  );
}
