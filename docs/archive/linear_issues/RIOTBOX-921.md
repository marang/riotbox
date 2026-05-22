# `RIOTBOX-921` Extract scene timing label helpers from timing rail shard

- Ticket: `RIOTBOX-921`
- Title: `Extract scene timing label helpers from timing rail shard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-921/extract-scene-timing-label-helpers-from-timing-rail-shard`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-921-scene-timing-label-helpers`
- Linear branch: `feature/riotbox-921-extract-scene-timing-label-helpers-from-timing-rail-shard`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#914 (https://github.com/marang/riotbox/pull/914)`
- Merge commit: `2e919ff3fc003c0d3c8660719ee0ed9916107870`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-921-rebased-just-ci.log just ci; GitHub Actions Rust CI run 26287816609 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-922 continues P015 TUI module ownership work`

## Why This Ticket Existed

P015 Productization Alpha needs scene timing labels separated from pending rail rendering before broader musician-facing Scene guidance becomes expensive to review.

## What Shipped

- Extracted scene compact labels, restore labels, energy deltas, scene-energy lookup, and quantization boundary labels into a semantic child module while keeping rail rendering unchanged.

## Notes

- No ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
