# `RIOTBOX-1484` Export the qualified W-30 hook as a real DAWproject arrangement

- Ticket: `RIOTBOX-1484`
- Title: `Export the qualified W-30 hook as a real DAWproject arrangement`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1484/export-the-qualified-w-30-hook-as-a-real-dawproject-arrangement`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-28`
- Started: `2026-08-28`
- Finished: `2026-08-28`
- Branch: `feature/riotbox-1484-export-the-qualified-w-30-hook-as-a-real-dawproject`
- Linear branch: `feature/riotbox-1484-export-the-qualified-w-30-hook-as-a-real-dawproject`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1502 (https://github.com/marang/riotbox/pull/1502)`
- Merge commit: `5d8bac1216f5e5373c5ca69bfbb4ad476ee48186`
- Deleted from Linear: `2026-08-28`
- Verification: `focused W-30 DAWproject and observer regression tests passed`; `adjacent 45-test DAW-session family passed`; `two fresh full just ci runs passed`; `GitHub rust-ci passed`
- Docs touched: `docs/README.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/technology_stack_spec.md`
- Follow-ups: `Next bounded P016 workflow from RIOTBOX-1036: exact host-import evidence or the first honest live-recording path; host playback, TUI/Ghost, Holdout, and release remain separate.`

## Why This Ticket Existed

The qualified RIOTBOX-1482/1483 W-30 hook was reusable as a WAV handoff but still required a musician to create a DAW project, set tempo, and place the loop manually.

## What Shipped

- Added one versioned export.daw_session action that writes a valid DAWproject 1.0 archive with the byte-identical qualified hook at beat zero for eight beats in 4/4 at confirmed Session tempo.
- Bound exact embedded audio, project XML, proof, source/timing references, and capture lineage through Action, Session receipt, replay, observer, deterministic typed readback, and fail-closed no-replace publication.
- Aligned generic DAW readiness and surface projections so the archive readback is the writer proof, the older JSON skeleton is not applicable, and honest host-import and audible-output blockers remain.

## Notes

- No source, Holdout, commercial reference, audio playback, rerender, or listening verdict was used; the embedded accepted WAV remains byte-identical.
- Branch code/Rust review findings around receipt hydration, no-clobber staging, and generic readiness projection were fixed before merge.
