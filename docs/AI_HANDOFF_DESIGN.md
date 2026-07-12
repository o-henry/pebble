# AI Region Questions

Pebble can answer a question about the selected region without an API key.
This path is optional and never participates in continuous monitoring.

## User Flow

```text
select region -> choose provider -> connect once -> type question -> Ask -> concise answer
```

The **Send** action is the consent boundary. Pebble does not call AI on a
timer, on visual change, at startup, or in the background.

## Runtime

Pebble bundles the OpenAI Codex app-server as a Tauri sidecar. Claude support
uses a separately installed official Claude CLI and does not add another large
binary to Pebble.
The React webview can invoke three typed Rust commands:

```text
get_ai_connection_status
connect_ai_provider
ask_selected_region
```

The webview has no shell, opener, filesystem, or network plugin permission.
Rust starts only the fixed `codex` sidecar or a Claude executable found at one
of three fixed locations and rejected when group- or world-writable. OpenAI
login opens only a validated `https://chatgpt.com` or `https://auth.openai.com`
URL. If Claude is absent, Rust opens only its fixed official installation page.

## Account Isolation

- No API key is requested.
- Browser cookies are never read.
- Other Codex, AI app, or browser tokens are never imported.
- The sidecar uses Pebble's private app data directory as `CODEX_HOME`.
- The directory mode is 0700 on Unix.
- Credentials use the OS keychain.
- On macOS, only the real `HOME` path is provided so the system can locate the
  default login keychain; Codex state remains isolated by `CODEX_HOME`.
- Every AI child environment is cleared before launch, preventing inherited
  API-key or proxy variables from becoming an accidental auth path.
- Claude uses its official Pro/Max login and system credential storage; Pebble
  never reads or persists the resulting credential.

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
8. Sends exactly one image and one bounded question through the selected
   provider's documented local process protocol.

No frame or prompt is written to a screenshot, temp, history, log, config, or
thread file.

## Usage Limits

- Question: 1 to 1,000 Unicode characters.
- Image: the user-selected display region, with no application-level size cap.
- OpenAI model: prefer `gpt-5.6-terra`, allow `gpt-5.6-luna` only as fallback.
- Claude model: Claude Sonnet 5.
- Reasoning effort: `medium`.
- Reasoning summary: disabled.
- Answer: at most 4,000 Unicode characters.
- Concurrency: one connection or question operation at a time.
- Conversation: a new ephemeral thread for every question.

If neither supported OpenAI model is available for the signed-in subscription
or the official Claude CLI is unavailable, Pebble reports that condition. It
does not silently fall back to mini, Haiku, or a premium flagship model.

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

Claude starts with safe mode, slash commands disabled, strict empty MCP
configuration, `--tools ""`, all tools explicitly denied, low effort, and one
turn maximum. Stream output containing a tool-use item is rejected.

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

Errors are recoverable and do not include account email, auth URLs, tokens,
screen bytes, prompts, sidecar stderr, or local paths.

## Tests

Automated tests cover:

- Question normalization and limits.
- Official OAuth host validation.
- Terra-first model selection with Luna-only fallback and mini rejection.
- Claude stream parsing, model/duration metadata, and tool-use rejection.
- Locale validation and question-language fallback instructions.
- Memory-only PNG data URL encoding.
- Privacy blank and missing-region rejection.
- Reselection and display reconfiguration invalidation.
- Webview permission denial for shell and opener plugins.
- In-memory frame storage policy.

The manual smoke checklist covers OAuth completion and one real selected-region
question on macOS.
