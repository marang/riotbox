# `RIOTBOX-353` Split TUI production and test files below 500-line budget

- Ticket: `RIOTBOX-353`
- Title: `Split TUI production and test files below 500-line budget`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-353/split-tui-production-and-test-files-below-500-line-budget`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-353-split-tui-files`
- Linear branch: `feature/riotbox-353-split-tui-production-and-test-files-below-500-line-budget`
- Assignee: `Markus`
- Labels: `review-followup`, `ux`, `workflow`
- PR: `#343`
- Merge commit: `64f99509143424e8e52e4b7c8d4cb27cd046b36b`
- Deleted from Linear: `2026-04-29`
- Verification: `cargo test -p riotbox-app`, `cargo test`, `just ci`, GitHub Actions `rust-ci`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-354`, `RIOTBOX-355`, `RIOTBOX-356`

## Why This Ticket Existed

The TUI implementation and snapshot tests were concentrated in large files, making UI work expensive to inspect and increasing the risk that small UX changes required loading thousands of unrelated lines.

## What Shipped

- replaced `crates/riotbox-app/src/ui.rs` with a small include index
- split TUI implementation by screen, panel, formatter, and rendering responsibility
- split TUI tests into focused snapshot and behavior files
- kept all `.rs` files below the repo line-budget target

## Notes

- this was a behavior-preserving organization slice
- the split prepared later musician-facing TUI work by reducing context cost
