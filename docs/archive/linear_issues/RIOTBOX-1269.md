# `RIOTBOX-1269` MC-202 cross-source diversity and template-collapse QA gates

- Ticket: `RIOTBOX-1269`
- Title: `MC-202 cross-source diversity and template-collapse QA gates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1269/mc-202-cross-source-diversity-and-template-collapse-qa-gates`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1269-mc202-cross-source-diversity-template-collapse`
- Linear branch: `feature/riotbox-1269-mc-202-cross-source-diversity-and-template-collapse-qa-gates`
- Assignee: `Markus`
- Labels: None
- PR: `#1244 (https://github.com/marang/riotbox/pull/1244)`
- Merge commit: `60e1b3d4`
- Deleted from Linear: `2026-06-15`
- Verification: `just ci: green (/tmp/riotbox-1269-just-ci.log); GitHub rust-ci: pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1270 structured listening review/demo-bank promotion gate for producer-grade MC-202 examples`

## Why This Ticket Existed

MC-202 source-composed phrases needed automated gates that reject template collapse and prove cross-source source-specific output before any quality claim.

## What Shipped

- Added dedicated quality-gate tests for same-source determinism, four measured source-family diversity cases, phrase-plan/render-mask/render-buffer separation, and a neutralized-feature negative gate that downgrades to non-source-derived silence.

## Notes

- Automated gates do not replace structured human listening approval; RIOTBOX-1264 remains open until producer-grade listening review passes.
