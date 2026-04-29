# `RIOTBOX-378` Require control-path evidence in observer/audio JSON smoke

- Ticket: `RIOTBOX-378`
- Title: `Require control-path evidence in observer/audio JSON smoke`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-378/require-control-path-evidence-in-observeraudio-json-smoke`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-378-observer-audio-json-control-path-smoke`
- Linear branch: `feature/riotbox-378-require-control-path-evidence-in-observeraudio-json-smoke`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#366`
- Merge commit: `df4f6ba083706ca61c882b2e65532b0ec20ff6b4`
- Deleted from Linear: `Not deleted yet`
- Verification: `just observer-audio-correlate-json-fixture`, `just audio-qa-ci`, `cargo fmt --check`, `git diff --check`, `just ci`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The observer/audio JSON smoke already checked the summary schema marker and output-path verdict, but it also needed to enforce committed control-path evidence so the machine-readable path stayed aligned with Riotbox's control-plus-output proof rule.

## What Shipped

- required `control_path.present == true` in the committed observer/audio JSON fixture smoke
- mirrored the stricter assertion in the GitHub Actions audio QA step
- documented the machine-readable control-plus-output proof rule in the audio QA workflow spec

## Notes

- this was a CI/QA assertion slice only; no audio rendering behavior changed
