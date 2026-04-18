# `RIOTBOX-24` Ticket Archive

- Ticket: `RIOTBOX-24`
- Title: `Fix whole-codebase review findings before the next feature slice`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-24/fix-whole-codebase-review-findings-before-the-next-feature-slice`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-12`
- Finished: `2026-04-12`
- Branch: `riotbox-24-review-findings-hardening`
- Assignee: `Markus`
- Labels: `Infra`, `TUI`, `Core`
- PR: `#18`
- Merge commit: `60bb0fd`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/reviews/whole_codebase_review_2026-04-13.md`
- Follow-ups: `RIOTBOX-25`

## Why This Ticket Existed

The whole-codebase review found four concrete issues that were significant enough to fix before continuing feature work.

## What Shipped

- Guaranteed terminal cleanup on early TUI startup failure.
- Made app load/save honor embedded versus external graph-storage behavior.
- Added sidecar request-id validation in the Rust client.
- Stopped transport reload from silently zeroing bar and phrase context.
- Added targeted regression tests for the new behavior.

## Notes

- This was a deliberate hardening pass, not a feature slice.
- It closed the first comprehensive review loop before Jam feature work resumed.
