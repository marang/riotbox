# `RIOTBOX-351` Document 500-line Rust file budget for production and tests

- Ticket: `RIOTBOX-351`
- Title: `Document 500-line Rust file budget for production and tests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-351/document-500-line-rust-file-budget-for-production-and-tests`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-351-document-500-line-rust-file-budget`
- Linear branch: `feature/riotbox-351-document-500-line-rust-file-budget-for-production-and-tests`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#341`
- Merge commit: `58c1b639b73464503e163eca20a1cdd99a30816c`
- Deleted from Linear: `2026-04-29`
- Verification: `cargo fmt --check`, `cargo test`, `just ci`, GitHub Actions `rust-ci`
- Docs touched: `AGENTS.md`, `docs/workflow_conventions.md`, local `code-review` skill
- Follow-ups: `RIOTBOX-352`, `RIOTBOX-353`, `RIOTBOX-354`, `RIOTBOX-355`, `RIOTBOX-356`

## Why This Ticket Existed

The codebase had accumulated large Rust production, test, fixture, and bin-helper files that raised review cost and agent context cost. The repo needed an explicit rule that every `.rs` file over roughly 500 lines is a refactor candidate.

## What Shipped

- documented the 500-line Rust file budget in repo workflow guidance
- clarified that tests and bin helpers count, not only production modules
- aligned branch review expectations so future PR reviews flag large-file growth
- created the follow-up cleanup lane for existing over-budget files

## Notes

- this was a workflow/documentation slice only
- later cleanup tickets performed the actual Rust file splits
