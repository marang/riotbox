# `RIOTBOX-883` Prioritize unavailable Source Timing cue over manual-confirm wording

- Ticket: `RIOTBOX-883`
- Title: `Prioritize unavailable Source Timing cue over manual-confirm wording`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-883/prioritize-unavailable-source-timing-cue-over-manual-confirm-wording`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-883-prioritize-unavailable-source-timing-cue`
- Linear branch: `feature/riotbox-883-prioritize-unavailable-source-timing-cue-over-manual-confirm`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#877 (https://github.com/marang/riotbox/pull/877)`
- Merge commit: `4c5ced83e7ab2581bac55b1e71acff9adac808ee`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; cargo test -p riotbox-core source_timing_readiness_labels; cargo test -p riotbox-audio --bin source_timing_probe; cargo test -p riotbox-audio --bin feral_grid_pack; cargo test -p riotbox-app --bin observer_audio_correlate; cargo clippy -p riotbox-core -- -D warnings; cargo clippy -p riotbox-app --bin observer_audio_correlate -- -D warnings; just source-timing-probe-json-validator-fixtures; just source-timing-example-probe-report-fixtures; just source-timing-grid-use-contract-fixtures; just listening-manifest-validator-fixtures; just observer-audio-summary-validator-fixtures; just source-timing-example-probe-report-local; just ci; GitHub Rust CI #2173 success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Unavailable Source Timing rows were still using manual-confirm language, which told musicians to confirm a grid that did not exist.

## What Shipped

- Shared readiness labels now prioritize unavailable timing over manual confirmation; validators, generated contract fixtures, example expectations, and Source Timing spec all enforce not available / timing unavailable for unavailable rows.

## Notes

- None
