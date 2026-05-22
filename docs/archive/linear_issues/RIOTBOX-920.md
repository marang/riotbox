# `RIOTBOX-920` Extract W-30 cue label helpers from capture/log source shard

- Ticket: `RIOTBOX-920`
- Title: `Extract W-30 cue label helpers from capture/log source shard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-920/extract-w-30-cue-label-helpers-from-capturelog-source-shard`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-920-w30-cue-label-helpers`
- Linear branch: `feature/riotbox-920-extract-w-30-cue-label-helpers-from-capturelog-source-shard`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#913 (https://github.com/marang/riotbox/pull/913)`
- Merge commit: `5b9f02885487bcd98a305eca6e6bc303f825bc8d`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-920-just-ci.log just ci; GitHub Actions Rust CI run 26287319299 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-921 continues P015 TUI module ownership work`

## Why This Ticket Existed

P015 Productization Alpha needs W-30 capture/log/source cue labels separated from mixed list rendering before musician-facing diagnostics become expensive to review.

## What Shipped

- Extracted W-30 pending cue labels, committed W-30 action lookup, and compact W-30 action labels into a semantic child module without changing UI copy or behavior.

## Notes

- No ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
