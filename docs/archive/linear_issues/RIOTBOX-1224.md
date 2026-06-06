# `RIOTBOX-1224` Extract oversized Justfile audio-QA JSON contracts into validators

- Ticket: `RIOTBOX-1224`
- Title: `Extract oversized Justfile audio-QA JSON contracts into validators`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1224/extract-oversized-justfile-audio-qa-json-contracts-into-validators`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1224-extract-oversized-justfile-audio-qa-json-contracts-into-validators`
- Linear branch: `feature/riotbox-1224-extract-oversized-justfile-audio-qa-json-contracts-into`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1208 (https://github.com/marang/riotbox/pull/1208)`
- Merge commit: `e4a76f80`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile scripts/validate_professional_output_suite_contract.py; git diff --check; just --list; python3 scripts/validate_professional_output_suite_contract.py artifacts/audio_qa/local-riotbox-1233-suite-smoke/professional-output-suite.json --output artifacts/audio_qa/local-riotbox-1233-suite-smoke; just professional-output-suite-smoke artifacts/audio_qa/local-riotbox-1224-suite-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

The professional-output suite contract had grown into oversized inline jq expressions in Justfile, making the command catalog hard to scan and the audio-QA contract hard to review.

## What Shipped

- Added scripts/validate_professional_output_suite_contract.py, routed just professional-output-suite-smoke through it, kept compact negative mutations for non-source-derived hook/chop and scripted demo-readiness promotion, and documented the validator-over-inline-JQ rule.

## Notes

- No audio-QA threshold was intentionally weakened; the validator preserves the cross-report checks while moving them into named Python failure codes.
