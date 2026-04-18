# `RIOTBOX-104` Split Jam help and footer into primary vs advanced gesture groups

- Ticket: `RIOTBOX-104`
- Title: `Split Jam help and footer into primary vs advanced gesture groups`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-104/split-jam-help-and-footer-into-primary-vs-advanced-gesture-groups`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `Jam-First Playable Slice`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-104-gesture-ranks`
- Linear branch: `feature/riotbox-104-split-jam-help-and-footer-into-primary-vs-advanced-gesture`
- Assignee: `Markus`
- Labels: `None`
- PR: `#97`
- Merge commit: `25b0a662a5fe9f92e1be9f1f30e7d48c38bd1955`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#276`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-105`, `RIOTBOX-106`

## Why This Ticket Existed

The Jam shell was already more musical in tone, but its footer and help copy still made too many controls feel equal in urgency. Riotbox needed one bounded pass that taught a small primary gesture set first while still keeping the deeper lane-shaping vocabulary visible for players who wanted more control.

## What Shipped

- narrowed the default Jam footer primary line to the highest-frequency play gestures
- moved lower-frequency and lane-shaping controls into an explicit advanced footer line
- aligned the help overlay to the same primary-versus-advanced rank split
- added snapshot assertions so the new footer rank split does not silently collapse back into one undifferentiated action list

## Notes

- this slice changed presentation only; it preserved the existing keymap and action semantics
- `answer` deliberately moved out of the first-line primary gesture group so the shell now teaches jump/follow/fill/capture/hit/undo before the richer phrase-shaping vocabulary
- the next honest follow-up is the post-commit `what changed / what next` cue, followed by the broader lane-card compression pass
