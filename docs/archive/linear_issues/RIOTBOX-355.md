# `RIOTBOX-355` Split remaining Rust files above 500-line budget

- Ticket: `RIOTBOX-355`
- Title: `Split remaining Rust files above 500-line budget`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-355/split-remaining-rust-files-above-500-line-budget`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-355-split-remaining-rust-hotspots`
- Linear branch: `feature/riotbox-355-split-remaining-rust-files-above-500-line-budget`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#345`
- Merge commit: `8413612c5a37ca723b1ae1a6af1242ea14158b4b`
- Deleted from Linear: `2026-04-29`
- Verification: `cargo fmt --check`, `git diff --check`, `cargo test`, `just ci`, GitHub Actions `rust-ci`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-356`, `RIOTBOX-360`

## Why This Ticket Existed

After the focused Jam app, TUI, and runtime splits, several Rust hotspots still exceeded the repo's 500-line budget. The remaining safe candidates needed mechanical splitting before feature work continued to pile onto them.

## What Shipped

- split the remaining safe Rust production, test, and bin-helper hotspots into smaller include shards
- kept behavior equivalent to `main` with reconstruction and full test checks
- left app orchestration hotspots for a separate semantic extraction slice

## Notes

- the first pass used mechanical shard boundaries to minimize behavior-change risk
- RIOTBOX-360 later replaced durable numbered shard names with semantic responsibility names
