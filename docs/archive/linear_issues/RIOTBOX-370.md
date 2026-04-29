# `RIOTBOX-370` Use manifest envelope validation in observer/audio strict mode

- Ticket: `RIOTBOX-370`
- Title: `Use manifest envelope validation in observer/audio strict mode`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-370/use-manifest-envelope-validation-in-observeraudio-strict-mode`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-370-use-manifest-envelope-validation-in-observeraudio-strict`
- Linear branch: `feature/riotbox-370-use-manifest-envelope-validation-in-observeraudio-strict`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#358`
- Merge commit: `66a4f650362bd2313ce0a2d9bf90a439f3f49e60`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate`, `just ci`, `git diff --check`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-371`

## Why This Ticket Existed

Strict observer/audio correlation needed to validate the shared manifest envelope before treating pack-specific output metrics as acceptable evidence.

## What Shipped

- strict mode validates the manifest v1 envelope first
- non-strict local summary mode remains tolerant for inspection
- committed observer/audio fixture manifest gained stable artifact `kind` fields

## Notes

- this connected the new manifest validator to the observer/audio control-plus-output proof path
