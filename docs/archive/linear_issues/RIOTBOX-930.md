# `RIOTBOX-930` Surface downbeat ambiguity in shared Source Timing summary

- Ticket: `RIOTBOX-930`
- Title: `Surface downbeat ambiguity in shared Source Timing summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-930/surface-downbeat-ambiguity-in-shared-source-timing-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-930-downbeat-ambiguity-summary`
- Linear branch: `feature/riotbox-930-surface-downbeat-ambiguity-in-shared-source-timing-summary`
- Assignee: `Markus`
- Labels: `review-followup`, `timing`, `ux`
- PR: `#923 (https://github.com/marang/riotbox/pull/923)`
- Merge commit: `b1c23eed3f812af491604bd3611c42c7d23a4b4f`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-core source_timing_summary; cargo test -p riotbox-app source_timing; cargo test -p riotbox-app shell_state; cargo test -p riotbox-app help_restore_cues; cargo test -p riotbox-app jam_scene_pending_onramp; cargo test -p riotbox-app --bin observer_audio_correlate; just ci; GitHub Actions Rust CI run 26291812237 passed`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md; docs/specs/tui_screen_spec.md`
- Follow-ups: `RIOTBOX-931 validates malformed observer ambiguity fields while keeping older fixtures compatible.`

## Why This Ticket Existed

Make Beat20-style downbeat ambiguity visible through the shared Source Timing summary instead of treating every needs-confirm source as generic.

## What Shipped

- Added compact downbeat score, score-gap, and alternate-phase evidence to SourceTimingSummaryView and surfaced it through Source, Help/Start-Here, observer snapshots, and observer/audio summaries.

## Notes

- No analyzer confidence, ActionCommand, Session, JamAppState, or audio-output behavior changed; Jam Trust remains compact to avoid layout wrapping.
