# `RIOTBOX-67` Add first bounded W-30 pad-bank stepping on the committed preview seam

- Ticket: `RIOTBOX-67`
- Title: `Add first bounded W-30 pad-bank stepping on the committed preview seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-67/add-first-bounded-w-30-pad-bank-stepping-on-the-committed-preview-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-67-w30-pad-bank-step`
- Linear branch: `feature/riotbox-67-add-first-bounded-w-30-pad-bank-stepping-on-the-committed`
- Assignee: `Markus`
- Labels: `None`
- PR: `#65`
- Merge commit: `b089d96ca358e06bf8110f6311f1ce5218685c65`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `Restore 1-5 near-next backlog tickets after the active W-30 slice`

## Why This Ticket Existed

The W-30 MVP already had committed preview, trigger, recall, audition, and resample behavior on one replay-safe seam, but it still lacked one honest way to step between promoted pads without inventing a shell-only cursor or opening a full pad-bank editor early.

## What Shipped

- added explicit committed `w30.step_focus` on the existing `NextBeat` W-30 preview seam
- surfaced pending focus-step targets through the core `JamViewModel.lanes` contract
- added a bounded shell keybinding and W-30 cue/log wording for the new step action
- covered queue, commit, and shell rendering behavior with focused app and UI regressions

## Notes

- the committed step action stays on the same W-30 preview seam as recall and trigger behavior instead of overloading `w30.swap_bank`
- the step target is derived from actual promoted W-30 pad targets, not from a separate shell cursor model
