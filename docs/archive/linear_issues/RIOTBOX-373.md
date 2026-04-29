# `RIOTBOX-373` Surface observer/audio evidence issues in summary output

- Ticket: `RIOTBOX-373`
- Title: `Surface observer/audio evidence issues in summary output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-373/surface-observeraudio-evidence-issues-in-summary-output`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-373-surface-observeraudio-evidence-issues-in-summary-output`
- Linear branch: `feature/riotbox-373-surface-observeraudio-evidence-issues-in-summary-output`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#361`
- Merge commit: `49062792d08d5d69dfd1f996aa3259d8a14b9ae7`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate`, `just ci`, `git diff --check`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-374`

## Why This Ticket Existed

Non-strict Markdown summaries needed to show the same output-evidence issue list so local review could notice weak evidence before strict mode failed.

## What Shipped

- Markdown summaries include `Output path issues`
- summary output reuses the strict output evidence failure classifier
- tests cover the happy-path `none` case and missing/collapsed metric details

## Notes

- this made local QA summaries more useful without making non-strict mode fatal
