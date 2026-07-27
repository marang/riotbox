# `RIOTBOX-1424` P023: Prequalify trusted-grid and W-30 Hard-recipe reachability before frozen holdout

- Ticket: `RIOTBOX-1424`
- Title: `P023: Prequalify trusted-grid and W-30 Hard-recipe reachability before frozen holdout`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1424/p023-prequalify-trusted-grid-and-w-30-hard-recipe-reachability-before`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-27`
- Started: `2026-07-27`
- Finished: `2026-07-27`
- Branch: `feature/riotbox-1424-p023-prequalify-trusted-grid-and-w-30-hard-recipe`
- Linear branch: `feature/riotbox-1424-p023-prequalify-trusted-grid-and-w-30-hard-recipe`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1378 (https://github.com/marang/riotbox/pull/1378)`
- Merge commit: `1512433c0d853e7a0ce67a005bc5a15033b6ab5e`
- Deleted from Linear: `2026-07-27`
- Verification: `cargo test; cargo clippy --all-targets --all-features -- -D warnings; cargo fmt --all -- --check; just source-holdout-rotation-fixtures; GitHub Rust CI`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/specs/fixture_corpus_spec.md; docs/research_decision_log.md; docs/reviews/riotbox_1424_w30_holdout_reachability_review_2026-07-27.md`
- Follow-ups: `RIOTBOX-1422: freeze one causally reachable fresh exact-path holdout and proceed to structured human listening only after technical gates pass.`

## Why This Ticket Existed

Prevent fresh W-30 Hard holdouts from being consumed before the exact product path and source_hit_shaper_v3 calibration are causally reachable.

## What Shipped

- Added a typed development-only W-30 reachability preflight that reports timing confirmation, selected Hard recipe, exact callback calibration, and fail-closed candidate-WAV eligibility.
- Added a multi-family holdout applicability contract and regression fixtures for consumed H14/H15/H16, timing-unavailable, and successful exact-calibration cases.

## Notes

- Fresh Holdout A and B remained uninspected, unprobed, unrendered, and unheard. This contract enabler claims no musical-quality progress.
