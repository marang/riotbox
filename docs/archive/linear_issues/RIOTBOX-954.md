# `RIOTBOX-954` Show bar-grid count in Source Timing panel

- Ticket: `RIOTBOX-954`
- Title: `Show bar-grid count in Source Timing panel`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-954/show-bar-grid-count-in-source-timing-panel`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-954-show-bar-grid-count-in-source-timing-panel`
- Linear branch: `feature/riotbox-954-show-bar-grid-count-in-source-timing-panel`
- Assignee: `Markus`
- Labels: None
- PR: `#947 (https://github.com/marang/riotbox/pull/947)`
- Merge commit: `514c682d9346f56bb1663ab776f1e5ca565e9fb8`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo test -p riotbox-app ui::tests::shell_state_source -- --nocapture`; `cargo fmt --check`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-954-just-ci.log just ci`; `GitHub Actions Rust CI run 26301902224 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 says Jam / Source surfaces should show beat, bar, phrase, timing quality, and degraded timing state, but the Source Timing panel only showed beat/downbeat/phrase and left bar-grid availability indirect.

## What Shipped

- Added explicit bars count evidence to the Source Timing panel grid-readiness line.
- Compacted the line to beat/phase wording so the Timing card keeps action and warning rows visible.
- Updated Source screen snapshot coverage for the manual-confirm timing case.

## Notes

- UI-rendering slice only; no analyzer, ActionCommand, queue, Session/replay, JamAppState, realtime audio, observer schema, or render behavior changed.
