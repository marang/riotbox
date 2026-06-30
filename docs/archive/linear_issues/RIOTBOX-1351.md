# `RIOTBOX-1351` P023: Strengthen W-30 hook/chop policy from weak-output routing

- Ticket: `RIOTBOX-1351`
- Title: `P023: Strengthen W-30 hook/chop policy from weak-output routing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1351/p023-strengthen-w-30-hookchop-policy-from-weak-output-routing`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-30`
- Started: `2026-06-30`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1351-p023-strengthen-w-30-hookchop-policy-from-weak-output`
- Linear branch: `feature/riotbox-1351-p023-strengthen-w-30-hookchop-policy-from-weak-output`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1315 (https://github.com/marang/riotbox/pull/1315)`
- Merge commit: `d5ece58643a2572f554099cdbb98e61c6fb2c198`
- Deleted from Linear: `2026-06-30`
- Verification: `python3 -m py_compile scripts/generate_dense_break_performance_pack.py scripts/validate_professional_output_suite_contract.py`; `just pro-pressure-source-matrix-smoke`; `just professional-source-wav-pack-smoke`; `just professional-output-suite-smoke`; `just weak-output-fix-routing-fixtures`; `just professional-output-listening-pack-smoke`; `just mc202-producer-grade-closeout-smoke`; `just audio-qa-ci`; `just ci`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`
- Follow-ups: `Continue P023 with the next weak-output production-fix candidate, likely bass_movement or destructive_gesture, while preserving source-backed diagnostic boundaries.`

## Why This Ticket Existed

Turn the highest-ranked weak-output production-fix candidate, chop_policy, into a real W-30 Hook/Chop render-policy improvement.

## What Shipped

- Dense/tonal Hook/Chop diagnostics now require hook_chop_w30_to_source_margin >= 0.10 instead of 0.025.
- Dense and tonal W-30 source-relative gain, hook/chop W-30 mix gains, and source-derived riff impact are pushed forward so the first two bars read more like a hook.
- Tonal MC-202 gain is raised enough to preserve the MC-202/W-30 support floor after the stronger W-30 hook lift.
- Documented the new P023 Hook/Chop boundary in the audio-QA spec, roadmap, and decision log.

## Notes

- Measured output evidence: dense W-30/source ratio 0.309 -> 0.346; dense hook/chop margin 0.089 -> 0.126; tonal hook/chop margin 0.158; tonal MC-202/W-30 0.209.
