# `RIOTBOX-1335` P023: Make Blend and four performance gestures reachable in the first-playable live flow

- Ticket: `RIOTBOX-1335`
- Title: `P023: Make Blend and four performance gestures reachable in the first-playable live flow`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1335/p023-make-blend-and-four-performance-gestures-reachable-in-the-first`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M2 | Live Dense-Break Golden Path`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-07-15`
- Finished: `2026-07-17`
- Branch: `feature/riotbox-1335-p023-live-blend-performance-gestures`
- Linear branch: `feature/riotbox-1335-p023-make-blend-and-four-performance-gestures-reachable-in`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `TUI`, `ux`
- PR: `#1367 (https://github.com/marang/riotbox/pull/1367)`
- Merge commit: `01c30c466cee3943963f5cd1988c52838c18893c`
- Deleted from Linear: `2026-07-17`
- Verification: `GitHub rust-ci: fmt, tests, audio QA smoke, and Clippy passed`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/audio_core_spec.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1401, RIOTBOX-1402`

## Why This Ticket Existed

Make Blend and four performance gestures reachable through the exact first-playable live product path.

## What Shipped

- Reachable Source, Blend, and Riotbox monitoring with musician-readable readiness and degraded states.
- Visible w, f, s, and y/Y gestures through queue, commit, Session/replay, Undo, observer, and exact RuntimeMix proof.
- Confirmed source-bar phase and typed half-bar PhraseDriveBreakCutStompV1 Fill takeover.

## Notes

- Instrument reachability shipped; human musical-alpha and demo-readiness verdicts remain unverified and belong to RIOTBOX-1401/1402.
