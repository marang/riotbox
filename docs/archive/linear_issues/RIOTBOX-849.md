# `RIOTBOX-849` Validate observer/audio lane recipe metric ranges

- Ticket: `RIOTBOX-849`
- Title: `Validate observer/audio lane recipe metric ranges`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-849/validate-observeraudio-lane-recipe-metric-ranges`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-849-observer-lane-recipe-metric-ranges`
- Linear branch: `feature/riotbox-849-validate-observeraudio-lane-recipe-metric-ranges`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#844 (https://github.com/marang/riotbox/pull/844)`
- Merge commit: `853d281c0f5530310c3d90e780087264fa96e5a2`
- Verification: `Local checks passed before PR: py_compile for validator scripts, just observer-audio-summary-validator-fixtures, git diff --check, and just ci. GitHub Rust CI #2071 passed on PR #844.`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio lane recipe proof fields needed range validation so impossible MC-202 phrase-grid metrics cannot pass as output-path evidence.

## What Shipped

- Validator now rejects negative lane recipe RMS/delta fields, out-of-range hit ratios, negative onset counts/offsets, and aligned-onset counts greater than candidate counts; fixture coverage is wired into the observer/audio summary validator gate; benchmark docs state the metric range contract.

## Notes

- None
