# `RIOTBOX-1077` Split export QA tests before next gate expansion

- Ticket: `RIOTBOX-1077`
- Title: `Split export QA tests before next gate expansion`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1077/split-export-qa-tests-before-next-gate-expansion`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1077-export-qa-test-split`
- Linear branch: `feature/riotbox-1077-split-export-qa-tests-before-next-gate-expansion`
- Assignee: `Markus`
- Labels: None
- PR: `#1053 (https://github.com/marang/riotbox/pull/1053)`
- Merge commit: `909a75176ac0f6f3defac7b3ca15782ce951126a`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core export_qa -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1053`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The next export QA gate expansion needs review headroom before adding fallback-collapse or lineage behavior.

## What Shipped

- Moved stem-package export QA tests into a semantic test module.
- Kept production API and behavior unchanged.
- Reduced export_qa.rs to 235 lines and avoided numbered shards.

## Notes

- No new export feature behavior, stem writing, DAW export, live recording export, or QA enforcement shipped.
