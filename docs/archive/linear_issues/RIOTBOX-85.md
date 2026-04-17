# `RIOTBOX-85` Add first bounded W-30 slice-pool browse control on the current lineage seam

- Ticket: `RIOTBOX-85`
- Title: `Add first bounded W-30 slice-pool browse control on the current lineage seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-85/add-first-bounded-w-30-slice-pool-browse-control-on-the-current`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-85-w30-slice-pool-browse`
- Linear branch: `feature/riotbox-85-add-first-bounded-w-30-slice-pool-browse-control-on-the`
- Assignee: `Markus`
- Labels: `None`
- PR: `#79`
- Merge commit: `90c6f32051b9e79c56e00330642cfe9827127df3`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#218`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-86`, `RIOTBOX-87`, `RIOTBOX-88`

## Why This Ticket Existed

The W-30 MVP already had live recall, audition, bank-manager, damage-profile, internal-resample, and loop-freeze seams, but loop-freeze reuse could now leave multiple captures on the same W-30 pad target with no honest way to step through that bounded pool. The next slice needed to stay on the current pad-lineage seam instead of opening a second browser or inventory model.

## What Shipped

- added bounded `w30.browse_slice_pool` on the existing W-30 lane and action lexicon
- queued the browse move on `NextBeat` against captures already assigned to the current focused W-30 bank/pad
- committed the selected capture back through the same W-30 live-recall preview seam by updating bank, pad, `last_capture`, and preview mode
- surfaced the pending browse cue in the shell and extended queue, committed-state, and shell tests for the new seam

## Notes

- this slice intentionally stays on the current pad target only; it does not introduce cross-pad slice browsing or a separate W-30 browser model
- richer diagnostics, replay fixtures, and preview profiling for slice-pool browsing remain split into the follow-up tickets
