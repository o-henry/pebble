import type { AiProvider } from "../features/ai/regionQuestion";
import openAiIcon from "../assets/brands/openai.svg";
import claudeIcon from "../assets/brands/claude.svg";

const PROVIDERS: ReadonlyArray<{
  id: AiProvider;
  label: string;
  icon: string;
}> = [
  { id: "openAi", label: "OPENAI", icon: openAiIcon },
  { id: "claude", label: "CLAUDE", icon: claudeIcon }
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
          <span
            className="provider-switch__icon"
            aria-hidden="true"
            style={{
              maskImage: `url("${item.icon}")`,
              WebkitMaskImage: `url("${item.icon}")`
            }}
          />
        </button>
      ))}
    </div>
  );
}
