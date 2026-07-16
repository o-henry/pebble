import {
  isSmartWatchInterval,
  type SmartWatchIntervalMinutes
} from "./smartWatch";

export const WATCH_RECIPE_STORAGE_KEY = "pebble.watch-recipes.v1";
export const MAX_WATCH_RECIPES = 20;
const MAX_RECIPE_NAME_CHARS = 40;
const MAX_RECIPE_INTENT_CHARS = 500;

export interface WatchRecipe {
  id: string;
  name: string;
  intent: string;
  recommendedIntervalMinutes: SmartWatchIntervalMinutes;
}

interface RecipeStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export const BUILT_IN_WATCH_RECIPES: readonly WatchRecipe[] = [
  builtIn(
    "stuck-after-activity",
    "NO PROGRESS",
    "Tell me when this region stops changing after activity"
  ),
  builtIn("build-failed", "BUILD FAILED", "Tell me when 'BUILD FAILED' appears"),
  builtIn("error-appears", "ERROR APPEARS", "Tell me when 'ERROR' appears"),
  builtIn("progress-complete", "PROGRESS 100%", "Tell me when progress reaches 100%"),
  builtIn("queue-empty", "QUEUE EMPTY", "Tell me when 'QUEUE EMPTY' appears")
];

export function loadWatchRecipes(storage: RecipeStorage): WatchRecipe[] {
  try {
    const value = storage.getItem(WATCH_RECIPE_STORAGE_KEY);
    if (!value) return [];
    const parsed: unknown = JSON.parse(value);
    if (!isRecord(parsed) || parsed.version !== 1 || !Array.isArray(parsed.recipes)) {
      return [];
    }
    return parsed.recipes
      .map(validRecipe)
      .filter((recipe): recipe is WatchRecipe => recipe !== null)
      .slice(0, MAX_WATCH_RECIPES);
  } catch {
    return [];
  }
}

export function saveWatchRecipe(
  storage: RecipeStorage,
  intent: string,
  recommendedIntervalMinutes: SmartWatchIntervalMinutes,
  id = createRecipeId()
): WatchRecipe[] {
  const normalizedIntent = safeText(intent, MAX_RECIPE_INTENT_CHARS);
  if (!normalizedIntent || !isSmartWatchInterval(recommendedIntervalMinutes)) {
    return loadWatchRecipes(storage);
  }
  const current = loadWatchRecipes(storage);
  const duplicate = current.find((recipe) => recipe.intent === normalizedIntent);
  const recipe: WatchRecipe = duplicate
    ? { ...duplicate, recommendedIntervalMinutes }
    : {
        id: safeRecipeId(id) ?? createRecipeId(),
        name: recipeName(normalizedIntent),
        intent: normalizedIntent,
        recommendedIntervalMinutes
      };
  const next = [recipe, ...current.filter((item) => item.id !== recipe.id)]
    .slice(0, MAX_WATCH_RECIPES);
  writeRecipes(storage, next);
  return next;
}

export function removeWatchRecipe(
  storage: RecipeStorage,
  id: string
): WatchRecipe[] {
  const next = loadWatchRecipes(storage).filter((recipe) => recipe.id !== id);
  writeRecipes(storage, next);
  return next;
}

function builtIn(id: string, name: string, intent: string): WatchRecipe {
  return {
    id: `built-in-${id}`,
    name,
    intent,
    recommendedIntervalMinutes: 5
  };
}

function validRecipe(value: unknown): WatchRecipe | null {
  if (!isRecord(value)) return null;
  const id = safeRecipeId(value.id);
  const name = safeText(value.name, MAX_RECIPE_NAME_CHARS);
  const intent = safeText(value.intent, MAX_RECIPE_INTENT_CHARS);
  const interval = value.recommendedIntervalMinutes;
  if (!id || !name || !intent || typeof interval !== "number" || !isSmartWatchInterval(interval)) {
    return null;
  }
  return { id, name, intent, recommendedIntervalMinutes: interval };
}

function writeRecipes(storage: RecipeStorage, recipes: WatchRecipe[]): void {
  try {
    storage.setItem(WATCH_RECIPE_STORAGE_KEY, JSON.stringify({ version: 1, recipes }));
  } catch {
    return;
  }
}

function safeRecipeId(value: unknown): string | null {
  return typeof value === "string" && /^[a-z0-9-]{1,80}$/.test(value)
    ? value
    : null;
}

function safeText(value: unknown, maxChars: number): string | null {
  if (
    typeof value !== "string"
    || Array.from(value).some((character) => {
      const code = character.codePointAt(0) ?? 0;
      return (code < 32 && ![9, 10, 13].includes(code)) || code === 127;
    })
  ) return null;
  const normalized = value.split(/\s+/u).join(" ").trim();
  const valid = normalized.length > 0
    && Array.from(normalized).length <= maxChars;
  return valid ? normalized : null;
}

function recipeName(intent: string): string {
  const characters = Array.from(intent.toUpperCase());
  return characters.length <= MAX_RECIPE_NAME_CHARS
    ? characters.join("")
    : `${characters.slice(0, MAX_RECIPE_NAME_CHARS - 3).join("")}...`;
}

function createRecipeId(): string {
  return `recipe-${globalThis.crypto.randomUUID().toLowerCase()}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
