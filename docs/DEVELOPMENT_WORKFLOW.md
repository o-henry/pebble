# Development Workflow

This document defines how Pebble work is planned, reviewed, committed, and
pushed.

## Atomic Task Loop

Every task must follow this loop:

1. Define the smallest coherent scope.
2. Identify the files and modules owned by the change.
3. Write or update tests first when the behavior is already clear.
4. Implement the change.
5. Run the relevant checks.
6. Inspect the diff for secrets, private artifacts, broad permissions, and
   security-sensitive behavior.
7. Run a separate review/refactor agent on the diff.
8. Apply narrow fixes from review.
9. Re-run relevant checks.
10. Inspect the final diff again.
11. Commit the atomic change.
12. Re-check the latest commit and working tree.
13. Push.

Do not start a second feature before the current feature is committed and pushed,
unless the user explicitly asks for a combined branch.

## Review Agent Checklist

The review/refactor agent must evaluate:

- Correctness bugs.
- Missing tests.
- Security or privacy regressions.
- Capture lifecycle leaks.
- Frame persistence by mistake.
- Excessive permissions.
- AI connector enabled by default.
- Overly large files or functions.
- Duplicated constants, command names, error codes, or coordinate logic.
- Confusing names or architecture boundaries.
- Unnecessary abstractions.

The review agent should recommend narrow fixes. It should not invent unrelated
features or broad rewrites.

## Premortem For Risky Work

Run a lightweight premortem before high-blast-radius changes:

- What could fail?
- What assumption would invalidate this plan?
- What is the rollback path?
- What is the minimum verification evidence?

High-risk examples:

- Screen capture implementation.
- Permission changes.
- Storage schema changes.
- AI connector integration.
- OCR pipeline.
- Notification or hotkey background behavior.
- Installer, auto-update, or release packaging.

## Commit Rules

Use focused commits with plain messages:

```text
docs: add engineering workflow
feat: add performance limit validation
test: cover capture lifecycle cancellation
fix: stop capture on privacy blank
refactor: split region coordinate mapper
```

Each commit must be pushable and reviewable on its own.

Before committing:

- Confirm staged files are intentional.
- Confirm `.env`, keys, tokens, cookies, captured frames, screenshots, OCR
  history, private logs, and local account data are not staged.
- Run whitespace checks with `git diff --check` or `git diff --cached --check`.
- Run relevant tests or state why they cannot run yet.

Before pushing a completed feature:

- Re-run relevant checks after review fixes.
- Inspect the final commit one more time.
- Confirm the feature slice is complete and does not include unrelated work.
- Confirm no sensitive file or private capture artifact was added.

## Verification Reporting

Each completion report should be concise and natural. Include:

- Whether the requested work is done.
- Commands run or why checks could not run.
- What changed.
- Anything that could not be completed, in plain language.
- Residual risk, briefly.

Do not use rigid status buckets unless the user explicitly asks for that format.
If checks cannot run because tooling does not exist yet, say that directly.
