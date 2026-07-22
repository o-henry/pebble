import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { LiveTileControls } from "./LiveTileControls";

const baseProps = {
  aiExpanded: false,
  disabled: false,
  privacyBlankActive: false,
  onLive: () => undefined,
  onPause: () => undefined,
  onReselect: () => undefined,
  onToggleAi: () => undefined,
  onTogglePrivacy: () => undefined
};

describe("live tile controls", () => {
  it("shows one capture toggle instead of separate Live and Pause buttons", () => {
    const liveMarkup = renderToStaticMarkup(createElement(LiveTileControls, {
      ...baseProps,
      mode: "live"
    }));
    const pausedMarkup = renderToStaticMarkup(createElement(LiveTileControls, {
      ...baseProps,
      mode: "paused"
    }));

    expect(liveMarkup).toContain(">PAUSE</button>");
    expect(liveMarkup).toContain('aria-label="PAUSE LIVE CAPTURE"');
    expect(liveMarkup).not.toContain(">LIVE</button>");
    expect(pausedMarkup).toContain(">LIVE</button>");
    expect(pausedMarkup).toContain('aria-label="RESUME LIVE CAPTURE"');
    expect(pausedMarkup).not.toContain(">PAUSE</button>");
    expect(pausedMarkup).not.toContain("is-active");
  });
});
