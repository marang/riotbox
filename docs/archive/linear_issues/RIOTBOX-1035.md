# `RIOTBOX-1035` P013+: Promote MC-202 source-derived phrase planning beyond contour support

- Ticket: `RIOTBOX-1035`
- Title: `P013+: Promote MC-202 source-derived phrase planning beyond contour support`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1035/p013-promote-mc-202-source-derived-phrase-planning-beyond-contour`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-30`
- Started: `2026-06-14`
- Finished: `2026-06-14`
- Branch: `feature/riotbox-1035-mc202-source-phrase-plan`
- Linear branch: `feature/riotbox-1035-p013-promote-mc-202-source-derived-phrase-planning-beyond`
- Assignee: `Markus`
- Labels: None
- PR: `#1237 (https://github.com/marang/riotbox/pull/1237)`
- Merge commit: `2979e5bb995fad807ff5031fa08c464a3dd6f83b`
- Deleted from Linear: `2026-06-14`
- Verification: `just ci (passed locally, log: /tmp/riotbox-1035-just-ci-final.log)`; `GitHub Actions rust-ci passed on PR #1237`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`, `docs/execution_roadmap.md`, `docs/phase_definition_of_done.md`, `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `Continue strengthening source-listening phrase extraction beyond this first Source Graph phrase-slot planner.`

## Why This Ticket Existed

MC-202 needed a replayable source-derived bass/answer phrase path instead of treating the static primitive answer/pressure shapes as product-quality proof.

## What Shipped

- Added typed Session/Core MC-202 source phrase plan state with undo, replay, and source-timing revert behavior.
- Derived deterministic MC-202 answer plans from trusted Source Graph phrase slots and source evidence, while rejecting commit boundaries outside phrase slots.
- Projected source-derived plans through the existing app/runtime/audio render seam, keeping primitive answer output as fallback/control instead of leaking it into product audio.
- Persisted the MC-202 source phrase planning plan in docs/plans and updated roadmap, DoD, specs, and decision log so the improvement path is explicit.

## Notes

- No formal human listening-review verdict exists for this slice; output proof is automated buffer/state/render regression evidence plus source-vs-control assertions.
