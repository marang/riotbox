# `RIOTBOX-1200` Import human verdicts into professional output listening packs

- Ticket: `RIOTBOX-1200`
- Title: `Import human verdicts into professional output listening packs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1200/import-human-verdicts-into-professional-output-listening-packs`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1200-import-human-verdicts-into-professional-output-listening`
- Linear branch: `feature/riotbox-1200-import-human-verdicts-into-professional-output-listening`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1178 (https://github.com/marang/riotbox/pull/1178)`
- Merge commit: `e90a1dd3`
- Deleted from Linear: `2026-06-04`
- Verification: `python3 -m py_compile scripts/generate_professional_output_listening_pack.py scripts/import_listening_review_label.py; just professional-output-listening-pack-smoke artifacts/audio_qa/local-professional-output-listening-pack; just professional-output-listening-verdict-import-fixtures artifacts/audio_qa/local-professional-output-listening-pack; just professional-output-suite-smoke artifacts/audio_qa/local-professional-output-suite; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/professional_output_suite_v1_2026-06-04.md; docs/benchmarks/human_listening_label_corpus_v1_2026-06-04.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1201 adds rendered weak-output examples for professional source families.`

## Why This Ticket Existed

Professional-output listening packs could prepare review prompts, but recorded human verdicts did not yet have a hash-bound import path back into the label corpus.

## What Shipped

- Added audio_judge_label metadata to professional-output review packs, hash-verifying import mode, verified/unverified/stale import fixtures, audio-qa-ci coverage, and docs for the human-verdict boundary.

## Notes

- Generated packs still begin with human_verdict unverified; this slice enables later recorded human reviews to become typed labels without accepting stale artifacts.
