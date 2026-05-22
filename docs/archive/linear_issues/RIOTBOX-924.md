# `RIOTBOX-924` Review P015 TUI module ownership after split batch

- Ticket: `RIOTBOX-924`
- Title: `Review P015 TUI module ownership after split batch`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-924/review-p015-tui-module-ownership-after-split-batch`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-924-p015-tui-module-review`
- Linear branch: `feature/riotbox-924-review-p015-tui-module-ownership-after-split-batch`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#917 (https://github.com/marang/riotbox/pull/917)`
- Merge commit: `9ea44db41951f3cd35a3321e43ad49c52bebd6a9`
- Deleted from Linear: `2026-05-22`
- Verification: `git diff --check main...HEAD; GitHub Actions Rust CI run 26289055981 passed`
- Docs touched: `docs/reviews/p015_tui_module_ownership_review_2026-05-22.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-925 starts the first recommended P015 TUI test split`

## Why This Ticket Existed

The repo workflow calls for a broad review after the split batch so remaining TUI ownership work stays semantic and roadmap-aligned.

## What Shipped

- Added the P015 TUI module ownership review and RBX-035 decision to continue semantic helper extraction slices while avoiding mechanical include churn.

## Notes

- Documentation/review only; no ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
