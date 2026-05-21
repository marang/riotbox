# `RIOTBOX-882` Require Source Timing cue and actionability in generated Feral-grid manifest validation

- Ticket: `RIOTBOX-882`
- Title: `Require Source Timing cue and actionability in generated Feral-grid manifest validation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-882/require-source-timing-cue-and-actionability-in-generated-feral-grid`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-882-require-source-timing-cue-actionability-manifest-validation`
- Linear branch: `feature/riotbox-882-require-source-timing-cue-and-actionability-in-generated`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#876 (https://github.com/marang/riotbox/pull/876)`
- Merge commit: `72d20e408f212b54a9032de90d077332488bd2ab`
- Deleted from Linear: `2026-05-21`
- Verification: `just listening-manifest-validator-fixtures; just observer-audio-summary-validator-fixtures; cargo test -p riotbox-app --bin observer_audio_correlate; git diff --check; GitHub Rust CI #2170 passed`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-883`

## Why This Ticket Existed

Generated Feral-grid manifests were required by spec to preserve source_timing.cue and source_timing.actionability, but the generic listening-manifest validator still treated those fields as optional.

## What Shipped

- Required source_timing.cue and source_timing.actionability for feral-grid-demo grid-BPM manifests in scripts/validate_listening_manifest_json.py.
- Added a missing-actionability negative fixture and updated valid Feral-grid/observer manifest fixtures with the practical timing phrase.

## Notes

- None
