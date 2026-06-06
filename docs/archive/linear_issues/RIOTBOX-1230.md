# `RIOTBOX-1230` Tighten source-character survival proof for rebuilt outputs

- Ticket: `RIOTBOX-1230`
- Title: `Tighten source-character survival proof for rebuilt outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1230/tighten-source-character-survival-proof-for-rebuilt-outputs`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1230-tighten-source-character-survival-proof-for-rebuilt-outputs`
- Linear branch: `feature/riotbox-1230-tighten-source-character-survival-proof-for-rebuilt-outputs`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1204 (https://github.com/marang/riotbox/pull/1204)`
- Merge commit: `56c84563493b0ce544f0e74904189dd2721f4f88`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile targeted professional-output scripts; git diff --check; targeted dense/matrix/source-wav/edge/suite smokes; just ci; GitHub rust-ci`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224 remains the Justfile/audio-QA validator extraction follow-up for the growing smoke gates.`

## Why This Ticket Existed

P022 rebuild-only output needed proof that transformed source character survives after the raw source layer is removed, not just non-silence and low waveform correlation.

## What Shipped

- Added rebuild-only source-character proof using spectral similarity, transient retention, RMS retention, and a survival score; surfaced it through dense, matrix, source-WAV, edge diagnostics, and suite reports; added gates and negative mutations while keeping quality_proof false and human_verdict unverified.

## Notes

- Diagnostic scripted evidence only; survival score guides QA but is not musical approval.
