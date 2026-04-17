# `RIOTBOX-86` Surface W-30 slice-pool browse diagnostics in Capture and Log shell

- Ticket: `RIOTBOX-86`
- Title: `Surface W-30 slice-pool browse diagnostics in Capture and Log shell`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-86/surface-w-30-slice-pool-browse-diagnostics-in-capture-and-log-shell`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-86-w30-slice-pool-diagnostics`
- Linear branch: `feature/riotbox-86-surface-w-30-slice-pool-browse-diagnostics-in-capture-and`
- Assignee: `Markus`
- Labels: `None`
- PR: `#80`
- Merge commit: `58481839eb8ad5f816b5a4943da83954ea8bae19`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#221`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-87`, `RIOTBOX-88`

## Why This Ticket Existed

`RIOTBOX-85` introduced the first bounded W-30 slice-pool browse control on the current pad-lineage seam, but the existing shell still hid too much of that state. The next slice needed to keep current-vs-next browse intent legible in the existing `Capture` and `Log` surfaces instead of opening a second W-30 page or diagnostics-only browser.

## What Shipped

- projected pending W-30 slice-pool capture ids through the core `JamViewModel` so shell surfaces can see current vs next browse state without direct queue scans
- deepened `Capture -> Routing / Promotion` so current-pad slice-pool state is visible alongside the pending browse cue
- deepened the compact W-30 `Log` lane so committed browse state stays legible while preserving lineage-priority diagnostics
- added fixture-backed shell regressions for pending browse cues and committed browse diagnostics

## Notes

- this slice stays presentation-only on top of the committed browse seam from `RIOTBOX-85`; it does not add new runtime or audio behavior
- replay-safe browse fixtures and later preview profiling remain split into the follow-up tickets
