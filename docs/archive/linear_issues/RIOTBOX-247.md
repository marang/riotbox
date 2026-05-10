# `RIOTBOX-247` Refresh Jam baseline for compact TR-909 lift cue

- Ticket: `RIOTBOX-247`
- Title: `Refresh Jam baseline for compact TR-909 lift cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-247/refresh-jam-baseline-for-compact-tr-909-lift-cue`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-247-refresh-jam-baseline-for-compact-tr-909-lift-cue`
- Linear branch: `feature/riotbox-247-refresh-jam-baseline-for-compact-tr-909-lift-cue`
- PR: `#237`
- Merge commit: `e1d154a`
- Labels: `benchmark`, `ux`
- Follow-ups: `RIOTBOX-248`

## Why This Ticket Existed

`RIOTBOX-246` added a compact `909 lift` cue to Scene post-commit guidance on the primary Jam surface. The repo needed the smallest matching benchmark artifact refreshed so future UI work preserves that cue intentionally instead of relying only on unit assertions.

## What Shipped

- Refreshed `docs/benchmarks/scene_post_landed_energy_cue_baseline_2026-04-25.md`.
- Recorded both jump and restore post-landed cue examples with `909 lift`.
- Clarified that `909 lift` is a compact Jam hint while Log/Inspect remain the diagnostic surfaces.
- Synced `docs/README.md` and `docs/benchmarks/README.md` summaries with the refreshed baseline.

## Verification

- `git diff --check`
- `rg -n '909 lift|scene_post_landed_energy_cue_baseline_2026-04-25' docs/README.md docs/benchmarks/README.md docs/benchmarks/scene_post_landed_energy_cue_baseline_2026-04-25.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/benchmark slice only; no runtime behavior, audio behavior, screenshot generation, or broad cue-ladder rewrite changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
