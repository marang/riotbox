# `RIOTBOX-107` Add first bounded scene restore action on the current transport seam

- Ticket: `RIOTBOX-107`
- Title: `Add first bounded scene restore action on the current transport seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-107/add-first-bounded-scene-restore-action-on-the-current-transport-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-107-scene-restore`
- Linear branch: `feature/riotbox-107-add-first-bounded-scene-restore-action-on-the-current`
- Assignee: `Markus`
- Labels: `None`
- PR: `#100`
- Merge commit: `85641dead987917e8f736f31153e77b086f24b3f`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `GitHub Actions Rust CI #284`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-108`, `RIOTBOX-109`, `RIOTBOX-110`

## Why This Ticket Existed

Scene Brain already had deterministic candidate projection, scene launch, diagnostics, and regression fixtures, but it still lacked an explicit recovery move. Riotbox needed the smallest restore action that reused the committed session model and transport seam instead of opening a second scene transition architecture.

## What Shipped

- added a bounded `scene.restore` action on the existing `NextBar` queue and commit seam
- reused the committed `restore_scene` pointer as the restore target so the recovery path stays replay-safe
- flipped the restore pointer to the previously active scene when a restore lands so recovery remains explicit
- added the first shell affordance for queuing restore and a minimal pending restore cue
- covered the restore path with committed-state and shell-side regressions

## Notes

- this slice intentionally stopped at one deterministic recovery move and did not open a richer scene transition policy
- deeper restore visibility and regression fixture work were split immediately into `RIOTBOX-108` and `RIOTBOX-109`
