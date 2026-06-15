# `RIOTBOX-1271` MC-202 source phrase groove spacing from transient and hook evidence

- Ticket: `RIOTBOX-1271`
- Title: `MC-202 source phrase groove spacing from transient and hook evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1271/mc-202-source-phrase-groove-spacing-from-transient-and-hook-evidence`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1271-mc202-source-phrase-groove-spacing`
- Linear branch: `feature/riotbox-1271-mc-202-source-phrase-groove-spacing-from-transient-and-hook`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1246 (https://github.com/marang/riotbox/pull/1246)`
- Merge commit: `b6e804b5`
- Deleted from Linear: `2026-06-15`
- Verification: `cargo test -p riotbox-app mc202 -- --nocapture; git diff --check; just ci (/tmp/riotbox-1271-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1264 remains open for actual structured listening review and further production-quality fixes until producer-grade dense and non-dense MC-202 examples pass.`

## Why This Ticket Existed

MC-202 source phrase candidates still risked sounding like related fixed question/answer templates because step placement came mostly from feature buckets rather than source timing anchors.

## What Shipped

- Added source phrase groove-map placement for pressure, answer, callback, hook-safe, and pickup steps from Source Graph timing anchors; recorded groove provenance; added commit-path tests for source AnswerSlot movement and hook-restraint downbeat avoidance; preserved the existing Source Graph -> Session -> projection -> render path.

## Notes

- This is an implementation and regression-test quality improvement, not a structured human listening pass or demo-readiness claim.
