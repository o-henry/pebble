import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { AiPanelHeader } from "./AiPanelHeader";

const baseProps = {
  browserPreview: false,
  connection: "connected" as const,
  provider: "openAi" as const,
  model: "gpt-5.6-terra",
  watchIntent: "",
  disabled: false,
  privacyBlankActive: false,
  intervalMinutes: 1 as const,
  onProviderChange: () => undefined,
  onToggleOptions: () => undefined,
  onIntervalChange: () => undefined,
  onBusyChange: () => undefined,
  onError: () => undefined
};

describe("AI panel header", () => {
  it("keeps the default Watch path compact", () => {
    const markup = renderToStaticMarkup(createElement(AiPanelHeader, {
      ...baseProps,
      optionsExpanded: false
    }));

    expect(markup).toContain("AI");
    expect(markup).toContain("WATCH");
    expect(markup).toContain("OPTIONS");
    expect(markup).not.toContain("INTERVAL");
    expect(markup).not.toContain("OPENAI");
    expect(markup).not.toContain("CLAUDE");
  });

  it("reveals interval and provider choices only in Options", () => {
    const markup = renderToStaticMarkup(createElement(AiPanelHeader, {
      ...baseProps,
      optionsExpanded: true
    }));

    expect(markup).toContain("DONE");
    expect(markup).toContain("INTERVAL");
    expect(markup).toContain("1 MIN");
    expect(markup).toContain("PROVIDER");
    expect(markup).toContain("OPENAI");
    expect(markup).toContain("CLAUDE");
  });
});
