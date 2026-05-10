# `RIOTBOX-234` Capture Scene-target TR-909 context readability baseline

- Ticket: `RIOTBOX-234`
- Title: `Capture Scene-target TR-909 context readability baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-234/capture-scene-target-tr-909-context-readability-baseline`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-234-scene-tr909-context-baseline`
- Linear branch: `feature/riotbox-234-capture-scene-target-tr-909-context-readability-baseline`
- PR: `#224`
- Merge commit: `e18773a`
- Labels: `benchmark`, `ux`
- Follow-ups: `RIOTBOX-235`

## Why This Ticket Existed

The Scene Brain cue stack now exposes TR-909 `scene_target` / `transport_bar` diagnostics and the recipes explain how to read them. The benchmark archive needed a bounded readability baseline for the current Log/Inspect wording before later TUI simplification or audio QA work moves those labels.

## What Shipped

- Added `docs/benchmarks/scene_tr909_support_context_baseline_2026-04-26.md`.
- Documented the current `scene_target`, `transport_bar`, and `unset` support-context label contract.
- Recorded the current limitation that this is render-state/profile coupling, not a finished transition engine.
- Indexed the new baseline in `docs/benchmarks/README.md` and `docs/README.md`.

## Verification

- `git diff --check`
- `rg -n "scene_tr909_support_context|source_support via drum_bus_support|scene_target|transport_bar|TR-909 support-context" docs/benchmarks/scene_tr909_support_context_baseline_2026-04-26.md docs/benchmarks/README.md docs/README.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/readability slice only; no runtime behavior, audio behavior, or TUI wording changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
