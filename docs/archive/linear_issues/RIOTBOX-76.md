# `RIOTBOX-76` Add replay-safe W-30 bank-manager and pad-forge regression fixtures

- Ticket: `RIOTBOX-76`
- Title: `Add replay-safe W-30 bank-manager and pad-forge regression fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-76/add-replay-safe-w-30-bank-manager-and-pad-forge-regression-fixtures`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-76-w30-regression-fixtures`
- Linear branch: `feature/riotbox-76-add-replay-safe-w-30-bank-manager-and-pad-forge-regression`
- Assignee: `Markus`
- Labels: `None`
- PR: `#70`
- Merge commit: `b0d8cacc11f17146ffffea90631888ea58c5e718`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#191`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The W-30 MVP already had a shared replay-safe regression corpus for live recall and promoted audition, but the newly shipped bank-manager and pad-forge controls were still relying on one-off tests. The repo needed those new committed preview-lane actions to use the same fixture-backed hardening path before more W-30 behavior landed.

## What Shipped

- extended `w30_regression.json` with committed `swap_bank` and `apply_damage_profile` cases
- widened the app-side committed-state regression test so the shared corpus now asserts lane state, preview projection, grit, and result summaries for those new controls
- widened the shell regression test so the same corpus now asserts Jam, Capture, and Log visibility for committed bank-manager and pad-forge diagnostics
- kept the slice verification-only and recorded the shared-corpus decision in `docs/research_decision_log.md`

## Notes

- this slice intentionally hardens existing W-30 behavior without changing the shipped bank-swap or damage-profile semantics
- later W-30 controls on the same committed preview seam should extend the shared fixture corpus unless they genuinely add a new runtime dimension
