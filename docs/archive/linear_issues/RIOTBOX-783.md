# `RIOTBOX-783` Prove manual-confirm short-loop Source Timing fixture boundary

- Ticket: `RIOTBOX-783`
- Title: `Prove manual-confirm short-loop Source Timing fixture boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-783/prove-manual-confirm-short-loop-source-timing-fixture-boundary`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-783-prove-manual-confirm-short-loop-source-timing-fixture`
- Linear branch: `feature/riotbox-783-prove-manual-confirm-short-loop-source-timing-fixture`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#777 (https://github.com/marang/riotbox/pull/777)`
- Merge commit: `3029efb00af27a9abbb97a456a5f4586445abf1a`
- Deleted from Linear: `2026-05-11`
- Verification: `python3 -m py_compile scripts/validate_source_timing_short_loop_fixture.py`; `just source-timing-probe-json-validator-fixtures`; `cargo test -p riotbox-core source_timing_probe_readiness_keeps_short_loop_manual_confirm_in_review -- --nocapture`; `cargo fmt --check`; `git diff --check`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

P012 needed an explicit committed proof for the conservative short-loop Source Timing boundary: stable beat/downbeat evidence may be useful, but insufficient phrase evidence must remain manual-confirm and must not be treated as locked timing.

## What Shipped

- Added semantic validation for the existing short-loop Source Timing probe fixture, covering cue, readiness, manual-confirm, grid_use, stable beat/downbeat, phrase uncertainty, anchor evidence, and warning presence.
- Wired the short-loop semantic validator into the probe JSON fixture recipe alongside the locked-grid fixture.
- Tightened the core short-loop readiness test to assert the PhraseUncertain warning.

## Notes

- No detector thresholds, lane render behavior, or production arbitrary-audio claims changed.
