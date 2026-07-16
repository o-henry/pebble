# AI Region Questions

Pebble can answer a question about the selected region through account access
or an optional Anthropic API key. Manual questions run only after **Send**;
semantic Watch is a separate opt-in, locally gated path.

## User Flow

```text
select region -> choose provider and model -> connect once -> type question -> Send -> concise answer
```

The **Send** action is the manual-question consent boundary. Pebble does not
call AI at startup. Watch may call the selected provider only after explicit
opt-in, a stable local material-change gate, Apple Vision OCR, and the selected
1, 5, 30, or 60 minute analysis interval.

## Runtime

Pebble bundles the OpenAI Codex app-server as a Tauri sidecar. Claude uses a
separately installed official Claude CLI when no API key is configured. When a
key is configured, Rust calls only the fixed Anthropic Messages and model URLs.
The React webview can invoke these typed Rust commands:

```text
get_ai_connection_status
connect_ai_provider
ask_selected_region
get_claude_credential_status
set_claude_api_key
delete_claude_api_key
```

The webview has no shell, opener, filesystem, or network plugin permission.
Rust starts only the fixed `codex` sidecar or a Claude executable found at one
of three fixed locations and rejected when group- or world-writable. OpenAI
login opens only a validated `https://chatgpt.com` or `https://auth.openai.com`
URL. If Claude is absent and no key is saved, Rust opens only its fixed official
installation page. Direct API requests use HTTPS, reject redirects, have bounded
timeouts and responses, and never expose raw server bodies to the UI.

## Account Isolation

- OpenAI API keys are not accepted.
- Claude API keys are optional and accepted only by the visible Pebble window.
- Claude API keys are stored only as a macOS Keychain generic password and are
  never returned to the webview, config, logs, tests, or Markdown journal.
- Browser cookies are never read.
- Other Codex, AI app, or browser tokens are never imported.
- The sidecar uses Pebble's private app data directory as `CODEX_HOME`.
- The directory mode is 0700 on Unix.
- Subscription credentials use provider-owned credential storage.
- On macOS, only the real `HOME` path is provided so the system can locate the
  default login keychain; Codex state remains isolated by `CODEX_HOME`.
- Every AI child environment is cleared before launch, preventing inherited
  API-key or proxy variables from becoming an accidental auth path.
- Claude uses its official Pro/Max login and system credential storage; Pebble
  never reads or persists the resulting subscription credential.
- A configured Claude API key takes precedence. Invalid API authentication
  fails visibly and never falls back silently to subscription billing.

## Image Boundary

For every question, Rust:

1. Verifies that the caller is the visible, non-minimized Pebble window.
2. Reads the selected region from backend session state, never from request
   coordinates.
3. Rejects privacy-blanked or missing regions.
4. Revalidates display identity, bounds, scale, and session revision.
5. Captures only that physical crop.
6. Encodes the RGBA bytes as an in-memory PNG data URL.
7. Revalidates the session and display immediately before `turn/start`.
8. Sends exactly one image and one bounded question through the selected local
   process, or through Anthropic's fixed Messages endpoint when API-key mode is
   explicitly configured.

No frame or prompt is written to a screenshot, temp, history, log, config, or
thread file.

## Usage Limits

- Question: 1 to 1,000 Unicode characters.
- Image: the user-selected display region, with no application-level size cap.
- OpenAI models: user-selected Sol, Terra, or Luna when the connected account
  reports the model as image-capable.
- Claude subscription models: user-selected Sonnet or Opus aliases.
- Claude API models: user-selected Sonnet or Opus model IDs returned by the
  fixed Anthropic models endpoint.
- Reasoning effort: `medium`.
- Reasoning summary: disabled.
- Answer: at most 4,000 Unicode characters.
- Concurrency: one connection or question operation at a time.
- Conversation: a new ephemeral thread for every question.

If a selected model is unavailable, the official Claude CLI is unavailable,
or the configured Claude API key cannot use the selected model, Pebble reports
that condition. It does not silently switch models or authentication paths.

## Tool Denial

The OpenAI app-server starts with:

- Read-only sandbox.
- Approval policy `never`.
- Web search disabled.
- Empty MCP server configuration.
- Analytics disabled.
- Instructions forbidding tools, files, shell, web, plugins, skills, memory,
  and outside context.

Pebble additionally inspects streamed items. Any command, file change,
web search, plugin, MCP, dynamic tool, image generation, or server approval
request aborts the response.

Claude subscription mode starts with safe mode, slash commands disabled, strict
empty MCP configuration, `--tools ""`, all tools explicitly denied, medium
effort, and one turn maximum. Stream output containing a tool-use item is
rejected. Claude API mode defines no tools, sends no tool-choice field, rejects
tool-use response blocks, follows no redirects, and has no web-search path.

## Failure Behavior

The operation fails closed when:

- The selected AI provider is unavailable or not connected.
- The question is empty, oversized, or contains unsafe control characters.
- The selected region is hidden, removed, or reselected.
- The display is disconnected, rearranged, resized, or rescaled.
- The Pebble window is hidden or minimized before upload.
- The sidecar exits, times out, or returns invalid protocol data.
- No supported balanced image model exists.
- The model attempts any action outside image reasoning.
- The saved Claude API key is rejected, rate-limited, or cannot access the selected model.

Errors are recoverable and do not include account email, auth URLs, tokens,
screen bytes, prompts, sidecar stderr, or local paths.

## Tests

Automated tests cover:

- Question normalization and limits.
- Official OAuth host validation.
- Account-reported OpenAI model selection and backend revalidation without silent fallback.
- Claude subscription alias and API model-list validation.
- Claude stream parsing, model/duration metadata, and tool-use rejection.
- Claude API payload boundaries, HTTP error sanitization, and tool-use rejection.
- Claude API-key format limits and explicit subscription/API billing labels.
- Keychain commands restricted to the visible, non-minimized Pebble window.
- Locale validation and question-language fallback instructions.
- Memory-only PNG data URL encoding.
- Privacy blank and missing-region rejection.
- Reselection and display reconfiguration invalidation.
- Webview permission denial for shell and opener plugins.
- In-memory frame storage policy.
- Typed Watch intent matching, confidence validation, and unmatched-result suppression.
- Ephemeral Apple Vision OCR with prompt-injection-resistant boundaries.

The manual smoke checklist covers OAuth completion and one real selected-region
question on macOS.
