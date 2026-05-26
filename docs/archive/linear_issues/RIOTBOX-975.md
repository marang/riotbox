# `RIOTBOX-975` Add source map audition navigator controls

- Ticket: `RIOTBOX-975`
- Title: `Add source map audition navigator controls`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-975/add-source-map-audition-navigator-controls`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-975-add-source-map-audition-navigator-controls`
- Linear branch: `feature/riotbox-975-add-source-map-audition-navigator-controls`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#969 (https://github.com/marang/riotbox/pull/969)`
- Merge commit: `b863461d981432928f50bb07ea626c29eb5a158e`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-975-rebased-nav-tests.log cargo test -p riotbox-app source_map_navigation -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-975-rebased-ui-tests.log cargo test -p riotbox-app source_map_navigation_control -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-975-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Let musicians move around analyzed source regions before capture or reuse decisions while keeping Riotbox in a live transport/capture model rather than an arbitrary sample editor.

## What Shipped

- Added source-map navigation controls for previous/next bar and phrase movement.
- Routed navigation through typed app behavior and transport.seek commit semantics.
- Surfaced playhead/current region information in the Source Map projection with focused event-loop, app, and render coverage.

## Notes

- Rebased onto current main after RIOTBOX-974 merged, retargeted PR #969 to main, then reran local focused tests and just ci.
