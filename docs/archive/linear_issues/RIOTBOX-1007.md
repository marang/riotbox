# `RIOTBOX-1007` P012: Add ActionCommand coverage guardrails

- Ticket: `RIOTBOX-1007`
- Title: `P012: Add ActionCommand coverage guardrails`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1007/p012-add-actioncommand-coverage-guardrails`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `In Review`
- Created: `2026-05-26`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/review-codebase-fixes`
- Linear branch: `feature/riotbox-1007-p012-add-actioncommand-coverage-guardrails`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`, `workflow`
- PR: `#994 (https://github.com/marang/riotbox/pull/994)`
- Merge commit: `19d18ce670c773aac92e7a56f23cf600fe3b2cad`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-core`; `cargo test -p riotbox-app`; `just ci`; `GitHub Actions Rust CI run 26494614089 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

ActionCommand coverage was spread across queue, commit, replay, runtime warning, and observer/UI tables without a guardrail against future drift.

## What Shipped

- Added ActionCommand coverage guardrails and aligned mutate.scene replay coverage with the committed product effect.

## Notes

- None
