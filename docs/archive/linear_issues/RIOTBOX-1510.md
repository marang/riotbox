# `RIOTBOX-1510` Replace Source Graph timing-candidate include namespace with semantic Rust modules

- Ticket: `RIOTBOX-1510`
- Title: `Replace Source Graph timing-candidate include namespace with semantic Rust modules`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1510/replace-source-graph-timing-candidate-include-namespace-with-semantic`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-09-30`
- Started: `2026-09-30`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1510-semantic-timing-candidates`
- Linear branch: `feature/riotbox-1510-replace-source-graph-timing-candidate-include-namespace-with`
- Assignee: `Markus`
- Labels: `Improvement`, `review-followup`
- PR: `#1564 (https://github.com/marang/riotbox/pull/1564)`
- Merge commit: `a5306fe4d5985fc5cb3ecd92ea2745c1525f4135`
- Deleted from Linear: `2026-10-01`
- Verification: `168 complete definitions accounted for, 15 public types and 12 signatures preserved; 470 Core and 771 App result entries identical; full source-free just ci and strict Clippy green; native run 36783129701 exact head green, no review comments.`
- Docs touched: `docs/engineering/module_policy.md`, `docs/engineering/textual_include_inventory_2026-06-29.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1510_semantic_timing_candidates_2026-09-30.md`
- Follow-ups: `RIOTBOX-1511 runtime regression ownership; RIOTBOX-1509 native Windows startup remains an investigation, not fixed.`

## Why This Ticket Existed

Remove the remaining shared timing candidate include namespace without altering timing trust or algorithms.

## What Shipped

- Real semantic candidate evidence/scoring/hypothesis/grid/model/report modules with explicit compatibility exports and test-only fixtures; twelve includes removed.

## Notes

- RBX-387; all algorithms, thresholds, schemas and frozen Stage-A contracts unchanged. Sequential solo review, not independent panel. No real source, holdout, device, DAW or human playback.
