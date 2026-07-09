# Development Workflow

This document defines how ScreenPebble work is planned, reviewed, committed, and
pushed.

## Atomic Task Loop

Every task must follow this loop:

1. Define the smallest coherent scope.
2. Identify the files and modules owned by the change.
3. Write or update tests first when the behavior is already clear.
4. Implement the change.
5. Run the relevant checks.
6. Run a separate review/refactor agent on the diff.
7. Apply narrow fixes from review.
8. Re-run relevant checks.
9. Inspect the final diff.
10. Commit and push.

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

## Verification Reporting

Each completion report must include:

- 완료 or 미완료.
- Commands run.
- What changed.
- 해결됨 items.
- 검증 필요 items for non-blocking residual confidence work.
- 검증된 미해결 items only when an acceptance criterion is unmet.
- 범위 밖 items only when intentionally deferred.
- Residual risk.

If checks cannot run because tooling does not exist yet, say that directly.
