# `RIOTBOX-1275` MC-202 composer chooses bass and answer motifs from source expression

- Ticket: `RIOTBOX-1275`
- Title: `MC-202 composer chooses bass and answer motifs from source expression`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1275/mc-202-composer-chooses-bass-and-answer-motifs-from-source-expression`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1275-mc202-expression-composer`
- Linear branch: `feature/riotbox-1275-mc-202-composer-chooses-bass-and-answer-motifs-from-source`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1250 (https://github.com/marang/riotbox/pull/1250)`
- Merge commit: `0cb334460f7060274535385d9bcabf2a956ba201`
- Deleted from Linear: `2026-06-15`
- Verification: `cargo test -p riotbox-core mc202_source_phrase_features -- --nocapture; cargo test -p riotbox-app committed_mc202_answer -- --nocapture; cargo test -p riotbox-app source_timing_side_effect_revert_clears_matching_mc202_source_phrase_plan -- --nocapture; git diff --check; just ci (/tmp/riotbox-1275-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1276 should use the expression-composed motifs for the production sound-design pass on the existing MC-202 render seam. RIOTBOX-1264 remains open through render, quality-gate, listening-pack, and closeout work.`

## Why This Ticket Existed

MC-202 had a typed source-expression vector, but candidate-family internals still adapted motifs from raw feature heuristics instead of composing from the replayable expression contract.

## What Shipped

- Moved MC-202 candidate construction, rejection, scoring, scorecards, groove fallback placement, contour, note-budget, and accent behavior to the Session source-expression axes; retained SourceGraph timing anchors and raw feature provenance; kept fallback/stay-out as control/failure paths; preserved source-derived output tests and cross-source diversity gates.

## Notes

- This improves motif composition but is not final producer-grade sound design; no human listening verdict or demo-readiness claim was added.
