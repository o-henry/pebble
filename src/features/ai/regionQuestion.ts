export const MAX_REGION_QUESTION_LENGTH = 1_000;

export type AiProvider = "openAi" | "claude";

export interface AiConnectionStatus {
  provider: AiProvider;
  available: boolean;
  connected: boolean;
  model: string;
  installUrl: string | null;
}

export interface AiAnswer {
  answer: string;
  provider: AiProvider;
  model: string;
  durationMs: number;
}

export function normalizedRegionQuestion(value: string): string | null {
  const question = value.trim();
  const characters = Array.from(question);
  if (
    characters.length === 0 ||
    characters.length > MAX_REGION_QUESTION_LENGTH ||
    characters.some(isUnsafeControlCharacter)
  ) {
    return null;
  }
  return question;
}

function isUnsafeControlCharacter(character: string): boolean {
  const codePoint = character.codePointAt(0);
  if (codePoint === undefined) {
    return true;
  }
  return (
    (codePoint < 32 && ![9, 10, 13].includes(codePoint)) || codePoint === 127
  );
}
