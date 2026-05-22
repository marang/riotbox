# `RIOTBOX-923` Extract W-30 preview label helpers from diagnostics shard

- Ticket: `RIOTBOX-923`
- Title: `Extract W-30 preview label helpers from diagnostics shard`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-923/extract-w-30-preview-label-helpers-from-diagnostics-shard`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-923-w30-preview-label-helpers`
- Linear branch: `feature/riotbox-923-extract-w-30-preview-label-helpers-from-diagnostics-shard`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#916 (https://github.com/marang/riotbox/pull/916)`
- Merge commit: `d1c85e27634d32dfd893e68666c17323e04e4cc6`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-923-rebased-just-ci.log just ci; GitHub Actions Rust CI run 26288696942 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-924 records the P015 TUI broad-review cadence findings`

## Why This Ticket Existed

P015 Productization Alpha needs W-30 preview/source-readiness label behavior separated from diagnostics line assembly before Log/Capture/footer guidance becomes expensive to review.

## What Shipped

- Extracted W-30 preview mode/profile labels, log labels, source suffix, and source-readiness helpers into a semantic child module without changing rendered behavior.

## Notes

- No ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
