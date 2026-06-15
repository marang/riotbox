# `RIOTBOX-1274` MC-202 source expression vector for producer-grade phrase decisions

- Ticket: `RIOTBOX-1274`
- Title: `MC-202 source expression vector for producer-grade phrase decisions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1274/mc-202-source-expression-vector-for-producer-grade-phrase-decisions`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-15`
- Finished: `2026-06-15`
- Branch: `feature/riotbox-1274-mc202-source-expression-vector`
- Linear branch: `feature/riotbox-1274-mc-202-source-expression-vector-for-producer-grade-phrase`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1249 (https://github.com/marang/riotbox/pull/1249)`
- Merge commit: `605be8de42d93bf0f2389f9141d4965849b641f3`
- Deleted from Linear: `2026-06-15`
- Verification: `cargo test -p riotbox-core mc202_source_phrase_features -- --nocapture; cargo test -p riotbox-app committed_mc202_answer -- --nocapture; cargo test -p riotbox-app source_timing_side_effect_revert_clears_matching_mc202_source_phrase_plan -- --nocapture; git diff --check; just ci (/tmp/riotbox-1274-just-ci.log); GitHub rust-ci pass`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `RIOTBOX-1275 should use the source-expression vector to compose bass/answer motifs rather than mutating fixed templates. RIOTBOX-1264 remains open through composer, render, gate, listening-pack, and closeout work.`

## Why This Ticket Existed

MC-202 source phrase planning had source-backed candidate families, but the next composer still lacked a typed, replayable expression vector that summarized what the source musically asked for.

## What Shipped

- Extended MC-202 Source Graph phrase features with low-band movement, backbeat density, spectral roughness, and spectral brightness; added a Session-backed MC-202 source-expression state with bass pressure, low-pressure contour, transient/backbeat pressure, offbeat answer space, phrase density, hook restraint, stab bite, stay-out pressure, confidence, and provenance; added reproducibility, cross-source diversity, measured-audio removal, and neutralized-source tests.

## Notes

- This is a composer-input contract, not final producer-grade MC-202 output; no human listening verdict or demo-readiness claim was added.
