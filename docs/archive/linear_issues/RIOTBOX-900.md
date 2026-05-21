# `RIOTBOX-900` Preserve Source Timing phrase counts in Feral-grid manifests

- Ticket: `RIOTBOX-900`
- Title: `Preserve Source Timing phrase counts in Feral-grid manifests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-900/preserve-source-timing-phrase-counts-in-feral-grid-manifests`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-900-feral-grid-phrase-counts`
- Linear branch: `feature/riotbox-900-preserve-source-timing-phrase-counts-in-feral-grid-manifests`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#893 (https://github.com/marang/riotbox/pull/893)`
- Merge commit: `80ce563fcb6026e009be4ce4baade15a0908e09a`
- Deleted from Linear: `2026-05-21`
- Verification: `cargo fmt --check; python3 -m py_compile scripts/validate_listening_manifest_json.py; just listening-manifest-validator-fixtures; cargo test -p riotbox-audio --bin feral_grid_pack; just p012-all-lane-source-grid-output-proof; git diff --check; just ci; GitHub Rust CI #2222`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Generated Feral-grid output QA could see phrase status but not the phrase-count evidence behind short-loop/manual-confirm source timing.

## What Shipped

- Added primary phrase count and phrase bar count to generated Feral-grid source_timing manifests.
- Added the same counts to Feral-grid README and grid-report phrase lines.
- Required the fields in listening-manifest validation and updated manifest fixtures.
- Updated the Source Timing spec with the generated Feral-grid manifest phrase-evidence contract.

## Notes

- None
