# `RIOTBOX-876` Surface Source Timing actionability in observer/audio correlation

- Ticket: `RIOTBOX-876`
- Title: `Surface Source Timing actionability in observer/audio correlation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-876/surface-source-timing-actionability-in-observeraudio-correlation`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-876-surface-source-timing-actionability-in-observeraudio-correlation`
- Linear branch: `feature/riotbox-876-surface-source-timing-actionability-in-observeraudio`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`, `ux`
- PR: `#870 (https://github.com/marang/riotbox/pull/870)`
- Merge commit: `7c50ea1050886cbd5e5e94fe2015b7d78d89e595`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo test -p riotbox-app source_timing_cues; cargo test -p riotbox-app --bin observer_audio_correlate observer_source_timing; cargo test -p riotbox-app --bin observer_audio_correlate summary_smoke; cargo test -p riotbox-app --bin observer_audio_correlate; cargo test -p riotbox-app source_timing_observer; git diff --check; just ci; GitHub Rust CI success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-874/875 made Source Timing actionability visible in Jam/Source/Help/observer snapshots, but observer/audio correlation still rendered cue/grid/quality/policy without the action phrase.

## What Shipped

- Parsed optional observer source_timing.actionability, validated it against degraded policy when present, rendered it in Markdown and JSON summaries, and added mismatch rejection/smoke coverage.

## Notes

- None
