# `RIOTBOX-793` Add Feral-grid full-mix drift regression against lane aliasing

- Ticket: `RIOTBOX-793`
- Title: `Add Feral-grid full-mix drift regression against lane aliasing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-793/add-feral-grid-full-mix-drift-regression-against-lane-aliasing`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-793-full-mix-drift-regression`
- Linear branch: `feature/riotbox-793-add-feral-grid-full-mix-drift-regression-against-lane`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#788 (https://github.com/marang/riotbox/pull/788)`
- Merge commit: `8a93bf6af350d1e5b3953ce3c1faf852f8b7bc4e`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-793-feral-grid-pack-tests.log cargo test -p riotbox-audio --bin feral_grid_pack`; `scripts/run_compact.sh /tmp/riotbox-793-cargo-fmt-check.log cargo fmt --check`; `scripts/run_compact.sh /tmp/riotbox-793-audio-qa-ci.log just audio-qa-ci`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-792 corrected pack-level Feral-grid drift measurement; the regression proof was needed so metrics.source_grid_output_drift cannot silently alias a lane again.

## What Shipped

- Added a typed SourceGridAlignmentReport seam and a regression test proving pack-level source_grid_output_drift follows the generated-support mix even when TR-909 lane alignment passes.

## Notes

- No audio rendering behavior changed; this added regression coverage for QA measurement provenance.
