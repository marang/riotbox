# `RIOTBOX-92` Add replay-safe Scene Brain committed-state and shell regression fixtures

- Ticket: `RIOTBOX-92`
- Title: `Add replay-safe Scene Brain committed-state and shell regression fixtures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-92/add-replay-safe-scene-brain-committed-state-and-shell-regression`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-92-scene-fixtures`
- Linear branch: `feature/riotbox-92-add-replay-safe-scene-brain-committed-state-and-shell`
- Assignee: `Markus`
- Labels: `None`
- PR: `#86`
- Merge commit: `fa952211023d1588cef5273e0a22b1be6a1bfe6e`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#239`
- Follow-ups: `RIOTBOX-93`, `RIOTBOX-94`

## Why This Ticket Existed

`RIOTBOX-89`, `RIOTBOX-90`, and `RIOTBOX-91` had already made the first Scene Brain seams real: deterministic scene candidates, one committed `scene.launch` action, and visible scene diagnostics in the current shell. The next honest step was verification, not new behavior. Scene Brain needed the same replay-safe regression pattern already established for TR-909, MC-202, and W-30 before deeper launch or restore logic could scale.

## What Shipped

- added a shared `scene_regression.json` fixture corpus for scene-candidate projection and the first committed `scene.launch` behavior
- asserted replay-safe committed session state and result summaries from that corpus in `jam_app`
- asserted Jam and Log visible scene diagnostics from the same corpus in `ui`
- kept the slice verification-only and avoided opening a second Scene Brain execution or presentation path

## Notes

- the fixture corpus deliberately covers both candidate projection and the first committed scene-select seam
- richer Scene Brain behavior such as restore, transition, or deeper launch controls remains follow-up work
