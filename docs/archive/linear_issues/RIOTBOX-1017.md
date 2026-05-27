# `RIOTBOX-1017` P012: Reject silent MC-202 Feral-grid output in observer/audio strict evidence

- Ticket: `RIOTBOX-1017`
- Title: `P012: Reject silent MC-202 Feral-grid output in observer/audio strict evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1017/p012-reject-silent-mc-202-feral-grid-output-in-observeraudio-strict`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-27`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/riotbox-1017-p012-reject-silent-mc-202-feral-grid-output-in-observeraudio`
- Linear branch: `feature/riotbox-1017-p012-reject-silent-mc-202-feral-grid-output-in-observeraudio`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `timing`
- PR: `#1000 (https://github.com/marang/riotbox/pull/1000)`
- Merge commit: `c0ee216376393c9d63fb388d0309bf71fae8b2df`
- Deleted from Linear: `2026-05-27`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate; just observer-audio-correlate-fixture; cargo fmt --check; just audio-qa-ci; just ci; GitHub Rust CI run 26522650789 success`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio strict evidence still allowed the old silent MC-202 compatibility branch after RIOTBOX-1015 made MC-202 a required primitive source-grid proof lane.

## What Shipped

- Feral-grid strict evidence now rejects compatibility_silent MC-202 output, requires mc202_bass_pressure.applied=true, and always validates mc202_source_grid_alignment against the current hit-ratio and offset budget.

## Notes

- The previous allow test was inverted into a compatibility-silent rejection test; committed observer/audio fixtures now include primitive MC-202 proof metadata.
