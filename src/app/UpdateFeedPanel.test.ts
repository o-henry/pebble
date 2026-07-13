import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { UpdateFeedPanel } from "./UpdateFeedPanel";

describe("UpdateFeedPanel", () => {
  it("keeps the divider without rendering empty-state placeholders", () => {
    const markup = renderToStaticMarkup(createElement(UpdateFeedPanel));

    expect(markup).toContain("update-feed--empty");
    expect(markup).not.toContain("UPDATES 0");
    expect(markup).not.toContain("DOWNLOADS/PEBBLE");
    expect(markup).not.toContain("NO SAVED UPDATES YET");
  });
});
