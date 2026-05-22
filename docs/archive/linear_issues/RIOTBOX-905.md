# `RIOTBOX-905` Centralize Source Timing policy cue/actionability labels

- Ticket: `RIOTBOX-905`
- Title: `Centralize Source Timing policy cue/actionability labels`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-905/centralize-source-timing-policy-cueactionability-labels`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-905-centralize-source-timing-policy-labels`
- Linear branch: `feature/riotbox-905-centralize-source-timing-policy-cueactionability-labels`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#898 (https://github.com/marang/riotbox/pull/898)`
- Merge commit: `3cf8058d29180155d2916fb6d346fb409f7ff57a`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-core source_timing_; cargo test -p riotbox-app source_timing_cues; cargo test -p riotbox-app --bin observer_audio_correlate; cargo test -p riotbox-app shell_state_jam_snapshot; git diff --check; just ci; GitHub Rust CI #2237`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Source Timing readiness labels were centralized, but degraded-policy cue/actionability labels still had parallel Rust string tables between the core Jam summary path and app helper path.

## What Shipped

- Added shared core Source Timing policy-label helpers and used them from SourceTimingSummaryView and app source_timing_cues without changing visible labels.

## Notes

- None
