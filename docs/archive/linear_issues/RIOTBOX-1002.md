# `RIOTBOX-1002` Add local example audio listening analysis report

- Ticket: `RIOTBOX-1002`
- Title: `Add local example audio listening analysis report`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1002/add-local-example-audio-listening-analysis-report`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-1002-add-local-example-audio-listening-analysis-report`
- Linear branch: `feature/riotbox-1002-add-local-example-audio-listening-analysis-report`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`
- PR: None
- Merge commit: `a00eaf07`
- Deleted from Linear: `2026-05-27`
- Verification: `just source-timing-example-probe-report-fixtures (passed 2026-05-27)`; `just source-timing-example-probe-report-local /tmp/riotbox-next-source-timing-report.md (passed 2026-05-27)`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The P012 real-source timing work needed a lightweight local report that combines example-WAV Source Timing probe output with basic audio descriptors, so reviewers can inspect local example behavior without dumping full WAV metadata into chat or mistaking fixture smokes for real-source evidence.

## What Shipped

- Added compact audio descriptor rows to the local Source Timing example probe report path.
- Kept missing local WAVs as skipped rows so fresh clones are not blocked by untracked example audio.
- Documented the optional local example report behavior in the Source Timing Intelligence spec.

## Notes

- No analyzer, Session, UI, or audio-render behavior changed; this was a local QA/reporting surface.
