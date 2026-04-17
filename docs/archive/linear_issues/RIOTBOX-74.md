# `RIOTBOX-74` Add first bounded W-30 damage-profile control on the current pad-forge seam

- Ticket: `RIOTBOX-74`
- Title: `Add first bounded W-30 damage-profile control on the current pad-forge seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-74/add-first-bounded-w-30-damage-profile-control-on-the-current-pad-forge`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-74-w30-damage-profile-control`
- Linear branch: `feature/riotbox-74-add-first-bounded-w-30-damage-profile-control-on-the-current`
- Assignee: `Markus`
- Labels: `None`
- PR: `#68`
- Merge commit: `47a7206659ff16aeb751304c5f02cb87b5ad511a`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#185`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-75`, `RIOTBOX-76`

## Why This Ticket Existed

The W-30 MVP already had replay-safe preview, trigger, recall, bank-swap, and resample behavior on one committed seam, but it still had no honest pad-forge move. `w30.apply_damage_profile` existed only as a name in the action lexicon; the repo still needed one bounded real control on the current W-30 preview seam before broader forge work could land honestly.

## What Shipped

- added the first explicit `w30.apply_damage_profile` queue path on the existing W-30 preview seam
- targeted the current W-30 pad capture and committed the action on `NextBar`
- raised the existing `w30_grit` path through one bounded `shred` profile instead of inventing a separate forge editor or a per-pad persistence model
- surfaced pending and committed damage cues through the core Jam view model, the current shell, and the event loop status messaging
- added queue, commit, and shell regressions for the bounded pad-forge control

## Notes

- this slice is intentionally limited to one explicit `shred` profile on top of the existing global `w30_grit` seam
- per-pad forge persistence, multiple named damage profiles, and broader bank-grid editing remain later work on top of the same committed W-30 path
