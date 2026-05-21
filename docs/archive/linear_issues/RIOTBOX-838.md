# `RIOTBOX-838` Split SourceTimingSummaryView tests out of the core summary module

- Ticket: `RIOTBOX-838`
- Title: `Split SourceTimingSummaryView tests out of the core summary module`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-838/split-sourcetimingsummaryview-tests-out-of-the-core-summary-module`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-838-source-timing-summary-tests-split`
- Linear branch: `feature/riotbox-838-split-sourcetimingsummaryview-tests-out-of-the-core-summary`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#833 (https://github.com/marang/riotbox/pull/833)`
- Merge commit: `170e4780d2fb2e510aaba4d2fa9fafbe61900eda`
- Verification: `cargo fmt --check`; `cargo test -p riotbox-core source_timing_summary -- --nocapture`; `just ci`; GitHub Actions Rust CI #2037.
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

The shared Source Timing summary module was over the repo's soft 500-line review budget after RIOTBOX-837, with inline tests dominating the file.

## What Shipped

- Moved SourceTimingSummaryView tests into crates/riotbox-core/src/view/jam/source_timing_summary_tests.rs while preserving behavior and keeping production/test modules below the review-size budget.

## Notes

- None
