# `RIOTBOX-1268` MC-202 source-composed render pressure and destructive contrast

- Ticket: `RIOTBOX-1268`
- Title: `MC-202 source-composed render pressure and destructive contrast`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1268/mc-202-source-composed-render-pressure-and-destructive-contrast`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1268-mc202-render-pressure-contrast`
- Linear branch: `feature/riotbox-1268-mc-202-source-composed-render-pressure-and-destructive`
- Assignee: `Markus`
- Labels: None
- PR: `#1243 (https://github.com/marang/riotbox/pull/1243)`
- Merge commit: `286a4d2f`
- Deleted from Linear: `2026-06-15`
- Verification: `just ci: green (/tmp/riotbox-1268-just-ci.log); GitHub rust-ci: pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1269 cross-source diversity/template-collapse gates; RIOTBOX-1270 structured listening review/demo-bank promotion`

## Why This Ticket Existed

Selected MC-202 source phrase candidates needed to affect the actual render seam, not just planner metadata, so strong sources could produce audible bass pressure and contrast.

## What Shipped

- Projected selected scorecards into source render accent/destructive masks plus pressure/contrast scalars, carried them through realtime shared state, and rendered source-only gain, gate, drive, accent, pitch-dive, and cut behavior with output-path tests.

## Notes

- No structured human listening pack in this slice; RIOTBOX-1264 remains open for producer-grade listening approval.
