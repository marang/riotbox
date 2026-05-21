# `RIOTBOX-878` Surface Source Timing actionability in P012 all-lane proof manifests

- Ticket: `RIOTBOX-878`
- Title: `Surface Source Timing actionability in P012 all-lane proof manifests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-878/surface-source-timing-actionability-in-p012-all-lane-proof-manifests`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-878-source-timing-actionability-p012-proof`
- Linear branch: `feature/riotbox-878-surface-source-timing-actionability-in-p012-all-lane-proof`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`, `ux`
- PR: `#872 (https://github.com/marang/riotbox/pull/872)`
- Merge commit: `52c03d72f51073c3591e59ee6737d5f4ee889581`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio --bin feral_grid_pack; cargo test -p riotbox-app --bin observer_audio_correlate; cargo clippy -p riotbox-app --bin observer_audio_correlate -- -D warnings; just listening-manifest-validator-fixtures; just observer-audio-summary-validator-fixtures; just source-timing-grid-use-contract-fixtures; just p012-all-lane-source-grid-output-proof; git diff --check; just ci; GitHub Rust CI success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `Consider adding the same actionability field to the standalone source_timing_probe CLI summary so probe JSON/text matches manifest and observer surfaces.`

## Why This Ticket Existed

Jam, Source, Help, observer snapshots, and observer/audio summaries carried Source Timing actionability, but the P012 all-lane proof still showed only grid-use policy fields from Recipe 15 manifests.

## What Shipped

- Added Source Timing cue/actionability to generated Feral-grid manifests, preserved actionability in observer/audio summaries, validated the language in manifest and summary validators, and added an Action column to the compact P012 all-lane proof summary.

## Notes

- None
