# `RIOTBOX-1496` Workflow: require completion challenge before ending an active assignment

- Ticket: `RIOTBOX-1496`
- Title: `Workflow: require completion challenge before ending an active assignment`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1496/workflow-require-completion-challenge-before-ending-an-active`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-09`
- Started: `2026-09-09`
- Finished: `2026-09-09`
- Branch: `feature/riotbox-1496-completion-challenge`
- Linear branch: `feature/riotbox-1496-workflow-require-completion-challenge-before-ending-an`
- Assignee: `Markus`
- Labels: `workflow`
- PR: `#1519 (https://github.com/marang/riotbox/pull/1519)`
- Merge commit: `4c3f94e1ff27b19f9744f304b8fc531f25f494cb`
- Deleted from Linear: `2026-09-09`
- Verification: `Source-free just ci passed; /tmp/riotbox-1496-ci.log.`; `Independent docs review: explicit-stop wording corrected; re-review and self-review zero remaining findings. Scenario checks and git diff --check passed.`; `GitHub Rust CI 34367887536 passed before PR1519 merge; no open review comments.`
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`
- Follow-ups: `Rule compliance must be demonstrated in subsequent work; scenario review does not guarantee future model behavior. RIOTBOX-1495 remains Todo.`

## Why This Ticket Existed

Repeated premature stops after intervening status questions left already-authorized workflow obligations unfinished.

## What Shipped

- Workflow v0.5 requires a fresh completion challenge at every ending boundary; AGENTS provides the always-loaded routing pin.
- Intervening messages preserve active scope; blockers apply only to dependent steps. Explicit stop, bounded completion and all existing authority/safety gates remain intact.

## Notes

- Workflow-only maintenance; no product/audio changes, source access or playback.
