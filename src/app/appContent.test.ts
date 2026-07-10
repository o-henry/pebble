import { describe, expect, it } from "vitest";
import { appStatus, docReferences, principles } from "./appContent";

describe("app shell content", () => {
  it("identifies the current macOS phase with capture enabled and AI off", () => {
    expect(appStatus).toEqual({
      phase: "pre-alpha",
      scaffoldReady: true,
      captureEnabled: true,
      aiEnabled: false
    });
  });

  it("points implementers to the required local planning documents", () => {
    expect(docReferences.map((doc) => doc.label)).toEqual([
      "Product Spec",
      "Architecture",
      "Implementation Plan",
      "Security Policy"
    ]);
    expect(
      docReferences.every(
        (doc) => doc.path.startsWith("docs/") && doc.path.endsWith(".md")
      )
    ).toBe(true);
  });

  it("keeps the launch principles focused on privacy and local monitoring", () => {
    expect(principles).toHaveLength(4);
    expect(principles.some((item) => item.title.includes("AI"))).toBe(true);
  });
});
