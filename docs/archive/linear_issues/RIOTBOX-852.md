# `RIOTBOX-852` Validate observer/audio MC-202 phrase-grid hit-ratio derivation

- Ticket: `RIOTBOX-852`
- Title: `Validate observer/audio MC-202 phrase-grid hit-ratio derivation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-852/validate-observeraudio-mc-202-phrase-grid-hit-ratio-derivation`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-852-observer-mc202-phrase-grid-hit-ratio-derivation`
- Linear branch: `feature/riotbox-852-validate-observeraudio-mc-202-phrase-grid-hit-ratio`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#847 (https://github.com/marang/riotbox/pull/847)`
- Merge commit: `e5338c5bc2bb8cbd597f7264157ab3afcf74b666`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_observer_audio_mc202_phrase_grid_hit_ratio_fixtures.py scripts/validate_observer_audio_mc202_phrase_grid_pass_fixtures.py; just observer-audio-summary-validator-fixtures; just ci; GitHub Actions Rust CI #2080 success`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

The observer/audio summary validator needed to reject MC-202 phrase-grid hit ratios that contradicted onset counts, so QA automation cannot accept incoherent phrase-grid evidence.

## What Shipped

- Validated mc202_phrase_grid.hit_ratio against grid_aligned_onset_count / candidate_onset_count, required 0.0 for zero candidate onsets, added dedicated validator fixtures, and documented the contract.

## Notes

- None
