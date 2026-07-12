import type { AiProvider } from "../features/ai/regionQuestion";

const PROVIDERS: ReadonlyArray<{
  id: AiProvider;
  label: string;
}> = [
  { id: "openAi", label: "OPENAI" },
  { id: "claude", label: "CLAUDE" }
];

export function AiProviderSwitch({
  provider,
  disabled,
  onChange
}: {
  provider: AiProvider;
  disabled: boolean;
  onChange: (provider: AiProvider) => void;
}) {
  return (
    <div className="provider-switch" role="group" aria-label="AI PROVIDER">
      {PROVIDERS.map((item) => (
        <button
          key={item.id}
          type="button"
          className={provider === item.id ? "is-active" : ""}
          aria-label={item.label}
          title={item.label}
          aria-pressed={provider === item.id}
          disabled={disabled}
          onClick={() => onChange(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
