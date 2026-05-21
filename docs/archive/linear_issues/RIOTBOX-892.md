# `RIOTBOX-892` Centralize Source Timing readiness actionability labels for Rust producers

- Ticket: `RIOTBOX-892`
- Title: `Centralize Source Timing readiness actionability labels for Rust producers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-892/centralize-source-timing-readiness-actionability-labels-for-rust`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-892-readiness-actionability-labels`
- Linear branch: `feature/riotbox-892-centralize-source-timing-readiness-actionability-labels-for`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#886 (https://github.com/marang/riotbox/pull/886)`
- Merge commit: `0aeb96090c6dae369e9b6ab62b4854c2053f1183`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check && git diff --check`; `cargo test -p riotbox-app source_timing_cues`; `cargo test -p riotbox-app --bin observer_audio_correlate`; `just observer-audio-summary-validator-fixtures`; `just ci`; `GitHub Rust CI #2200 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-892 existed to remove a remaining Rust producer readiness cue/actionability phrase table from the app path and keep Jam/observer readiness language delegated to the shared Source Timing core helper.

## What Shipped

- App readiness cue/actionability labels now delegate to the shared core `source_timing_readiness_labels(...)` helper.
- Existing musician-facing wording and unknown fallback behavior remain unchanged.
- No ActionCommand, queue/session/replay state, JamAppState state, timing policy, lane behavior, or audio-producing behavior changed.

## Notes

- None
