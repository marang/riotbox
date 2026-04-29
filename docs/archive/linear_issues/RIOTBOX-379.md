# `RIOTBOX-379` Document observer/audio JSON summary contract

- Ticket: `RIOTBOX-379`
- Title: `Document observer/audio JSON summary contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-379/document-observeraudio-json-summary-contract`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-379-observer-audio-json-contract`
- Linear branch: `feature/riotbox-379-document-observeraudio-json-summary-contract`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#368`
- Merge commit: `34c7602c9b94f1cd00823cd230b7f64934832b7f`
- Deleted from Linear: `Not deleted yet`
- Verification: `git diff --check`, docs-only review
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`, `docs/benchmarks/README.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-380`

## Why This Ticket Existed

The observer/audio JSON summary path had a schema marker and CI smoke coverage, but the minimal contract still lived only in implementation details.

## What Shipped

- added the `riotbox.observer_audio_summary.v1` JSON contract note
- documented stable top-level, control-path, and output-path fields
- documented compatibility rules and current CI smoke requirements
- linked the contract from the benchmark index and audio QA workflow spec

## Notes

- this was documentation-only and did not change output shape or audio behavior
