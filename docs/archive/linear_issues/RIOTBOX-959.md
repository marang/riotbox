# `RIOTBOX-959` Validate Source Timing probe beat/bar count consistency

- Ticket: `RIOTBOX-959`
- Title: `Validate Source Timing probe beat/bar count consistency`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-959/validate-source-timing-probe-beatbar-count-consistency`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-959-validate-source-timing-probe-beatbar-count-consistency`
- Linear branch: `feature/riotbox-959-validate-source-timing-probe-beatbar-count-consistency`
- Assignee: `Markus`
- Labels: None
- PR: `#952 (https://github.com/marang/riotbox/pull/952)`
- Merge commit: `c05ae0aaeae1804fa5b74a6effcc97da11b4ee15`
- Deleted from Linear: `2026-05-22`
- Verification: `python3 -m py_compile scripts/validate_source_timing_probe_json.py scripts/validate_source_timing_grid_use_contract_fixtures.py`; `scripts/run_compact.sh /tmp/riotbox-959-probe-validator-final.log just source-timing-probe-json-validator-fixtures`; `scripts/run_compact.sh /tmp/riotbox-959-grid-use-fixtures-final.log just source-timing-grid-use-contract-fixtures`; `git diff --check`; `scripts/run_compact.sh /tmp/riotbox-959-just-ci.log just ci`; `GitHub Actions Rust CI run 26304253654 passed`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-958 exposed primary_beat_count and primary_bar_count, but the probe JSON validator initially only required non-negative integers and allowed contradictory status/count payloads.

## What Shipped

- Added beat/bar count consistency checks for stable and unavailable probe statuses.
- Added focused source-timing-probe validator fixture mutations for stable-with-zero and unavailable-with-positive count cases.
- Updated grid-use contract fixture generation so unavailable probe cases clear the new counts, and documented the validator contract.

## Notes

- Validator/spec/fixture slice only; no analyzer scoring, ActionCommand, queue, Session/replay, JamAppState, observer schema, realtime audio, or render behavior changed.
