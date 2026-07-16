# Growth and Product Strategy

This document defines how Pebble can earn adoption as a free and open-source
macOS utility. It covers distribution, positioning, community growth, product
gaps, and the product capability most likely to make Pebble meaningfully
different.

It is a strategy document, not a claim that every item is already implemented.
Shipped behavior must continue to be described by the README, release notes,
and security documentation.

## Goal And Constraints

Pebble should optimize for useful installs, retained use, community trust, and
contributors. GitHub stars are a useful public signal, but they are an outcome
of those things rather than the primary product metric.

The standing constraints are:

- Pebble remains free and MIT licensed.
- There is no paid tier, subscription, feature gate, advertising, or sale of
  user data.
- Third-party AI subscriptions or API usage may still have costs imposed by
  the selected provider. Pebble must label that distinction plainly.
- Capture remains read-only. Pebble must not click, type, scroll, or otherwise
  control the selected application.
- The user selects the region. Whole-screen or hidden surveillance is outside
  the product.
- Captured frames are not stored as history.
- Local checks come before any network handoff.
- Public claims must match the implementation and provider terms.

## Positioning

Pebble should not be positioned as a screenshot utility, generic AI chat box,
browser extension, screen recorder, or computer-control agent.

The recommended promise is:

> Pin any part of your Mac. Pebble keeps it visible, explains meaningful
> changes, and never controls your computer.

The shorter category statement is:

> A free, open-source, read-only AI watch for anything visible on macOS.

The message hierarchy is:

1. It works across browsers and native apps, not only web pages.
2. It notices a state the user would otherwise keep checking.
3. It explains what changed instead of reporting only pixel movement.
4. It is region-scoped and cannot control the computer.
5. It is free, open source, and inspectable.

Do not lead with Tauri, Rust, model names, frame rates, or change percentages.
Those are implementation proof. The first message must be the time and
attention Pebble gives back to the user.

## Market Shape

Pebble sits between three established product categories:

| Category | Existing value | Gap Pebble can own |
| --- | --- | --- |
| Static pinning tools | Keep a screenshot or reference above other windows | The source does not stay live and the tool does not understand state changes |
| Screen alert tools | OCR, numeric rules, visual thresholds, and notifications | The user must configure mechanics and receives limited semantic explanation |
| Screen memory products | Record broad screen history for later search and AI context | Continuous collection, storage, resource cost, and a much larger trust surface |
| Browser AI extensions | Ask about web content with page or tab context | They stop at the browser and cannot cover native apps, games, simulators, or remote desktops |

Current examples illustrate this split:

