# `RIOTBOX-357` Consume Feral readiness in W-30 slice-pool targeting with output proof

- Ticket: `RIOTBOX-357`
- Title: `Consume Feral readiness in W-30 slice-pool targeting with output proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-357/consume-feral-readiness-in-w-30-slice-pool-targeting-with-output-proof`
- Project: `P009 | Feral Policy Layer`
- Milestone: `P009 | Feral Policy Layer`
- Status: `Duplicate`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Deleted from Linear: `2026-04-30`
- Branch: `feature/riotbox-357-consume-feral-readiness-in-w-30-slice-pool-targeting-with`
- Linear branch: `feature/riotbox-357-consume-feral-readiness-in-w-30-slice-pool-targeting-with`
- Assignee: `Markus`
- Labels: `benchmark`, `Audio`
- PR: `None`
- Merge commit: `None`
- Verification: `Existing implementation/test references recorded in the duplicate closure note.`
- Follow-ups: `None`

## Why This Ticket Existed

Closed as duplicate/superseded by existing P009 work.

The requested bounded W-30 Feral consumer already exists:

* `JamAppState::next_w30_slice_pool_capture` prefers Feral-ready capture candidates when `SourceGraph::has_feral_break_support_evidence()` is true.
* `queue_w30_browse_slice_pool_prefers_feral_capture_and_changes_preview_window` proves control-path target selection (`cap-03` vs `cap-02`) and source-window preview output delta.
* UI tests surface the `feral` slice-pool reason in Capture/Jam surfaces.

No new code slice needed.

## What Shipped

- No new code shipped for this ticket; existing implementation and tests already covered the requested scope.

## Notes

- Closed as duplicate/superseded; the closure note above names the existing implementation/test coverage.
