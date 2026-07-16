import type { AiModelOption } from "../features/ai/regionQuestion";

export function AiModelSwitch({
  models,
  selectedModel,
  disabled,
  onChange
}: {
  models: readonly AiModelOption[];
  selectedModel: string;
  disabled: boolean;
  onChange: (model: string) => void;
}) {
  return (
    <div className="model-switch" role="group" aria-label="AI MODEL">
      {models.map((model) => (
        <button
          key={model.id}
          type="button"
          className={selectedModel === model.id ? "is-active" : ""}
          aria-pressed={selectedModel === model.id}
          title={model.id.toUpperCase()}
          disabled={disabled}
          onClick={() => onChange(model.id)}
        >
          {model.label}
        </button>
      ))}
    </div>
  );
}
