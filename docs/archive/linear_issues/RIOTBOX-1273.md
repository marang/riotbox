# `RIOTBOX-1273` MC-202 source phrase render articulation for bass pressure vs answer stabs

- Ticket: `RIOTBOX-1273`
- Title: `MC-202 source phrase render articulation for bass pressure vs answer stabs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1273/mc-202-source-phrase-render-articulation-for-bass-pressure-vs-answer`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1273-mc202-render-articulation`
- Linear branch: `feature/riotbox-1273-mc-202-source-phrase-render-articulation-for-bass-pressure`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1248 (https://github.com/marang/riotbox/pull/1248)`
- Merge commit: `fadc6d1302a0689412f6f0c31e1262fd9a16c1a6`
- Deleted from Linear: `2026-06-15`
- Verification: `cargo test -p riotbox-audio mc202 -- --nocapture; cargo test -p riotbox-app mc202 -- --nocapture; git diff --check; just ci (/tmp/riotbox-1273-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1264 remains open for structured listening review and additional production-quality fixes toward producer-grade dense/non-dense MC-202 output.`

## Why This Ticket Existed

MC-202 source-derived pressure and answer plans still shared mostly the same render articulation, making bass pressure and stabs less distinct than the planner's selected family implied.

## What Shipped

- Added typed bass_weight, stab_bite, and gate_snap articulation to source phrase render plans; derived those values from selected candidate family and scorecard; applied them in the audio renderer and realtime shared state; added audio/app tests and split MC-202 tests into semantic files under the review budget.

## Notes

- No human listening verdict or demo-readiness claim was added.
