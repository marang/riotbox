# `RIOTBOX-1411` Split remaining riotbox-app UI and Jam test include shells into semantic modules

- Ticket: `RIOTBOX-1411`
- Title: `Split remaining riotbox-app UI and Jam test include shells into semantic modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1411/split-remaining-riotbox-app-ui-and-jam-test-include-shells-into`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-19`
- Started: `2026-09-30`
- Finished: `2026-09-30`
- Branch: `feature/riotbox-1411-semantic-ui-tests`
- Linear branch: `feature/riotbox-1411-split-remaining-riotbox-app-ui-and-jam-test-include-shells`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1555 (https://github.com/marang/riotbox/pull/1555)`
- Merge commit: `e54b644e546bfcada10c9a5c6897fd4125c7bd20`
- Deleted from Linear: `2026-09-30`
- Verification: `Full local just ci and exact-head native run 36775673935 green; App 770 with one unchanged ignored test; 1126 selected items accounted for; solo review/self-review zero unresolved findings.`
- Docs touched: `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/engineering/textual_include_allowlist.txt`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1411_semantic_ui_tests_2026-09-30.md`
- Follow-ups: `RIOTBOX-1507 owns the due broader architecture checkpoint; no new runtime defect found in branch review.`

## Why This Ticket Existed

Replace shared lexical UI and Jam-test include ownership with explicit semantic Rust modules.

## What Shipped

- 109 includes removed; public shell/render and four crate-internal fixture paths preserved; no behavior change; inventory now 94 sites in 13 owners.

## Notes

- RBX-385. No real source, holdout, device, DAW or playback; synthetic evidence grants no musical or human pass. No subagents.
