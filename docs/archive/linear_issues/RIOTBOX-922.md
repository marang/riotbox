# `RIOTBOX-922` Extract W-30 slice pool helpers from capture source shard

- Ticket: `RIOTBOX-922`
- Title: `Extract W-30 slice pool helpers from capture source shard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-922/extract-w-30-slice-pool-helpers-from-capture-source-shard`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-922-w30-slice-pool-helpers`
- Linear branch: `feature/riotbox-922-extract-w-30-slice-pool-helpers-from-capture-source-shard`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#915 (https://github.com/marang/riotbox/pull/915)`
- Merge commit: `dc26e8293ffd2e58a3e5bc59cf4eaecad40c1973`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-922-rebased-just-ci.log just ci; GitHub Actions Rust CI run 26288230630 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-923 continues P015 TUI module ownership work`

## Why This Ticket Existed

P015 Productization Alpha needs W-30 slice-pool collection and labels separated from mixed capture/source helpers before Capture and Log guidance becomes expensive to review.

## What Shipped

- Extracted W-30 slice-pool collection, current-position lookup, relevance, compact labels, and log labels into a semantic child module without changing rendered behavior.

## Notes

- No ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
