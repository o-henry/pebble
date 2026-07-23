import { useState } from "react";
import type { SmartWatchIntervalMinutes } from "../features/ai/smartWatch";
import {
  BUILT_IN_WATCH_RECIPES,
  clearWatchRecipes,
  loadWatchRecipes,
  removeWatchRecipe,
  saveWatchRecipe,
  type WatchRecipe
} from "../features/ai/watchRecipes";

export function WatchRecipeBar({
  intent,
  disabled,
  intervalMinutes,
  onApply
}: {
  intent: string;
  disabled: boolean;
  intervalMinutes: SmartWatchIntervalMinutes;
  onApply: (recipe: WatchRecipe) => void;
}) {
  const [saved, setSaved] = useState(() =>
    loadWatchRecipes(globalThis.localStorage)
  );
  const [selectedSavedId, setSelectedSavedId] = useState<string | null>(null);
  const canSave = intent.trim().length > 0 && intent.trim().length <= 500;

  function apply(recipe: WatchRecipe, savedRecipe: boolean) {
    setSelectedSavedId(savedRecipe ? recipe.id : null);
    onApply(recipe);
  }

  function save() {
    if (!canSave) return;
    const next = saveWatchRecipe(
      globalThis.localStorage,
      intent,
      intervalMinutes
    );
    setSaved(next);
    setSelectedSavedId(next[0]?.id ?? null);
  }

  function remove() {
    if (!selectedSavedId) return;
    setSaved(removeWatchRecipe(globalThis.localStorage, selectedSavedId));
    setSelectedSavedId(null);
  }

  function clear() {
    setSaved(clearWatchRecipes(globalThis.localStorage));
    setSelectedSavedId(null);
  }

  return (
    <div className="watch-recipes" aria-label="WATCH PRESETS">
      <span>LINK REGIONS</span>
      {BUILT_IN_WATCH_RECIPES.map((recipe) => (
        <RecipeButton
          key={recipe.id}
          recipe={recipe}
          disabled={disabled}
          active={intent.trim() === recipe.intent}
          onClick={() => apply(recipe, false)}
        />
      ))}
      {saved.length > 0 ? <span>SAVED</span> : null}
      {saved.map((recipe) => (
        <RecipeButton
          key={recipe.id}
          recipe={recipe}
          disabled={disabled}
          active={intent.trim() === recipe.intent}
          onClick={() => apply(recipe, true)}
        />
      ))}
      <button
        type="button"
        title="SAVES THIS WATCH INSTRUCTION ON THIS MAC"
        disabled={disabled || !canSave}
        onClick={save}
      >
        SAVE
      </button>
      {selectedSavedId ? (
        <button type="button" disabled={disabled} onClick={remove}>
          REMOVE
        </button>
      ) : null}
      {saved.length > 0 ? (
        <button
          type="button"
          title="REMOVES ALL SAVED WATCH INSTRUCTIONS FROM THIS MAC"
          disabled={disabled}
          onClick={clear}
        >
          CLEAR
        </button>
      ) : null}
    </div>
  );
}

function RecipeButton({
  recipe,
  disabled,
  active,
  onClick
}: {
  recipe: WatchRecipe;
  disabled: boolean;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      className={active ? "is-active" : undefined}
      title={recipe.intent}
      disabled={disabled}
      onClick={onClick}
    >
      {recipe.name}
    </button>
  );
}
