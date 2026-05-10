# `RIOTBOX-227` Add Scene contrast launch readability baseline

- Ticket: `RIOTBOX-227`
- Title: `Add Scene contrast launch readability baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-227/add-scene-contrast-launch-readability-baseline`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-227-scene-contrast-baseline`
- Linear branch: `feature/riotbox-227-add-scene-contrast-launch-readability-baseline`
- PR: `#217`
- Merge commit: `7d40e84`
- Labels: `benchmark`, `ux`
- Follow-ups: `RIOTBOX-228`

## Why This Ticket Existed

`RIOTBOX-226` made Scene launch target selection prefer a known energy contrast before falling back to ordered adjacency. That changed the first visible Scene Brain behavior enough that docs and benchmarks needed to capture how a performer can verify why the selected target is musically intentional.

## What Shipped

- Added `docs/benchmarks/scene_contrast_launch_baseline_2026-04-25.md`.
- Captured the expected `scene-01-drop -> scene-03-intro` contrast-target behavior.
- Updated benchmark indexes so the baseline is discoverable from `docs/README.md` and `docs/benchmarks/README.md`.

## Verification

- `git diff --check`
- `rg -n "scene_contrast_launch|Scene Contrast Launch|contrast launch target" docs/benchmarks/README.md docs/README.md docs/benchmarks/scene_contrast_launch_baseline_2026-04-25.md`
- GitHub Actions `rust-ci`

## Notes

- Documentation and benchmark slice only; no runtime behavior, broad recipe rewrite, or new audio QA harness changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
