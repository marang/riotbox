# `RIOTBOX-1297` Strengthen destructive gesture impact proof

- Ticket: `RIOTBOX-1297`
- Title: `Strengthen destructive gesture impact proof`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1297/strengthen-destructive-gesture-impact-proof`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-28`
- Started: `2026-06-28`
- Finished: `2026-06-28`
- Branch: `feature/riotbox-1297-destructive-gesture-impact`
- Linear branch: `feature/riotbox-1297-strengthen-destructive-gesture-impact-proof`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`
- PR: `#1271 (https://github.com/marang/riotbox/pull/1271)`
- Merge commit: `d951e04493fc3bfd7580d411e92e8b94386e391f`
- Deleted from Linear: `2026-06-28`
- Verification: `python3 -m py_compile scripts/validate_destructive_variation_professional.py scripts/validate_professional_output_suite_contract.py scripts/generate_dense_break_performance_pack.py scripts/generate_professional_output_suite.py scripts/generate_rendered_weak_professional_outputs.py; just destructive-variation-professional-smoke artifacts/audio_qa/local-riotbox-1297-destructive-rerun; just rendered-weak-professional-output-fixtures artifacts/audio_qa/local-riotbox-1297-rendered-weak-rerun; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1297-suite-rerun; git diff --check; just ci; GitHub rust-ci pass on PR #1271`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Destructive gesture diagnostics needed direct cut-depth and restore-from-cut proof so flat stutter/dropout/restore edits cannot pass as stage-meaningful gestures.

## What Shipped

- Added dropout_silence_to_stutter_rms_ratio and restore_to_dropout_silence_rms_ratio proof fields; validated them in destructive-variation and professional-output suite contracts; added missing-metric guards, suite mutations, rendered weak-output expectations, and docs/roadmap notes.

## Notes

- Evidence remains diagnostic with quality_proof false until structured listening review accepts the output. Passing destructive values observed locally: dropout_silence_to_stutter_rms_ratio=0.008897 and restore_to_dropout_silence_rms_ratio=133.444410.
