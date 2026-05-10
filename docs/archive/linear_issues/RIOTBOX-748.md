# `RIOTBOX-748` Add core MC-202 role and phrase intent conversion helpers

- Ticket: `RIOTBOX-748`
- Title: `Add core MC-202 role and phrase intent conversion helpers`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-748`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Deleted from Linear: `2026-05-10`
- Branch: `Not archived in grouped source row`
- Assignee: `Markus`
- Labels: `review-followup`
- PR: `#740 (https://github.com/marang/riotbox/pull/740)`
- Merge commit: `9eecf29`
- Verification: `Merged PR and repository CI/review gate for the shipped slice when PR metadata was present in the grouped source row.`
- Follow-ups: `Tracked by current roadmap/backlog where needed.`

## Why This Ticket Existed

This ticket represented a bounded Riotbox roadmap/workflow slice in `P000 | Repo Ops / QA / Workflow` and was completed before Linear cleanup.

## What Shipped

- Added `Mc202RoleState` and `Mc202PhraseIntentState` helper boundaries, moved MC-202 session lane types into `session/mc202_types.rs`, and preserved existing session JSON shape; verified with core MC-202 tests, `just ci`, and GitHub CI.

## Notes

- Split from the former grouped May 2026 archive so each archived Linear issue has one canonical file.
