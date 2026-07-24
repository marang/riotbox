# `RIOTBOX-1423` P023: Expand the legal real-source corpus and rotate cross-family holdouts

- Ticket: `RIOTBOX-1423`
- Title: `P023: Expand the legal real-source corpus and rotate cross-family holdouts`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1423/p023-expand-the-legal-real-source-corpus-and-rotate-cross-family`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-23`
- Started: `2026-07-23`
- Finished: `2026-07-24`
- Branch: `feature/riotbox-1423-p023-expand-the-legal-real-source-corpus-and-rotate-cross`
- Linear branch: `feature/riotbox-1423-p023-expand-the-legal-real-source-corpus-and-rotate-cross`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1376 (https://github.com/marang/riotbox/pull/1376)`
- Merge commit: `261d40bb1079f745786b86850dd04c1851ddb655`
- Deleted from Linear: `2026-07-24`
- Verification: `source-holdout-rotation fixtures/local-files: pass; cargo fmt --check: pass; GitHub rust-ci: pass`
- Docs touched: `docs/benchmarks/source_holdout_rotation_v1.json; docs/specs/fixture_corpus_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/execution_roadmap.md; docs/phase_definition_of_done.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1422`

## Why This Ticket Existed

RIOTBOX-1422 exposed overfitting to one narrow loop family and had no fresh legal holdouts left for another source-aware W-30 Hard candidate.

## What Shipped

- Versioned a local-ignored CC0 corpus with 15 eligible sources across 14 source packs, six development families, and one dense-full-mix stress source.
- Added two disjoint unheard multi-family holdout sets with explicit consumption and replacement provenance.
- Added CI-safe contract/mutation validation plus optional local SHA-256, WAV-format, duration, and clipping verification.

## Notes

- Contract enabler only: quality_proof false; no source audio or commercial reference recording was committed.
