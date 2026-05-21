# `RIOTBOX-887` Centralize app-side Source Timing readiness actionability fallback

- Ticket: `RIOTBOX-887`
- Title: `Centralize app-side Source Timing readiness actionability fallback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-887/centralize-app-side-source-timing-readiness-actionability-fallback`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-887-centralize-app-readiness-actionability`
- Linear branch: `feature/riotbox-887-centralize-app-side-source-timing-readiness-actionability`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#881 (https://github.com/marang/riotbox/pull/881)`
- Merge commit: `6289471ed17da59e3826b7f2648ab93c95018e6e`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; cargo test -p riotbox-app source_timing_readiness_actionability_prioritizes_unavailable_then_manual_confirm; cargo test -p riotbox-app --bin observer_audio_correlate; cargo clippy -p riotbox-app --bin observer_audio_correlate -- -D warnings; just observer-audio-summary-validator-fixtures; just ci; git diff --check main...HEAD; GitHub Rust CI #2185 success`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-888 exposes downbeat phase confidence in generated Feral-grid Source Timing manifests`

## Why This Ticket Existed

Observer/audio summary fallback rendering still carried a local readiness/manual-confirm-to-actionability phrase table after the rest of the Source Timing actionability surface had moved behind shared helpers.

## What Shipped

- Added a shared app-side Source Timing readiness actionability fallback helper and switched observer/audio summary rendering to use it when evidence omits an explicit actionability label.

## Notes

- None
