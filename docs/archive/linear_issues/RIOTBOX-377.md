# `RIOTBOX-377` Add observer/audio summary JSON schema marker

- Ticket: `RIOTBOX-377`
- Title: `Add observer/audio summary JSON schema marker`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-377/add-observeraudio-summary-json-schema-marker`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-377-observer-audio-summary-json-schema`
- Linear branch: `feature/riotbox-377-add-observeraudio-summary-json-schema-marker`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#365`
- Merge commit: `40533ee8fbd164cb4953aba7d7be8523da65fc6d`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate`, `just observer-audio-correlate-json-fixture`, `just audio-qa-ci`, `cargo fmt --check`, `git diff --check`, `just ci`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-378`

## Why This Ticket Existed

The machine-readable observer/audio summary needed a stable schema marker so automation could cheaply reject an unexpected summary shape before making QA decisions.

## What Shipped

- added top-level `schema` and `schema_version` fields to `observer_audio_correlate --json`
- asserted the marker in unit tests and the committed fixture JSON smoke
- mirrored the stricter assertion in GitHub Actions
- documented the schema marker in the audio QA workflow spec

## Notes

- this did not add a full JSON Schema file or change audio behavior
