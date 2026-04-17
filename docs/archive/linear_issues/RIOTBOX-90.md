# `RIOTBOX-90` Add first committed scene-select action on the current transport seam

- Ticket: `RIOTBOX-90`
- Title: `Add first committed scene-select action on the current transport seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-90/add-first-committed-scene-select-action-on-the-current-transport-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-90-scene-select`
- Linear branch: `feature/riotbox-90-add-first-committed-scene-select-action-on-the-current`
- Assignee: `Markus`
- Labels: `None`
- PR: `#84`
- Merge commit: `e678fd3cdfc53ee262b06fcd7e4fb829b88ad78d`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#233`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-91`, `RIOTBOX-92`, `RIOTBOX-93`

## Why This Ticket Existed

`RIOTBOX-89` had already made scene candidates real by deriving them from analyzed source sections and projecting them into the existing session scene state. The next honest `Scene Brain` step was to prove Riotbox could now move between those committed scene candidates on the existing queue and transport seam, instead of opening a second arrangement or transition model.

## What Shipped

- added one bounded `scene.launch` action on the existing action queue and commit-boundary path
- cycles to the next committed scene candidate and queues it on `NextBar`
- commits scene selection into both session `active_scene` and transport `current_scene`
- keeps runtime transport and render state aligned with the committed scene change
- added focused regressions for queueing, duplicate pending protection, and committed scene-state updates
- added a minimal Jam-shell key path for the first scene-select control

## Notes

- this slice is intentionally bounded to first committed scene selection; richer scene launch, restore, recovery, and transition logic remain follow-up work
- the action stays explicit, logged, and replay-safe on the existing transport seam rather than creating a second Scene Brain execution path
