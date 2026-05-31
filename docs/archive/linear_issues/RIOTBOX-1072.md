# `RIOTBOX-1072` Add stem package export QA gate skeleton without writing stems

- Ticket: `RIOTBOX-1072`
- Title: `Add stem package export QA gate skeleton without writing stems`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1072/add-stem-package-export-qa-gate-skeleton-without-writing-stems`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1072-stem-package-qa-gate`
- Linear branch: `feature/riotbox-1072-add-stem-package-export-qa-gate-skeleton-without-writing`
- Assignee: `Markus`
- Labels: None
- PR: `#1047 (https://github.com/marang/riotbox/pull/1047)`
- Merge commit: `f3f4b018ee475a3c4f16bd6748a77a6ad260ccd1`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core stem_package -- --nocapture`; `cargo test -p riotbox-core export_receipt -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1047`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `Next P016 implementation slice should attach real per-stem audio metrics before claiming non-silence or fallback-collapse proof.`

## Why This Ticket Existed

Stem package export needs a CI-safe evidence gate before Riotbox can safely implement or claim stem delivery.

## What Shipped

- Added riotbox_core export_qa stem-package artifact-set QA report.
- Required claimed stem roles to be typed stem roles with exactly one artifact entry.
- Failed clearly for missing role artifacts, duplicate role artifacts, missing artifact locations, missing hashes, and non-stem role claims.
- Marked per-stem non-silence and fallback-collapse checks deferred until real per-stem audio metrics exist.
- Documented the current skeleton boundary in the Audio QA workflow spec.

## Notes

- No stem files, stem package export action, or stronger audio-output claim shipped in this ticket.
