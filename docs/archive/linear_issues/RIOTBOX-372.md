# `RIOTBOX-372` Report strict observer/audio metric evidence failures

- Ticket: `RIOTBOX-372`
- Title: `Report strict observer/audio metric evidence failures`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-372/report-strict-observeraudio-metric-evidence-failures`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-372-report-strict-observeraudio-metric-evidence-failures`
- Linear branch: `feature/riotbox-372-report-strict-observeraudio-metric-evidence-failures`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#360`
- Merge commit: `3507aa5454ff301d3d286594f000f0387f611f83`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate`, `just ci`, `git diff --check`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-373`

## Why This Ticket Existed

Strict observer/audio failures needed actionable diagnostics so a failed QA run named the missing or collapsed metric instead of only saying output evidence was missing.

## What Shipped

- strict failures report missing or collapsed metric names
- collapsed metric diagnostics include the active output metric floor
- tests cover missing metric names and collapsed metric floor messages

## Notes

- this improved operator and agent debugging without changing render behavior
