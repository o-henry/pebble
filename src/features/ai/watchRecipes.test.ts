import { describe, expect, it } from "vitest";
import {
  BUILT_IN_WATCH_RECIPES,
  MAX_WATCH_RECIPES,
  WATCH_RECIPE_STORAGE_KEY,
  clearWatchRecipes,
  loadWatchRecipes,
  removeWatchRecipe,
  saveWatchRecipe
} from "./watchRecipes";

function memoryStorage(initial?: string) {
  const values = new Map<string, string>();
  if (initial) values.set(WATCH_RECIPE_STORAGE_KEY, initial);
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key)
  };
}

describe("watch recipes", () => {
  it("keeps only recipes that require explicit multi-region roles", () => {
    expect(BUILT_IN_WATCH_RECIPES.map((recipe) => recipe.name)).toEqual([
      "FOLLOW START",
      "FOLLOW RESULT",
      "CROSS CHECK"
    ]);
  });

  it("stores only the explicit recipe fields", () => {
    const storage = memoryStorage();
    const recipes = saveWatchRecipe(storage, "  Tell me when READY appears  ", 5, "recipe-ready");
    expect(recipes[0]).toEqual({
      id: "recipe-ready",
      name: "TELL ME WHEN READY APPEARS",
      intent: "Tell me when READY appears",
      recommendedIntervalMinutes: 5
    });
    const serialized = storage.getItem(WATCH_RECIPE_STORAGE_KEY) ?? "";
    expect(serialized).not.toMatch(/region|coordinate|frame|image|ocr|provider|model|token|key/i);
  });

  it("ignores malformed, oversized, and excessive records", () => {
    const recipes = Array.from({ length: MAX_WATCH_RECIPES + 5 }, (_, index) => ({
      id: `recipe-${index}`,
      name: `RECIPE ${index}`,
      intent: `Tell me when ITEM ${index} appears`,
      recommendedIntervalMinutes: 5
    }));
    recipes[0].intent = "x".repeat(501);
    const storage = memoryStorage(JSON.stringify({ version: 1, recipes }));
    expect(loadWatchRecipes(storage)).toHaveLength(MAX_WATCH_RECIPES);
    const malformed = memoryStorage("not json");
    expect(loadWatchRecipes(malformed)).toEqual([]);
    expect(malformed.getItem(WATCH_RECIPE_STORAGE_KEY)).toBeNull();
  });

  it("rewrites partially valid storage without unknown or sensitive fields", () => {
    const storage = memoryStorage(JSON.stringify({
      version: 1,
      recipes: [{
        id: "recipe-safe",
        name: "SAFE",
        intent: "Tell me when READY appears",
        recommendedIntervalMinutes: 5,
        capturedText: "private value"
      }]
    }));

    expect(loadWatchRecipes(storage)).toHaveLength(1);
    expect(storage.getItem(WATCH_RECIPE_STORAGE_KEY)).not.toContain("private value");
  });

  it("deduplicates and removes saved recipes", () => {
    const storage = memoryStorage();
    saveWatchRecipe(storage, "Tell me when READY appears", 5, "recipe-one");
    expect(saveWatchRecipe(storage, "Tell me when READY appears", 30, "recipe-two")).toHaveLength(1);
    expect(removeWatchRecipe(storage, "recipe-one")).toEqual([]);
  });

  it("clears every saved custom recipe without affecting built-ins", () => {
    const storage = memoryStorage();
    saveWatchRecipe(storage, "Tell me when READY appears", 5, "recipe-one");

    expect(clearWatchRecipes(storage)).toEqual([]);
    expect(loadWatchRecipes(storage)).toEqual([]);
    expect(BUILT_IN_WATCH_RECIPES).toHaveLength(3);
  });
});
