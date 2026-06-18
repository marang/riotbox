# `RIOTBOX-1290` Make professional-output QA recipes artifact-race resistant

- Ticket: `RIOTBOX-1290`
- Title: `Make professional-output QA recipes artifact-race resistant`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1290/make-professional-output-qa-recipes-artifact-race-resistant`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-18`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1290-artifact-race-resistant-qa`
- Linear branch: `feature/riotbox-1290-make-professional-output-qa-recipes-artifact-race-resistant`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1265 (https://github.com/marang/riotbox/pull/1265)`
- Merge commit: `98fa9087bcf7324d9328908d39560a6d86722d6e`
- Deleted from Linear: `2026-06-18`
- Verification: `Not recorded`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Prevent broad professional-output QA and CI runs from racing on shared local audio artifacts.

## What Shipped

- Added a repo-local broad-audio-QA lock wrapper; routed just audio-qa-ci through the lock; added lock fixtures to just ci; documented the exclusive broad-QA workflow and P023 roadmap note.

## Notes

- None
