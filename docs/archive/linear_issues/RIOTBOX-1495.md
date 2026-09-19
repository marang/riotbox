# `RIOTBOX-1495` Review follow-up: fix Graph recovery → save and define storage/integrity boundaries

- Ticket: `RIOTBOX-1495`
- Title: `Review follow-up: fix Graph recovery → save and define storage/integrity boundaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1495/review-follow-up-fix-graph-recovery-save-and-define-storageintegrity`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-08`
- Started: `2026-09-19`
- Finished: `2026-09-19`
- Branch: `feature/riotbox-1495-recovery-save`
- Linear branch: `feature/riotbox-1495-review-follow-up-fix-graph-recovery-save-and-define`
- Assignee: `Markus`
- Labels: `Bug`, `Core`, `review-followup`
- PR: `#1521 (https://github.com/marang/riotbox/pull/1521)`
- Merge commit: `b0d8571afeb1070e5604fb1d71623ef276eaa438`
- Deleted from Linear: `2026-09-19`
- Verification: `Original JSON failure and UTF8/relocation gaps reproduced red before corrections; 16 focused transaction tests and four Core publication tests passed.`; `Source-free just ci passed including 744 App tests, synthetic QA and strict Clippy; log /tmp/riotbox-1495-ci.log.`; `Independent App/Core/docs review plus self-review fixed three P2 findings and finished with no remaining blocking findings.`; `PR1521 remote Rust CI 35462169360 passed; no open comments or blocking reviews before merge.`
- Docs touched: `docs/specs/session_file_spec.md`, `docs/dev_environment.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1495_recovery_save_2026-09-19.md`
- Follow-ups: `RIOTBOX-1497 separately owns persisted capture-WAV content identity and legacy rules.`

## Why This Ticket Existed

A Session recovered from a corrupt Graph alias could not save edits despite a valid exact immutable generation; external publication had an implicit hardlink requirement.

## What Shipped

- Persisted-Session exact-generation authority now permits safe save after missing, invalid-JSON or invalid-UTF8 alias recovery, while preserving fresh graph destinations and failing closed on invalid authority.
- RBX-374 declares hardlink-required external Graph storage with a typed unsupported publication error and no copy fallback; publication and tests have a semantic Core module owner.

## Notes

- No real filesystem matrix, source/holdout access or human playback; no musical claim. Existing single-writer/process-interruption guarantee remains, not power-loss durability.
