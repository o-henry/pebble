# AI Handoff Design

AI support is a future optional capability. It must not be part of the core
capture path, and it must never make ScreenPebble feel like a hidden screen
watcher.

## Design Goal

Use local monitoring to minimize AI usage.

```text
local crop -> local diff -> local OCR -> dedupe -> optional AI text handoff
```

The AI should receive the smallest useful payload:

1. OCR text when available.
2. A compact local change summary.
3. A cropped image only when the user explicitly allows images for that region.

## Non-Negotiables

- AI is off by default.
- AI is enabled per region, not globally by accident.
- No AI whole-screen access.
- No continuous image streaming.
- No hidden background calls.
- No browser cookie scraping.
- No ChatGPT web automation.
- No API key theft or token reuse.
- No captured frames or OCR history persisted.

## User Consent Model

Each region has independent settings:

```text
AI: off | text on change | image on request | image on change
```

Default:

```text
AI: off
```

Before enabling AI, the UI must explain:

- What data may be handed off.
- Whether the payload is text or image.
- That handoff happens only for this region.
- That capture still stops when paused, hidden, blanked, closed, or deleted.

## Low-Usage Policy

The connector should use these gates before any AI handoff:

1. Region AI setting is enabled.
2. Tile is live and visible.
3. Privacy blank is off.
4. Local diff crosses threshold, or user explicitly requests a handoff.
5. Cooldown has passed.
6. OCR text or image payload differs from the previous handoff.

If any gate fails, do nothing.

## Payload Design

Text-first payload:

```text
Region: build-log
Event: changed
Observed text:
error: missing dependency @tauri-apps/api

Instruction:
Explain what likely happened and suggest the next local debugging step.
```

Image payload:

- Cropped region only.
- No full monitor frame.
- No unrelated screen content.
- No disk write required to create the payload.
- Allowed only by explicit region setting or explicit user action.

## Connector Interface

Keep connectors behind an adapter:

```text
AiConnector
  is_enabled(region_id)
  can_send_text(region_id)
  can_send_image(region_id)
  send_text(payload)
  send_image(payload)
```

The rest of the app should not know whether the connector is a local clipboard
flow, a local app integration, an MCP adapter, or another explicit integration.

## MCP Position

MCP can be a later adapter, not the product core.

Use MCP only if it preserves the same constraints:

- User-visible enablement.
- Per-region authorization.
- No shell, filesystem, browser history, or broad OS tools.
- No automatic whole-screen access.
- No continuous image streaming.

If MCP requires a tunnel or account setup, document that clearly. Do not present
it as required for ScreenPebble's core value.

## ChatGPT Account Direction

The desired user experience is "no OpenAI API key required." That does not mean
ScreenPebble may automate a logged-in ChatGPT browser session.

Allowed future directions:

- Official connector or app integration.
- User-initiated copy/open flow.
- Explicit local connector with a documented pairing flow.

Disallowed:

- Reading ChatGPT cookies.
- Controlling the ChatGPT web UI invisibly.
- Reusing account tokens from another app.

## Speaking Results

"AI can speak" should be implemented as local text-to-speech over a text response
that the user explicitly requested or configured for a region.

Do not implement always-listening or always-speaking behavior. Speaking must have
clear controls:

- Mute.
- Stop.
- Per-region enablement.
- Cooldown.
- Visible status while speaking.

## Tests Required Before Shipping AI Handoff

- AI disabled by default.
- Region without AI permission cannot send text or image.
- Privacy blank blocks handoff.
- Paused, hidden, closed, and deleted regions block handoff.
- Text dedupe works.
- Cooldown works.
- Image handoff rejects full-frame payloads.
- Connector errors are recoverable.
