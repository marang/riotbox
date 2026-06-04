# `RIOTBOX-1178` P016: Add live-recording readiness report just helper

- Ticket: `RIOTBOX-1178`
- Title: `P016: Add live-recording readiness report just helper`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1178/p016-add-live-recording-readiness-report-just-helper`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1178-p016-live-recording-readiness-report-just-helper`
- Linear branch: `feature/riotbox-1178-p016-add-live-recording-readiness-report-just-helper`
- Assignee: `Markus`
- Labels: None
- PR: `#1157 (https://github.com/marang/riotbox/pull/1157)`
- Merge commit: `d012bb5ff0f489096ef5713d0966010e84a9cde0`
- Deleted from Linear: `2026-06-04`
- Verification: `just --list | rg live-recording-readiness-report-smoke`; `just live-recording-readiness-report-smoke`; `git diff --check`; `just ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The live-recording readiness CLI existed, but it lacked the same discoverable repo-level smoke helper used by the other P016 operator-report surfaces. Operators needed a named command that proves the report remains read-only and correctly handles ready/blocked receipt evidence.

## What Shipped

- Added just live-recording-readiness-report-smoke as a discoverable helper for the built-binary smoke.
- Updated audio QA and Session specs to reference the helper and keep the read-only/no-capture boundary explicit.

## Notes

- PR #1157 merged after GitHub rust-ci passed.
