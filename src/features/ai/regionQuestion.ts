export const MAX_REGION_QUESTION_LENGTH = 1_000;

export type AiProvider = "openAi" | "claude";
export type AiConnectionMode = "account" | "apiKey" | "subscription";

export interface AiModelOption {
  id: string;
  label: string;
}

const DEFAULT_MODEL_IDS: Record<AiProvider, string> = {
  openAi: "gpt-5.6-terra",
  claude: "sonnet"
};

const MODEL_STORAGE_KEYS: Record<AiProvider, string> = {
  openAi: "pebble.ai-model.openai",
  claude: "pebble.ai-model.claude"
};

export function defaultAiModelLabel(provider: AiProvider) {
  return provider === "openAi" ? "GPT-5.6-TERRA" : "CLAUDE SONNET 5";
}

export function defaultAiModelId(provider: AiProvider): string {
  return DEFAULT_MODEL_IDS[provider];
}

export interface AiConnectionStatus {
  provider: AiProvider;
  available: boolean;
  connected: boolean;
  model: string;
  models: AiModelOption[];
  installUrl: string | null;
  connectionMode: AiConnectionMode | null;
}

interface ModelStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function selectedAiModel(
  provider: AiProvider,
  models: readonly AiModelOption[],
  storage: ModelStorage
): string {
  const remembered = storage.getItem(MODEL_STORAGE_KEYS[provider]);
  if (remembered && models.some((model) => model.id === remembered)) {
    return remembered;
  }
  const preferred = defaultAiModelId(provider);
  return models.find((model) => model.id === preferred)?.id ?? models[0]?.id ?? preferred;
}

export function rememberAiModel(
  provider: AiProvider,
  model: string,
  storage: ModelStorage
): void {
  storage.setItem(MODEL_STORAGE_KEYS[provider], model);
}

export function aiAccessLabel(mode: AiConnectionMode | null | undefined) {
  if (mode === "apiKey") return "API BILLING";
  if (mode === "subscription") return "SUBSCRIPTION";
  if (mode === "account") return "ACCOUNT";
  return "";
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
