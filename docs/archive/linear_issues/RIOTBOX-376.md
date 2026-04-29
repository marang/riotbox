# `RIOTBOX-376` Add observer/audio JSON summary smoke to audio QA CI

- Ticket: `RIOTBOX-376`
- Title: `Add observer/audio JSON summary smoke to audio QA CI`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-376/add-observeraudio-json-summary-smoke-to-audio-qa-ci`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-376-observer-audio-json-ci-smoke`
- Linear branch: `feature/riotbox-376-add-observeraudio-json-summary-smoke-to-audio-qa-ci`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#364`
- Merge commit: `1d50642c47e654734baee3e6222c007bb6c20d73`
- Deleted from Linear: `Not deleted yet`
- Verification: `just observer-audio-correlate-json-fixture`, `just audio-qa-ci`, `cargo fmt --check`, `git diff origin/main --check`, `just ci`, Rust file line budget check
- Docs touched: `AGENTS.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-377`, `RIOTBOX-378`

## Why This Ticket Existed

The observer/audio JSON summary path needed to be part of the same CI-safe audio QA smoke gate as strict Markdown evidence so future automation would not silently drift.

## What Shipped

- added `just observer-audio-correlate-json-fixture`
- wired the JSON fixture smoke into `just audio-qa-ci`
- mirrored the JSON fixture assertion in GitHub Actions
- documented the CI-covered JSON summary path

## Notes

- the initial JSON smoke checked output-path evidence; `RIOTBOX-378` tightened it to require control-path evidence too
