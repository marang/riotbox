# `RIOTBOX-1518` Reject non-finite W-30 preview render durations before artifact writes

- Ticket: `RIOTBOX-1518`
- Title: `Reject non-finite W-30 preview render durations before artifact writes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1518/reject-non-finite-w-30-preview-render-durations-before-artifact-writes`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1518-finite-w30-duration`
- Linear branch: `feature/riotbox-1518-reject-non-finite-w-30-preview-render-durations-before`
- Assignee: `Markus`
- Labels: `Bug`, `review-followup`
- PR: `#1580 (https://github.com/marang/riotbox/pull/1580)`
- Merge commit: `eaa4862393f810df95bdadb9e520da57e767f1f1`
- Deleted from Linear: `2026-10-01`
- Verification: `Debug and actual Release: ten unit and two CLI integration tests pass; 40 invalid CLI paths; 13 previous CLI results and six finite-control file hashes unchanged; full source-free just ci warning/error-free; exact-head native run 36800653012 both jobs SUCCESS.`
- Docs touched: `docs/reviews/riotbox_1518_finite_w30_duration_2026-10-01.md`
- Follow-ups: `RIOTBOX-1509 remains open; finite-duration resource bounds and tiny positive rounding are outside this fix.`

## Why This Ticket Existed

W-30 QA accepted NaN duration and wrote a successful header-only WAV; positive infinity was accepted by the parser.

## What Shipped

- Reject non-finite duration before source hydration, render-buffer allocation or artifact writes; preserve positive finite behavior.

## Notes

- Solo sequential review and self-review, no remaining changed-diff finding. Maintenance only; no real sources, playback, devices, DAW or musical verdict. Archive PR gates and actual cleanup recorded separately.
