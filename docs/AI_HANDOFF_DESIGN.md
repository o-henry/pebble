# AI Region Questions

Pebble can answer a question about the selected region without an API key.
This path is optional and never participates in continuous monitoring.

## User Flow

```text
select region -> connect ChatGPT once -> type question -> Ask -> concise answer
```

The **Ask** action is the consent boundary. Pebble does not call AI on a
timer, on visual change, at startup, or in the background.

## Runtime

Pebble bundles the official OpenAI Codex app-server as a Tauri sidecar.
The React webview can invoke three typed Rust commands:

```text
get_ai_connection_status
connect_chatgpt
ask_selected_region
```

The webview has no shell, opener, filesystem, or network plugin permission.
Rust starts only the fixed `codex` sidecar and opens only a validated
`https://chatgpt.com` or `https://auth.openai.com` OAuth URL. The login request
uses the hosted success page with the ChatGPT brand.

## Account Isolation

- No API key is requested.
- Browser cookies are never read.
- Other Codex or ChatGPT app tokens are never imported.
- The sidecar uses Pebble's private app data directory as `CODEX_HOME`.
- The directory mode is 0700 on Unix.
- Credentials use the OS keychain.
- On macOS, only the real `HOME` path is provided so the system can locate the
  default login keychain; Codex state remains isolated by `CODEX_HOME`.
- The child environment is cleared before launch, preventing inherited API-key
  or proxy variables from becoming an accidental auth path.

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
8. Sends exactly one image and one bounded question.

No frame or prompt is written to a screenshot, temp, history, log, config, or
thread file.

## Usage Limits

- Question: 1 to 1,000 Unicode characters.
- Image: the user-selected display region, with no application-level size cap.
- Model: an image-capable model whose id contains `mini` and supports low
  reasoning effort.
- Reasoning effort: `low`.
- Reasoning summary: disabled.
- Answer: at most 4,000 Unicode characters.
- Concurrency: one connection or question operation at a time.
- Conversation: a new ephemeral thread for every question.

If a compact compatible model is unavailable for the signed-in subscription,
Pebble reports that condition. It does not silently fall back to a larger
model.

## Tool Denial

The app-server starts with:

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

## Failure Behavior

The operation fails closed when:

- ChatGPT is not connected.
- The question is empty, oversized, or contains unsafe control characters.
- The selected region is hidden, removed, or reselected.
- The display is disconnected, rearranged, resized, or rescaled.
- The Pebble window is hidden or minimized before upload.
- The sidecar exits, times out, or returns invalid protocol data.
- No compatible compact image model exists.
- The model attempts any action outside image reasoning.

Errors are recoverable and do not include account email, auth URLs, tokens,
screen bytes, prompts, sidecar stderr, or local paths.

## Tests

Automated tests cover:

- Question normalization and limits.
- Official OAuth host validation.
- Compact image model selection without expensive fallback.
- Memory-only PNG data URL encoding.
- Privacy blank and missing-region rejection.
- Reselection and display reconfiguration invalidation.
- Webview permission denial for shell and opener plugins.
- In-memory frame storage policy.

The manual smoke checklist covers OAuth completion and one real selected-region
question on macOS.
