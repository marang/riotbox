# `RIOTBOX-881` Centralize Source Timing readiness cue and actionability labels for Rust producers

- Ticket: `RIOTBOX-881`
- Title: `Centralize Source Timing readiness cue and actionability labels for Rust producers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-881/centralize-source-timing-readiness-cue-and-actionability-labels-for`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-881-centralize-source-timing-readiness-cue-and-actionability`
- Linear branch: `feature/riotbox-881-centralize-source-timing-readiness-cue-and-actionability`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#875 (https://github.com/marang/riotbox/pull/875)`
- Merge commit: `062a2dae7d470bbde446566b85f5db20c1f3b95b`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; cargo test -p riotbox-core source_timing_readiness_labels; cargo test -p riotbox-audio --bin source_timing_probe; cargo test -p riotbox-audio --bin feral_grid_pack; cargo clippy -p riotbox-audio --bin source_timing_probe -- -D warnings; cargo clippy -p riotbox-audio --bin feral_grid_pack -- -D warnings; just source-timing-probe-json-validator-fixtures; just listening-manifest-validator-fixtures; git diff --check; GitHub Rust CI #2167 passed`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-882`

## Why This Ticket Existed

RIOTBOX-880 found producer-side duplicated Source Timing readiness cue/actionability mappings across the probe CLI and generated Feral-grid manifest builder.

## What Shipped

- Added shared core SourceTimingReadinessLabels helpers beside the typed Source Timing grid-use policy.
- Updated source_timing_probe and generated Feral-grid manifests to emit cue/actionability through the shared helper without changing output strings.
- Updated the Source Timing spec to require shared Rust producer helpers for readiness cue/actionability labels.

## Notes

- None
