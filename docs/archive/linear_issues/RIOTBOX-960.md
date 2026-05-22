# `RIOTBOX-960` Validate observer/audio source timing count consistency

- Ticket: `RIOTBOX-960`
- Title: `Validate observer/audio source timing count consistency`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-960/validate-observeraudio-source-timing-count-consistency`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-960-validate-observeraudio-source-timing-count-consistency`
- Linear branch: `feature/riotbox-960-validate-observeraudio-source-timing-count-consistency`
- Assignee: `Markus`
- Labels: None
- PR: `#953 (https://github.com/marang/riotbox/pull/953)`
- Merge commit: `7bd4d87fdb7d5a6a722ce75e5d050bd313959551`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py`; `scripts/run_compact.sh /tmp/riotbox-960-observer-audio-summary-validator-final.log just observer-audio-summary-validator-fixtures`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-960-just-ci.log just ci`; `GitHub Actions Rust CI run 26304605216 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio summaries read the same observer_source_timing beat/bar/phrase counts as user-session observer events, but the summary validator did not explicitly reject count/status contradictions already blocked by the raw observer gate.

## What Shipped

- Added observer_source_timing beat/bar/phrase count consistency checks to the observer/audio summary validator.
- Added fixture-gate mutations for locked observer beat, bar, phrase, and non-locked phrase-count contradictions.
- Documented the observer/audio validator count contract in the Source Timing spec.

## Notes

- Validator/spec/fixture slice only; no analyzer scoring, ActionCommand, queue, Session/replay, JamAppState, schema, realtime audio, or render behavior changed.
