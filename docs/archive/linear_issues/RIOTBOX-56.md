# `RIOTBOX-56` Ticket Archive

- Ticket: `RIOTBOX-56`
- Title: `Add live drum-bus level control to make TR-909 render audibly testable`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-56/add-live-drum-bus-level-control-to-make-tr-909-render-audibly-testable`
- Project: `Riotbox MVP Buildout`
- Milestone: `Jam-First Playable Slice`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-56-drum-bus-level-control`
- Linear branch: `feature/riotbox-56-add-live-drum-bus-level-control-to-make-tr-909-render`
- Assignee: `Markus`
- Labels: `None`
- PR: `#47`
- Merge commit: `246ac39`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The shipped TR-909 render seam could already be technically running while staying silent because the drum bus still sat at zero. Riotbox needed one bounded operator control that made the existing seam audibly testable by ear without widening scope into a separate mixer page or a second callback-side control path.

## What Shipped

- Added bounded live drum-bus up/down control on the existing persisted `mixer_state.drum_level` seam.
- Surfaced the control in Jam-shell key handling, footer help, and runtime mix summaries.
- Kept the TR-909 render warnings tied to the same app-derived mix summary so the zero-level silent-state remains visible.
- Added app and UI test coverage for drum-bus adjustment and the updated shell affordances.

## Notes

- This slice intentionally stopped at one bounded live drum-bus control and did not open a full mixer surface.
- Deeper mixer work should continue on the same session/runtime seam rather than bypassing it with callback-only state or a second UI path.
