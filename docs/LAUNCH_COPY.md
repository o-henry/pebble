# Pebble Launch Copy

Pebble is pre-alpha, open source, and currently built from source on macOS. Do
not describe it as packaged or production-ready until a signed installer ships.

## X

I built Pebble, a free and open-source macOS utility for the tiny part of your
screen you keep checking.

Select any region from a browser, terminal, IDE, native app, simulator, or
remote desktop. Pebble keeps it visible. Optional Watch mode uses a local
visual gate and, after a material change, sends only the previous and current
selected crops to the chosen AI provider at the interval you select. Watch is
off by default, with no frame history, telemetry, or continuous image upload.
You can also explicitly ask OpenAI or Claude about one fresh selected crop.
Zero-token local recipes can detect stalled activity, compare opposing status
regions, or verify that a change in one app is followed by another app.

It is pre-alpha and available on GitHub:
https://github.com/o-henry/pebble

## Show HN

**Title**

Show HN: Pebble - local, region-scoped monitoring for any macOS app

**Post**

Hi HN,

I built Pebble because many status surfaces have no useful webhook or native
notification: build logs, render jobs, upload progress, queues, dashboards,
timers, and remote desktop sessions.

Pebble works at the desktop layer rather than inside a browser extension. You
drag over any visible macOS region, and it becomes a small always-on-top live
tile. Optional Watch mode uses a low-FPS local gate. After a material change,
it sends only the previous and current selected crops to the chosen provider,
no more often than the interval selected by the user. It stores no frame
history and does not continuously upload the screen.

Local-only recipes also cover stalled regions, opposing status regions, and
cross-app follow-through deadlines without provider AI; the visual-only paths
do not run OCR.

For explicit questions, Pebble can send one freshly authorized selected crop to
OpenAI through account sign-in, or to Claude through its CLI subscription or an
optional Anthropic API key stored in macOS Keychain. That path is separate from
monitoring and runs only after pressing Ask.

The project is MIT licensed and pre-alpha. It currently requires building from
source on macOS. I would especially value feedback on the trust model, useful
non-browser workflows, and packaging priorities.

GitHub: https://github.com/o-henry/pebble

## Reddit

Suggested communities must be checked against their current self-promotion
rules immediately before posting. Lead with the problem and technical choices,
not a star request.

**Title**

I made an open-source macOS utility that watches one selected screen region locally

**Post**

I kept missing state changes in tools that do not have good notifications, so I
built Pebble.

You can select a small region from any browser or native macOS app and keep it
as an always-on-top live tile. A local Watch mode can notify on material visual
changes. It runs a low-FPS local gate and stores no frame history. When Watch is
enabled and a material change occurs, it sends only the before-and-after crops
to the chosen provider, no more often than the interval selected by the user.

It also includes local-only recipes for stalled activity, opposing states, and
checking whether one app responds after another changes.

There is also an optional Ask flow for OpenAI or Claude. It sends exactly one
fresh selected crop only when the user presses Ask. OpenAI uses account sign-in;
Claude can use its CLI subscription or an optional API key stored in macOS
Keychain.

The project is MIT licensed and still pre-alpha, so it currently builds from
source. Feedback on useful workflows and the privacy model would be genuinely
helpful:

https://github.com/o-henry/pebble

## Product Hunt Tagline

Keep any part of your Mac in view, with local change alerts.

## Launch Checklist

1. Ship a signed installer before using language such as "install in one click."
2. Record a synthetic demo showing a browser, terminal, and native app.
3. Publish the GitHub repository first and verify the README install commands.
4. Post to one community at a time and answer technical questions directly.
5. Ask for workflow and trust-model feedback; do not ask users to manufacture stars.
6. Never upload a private real-screen capture as launch media.
