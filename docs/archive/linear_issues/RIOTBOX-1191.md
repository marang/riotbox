# `RIOTBOX-1191` Add real listening label import path for audio judge

- Ticket: `RIOTBOX-1191`
- Title: `Add real listening label import path for audio judge`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1191/add-real-listening-label-import-path-for-audio-judge`
- Project: `P021 | Audio Judge / Musical Fitness`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-04`
- Started: `2026-06-04`
- Finished: `2026-06-04`
- Branch: `feature/riotbox-1191-add-real-listening-label-import-path-for-audio-judge`
- Linear branch: `feature/riotbox-1191-add-real-listening-label-import-path-for-audio-judge`
- Assignee: `Markus`
- Labels: None
- PR: `#1169 (https://github.com/marang/riotbox/pull/1169)`
- Merge commit: `e441517d`
- Deleted from Linear: `2026-06-04`
- Verification: `python3 -m py_compile scripts/import_listening_review_label.py scripts/listening_review_workflow.py scripts/validate_human_listening_label_corpus.py; just listening-review-label-import-fixtures; just human-listening-label-corpus-fixtures; just audio-judge-spike-fixtures; just audio-judge-spike-generated-smoke; just audio-qa-ci; just ci; GitHub rust-ci pass`
- Docs touched: `docs/benchmarks/human_listening_label_corpus_v1_2026-06-04.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `Next P021 work can use the import path with real local listening packs instead of fixture-only labels.`

## Why This Ticket Existed

Create a validated bridge from structured listening reviews into the audio-judge label corpus so calibrated future judgment can use real human evidence.

## What Shipped

- Added import_listening_review_label.py, valid/invalid import fixtures, listening-review-label-import-fixtures wired into audio-qa-ci, and docs for required audio_judge_label metadata and verdict mapping.

## Notes

- Importer rejects unverified or metadata-missing reviews and validates both the source listening_review.v1 shape and output human_listening_label_corpus.v1 shape.
