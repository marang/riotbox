# `RIOTBOX-1220` Add bad-timing cautious arrangement policy and user cue proof

- Ticket: `RIOTBOX-1220`
- Title: `Add bad-timing cautious arrangement policy and user cue proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1220/add-bad-timing-cautious-arrangement-policy-and-user-cue-proof`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-05`
- Started: `2026-06-05`
- Finished: `2026-06-05`
- Branch: `feature/riotbox-1220-add-bad-timing-cautious-arrangement-policy-and-user-cue`
- Linear branch: `feature/riotbox-1220-add-bad-timing-cautious-arrangement-policy-and-user-cue`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1195 (https://github.com/marang/riotbox/pull/1195)`
- Merge commit: `f0f06b4299a89cc8116cfc433823ac0d4f5cfe2e`
- Deleted from Linear: `2026-06-05`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/generate_edge_source_professional_diagnostics.py scripts/generate_professional_output_suite.py; just edge-source-professional-diagnostics-smoke artifacts/audio_qa/local-riotbox-1220-edge-smoke-8; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1220-suite-smoke-2; scripts/run_compact.sh /tmp/riotbox-1220-audio-qa-ci-final.log just audio-qa-ci; scripts/run_compact.sh /tmp/riotbox-1220-just-ci-final.log just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/dense_break_performance_pack_v1_2026-06-04.md; docs/benchmarks/professional_output_suite_v1_2026-06-04.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Bad-timing/manual-confirm-only sources needed an explicit cautious professional-output route instead of flowing through confident bar-locked policy.

## What Shipped

- Added bad_timing pressure policy and manual_confirm_cautious_arrangement timing policy, confirmation-cue proof/negative fixture, edge-source suite coverage, and P022 docs/decision-log updates.

## Notes

- None
