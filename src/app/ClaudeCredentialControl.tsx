import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  MAX_CLAUDE_API_KEY_LENGTH,
  normalizedClaudeApiKey,
  type ClaudeCredentialStatus
} from "../features/ai/claudeCredential";
import {
  deleteClaudeApiKey,
  getClaudeCredentialStatus,
  setClaudeApiKey
} from "../lib/invoke";
import { errorMessage } from "./usePebbleSession";

export function ClaudeCredentialControl({
  disabled,
  onChanged,
  onBusyChange,
  onError
}: {
  disabled: boolean;
  onChanged: () => void;
  onBusyChange: (busy: boolean) => void;
  onError: (message: string | null) => void;
}) {
  const [status, setStatus] = useState<ClaudeCredentialStatus | null>(null);
  const [editing, setEditing] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState(false);
  const normalizedKey = useMemo(() => normalizedClaudeApiKey(apiKey), [apiKey]);

  useEffect(() => {
    let active = true;
    getClaudeCredentialStatus()
      .then((nextStatus) => {
        if (active) setStatus(nextStatus);
      })
      .catch((reason: unknown) => {
        if (active) {
          onError(errorMessage(reason, "CLAUDE KEYCHAIN COULD NOT BE CHECKED."));
        }
      });
    return () => {
      active = false;
    };
  }, [onError]);

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!normalizedKey) {
      onError("ENTER A VALID ANTHROPIC API KEY.");
      return;
    }
    setApiKey("");
    await runCredentialAction(
      () => setClaudeApiKey(normalizedKey),
      "CLAUDE API KEY COULD NOT BE SAVED."
    );
  }

  async function remove() {
    setApiKey("");
    await runCredentialAction(
      deleteClaudeApiKey,
      "CLAUDE API KEY COULD NOT BE REMOVED."
    );
  }

  async function runCredentialAction(
    action: () => Promise<ClaudeCredentialStatus>,
    fallbackError: string
  ) {
    try {
      setBusy(true);
      onBusyChange(true);
      onError(null);
      setStatus(await action());
      setEditing(false);
      onChanged();
    } catch (reason) {
      onError(errorMessage(reason, fallbackError));
    } finally {
      setBusy(false);
      onBusyChange(false);
    }
  }

  const configured = status?.apiKeyConfigured === true;
  return (
    <div className="claude-credential">
      <div className="claude-credential__summary">
        <span>{status ? (configured ? "API BILLING" : "SUBSCRIPTION DEFAULT") : "CHECKING KEYCHAIN"}</span>
        <div className="claude-credential__actions">
          <button
            type="button"
            disabled={disabled || busy}
            onClick={() => setEditing(true)}
          >
            {configured ? "REPLACE KEY" : "ADD API KEY"}
          </button>
          {configured ? (
            <button
              type="button"
              title="REMOVE KEY AND USE CLAUDE SUBSCRIPTION"
              disabled={disabled || busy}
              onClick={() => void remove()}
            >
              USE SUBSCRIPTION
            </button>
          ) : null}
        </div>
      </div>

      {editing ? (
        <form className="claude-credential__editor" onSubmit={(event) => void save(event)}>
          <input
            type="password"
            aria-label="ANTHROPIC API KEY"
            placeholder="ANTHROPIC API KEY"
            value={apiKey}
            maxLength={MAX_CLAUDE_API_KEY_LENGTH}
            autoComplete="off"
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
            disabled={disabled || busy}
            onChange={(event) => setApiKey(event.currentTarget.value)}
          />
          <button type="submit" disabled={disabled || busy || !normalizedKey}>
            SAVE
          </button>
          <button
            type="button"
            disabled={disabled || busy}
            onClick={() => {
              setApiKey("");
              setEditing(false);
            }}
          >
            CANCEL
          </button>
        </form>
      ) : null}
    </div>
  );
}
