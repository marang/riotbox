# `RIOTBOX-974` Add explicit source timing grid revert action

- Ticket: `RIOTBOX-974`
- Title: `Add explicit source timing grid revert action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-974/add-explicit-source-timing-grid-revert-action`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-974-add-explicit-source-timing-grid-revert-action`
- Linear branch: `feature/riotbox-974-add-explicit-source-timing-grid-revert-action`
- Assignee: `Markus`
- Labels: `Feature`, `timing`, `ux`
- PR: `#966 (https://github.com/marang/riotbox/pull/966)`
- Merge commit: `d61623dfe936eef2e0cfba0828e7085a131cab82`
- Deleted from Linear: `2026-05-26`
- Verification: `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-974-rebased-tests.log cargo test -p riotbox-app source_timing_confirm_control -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-974-rebased-core-tests.log cargo test -p riotbox-core plan_executor_applies_supported_structural_actions_in_commit_order -- --nocapture`; `scripts/run_compact.sh /tmp/riotbox-974-rebased-ci.log just ci`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Add a replayable counterpart to source_timing.confirm_grid so musician trust in a timing grid can be removed without mutating Source Graph analysis evidence.

## What Shipped

- Added source_timing.revert_grid / ActionCommand::SourceTimingRevertGrid.
- Wired revert through queueing, commit side effects, replay executor support, observer labels, and shell handling.
- Added the R key path and tests proving only matching Session runtime confirmed-grid trust is cleared.

## Notes

- Rebased onto current main after RIOTBOX-973 merged, retargeted PR #966 to main, then reran local focused tests and just ci.
