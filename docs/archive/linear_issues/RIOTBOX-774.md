# `RIOTBOX-774` Propagate Source Timing grid-use into observer/audio summaries

- Ticket: `RIOTBOX-774`
- Title: `Propagate Source Timing grid-use into observer/audio summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-774/propagate-source-timing-grid-use-into-observeraudio-summaries`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-774-propagate-source-timing-grid-use-into-observeraudio`
- Linear branch: `feature/riotbox-774-propagate-source-timing-grid-use-into-observeraudio`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#768 (https://github.com/marang/riotbox/pull/768)`
- Merge commit: `00e6fecef9217d072ff6a42dac98e31e109f816a`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate; just observer-audio-summary-validator-fixtures; just observer-audio-correlate-json-fixture; just observer-audio-correlate-generated-feral-grid; git diff --check; just ci`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md; docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `RIOTBOX-775 rejects grid-use / grid BPM policy contradictions in strict observer/audio evidence.`

## Why This Ticket Existed

Observer/audio correlation summaries needed to expose manifest-side Source Timing grid-use classification so QA could see timing trust without opening raw Feral grid manifests.

## What Shipped

- Collected source_timing.grid_use from manifests, rendered it in Markdown and JSON observer/audio summaries, validated allowed values and consistency, added a negative validator fixture, and documented the additive contract field.

## Notes

- None
