# `RIOTBOX-1078` Add structural export QA gate for stem lineage evidence

- Ticket: `RIOTBOX-1078`
- Title: `Add structural export QA gate for stem lineage evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1078/add-structural-export-qa-gate-for-stem-lineage-evidence`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1078-export-qa-lineage-gate`
- Linear branch: `feature/riotbox-1078-add-structural-export-qa-gate-for-stem-lineage-evidence`
- Assignee: `Markus`
- Labels: None
- PR: `#1054 (https://github.com/marang/riotbox/pull/1054)`
- Merge commit: `b54eb52513b22db439a2e65660d63325f529d321`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core export_qa -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1054`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Future stem export claims need an opt-in structural gate for source/capture lineage before real stem writing lands.

## What Shipped

- Added an opt-in stem artifact-set QA policy requiring lineage evidence.
- Added MissingLineageEvidence failures for claimed stems without source graph, source capture, or capture-lineage refs when enabled.
- Preserved default behavior for current product-mix callers.
- Added focused core tests and documented the policy.

## Notes

- No stem writing, DAW export, live recording export, or audio fallback-collapse comparison shipped.
