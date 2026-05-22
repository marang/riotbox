# `RIOTBOX-926` Split W-30 preview and source-readiness UI tests

- Ticket: `RIOTBOX-926`
- Title: `Split W-30 preview and source-readiness UI tests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-926/split-w-30-preview-and-source-readiness-ui-tests`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-926-w30-preview-source-readiness-tests`
- Linear branch: `feature/riotbox-926-split-w-30-preview-and-source-readiness-ui-tests`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#919 (https://github.com/marang/riotbox/pull/919)`
- Merge commit: `cfea758a84cec106cb9e2df48bb4a68b81de0b3f`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; git diff --check main...HEAD; scripts/run_compact.sh /tmp/riotbox-926-just-ci.log just ci; GitHub Actions Rust CI run 26289982133 passed`
- Docs touched: `docs/archive/linear_issues/RIOTBOX-926.md; docs/archive/linear_issues/2026-05.md; docs/archive/linear_issues/index.md`
- Follow-ups: `RIOTBOX-927 continues the P015 first-run capture routing helper extraction`

## Why This Ticket Existed

The P015 review identified W-30 preview/source-readiness tests as a cohesive group inside capture_w30_cues.rs.

## What Shipped

- Split W-30 source-window, raw audition preview/fallback, and source-backed preview label tests into w30_preview_source_readiness.rs while preserving existing test names.

## Notes

- Test ownership split only; no ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
