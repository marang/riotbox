# `RIOTBOX-847` Validate observer/audio scalar metric ranges

- Ticket: `RIOTBOX-847`
- Title: `Validate observer/audio scalar metric ranges`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-847/validate-observeraudio-scalar-metric-ranges`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-847-observer-scalar-metric-ranges`
- Linear branch: `feature/riotbox-847-validate-observeraudio-scalar-metric-ranges`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#842 (https://github.com/marang/riotbox/pull/842)`
- Merge commit: `55055c82e83ab80985f2fe45b9f68a50ee456015`
- Verification: `Local checks passed before PR: py_compile for validator scripts, just observer-audio-summary-validator-fixtures, and just ci. GitHub Rust CI #2065 passed on PR #842.`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio scalar metrics needed explicit range validation so impossible RMS and ratio values cannot pass as output-path evidence.

## What Shipped

- Validator rejects negative scalar RMS/delta metrics and out-of-range W-30 active-sample ratios; fixture coverage is wired into the observer/audio summary validator gate; benchmark docs now state the range contract.

## Notes

- None
