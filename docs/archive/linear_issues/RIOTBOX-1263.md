# `RIOTBOX-1263` MC-202 Level 2 source feature vector for phrase planning

- Ticket: `RIOTBOX-1263`
- Title: `MC-202 Level 2 source feature vector for phrase planning`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1263/mc-202-level-2-source-feature-vector-for-phrase-planning`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-14`
- Started: `2026-06-14`
- Finished: `2026-06-14`
- Branch: `feature/riotbox-1263-mc202-source-feature-vector`
- Linear branch: `feature/riotbox-1263-mc-202-level-2-source-feature-vector-for-phrase-planning`
- Assignee: `Markus`
- Labels: None
- PR: `#1239 (https://github.com/marang/riotbox/pull/1239)`
- Merge commit: `a0017dc5b51423e517ce6cc2fdc1a63f52a28a0f`
- Deleted from Linear: `2026-06-14`
- Verification: `cargo fmt`; `cargo test -p riotbox-core mc202_source_phrase_features --quiet`; `cargo test -p riotbox-app mc202 --quiet`; `cargo test -p riotbox-core --quiet`; `cargo test -p riotbox-app --quiet`; `cargo clippy --all-targets --all-features -- -D warnings`; `git diff --check`; `just ci`
- Docs touched: `docs/plans/mc202_source_phrase_planning_plan.md`
- Follow-ups: `Implement MC-202 candidate-family generation and scoring against the new source feature vector, using richer measured audio features as they land.`

## Why This Ticket Existed

MC-202 source phrase planning needed a real Level-2 source feature contract so source-derived claims are backed by explicit source evidence, not only templates or fingerprints.

## What Shipped

- Added Mc202SourcePhraseFeatureVector with low-band pressure, transient density, offbeat density, hook restraint, source strength, confidence, stay-out, and provenance refs.
- Changed MC-202 source phrase derivation to consume the feature vector and fall back/silence weak source contexts instead of emitting fake source-derived answer audio.
- Added Core and app regressions for deterministic features, source-evidence variation, empty-evidence rejection, and render fallback behavior.

## Notes

- None
