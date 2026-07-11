import { describe, expect, it } from "vitest";
import { normalizedPublicSourceUrl } from "./publicSource";

describe("public source URL", () => {
  it("accepts only credential-free HTTPS URLs without fragments", () => {
    expect(normalizedPublicSourceUrl(" https://example.com/feed.xml ")).toBe(
      "https://example.com/feed.xml"
    );
    expect(normalizedPublicSourceUrl("http://example.com")).toBeNull();
    expect(normalizedPublicSourceUrl("https://user:pass@example.com")).toBeNull();
    expect(normalizedPublicSourceUrl("https://example.com/#private")).toBeNull();
  });
});
