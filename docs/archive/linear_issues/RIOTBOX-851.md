# `RIOTBOX-851` Validate observer/audio MC-202 phrase-grid pass consistency

- Ticket: `RIOTBOX-851`
- Title: `Validate observer/audio MC-202 phrase-grid pass consistency`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-851/validate-observeraudio-mc-202-phrase-grid-pass-consistency`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-851-observer-mc202-phrase-grid-pass-consistency`
- Linear branch: `feature/riotbox-851-validate-observeraudio-mc-202-phrase-grid-pass-consistency`
- Assignee: `Markus`
- Labels: `benchmark`, `review-followup`
- PR: `#846 (https://github.com/marang/riotbox/pull/846)`
- Merge commit: `830eca082b3d11f7e7c46ec868f79f13b626eb1c`
- Verification: `Local checks passed before PR: py_compile for validator scripts, just observer-audio-summary-validator-fixtures, git diff --check, and just ci. GitHub Rust CI #2077 passed on PR #846.`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

Observer/audio MC-202 phrase-grid pass evidence needed consistency validation so summaries cannot claim or deny passing phrase-grid proof contrary to their boundary, onset, hit-ratio, and offset evidence.

## What Shipped

- Validator now requires MC-202 phrase-grid passed to match phrase-boundary start, candidate onset presence, hit_ratio >= 0.95, and max onset offset within budget; fixture coverage is wired into the observer/audio summary validator gate; benchmark docs state the pass contract.

## Notes

- None
