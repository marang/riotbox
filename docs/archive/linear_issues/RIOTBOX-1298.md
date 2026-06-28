# `RIOTBOX-1298` Strengthen source-first mix-bus masking proof

- Ticket: `RIOTBOX-1298`
- Title: `Strengthen source-first mix-bus masking proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1298/strengthen-source-first-mix-bus-masking-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-28`
- Started: `2026-06-28`
- Finished: `2026-06-28`
- Branch: `feature/riotbox-1298-source-first-mix-bus-masking`
- Linear branch: `feature/riotbox-1298-strengthen-source-first-mix-bus-masking-proof`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1272 (https://github.com/marang/riotbox/pull/1272)`
- Merge commit: `8a914141c4028f02ae76e566b057462a95379d6f`
- Deleted from Linear: `2026-06-28`
- Verification: `python3 -m py_compile scripts/validate_automated_musical_fitness.py scripts/validate_representative_showcase_musical_quality.py scripts/generate_professional_output_suite.py scripts/validate_professional_output_suite_contract.py; just automated-musical-fitness-fixtures; just representative-source-showcase-musical-quality-fixtures; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1298-suite; just weak-output-fix-routing-fixtures artifacts/audio_qa/local-riotbox-1298-routing; git diff --check; just ci; GitHub rust-ci passed on PR #1272`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Source-first mix-bus diagnostics needed a stricter shared masking ceiling and explicit headroom so generated support cannot barely pass while still risking source-character burial.

## What Shipped

- Aligned automated-fitness and representative source-first masking ceilings to 0.16; added professional-suite per-case source_first_masking_headroom; added min 0.09 suite headroom gate and mutation fixture; documented the diagnostic contract.

## Notes

- Observed local suite max source-first generated/source ratio 0.04954682 and min masking headroom 0.11045318. This remains diagnostic evidence, not automated musical-pass proof.
