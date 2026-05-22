# `RIOTBOX-937` Validate observer/audio downbeat ambiguity alignment JSON

- Ticket: `RIOTBOX-937`
- Title: `Validate observer/audio downbeat ambiguity alignment JSON`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-937/validate-observeraudio-downbeat-ambiguity-alignment-json`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-937-downbeat-ambiguity-json-validator`
- Linear branch: `feature/riotbox-937-validate-observeraudio-downbeat-ambiguity-alignment-json`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#930 (https://github.com/marang/riotbox/pull/930)`
- Merge commit: `ec3f6035bfbb1ec8e9e6c6be59319e1eae3a817f`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py; just observer-audio-summary-validator-fixtures; just observer-audio-correlate-json-fixture; just observer-audio-correlate-locked-grid-json-fixture; just audio-qa-ci; just ci; git diff --check; GitHub Actions Rust CI run 26294281889 passed`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md; docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `Continue P012 source-timing spine with the next smallest roadmap-aligned proof.`

## Why This Ticket Existed

Validate observer/audio downbeat ambiguity alignment JSON so external QA tooling catches malformed compatibility summaries.

## What Shipped

- Required downbeat_ambiguity_compatibility in source timing alignment summaries, added issue-list coherence checks, fixture coverage, Justfile gate wiring, and contract/spec docs.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, realtime audio, or generated audio behavior changed.
