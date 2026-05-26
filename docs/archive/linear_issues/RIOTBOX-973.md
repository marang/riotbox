# `RIOTBOX-973` Expose source timing grid confirmation in Jam and Source controls

- Ticket: `RIOTBOX-973`
- Title: `Expose source timing grid confirmation in Jam and Source controls`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-973/expose-source-timing-grid-confirmation-in-jam-and-source-controls`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-973-expose-source-timing-grid-confirmation-in-jam-and-source`
- Linear branch: `feature/riotbox-973-expose-source-timing-grid-confirmation-in-jam-and-source`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#965 (https://github.com/marang/riotbox/pull/965)`
- Merge commit: `2ceb656374bd8e186cf81ddab34490d48ed261ec`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `cargo test -p riotbox-app source_timing_confirm_control -- --nocapture`; `cargo test -p riotbox-app renders_source_shell_snapshot_with_user_confirmed_grid -- --nocapture`; `cargo test -p riotbox-app source_timing_readiness_shows_user_confirmed_grid -- --nocapture`; `cargo test -p riotbox-app`; `scripts/run_compact.sh /tmp/riotbox-973-just-ci-final.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Make the source timing grid confirmation contract musician-operable from Jam and Source after audition, while preserving Source Graph analysis evidence as immutable analysis truth.

## What Shipped

- Added the C confirmation path for the current source timing grid in Jam/Source.
- Committed source_timing.confirm_grid immediately and surfaced clear confirmed/already pending/unavailable user status.
- Rendered user-confirmed grid state in Jam and Source timing surfaces without mutating analyzer evidence.

## Notes

- No audio rendering behavior changed; existing CI/audio gates stayed green.
