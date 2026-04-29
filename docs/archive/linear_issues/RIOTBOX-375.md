# `RIOTBOX-375` Add Justfile helper for observer/audio JSON summaries

- Ticket: `RIOTBOX-375`
- Title: `Add Justfile helper for observer/audio JSON summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-375/add-justfile-helper-for-observeraudio-json-summaries`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-375-observer-audio-json-helper`
- Linear branch: `feature/riotbox-375-add-justfile-helper-for-observeraudio-json-summaries`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#363`
- Merge commit: `7866231642af7fbbdc85b5fbe6b0db49db5527a6`
- Deleted from Linear: `Not deleted yet`
- Verification: `just observer-audio-correlate-json`, `cargo test -p riotbox-app --bin observer_audio_correlate`, `just ci`, Rust file line budget check
- Docs touched: `AGENTS.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-376`

## Why This Ticket Existed

The new JSON summary output needed a stable repo-local helper so agents and humans could invoke the machine-readable observer/audio correlation path consistently.

## What Shipped

- added `just observer-audio-correlate-json`
- kept the existing Markdown correlation helper unchanged
- added the helper to the AGENTS command shortlist
- documented the helper in the audio QA workflow spec

## Notes

- this made the JSON path easy to invoke but did not yet make it part of CI
