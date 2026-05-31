# `RIOTBOX-1079` Add structural export QA gate for fallback-collapse evidence

- Ticket: `RIOTBOX-1079`
- Title: `Add structural export QA gate for fallback-collapse evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1079/add-structural-export-qa-gate-for-fallback-collapse-evidence`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1079-export-qa-fallback-evidence`
- Linear branch: `feature/riotbox-1079-add-structural-export-qa-gate-for-fallback-collapse-evidence`
- Assignee: `Markus`
- Labels: None
- PR: `#1055 (https://github.com/marang/riotbox/pull/1055)`
- Merge commit: `5c95623d841015914e8a12b350ab2eb80a486e04`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core session::export_types -- --nocapture`; `cargo test -p riotbox-core export_qa -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1055`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Future stem QA needs a typed source-vs-fallback evidence slot before it can reject fallback-collapsed exports honestly.

## What Shipped

- Added optional fallback_comparison evidence to export artifact-set entries.
- Added an opt-in stem QA policy requiring fallback comparison evidence.
- Added MissingFallbackComparisonEvidence failures when the policy is enabled.
- Preserved product-mix compatibility and older artifact entry defaults.

## Notes

- No stem writing, DAW export, live recording export, actual source-vs-fallback rendering, or threshold interpretation shipped.
