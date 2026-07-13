export const MAX_CLAUDE_API_KEY_LENGTH = 512;

export interface ClaudeCredentialStatus {
  apiKeyConfigured: boolean;
}

export function normalizedClaudeApiKey(value: string): string | null {
  const key = value.trim();
  if (
    key.length < 20 ||
    key.length > MAX_CLAUDE_API_KEY_LENGTH ||
    !key.startsWith("sk-ant-") ||
    !/^[A-Za-z0-9_-]+$/.test(key)
  ) {
    return null;
  }
  return key;
}
