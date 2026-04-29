# `RIOTBOX-374` Add machine-readable observer/audio correlation summary output

- Ticket: `RIOTBOX-374`
- Title: `Add machine-readable observer/audio correlation summary output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-374/add-machine-readable-observeraudio-correlation-summary-output`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-374-add-machine-readable-observeraudio-correlation-summary`
- Linear branch: `feature/riotbox-374-add-machine-readable-observeraudio-correlation-summary`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#362`
- Merge commit: `90a74b540c07e4333019b1d540d4fa58efc91466`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate`, `just ci`, `git diff --check`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-375`

## Why This Ticket Existed

Observer/audio correlation summaries needed a machine-readable form so automation could inspect control-path verdicts, output-path verdicts, issue lists, and key metrics without parsing Markdown.

## What Shipped

- added opt-in `--json` output to `observer_audio_correlate`
- JSON includes control-path and output-path verdict data, issue list, pack id, manifest result, artifact count, and key metrics
- Markdown remains the default output

## Notes

- this introduced the machine-readable path but did not yet add a Justfile helper or CI smoke
