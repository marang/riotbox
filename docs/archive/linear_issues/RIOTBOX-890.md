# `RIOTBOX-890` Expose Source Timing anchor cue in observer snapshots

- Ticket: `RIOTBOX-890`
- Title: `Expose Source Timing anchor cue in observer snapshots`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-890/expose-source-timing-anchor-cue-in-observer-snapshots`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-890-source-timing-anchor-cue-observer`
- Linear branch: `feature/riotbox-890-expose-source-timing-anchor-cue-in-observer-snapshots`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`, `ux`
- PR: `#884 (https://github.com/marang/riotbox/pull/884)`
- Merge commit: `d14d1b06d51c1fc5277a0f96f9d0cb7038bc2dcc`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; git diff --check; python3 -m py_compile scripts/validate_observer_audio_summary_json.py; cargo test -p riotbox-app --bin observer_audio_correlate; cargo test -p riotbox-app --bin riotbox-app source_timing_observer; just observer-audio-summary-validator-fixtures; just observer-audio-correlate-locked-grid-json-fixture; just ci before final rebase; focused gates repeated after final rebase; GitHub Rust CI #2194 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Expose the existing musician-facing Source Timing anchor cue in observer and observer/audio QA surfaces so reviewers can read kick/backbeat vs transient-only evidence without decoding raw counts.

## What Shipped

- Added primary_anchor_cue to app observer source_timing snapshots, observer/audio correlate parsing, Markdown and JSON rendering, JSON validation, and valid observer/audio fixtures/tests.

## Notes

- No timing policy, lane behavior, ActionCommand, Session/replay, JamAppState, or audio-producing behavior changed.