- [FloatSnap](https://floatsnap.com/) focuses on always-on-top screenshots.
- [ScreenAlert](https://apps.apple.com/us/app/screenalert/id6761373715?mt=12)
  offers OCR, conditions, image-change alerts, Shortcuts, and window tracking.
- [screenpipe](https://github.com/screenpipe/screenpipe) records broad screen
  and audio history locally for search and AI context.

Pebble should not claim to be the first region monitor. Its defensible space is
the narrow combination of live pinning, user intent, semantic change
explanation, no computer control, no frame history, and desktop-wide reach.

## Launch Readiness Before Promotion

Promotion should start only when a new user can safely try the product without
building it from source.

Required launch assets:

- A signed and notarized Apple Silicon application.
- A downloadable DMG or ZIP attached to a GitHub Release.
- A stable release URL and checksums.
- A direct `Download for macOS` action on a small product page.
- A Homebrew Cask after the downloadable release is stable.
- A 15 to 25 second demonstration made with synthetic or intentionally public
  screen content.
- A permission walkthrough that says exactly what Screen Recording allows.
- A concise `What leaves your Mac` explanation for Watch and manual Ask.
- A tested uninstall path.
- Release notes and a visible pre-alpha label.
- Consistent README, product spec, security, and in-app privacy wording.

Apple recommends Developer ID signing and notarization for software distributed
outside the Mac App Store:
[Distributing software on macOS](https://developer.apple.com/macos/distribution/).

Homebrew provides a documented Cask path for graphical applications:
[Adding Software to Homebrew](https://docs.brew.sh/Adding-Software-to-Homebrew).

## Provider Policy Gate

Provider access is part of release readiness, not only an engineering detail.

- Anthropic currently says products and third-party tools should use API key
  authentication through Claude Console or a supported cloud provider. The
  Claude CLI subscription path must not be marketed for third-party use without
  a confirmed compliant basis. See
  [Anthropic account authentication guidance](https://support.claude.com/en/articles/13189465-log-in-to-your-claude-account).
- OpenAI documents ChatGPT-plan access for official Codex clients, but that
  alone does not establish a right to redistribute or embed the authentication
  flow in an unrelated third-party product. Confirm the public distribution
  path before launch. See
  [Using Codex with a ChatGPT plan](https://help.openai.com/en/articles/11369540-using-codex-with-chatgpt).
- API billing, subscription usage, fallback behavior, and model identity must
  be visible before the user sends a crop.

If compliant subscription access cannot be confirmed, Pebble should ship its
core local experience without requiring AI and offer clearly labeled optional
provider keys. A free product must not achieve convenience by creating an
account or billing surprise for users.

## Distribution Strategy

Each channel needs a native story. Reposting identical promotional copy across
communities is not a distribution strategy.

| Priority | Channel | Native approach |
| --- | --- | --- |
| Highest | Direct early users | Give developers, designers, operators, editors, and QA users a demo matching one real waiting problem |
| Highest | GitHub Releases | Make the repository itself a reliable product page with a binary, release notes, security policy, and screenshots |
| Highest | r/macapps | Use the current open-source title prefix, disclose free pricing, show a real workflow, and follow its promotion interval |
| Highest | Show HN | Launch only when the app is directly runnable; explain the personal problem and technical trust choices |
| High | MacMenuBar | Submit after the menu bar flow and download are stable |
| High | Homebrew Cask | Give technical users a one-command installation path |
| High | Product Hunt | Launch as a free product after users can download and use it immediately |
| High | Curated lists | Submit focused pull requests to active macOS, Tauri, Rust, privacy, and open-source app lists |
| Medium | X and LinkedIn | Publish one outcome per short video instead of a feature inventory |
| Medium | YouTube and short video | Demonstrate a complete waiting problem and resolution in under 30 seconds |
| Medium | Developer writing | Explain source-window tracking, local gating, capture privacy, and token reduction with code evidence |
| Long term | Search pages | Publish useful pages for concrete queries and honest comparisons |
| Long term | Creator outreach | Give small macOS and productivity creators a working build and a workflow chosen for their audience |

Relevant current channel rules and submission pages:

- [Show HN Guidelines](https://news.ycombinator.com/showhn.html)
- [r/macapps 2026 posting guidance](https://www.reddit.com/r/macapps/comments/1qghsc5/new_post_guidelines_and_updates_on_rmacapps/)
- [Product Hunt featuring guidelines](https://help.producthunt.com/en/articles/9883485-product-hunt-featuring-guidelines)
- [MacMenuBar submission](https://macmenubar.com/submit-your-menu-bar-app/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)

## Content System

Every public artifact should prove one job instead of listing every feature.

Recommended demonstrations:

- A background build succeeds or fails while the user works elsewhere.
- A render or export finishes and Pebble explains the final state.
- An upload, queue, or long-running job reaches a meaningful state.
- A remote desktop task changes while its source window is behind another app.
- The same selection workflow works first in a browser and then in a native app.
- A user asks what changed and sees the provider, model, and generation time.
- Pebble remains read-only while the source application stays untouched.
- Preview pause, background Watch, privacy blank, and per-region stop are shown
  as distinct states with visible proof.

Use synthetic accounts, local fixtures, public pages, or purpose-built demo
screens. Never publish a real personal desktop, bookmark bar, account name,
notification, token, private repository, customer record, or work dashboard.

The repeatable publishing cadence should be:

1. One real workflow.
2. One short video.
3. One technical or privacy fact.
4. One link to a runnable release.
5. One specific request for workflow feedback, not a request for manufactured
   votes or stars.

## Search And Discovery

The product page and documentation should target problem language used by
people who do not know Pebble exists:

- monitor any screen region on Mac
- keep part of screen always on top on Mac
- get notified when something changes on screen
- watch a build or render without integrations
- ask AI about any native Mac app
- read-only AI screen assistant
- open-source ScreenAlert alternative
- browser and native app screen watcher

Comparison pages must be factual. They should explain who each product is for,
not manufacture weaknesses or claim exclusivity.

## Community Growth

The repository should make contribution easier than private support channels.

Recommended community surfaces:

- GitHub Discussions with `Show how you use Pebble`, `Ideas`, and `Help`.
- A concise `CONTRIBUTING.md` with architecture, test commands, and security
  boundaries.
- Bug and feature issue forms that ask for macOS version, chip, display layout,
  and a synthetic reproduction instead of a private screenshot.
- `good first issue` tasks that do not touch capture permissions or credential
  handling.
- A public roadmap that separates committed work from exploration.
- Release notes that credit reporters and contributors.
- English, Korean, and Japanese user-facing installation and permission copy.

GitHub Discussions is intended for open-ended project conversation that does
not belong in an issue:
[GitHub Discussions quickstart](https://docs.github.com/en/discussions/quickstart).

## Measurement Without Product Telemetry

Pebble can measure adoption without hidden application analytics.

Track:

- Release asset downloads by version.
- Unique repository visitors and clones in GitHub Traffic.
- Stars, release watchers, forks, issues, discussions, and contributors.
- Channel-specific links to distinguish launch sources.
- Repeat commenters and people who return for later releases.
- Voluntary workflow reports in Discussions.
- Support questions per download as a signal of onboarding friction.

Do not treat raw stars as proof of retained use. A healthy sequence is:

```text
discovery -> safe download -> first successful selection -> useful alert
          -> repeated use -> workflow story -> star or contribution
```

## Product Gap Analysis

The current architecture already provides valuable foundations:

- A user-selected region instead of whole-screen access.
- A live always-on-top tile.
- Background capture tied to the selected session revision.
- Local low-frequency visual gating.
- Persistence checks before a visual change becomes a candidate.
- User-selected AI analysis intervals.
- Before-and-after semantic comparison.
- Notifications and a bounded activity feed.
- Explicit provider selection and read-only AI instructions.
- No frame-history product.

Watch now freezes the user's composer text as an explicit intent, runs Apple
Vision OCR after a stable local candidate, and asks the selected model for a
typed match decision and confidence only when local evaluation cannot decide.
Deterministic text appearance, disappearance, text-change, single-number
threshold, progress, and state rules now run locally. They can be activated
without an AI connection, and the UI states when token usage is zero.

The remaining reliability problems are:

1. Real-device source-window edge cases still need broader coverage across
   Spaces, full screen, minimize, restore, and mixed-DPI displays.
2. A meaningful state can be visually small and fail the current local signal
   ensemble.
3. Unsupported natural language still requires AI rather than a reviewable
   one-time constrained compile step.

The production OCR adapter, deterministic intent compiler, stable-candidate
animation suppression, and semantic fingerprint dedupe are implemented. A
synthetic benchmark and explicit CPU, memory, detection-time, and escalation
budgets are still missing.

## Candidate Product Kicks

| Candidate | User value | Distinctiveness | Privacy fit | Cost efficiency | Recommendation |
| --- | ---: | ---: | ---: | ---: | --- |
| More generic AI chat | 2 | 1 | 3 | 2 | Do not prioritize; crowded and not the core job |
| Broad web research after every change | 2 | 2 | 1 | 1 | Reject as a default; it breaks scope and cost discipline |
| Continuous screen memory | 3 | 1 | 1 | 1 | Reject; it competes directly with established recorders and expands risk |
| More visual threshold controls | 3 | 2 | 5 | 5 | Useful foundation, but competitors already offer this |
| Multiple monitoring tiles | 4 | 3 | 4 | 4 | Valuable after one tile is highly reliable |
| Shareable Watch recipes | 4 | 4 | 5 | 5 | Strong growth loop after the intent model exists |
| Intent Watch | 5 | 5 | 5 | 5 | Recommended product-defining capability |

## Recommended Kick: Intent Watch

The product-defining interaction should be:

> Point at anything. Say what matters. Pebble tells you when it happens.

After selecting a region, the user describes the desired event in one sentence:

- Tell me when this build succeeds or fails.
- Tell me when the render is finished.
- Tell me when this number goes above 100.
- Tell me when an error message appears.
- Tell me when the queue becomes empty.
- Tell me when the approval state changes.

This is not free-form autonomous surveillance. It is a narrow, visible,
read-only condition attached to one region by the user.

### User Experience

1. The user selects a region.
2. Pebble opens the live tile immediately.
3. The user enables Watch and enters one `Tell me when...` condition.
4. Pebble shows a plain execution plan: local check cadence, provider if any,
   minimum AI interval, and exactly what may leave the Mac.
5. Pebble captures a baseline and confirms `WATCHING FOR: ...`.
6. Local visual and OCR signals evaluate each stable candidate.
7. Pebble calls AI only when the condition is semantic or local confidence is
   insufficient and the user's interval permits it.
8. The notification states what happened, why it matched, confidence, provider,
   model, and elapsed time.
9. The user can mark the event useful or noisy without sending telemetry. That
   feedback tunes only the local region rule.

### Local Signal Pipeline

The local gate should evolve from a single generic score into a small ensemble:

```text
selected crop every 5 seconds
  -> perceptual hash and structural similarity
  -> edge and layout change map
  -> local OCR text and number diff
  -> color and status-token change
  -> persistence and animation suppression
  -> deterministic intent evaluation
  -> bounded AI escalation only when needed
  -> deduped semantic event
```

Recommended local evidence:

- Perceptual hash distance for broad visual changes.
- Structural similarity for layout-preserving changes.
- Edge-map change for text, icons, and progress-state transitions.
- Local OCR with normalized lines, numbers, and status words.
- Region-of-interest persistence across two of the last three checks.
- Motion suppression for cursors, spinners, clocks, and repeated animation.
- Baseline and last-notified semantic fingerprints for dedupe.

The system keeps only active baselines, current candidates, and compact local
fingerprints in memory. Per-region stop, privacy blank, Pebble removal, or app
quit clears associated Watch state. Hiding or closing Pebble and pausing the
visible preview intentionally preserve disclosed background Watch targets.
Reselection never retargets them. No image timeline should be introduced.

### Intent Representation

Common conditions should compile into a typed local rule rather than requiring
AI on every frame:

```json
{
  "subject": "visible number",
  "operator": "greater_than",
  "value": 100,
  "stabilityChecks": 2,
  "notifyOnceUntilReset": true
}
```

Initial rule types:

- text appears or disappears
- text changes from one stable value to another
- number crosses above or below a threshold
- progress reaches a value
- success, warning, or error state appears
- a user-named object or control appears
- the selected region becomes stable after sustained activity
- any meaningful semantic change, as the explicit fallback

A deterministic parser should handle common templates. An optional one-time AI
compile step may translate unsupported natural language into a constrained rule
schema, but the user must review the result before Watch starts.

### Token And Cost Discipline

Intent Watch should reduce AI use below the current generic semantic Watch, not
increase it.

- Local checks never consume provider tokens.
- OCR and deterministic numeric/text rules stay on-device.
- A visual candidate must persist before it can reach AI.
- The intent condition must plausibly match before AI escalation.
- Repeated semantic events are fingerprinted and suppressed.
- The user selects the minimum AI interval.
- Only one before-and-after selected crop pair is sent per analysis.
- There is no web search, tool use, shell access, file access, MCP, or computer
  control in Watch analysis.
- The UI shows the analysis count and active provider path.

### Privacy Receipt

Every AI-assisted event should have an ephemeral receipt showing:

- selected region identity
- reason the local gate escalated
- before-and-after crop thumbnails
- provider and billing path
- model and duration
- whether anything was persisted

The receipt must not become a screenshot history. It can exist in memory while
the event is open and disappear when dismissed. The durable Markdown activity
feed should contain text metadata only.

## Supporting Improvements

Intent Watch is the kick, but several supporting improvements determine whether
people keep using it.

### Reliability

- Test source tracking while occluded, moved, resized, placed on another Space,
  made full screen, minimized, and restored.
- Test display disconnects, scale changes, mirrored displays, and mixed-DPI
  external monitors.
- Make capture loss visible and never silently switch to another screen area.
- Add a synthetic Watch benchmark with text, number, color, layout, animation,
  and low-contrast transitions.
- Report false positives, false negatives, time-to-detect, CPU, memory, and AI
  escalation rate.

### Onboarding

- Make the first useful local-only Watch possible without connecting AI.
- Explain Screen Recording permission before macOS asks for it.
- Use a synthetic demo region so users can test Watch safely in under a minute.
- Replace feature descriptions with one guided `select -> intent -> alert`
  experience.
- Keep advanced provider and interval controls out of the first-run path.

### Notifications And Activity

- Make the event title describe the matched intent, not merely `change detected`.
- Provide `Show`, `Snooze`, `Mark noisy`, and `Stop Watch` actions where macOS
  supports them.
- Group repeated updates for one intent instead of producing a stream of alerts.
- Keep text logs compact and searchable without storing image or OCR history.

### Shareable Recipes

After Intent Watch is stable, users should be able to share privacy-safe recipe
files containing only:

- intent template
- local rule schema
- recommended interval
- provider optionality
- notification behavior

Recipes must never contain coordinates, screenshots, OCR output, app account
details, or captured content. A community recipe gallery can create organic
distribution without turning Pebble into a template marketplace.

## Delivery Order

Implemented foundation as of July 2026:

- Public Watch and Ask documentation reconciled with current behavior.
- Production Apple Vision OCR behind the stable-change and interval gates.
- One-sentence intent captured from the composer and frozen when Watch starts.
- User-selected OpenAI and Claude models validated again in the Rust backend.
- Typed matched/unmatched model result with confidence; unmatched candidates do
  not notify.
- Deterministic local rules for text, single-number thresholds, progress, and
  state words, including an AI-free activation path.
- Stable-candidate animation suppression and repeated semantic event dedupe.
- Privacy-safe Watch recipes that persist only intent metadata.
- Up to three independent source-window-bound regions with individual stop and
  revocable AI authorization.

### P0 - Trustworthy Public Build

- Resolve provider authentication policy for public distribution.
- Ship a signed, notarized, directly downloadable release.
- Publish a synthetic demo and permission explanation.
- Add community contribution and discussion surfaces.

### P1 - Local Understanding Foundation

- Add synthetic Watch benchmarks and resource budgets.
- Expand typed conditions only from measured false-negative cases.

### P2 - Intent Watch MVP

- Add an ephemeral decision receipt for AI-assisted events.
- Add a review UI before an optional one-time AI compile of unsupported intent.
- Measure local-versus-AI escalation rates against explicit budgets.

### P3 - Adoption Loop

- Add workflow examples and GitHub Discussions showcase categories.
- Localize onboarding and permission copy.
- Publish benchmark results and technical implementation notes.

### P4 - Scale Only After Evidence

- Validate three-region CPU and memory budgets on supported Mac hardware.
- Explore optional local vision models only if OCR and deterministic signals
  cannot meet measured use cases within the resource budget.
- Add new providers only when authentication, billing, security, and model
  behavior are explicit and tested.

## Acceptance Criteria For The Kick

Intent Watch is ready to market when:

- A new user can install, select, define an intent, and receive a useful alert
  without reading documentation.
- Text and numeric conditions work without an AI connection.
- The same intent works in at least one browser, terminal, and native app demo.
- Occlusion and window movement do not silently change the watched source.
- Animation fixtures do not create repeated alerts.
- Every AI event is attributable to an in-memory local candidate and the user's
  explicit region intent.
- No frame is written to disk.
- No mouse, keyboard, Accessibility, browser cookie, or automation permission is
  requested.
- Per-region stop, privacy blank, Pebble removal, and app quit clear Watch pixel
  buffers; hide, native close, preview pause, and reselection preserve and never
  retarget disclosed background targets.
- The UI accurately labels provider, billing path, interval, model, and duration.
- CPU, memory, time-to-detect, false-alert rate, and AI escalation rate are
  measured against explicit budgets before release.

## What Pebble Should Refuse To Become

Do not add features merely because they can be described as AI:

- no hot-news or social feed
- no unrelated web monitoring
- no whole-screen memory timeline
- no hidden employee monitoring
- no browser session or cookie automation
- no autonomous clicking or typing
- no trade execution or financial decision automation
- no continuous cloud image stream
- no generic multi-agent workspace
- no marketplace or paid upgrade layer

Pebble wins by being small, legible, and trustworthy. The product should watch
one thing the user cares about, recognize when that thing happens, explain it,
and then get out of the way.

## Thirty-Day Growth Sequence

1. Week 1: complete release trust gates, provider-policy decisions, installer,
   and documentation consistency.
2. Week 2: recruit direct testers and record five synthetic workflow demos.
3. Week 3: publish the release sequentially to r/macapps, MacMenuBar, relevant
   curated lists, X, and developer communities; incorporate feedback between
   posts.
4. Week 4: ship a feedback release, then launch on Show HN and Product Hunt.
5. Continue with one workflow demonstration and one meaningful release note per
   week, always linking to a directly runnable build.

Do not coordinate votes, request artificial stars, or mass-post identical copy.
The durable growth loop is a useful alert becoming a user story, that story
becoming a trustworthy demonstration, and that demonstration bringing the next
user.
