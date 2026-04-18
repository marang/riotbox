# `RIOTBOX-112` Add scene jump and restore workflow benchmark baseline

- Ticket: `RIOTBOX-112`
- Title: `Add scene jump and restore workflow benchmark baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-112/add-scene-jump-and-restore-workflow-benchmark-baseline`
- Project: `Riotbox MVP Buildout`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-112-scene-restore-benchmark`
- Linear branch: `feature/riotbox-112-add-scene-jump-and-restore-workflow-benchmark-baseline`
- Assignee: `Markus`
- Labels: `None`
- PR: `#105`
- Merge commit: `1abdf3430e8cdcdf36600077cfb8b93f40666174`
- Deleted from Linear: `Not deleted`
- Verification: `self-reviewed docs-only diff for benchmark consistency and repo indexing`, GitHub Actions `Rust CI` run `#299`
- Docs touched: `docs/README.md`, `docs/benchmarks/README.md`, `docs/benchmarks/scene_jump_restore_workflow_baseline_2026-04-18.md`
- Follow-ups: `RIOTBOX-113`

## Why This Ticket Existed

The first Scene Brain recovery loop was already real, visible, replay-safe, and documented, but Riotbox still lacked one honest benchmark artifact for how quickly a user could complete the first `scene jump -> restore` path. The product needed a repo-local baseline before adding more timing-visibility or scene-learning changes.

## What Shipped

- added the first explicit workflow benchmark baseline for the current Scene Brain `jump -> restore` loop
- recorded the fixture, interaction path, timing assumptions, and acceptable current-budget numbers in repo docs
- indexed the new benchmark alongside the existing Jam workflow baseline

## Notes

- this slice stayed benchmark-only and did not change any scene behavior or shell text
- the baseline remains intentionally manual and repo-local, using the shipped recipe path instead of a new telemetry subsystem
