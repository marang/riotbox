# `RIOTBOX-791` Expose lane recipe summary case count in audio QA docs and contract notes

- Ticket: `RIOTBOX-791`
- Title: `Expose lane recipe summary case count in audio QA docs and contract notes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-791/expose-lane-recipe-summary-case-count-in-audio-qa-docs-and-contract`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-14`
- Started: `2026-05-14`
- Finished: `2026-05-14`
- Branch: `feature/riotbox-791-lane-recipe-summary-contract-docs`
- Linear branch: `feature/riotbox-791-expose-lane-recipe-summary-case-count-in-audio-qa-docs-and`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#786 (https://github.com/marang/riotbox/pull/786)`
- Merge commit: `5660fefa7e52969c941c0dd2ee50f8d12574d185`
- Deleted from Linear: `2026-05-14`
- Verification: `scripts/run_compact.sh /tmp/riotbox-791-doc-grep.log rg -n lane_recipe_cases docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md scripts/validate_observer_audio_summary_json.py scripts/validate_recipe2_observer_audio_gate.sh`; `git diff --check`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`
- Follow-ups: `None`

## Why This Ticket Existed

P012 required the observer/audio summary JSON contract to document lane_recipe_cases after the field became required and generated Recipe 2 MC-202 summary evidence became visible.

## What Shipped

- Documented output_path.lane_recipe_cases, the compact lane recipe case evidence shape, and the generated Recipe 2 MC-202 required case proof expectations.

## Notes

- Documentation-only branch; no code behavior, ActionCommand, JamAppState, or audio-producing behavior changed.
