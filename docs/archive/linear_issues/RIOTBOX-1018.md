# `RIOTBOX-1018` Reject contradictory observer source timing counts in Rust summary correlation

- Ticket: `RIOTBOX-1018`
- Title: `Reject contradictory observer source timing counts in Rust summary correlation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1018/reject-contradictory-observer-source-timing-counts-in-rust-summary`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-28`
- Started: `2026-05-28`
- Finished: `2026-05-28`
- Branch: `feature/riotbox-1018-reject-contradictory-observer-source-timing-counts-in-rust`
- Linear branch: `feature/riotbox-1018-reject-contradictory-observer-source-timing-counts-in-rust`
- Assignee: `Markus`
- Labels: None
- PR: `#1001 (https://github.com/marang/riotbox/pull/1001)`
- Merge commit: `50106133b7e8d0bf6ca566fedec4f7092a7b271e`
- Deleted from Linear: `2026-05-28`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate observer_source_timing; just observer-audio-summary-validator-fixtures; just ci; GitHub Rust CI 26584742567 success`
- Docs touched: `none`
- Follow-ups: `none`

## Why This Ticket Existed

The Rust observer/audio summary collector could accept contradictory observer Source Timing count evidence even though the P012 validator contract rejects those contradictions.

## What Shipped

- Added Rust-side observer Source Timing count consistency rejection for grid, bar-locked, phrase-locked, and non-locked phrase-count contradictions, with focused strict-evidence tests.

## Notes

- The existing malformed observer source timing path is reused; no new summary schema or Source Timing model state was introduced.
