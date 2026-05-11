# `RIOTBOX-773` Propagate Source Timing grid-use into Feral grid manifests

- Ticket: `RIOTBOX-773`
- Title: `Propagate Source Timing grid-use into Feral grid manifests`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-773/propagate-source-timing-grid-use-into-feral-grid-manifests`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-773-propagate-source-timing-grid-use-into-feral-grid-manifests`
- Linear branch: `feature/riotbox-773-propagate-source-timing-grid-use-into-feral-grid-manifests`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#767 (https://github.com/marang/riotbox/pull/767)`
- Merge commit: `48e346bcf577a59b624b92faa68dbb12239dee1f`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio --bin feral_grid_pack; just listening-manifest-validator-fixtures; just listening-manifest-validate-generated-packs; just observer-audio-correlate-generated-feral-grid; just ci`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-774 propagates manifest grid_use into observer/audio summaries.`

## Why This Ticket Existed

Feral grid output manifests needed to preserve Source Timing grid-use classification so QA can distinguish locked, cautious short-loop manual-confirm, manual-only, fallback, and unavailable timing evidence without re-inferring it.

## What Shipped

- Added source_timing.grid_use to generated Feral grid manifests, validated the field in listening manifest JSON, asserted generated short-loop and locked classifications, and documented the manifest evidence field.

## Notes

- None
