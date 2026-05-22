# `RIOTBOX-919` Split post-commit and restore help UI tests

- Ticket: `RIOTBOX-919`
- Title: `Split post-commit and restore help UI tests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-919/split-post-commit-and-restore-help-ui-tests`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-919-split-post-commit-restore-tests`
- Linear branch: `feature/riotbox-919-split-post-commit-and-restore-help-ui-tests`
- Assignee: `Markus`
- Labels: `Improvement`, `TUI`, `review-followup`
- PR: `#912 (https://github.com/marang/riotbox/pull/912)`
- Merge commit: `0cbba3dd0463f219a96b67c1c9503c149c181ed1`
- Deleted from Linear: `2026-05-22`
- Verification: `cargo fmt --check; cargo test -p riotbox-app ui::tests; scripts/run_compact.sh /tmp/riotbox-919-just-ci.log just ci; GitHub Actions Rust CI run 26286933538 passed`
- Docs touched: `none`
- Follow-ups: `RIOTBOX-920 continues P015 TUI module ownership work`

## Why This Ticket Existed

P015 Productization Alpha needs mixed UI test shards split by surface before musician-facing guidance tests become expensive to review and extend.

## What Shipped

- Split post-commit scene cue tests from first-run/help/restore readiness tests while preserving test names and assertions.

## Notes

- No production code, ActionCommand, JamAppState, session/replay, queue, or audio-producing behavior changed.
