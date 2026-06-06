# `RIOTBOX-1229` Expose strongest-audible-element proof across professional outputs

- Ticket: `RIOTBOX-1229`
- Title: `Expose strongest-audible-element proof across professional outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1229/expose-strongest-audible-element-proof-across-professional-outputs`
- Project: `P022 | Professional Sound Output`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-06`
- Started: `2026-06-06`
- Finished: `2026-06-06`
- Branch: `feature/riotbox-1229-expose-strongest-audible-element-proof-across-professional`
- Linear branch: `feature/riotbox-1229-expose-strongest-audible-element-proof-across-professional`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1203 (https://github.com/marang/riotbox/pull/1203)`
- Merge commit: `edf76636caccefd001fa4d9755586405187ae17a`
- Deleted from Linear: `2026-06-06`
- Verification: `python3 -m py_compile targeted professional-output scripts; git diff --check; targeted dense/matrix/source-wav/edge/suite smokes; just ci; GitHub rust-ci`
- Docs touched: `docs/execution_roadmap.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1224 remains the Justfile/audio-QA validator extraction follow-up for the growing smoke gates.`

## Why This Ticket Existed

P022 needed machine-readable evidence for what audible element carries each professional-output render, rather than only pass/RMS/non-collapse status.

## What Shipped

- Added bounded strongest_audible_element proof, surfaced it through dense, matrix, professional source-WAV, edge diagnostics, and the professional-output suite, and gated missing/ambiguous evidence while keeping quality_proof false and human_verdict unverified.

## Notes

- Diagnostic scripted evidence only; strongest-element labels guide review but are not musical approval.
