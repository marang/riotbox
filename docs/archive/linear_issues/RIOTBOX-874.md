# `RIOTBOX-874` Move Source Timing actionability language into the shared Jam summary

- Ticket: `RIOTBOX-874`
- Title: `Move Source Timing actionability language into the shared Jam summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-874/move-source-timing-actionability-language-into-the-shared-jam-summary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-874-move-source-timing-actionability-language-into-shared-jam-summary`
- Linear branch: `feature/riotbox-874-move-source-timing-actionability-language-into-the-shared`
- Assignee: `Markus`
- Labels: `timing`, `ux`
- PR: `#868 (https://github.com/marang/riotbox/pull/868)`
- Merge commit: `ed3919e8b97c8b8b6889df2efe5fb8e7e82742e6`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo test -p riotbox-core source_timing_summary; cargo test -p riotbox-app shell_state_log_source; cargo test -p riotbox-app source_timing_observer; cargo test -p riotbox-app shell_state_jam_snapshot; cargo test -p riotbox-app post_commit_help_restore; cargo test -p riotbox-app jam_scene_pending_onramp; git diff --check; just ci; GitHub Rust CI success`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md; docs/specs/tui_screen_spec.md`
- Follow-ups: `Next P012 slice can either expose this actionability in a tighter Jam perform rail or begin translating trusted source timing into the next bounded all-lane gesture.`

## Why This Ticket Existed

Jam, Source, Help, and observer surfaces shared timing cue fields but the musician-facing action phrase was still assembled locally in app UI code.

## What Shipped

- Added shared Source Timing actionability to the Jam summary, used it in Jam Help and Source timing surfaces, included it in observer snapshots, and documented it as part of the shared presentation contract.

## Notes

- None
