# `RIOTBOX-58` Ticket Archive

- Ticket: `RIOTBOX-58`
- Title: `Add replay-safe W-30 live-recall and audition regression fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-58/add-replay-safe-w-30-live-recall-and-audition-regression-fixtures`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-58-w30-regression-fixtures`
- Linear branch: `feature/riotbox-58-add-replay-safe-w-30-live-recall-and-audition-regression`
- Assignee: `Markus`
- Labels: `None`
- PR: `#52`
- Merge commit: `4cf70a7`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-59`

## Why This Ticket Existed

The shipped W-30 seam already had live recall, promoted-material audition, and clearer shell diagnostics, but it still lacked the same replay-safe regression net already used for TR-909 and MC-202. Riotbox needed one bounded verification slice that hardened the current committed cue path before the lane grew into deeper preview or pad behavior.

## What Shipped

- Added a shared `w30_regression.json` fixture corpus in `riotbox-app`.
- Added fixture-backed committed-state regressions in `jam_app` for W-30 live recall and promoted audition.
- Added fixture-backed shell regressions in `ui` that assert Capture and Log output from the same corpus.
- Recorded the verification strategy in the research decision log so later W-30 work extends the same fixture net.

## Notes

- This slice stayed verification-only and did not add new W-30 musical behavior or any audio-facing preview seam.
- The existing MC-202 shell fixture assertions stayed in place but were intentionally kept token-based where panel-width changes make full-line strings brittle.
