# `RIOTBOX-758` Handle archive generator network failures cleanly

- Ticket: `RIOTBOX-758`
- Title: `Handle archive generator network failures cleanly`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-758/handle-archive-generator-network-failures-cleanly`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Branch: `feature/riotbox-758-handle-archive-generator-network-failures-cleanly`
- Linear branch: `feature/riotbox-758-handle-archive-generator-network-failures-cleanly`
- Assignee: `Markus`
- Labels: `archive`, `workflow`
- PR: `#752 (https://github.com/marang/riotbox/pull/752)`
- Merge commit: `153792e2b9483a428f074ce962e7872d3e191f4d`
- Deleted from Linear: `2026-05-10`
- Verification: `scripts/archive_linear_issue_smoke.sh`; `git diff --check`; `GitHub Rust CI success on PR #752`
- Docs touched: `scripts/archive_linear_issue.py`, `scripts/archive_linear_issue_smoke.sh`
- Follow-ups: `None`

## Why This Ticket Existed

Real closeout use of RIOTBOX-757 showed that temporary Linear/GitHub transport failures produced a Python traceback from the archive generator instead of a clean workflow-helper error.

## What Shipped

- Caught urllib URL errors for Linear GraphQL and GitHub PR metadata requests.
- Returned clean archive_linear_issue request-failed messages without Python tracebacks.
- Extended the archive generator smoke test with an unreachable endpoint case.

## Notes

- None
