import { describe, expect, it } from "vitest";
import {
  MAX_CLAUDE_API_KEY_LENGTH,
  normalizedClaudeApiKey
} from "./claudeCredential";

describe("Claude credentials", () => {
  it("normalizes a valid Anthropic API key", () => {
    const key = `sk-ant-${"x".repeat(24)}`;
    expect(normalizedClaudeApiKey(`  ${key}  `)).toBe(key);
  });

  it("rejects malformed or oversized credentials", () => {
    expect(normalizedClaudeApiKey("not-an-anthropic-key")).toBeNull();
    expect(normalizedClaudeApiKey("sk-ant-key with space")).toBeNull();
    expect(
      normalizedClaudeApiKey(`sk-ant-${"a".repeat(MAX_CLAUDE_API_KEY_LENGTH)}`)
    ).toBeNull();
  });
});
