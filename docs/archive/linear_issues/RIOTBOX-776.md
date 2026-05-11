# `RIOTBOX-776` Share Source Timing grid-use policy helper

- Ticket: `RIOTBOX-776`
- Title: `Share Source Timing grid-use policy helper`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-776/share-source-timing-grid-use-policy-helper`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-776-share-source-timing-grid-use-policy-helper`
- Linear branch: `feature/riotbox-776-share-source-timing-grid-use-policy-helper`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#770 (https://github.com/marang/riotbox/pull/770)`
- Merge commit: `190acb793f90bb2b3d687a08fc08ed43cfc63491`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo fmt --check; cargo check -p riotbox-audio --bin source_timing_probe --bin feral_grid_pack; cargo test -p riotbox-core source_timing_grid_use -- --nocapture; cargo test -p riotbox-core source_timing_probe_readiness -- --nocapture; cargo test -p riotbox-audio source_timing_grid_use -- --nocapture; cargo test -p riotbox-audio --bin source_timing_probe --bin feral_grid_pack; just source-timing-readiness-report; just generated-source-timing-probe-json-smoke; just observer-audio-correlate-generated-feral-grid; git diff --check; just ci`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-777 adds JSON validator parity fixtures for the shared grid-use contract.`

## Why This Ticket Existed

P012 grid_use classification needed one Rust policy truth so source_timing_probe and generated Feral grid manifests cannot silently diverge.

## What Shipped

- Added a typed SourceTimingGridUse core helper with stable public labels, reused it from source_timing_probe and feral_grid_pack, reused the cautious source-timing BPM predicate, added contract tests, and documented the shared Rust producer rule.

## Notes

- None
