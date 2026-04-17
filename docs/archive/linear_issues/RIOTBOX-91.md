# `RIOTBOX-91` Surface scene candidate and active-scene diagnostics in Jam and Log shell

- Ticket: `RIOTBOX-91`
- Title: `Surface scene candidate and active-scene diagnostics in Jam and Log shell`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-91/surface-scene-candidate-and-active-scene-diagnostics-in-jam-and-log`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-91-scene-diagnostics`
- Linear branch: `feature/riotbox-91-surface-scene-candidate-and-active-scene-diagnostics-in-jam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#85`
- Merge commit: `f619138fe191bda61e0384a819b263eea1b8dcdb`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#236`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-92`, `RIOTBOX-93`

## Why This Ticket Existed

`RIOTBOX-90` had already proven the first committed `scene.launch` seam on the existing queue and transport path. The next honest `Scene Brain` slice was to make that new seam visible in the operator shell immediately, the same way TR-909, MC-202, and W-30 state already stays legible in `Jam` and `Log`.

## What Shipped

- surfaced active scene, next scene candidate, and pending scene-launch state in the existing `Jam` overview
- folded committed and pending scene diagnostics into the current `Log` summary spine instead of opening a second scene page
- added focused shell regression coverage for the new Scene Brain diagnostics
- recorded the presentation-only Scene Brain decision in `docs/research_decision_log.md`

## Notes

- this slice stayed presentation-only on top of the shipped committed `scene.launch` behavior
- replay-safe Scene Brain fixtures and richer launch, restore, or transition behavior remain follow-up work
