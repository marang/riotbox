# `RIOTBOX-1003` Separate synthetic fixture showcase from real-source listening showcase

- Ticket: `RIOTBOX-1003`
- Title: `Separate synthetic fixture showcase from real-source listening showcase`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1003/separate-synthetic-fixture-showcase-from-real-source-listening`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-26`
- Started: `2026-05-26`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-1003-separate-synthetic-fixture-showcase-from-real-source`
- Linear branch: `feature/riotbox-1003-separate-synthetic-fixture-showcase-from-real-source`
- Assignee: `Markus`
- Labels: `Improvement`, `benchmark`, `workflow`
- PR: None
- Merge commit: `87e11c9f`
- Deleted from Linear: `2026-05-27`
- Verification: `just real-source-listening-showcase-validate (passed 2026-05-27)`; `just source-showcase-diversity-validator-fixtures (passed 2026-05-27)`
- Docs touched: `AGENTS.md`, `docs/specs/audio_qa_workflow_spec.md`, `docs/workflow_conventions.md`
- Follow-ups: `None`

## Why This Ticket Existed

The representative showcase path was still mixing deterministic synthetic fixture material with musician-facing real-source listening evidence. P012 needed those surfaces separated so fixture packs remain CI-safe while real-source listening reports stay honest about local inputs and musical verdicts.

## What Shipped

- Added a real-source local listening showcase path and manifest convention.
- Separated synthetic fixture showcase generation from real-source listening showcase generation.
- Removed hardcoded showcase assumptions from generated QA paths and kept real-source verdicts distinct from deterministic fixture proof.

## Notes

- No GitHub PR was found for this direct-main history; the archived merge commit records the latest RIOTBOX-1003 commit contained in main, with earlier commit 39551fa4 also included.
