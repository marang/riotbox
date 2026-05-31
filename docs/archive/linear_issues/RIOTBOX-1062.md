# `RIOTBOX-1062` Surface export readiness without claiming full export polish

- Ticket: `RIOTBOX-1062`
- Title: `Surface export readiness without claiming full export polish`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1062/surface-export-readiness-without-claiming-full-export-polish`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1062-export-readiness-surface`
- Linear branch: `feature/riotbox-1062-surface-export-readiness-without-claiming-full-export-polish`
- Assignee: `Markus`
- Labels: None
- PR: `#1038 (https://github.com/marang/riotbox/pull/1038)`
- Merge commit: `4f777a29e202ff62d089ba5621564e754ec9e8a9`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-app export_readiness -- --nocapture: pass`; `cargo test -p riotbox-app renders_jam_shell_inspect_snapshot -- --nocapture: pass`; `git diff --check: pass`; `just product-export-reproducibility-smoke: pass`; `just ci: pass`; `GitHub rust-ci on PR #1038: pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

After the export-readiness contract existed, musicians needed an inspectable surface that said what export proof exists and what remains unsupported.

## What Shipped

- Added export-readiness lines to Jam Inspect Material flow.
- Used riotbox-core::export_readiness types for role, boundary, and unsupported-scope list.
- Kept export readiness out of Jam Perform so it is not presented as a play control.
- Added focused UI snapshot coverage for inspect visibility and perform absence.
- Updated the existing inspect snapshot assertion.

## Notes

- None
