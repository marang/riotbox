# `RIOTBOX-70` Ticket Archive

- Ticket: `RIOTBOX-70`
- Title: `Make W-30 capture resolution follow committed lane focus`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-70/make-w-30-capture-resolution-follow-committed-lane-focus`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-70-w30-focus-resolution`
- Linear branch: `feature/riotbox-70-make-w-30-capture-resolution-follow-committed-lane-focus`
- Assignee: `Markus`
- Labels: `None`
- PR: `#62`
- Merge commit: `cdd9cd3`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`, `self-review`, `GitHub Actions Rust CI`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-67`, `RIOTBOX-69`, `RIOTBOX-71`

## Why This Ticket Existed

The periodic review in `RIOTBOX-68` found that W-30 pad-facing actions could still choose captures by history instead of by the committed lane focus already shown in the shell. That drift would make later pad-bank stepping partly cosmetic and would leave recall, audition, trigger, and internal resample actions inconsistent with the committed W-30 lane state.

## What Shipped

- Added one focus-aware W-30 capture resolver based on committed `active_bank` plus `focused_pad`.
- Routed W-30 recall, audition, trigger, and internal-resample helpers through that resolver whenever explicit lane focus exists.
- Preserved the older latest-pinned and latest-promoted fallback only for the no-focus case.
- Updated app and shell regression fixtures so the focused-lane contract stays locked in place.
- Recorded the focused-lane decision in `docs/research_decision_log.md`.

## Notes

- The first full test run exposed older W-30 tests that were still assuming capture lookup ignored committed focus; those fixtures were corrected instead of weakening the new rule.
- This slice intentionally stopped at selection correctness. Bounded pad-bank stepping and any later shell/core cleanup remain separate follow-up tickets on the same W-30 seam.
