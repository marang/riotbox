# `RIOTBOX-918` Extract Jam footer renderer from screen body shard

- Ticket: `RIOTBOX-918`
- Title: `Extract Jam footer renderer from screen body shard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-918/extract-jam-footer-renderer-from-screen-body-shard`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-918-footer-renderer-module`
- Linear branch: `feature/riotbox-918-extract-jam-footer-renderer-from-screen-body-shard`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#911 (https://github.com/marang/riotbox/pull/911)`
- Merge commit: `eea5502dc04ffca4a832e42654ee107b35e75ad1`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-918-just-ci.log just ci; GitHub Actions Rust CI run 26286531483 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-919 continues P015 UI test ownership split work`

## Why This Ticket Existed

P015 Productization Alpha needs persistent Jam footer controls separated from screen body renderers so musician-facing control cues remain easier to evolve.

## What Shipped

- Extracted render_footer and footer gesture line helpers into footer_renderer while preserving rendered footer behavior and existing footer styling tests.

## Notes

- No ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
