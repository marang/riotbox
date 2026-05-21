# `RIOTBOX-860` Document point-in-time review follow-up handling

- Ticket: `RIOTBOX-860`
- Title: `Document point-in-time review follow-up handling`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-860/document-point-in-time-review-follow-up-handling`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-860-review-followup-freshness-docs`
- Linear branch: `feature/riotbox-860-document-point-in-time-review-follow-up-handling`
- Assignee: `Markus`
- Labels: `review-followup`, `workflow`
- PR: `#854 (https://github.com/marang/riotbox/pull/854)`
- Merge commit: `f4499b6162f1c9a47f0a2c08b70b20cd09b308a9`
- Verification: `git diff --check; search checks for point-in-time/live backlog/already shipped/duplicate/superseded; GitHub Actions Rust CI passed for PR #854; main synced after merge.`
- Docs touched: `docs/reviews/README.md, docs/workflow_conventions.md, docs/README.md`
- Follow-ups: `None`

## Why This Ticket Existed

Older review findings can become stale after later tickets ship; without an explicit freshness rule, autonomous continuation can create duplicate tickets from point-in-time review artifacts.

## What Shipped

- Added docs/reviews/README.md; documented that review artifacts are point-in-time evidence, not live backlog truth; added workflow guidance to verify older review findings against current main, Linear, and archives before creating tickets; linked the rule from docs/README.md.

## Notes

- None
