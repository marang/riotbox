# `RIOTBOX-737` Reject invalid source timing fixture labels in Rust report path

- Ticket: `RIOTBOX-737`
- Title: `Reject invalid source timing fixture labels in Rust report path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-737`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `P012 | Source Timing Intelligence`
- Status: `Done`
- Created: `2026-05-10`
- Started: `2026-05-10`
- Finished: `2026-05-10`
- Deleted from Linear: `2026-05-10`
- Branch: `Not archived in grouped source row`
- Assignee: `Markus`
- Labels: `timing, review-followup, benchmark`
- PR: `#730 (https://github.com/marang/riotbox/pull/730)`
- Merge commit: `858963e`
- Verification: `Merged PR and repository CI/review gate for the shipped slice when PR metadata was present in the grouped source row.`
- Follow-ups: `Tracked by current roadmap/backlog where needed.`

## Why This Ticket Existed

This ticket represented a bounded Riotbox roadmap/workflow slice in `P012 | Source Timing Intelligence` and was completed before Linear cleanup.

## What Shipped

- Source timing fixture conversion now rejects unknown timing quality, degraded policy, warning, and alternative labels instead of silently falling back; verified with focused core tests, validator fixtures, `just ci`, and GitHub CI.

## Notes

- Split from the former grouped May 2026 archive so each archived Linear issue has one canonical file.
