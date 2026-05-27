# `RIOTBOX-1006` P012: Give scene mutation a real product effect or remove the live gesture

- Ticket: `RIOTBOX-1006`
- Title: `P012: Give scene mutation a real product effect or remove the live gesture`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1006/p012-give-scene-mutation-a-real-product-effect-or-remove-the-live`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `In Review`
- Created: `2026-05-26`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/review-codebase-fixes`
- Linear branch: `feature/riotbox-1006-p012-give-scene-mutation-a-real-product-effect-or-remove-the`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`
- PR: `#994 (https://github.com/marang/riotbox/pull/994)`
- Merge commit: `19d18ce670c773aac92e7a56f23cf600fe3b2cad`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-core`; `cargo test -p riotbox-app`; `just ci`; `GitHub Actions Rust CI run 26494614089 completed successfully`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The live mutate.scene gesture could commit as a log-only/no-op product action.

## What Shipped

- Gave mutate.scene a deterministic committed Session/log/replay effect by updating scene aggression and covering live/replay paths.

## Notes

- None
