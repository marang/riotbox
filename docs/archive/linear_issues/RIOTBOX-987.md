# `RIOTBOX-987` Prove source seek changes audible monitored excerpt

- Ticket: `RIOTBOX-987`
- Title: `Prove source seek changes audible monitored excerpt`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-987/prove-source-seek-changes-audible-monitored-excerpt`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-987-prove-source-seek-changes-audible-monitored-excerpt`
- Linear branch: `feature/riotbox-987-prove-source-seek-changes-audible-monitored-excerpt`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#979 (https://github.com/marang/riotbox/pull/979)`
- Merge commit: `31cc3259d76dfb96493c47eedbe00b5cdba3a853`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-987-rebased-audio.log cargo test -p riotbox-audio source_monitor_seeked_running_transport_changes_audible_source_excerpt -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-987-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Prove Source Map navigation affects the audible monitored source excerpt, not only UI state or transport.seek logs.

## What Shipped

- Added an offline source monitor output proof for running transport before/after seek from beat 0 to beat 16.
- Rendered through render_source_monitor_mix_offline using the same source monitor policy consumed by the realtime callback.
- Asserted both excerpts are non-silent and have a significant RMS delta so the path cannot collapse to the same window.

## Notes

- Audio-proof slice; no realtime file I/O or analysis work introduced.
