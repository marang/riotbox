# `RIOTBOX-1425` P023: Make W-30 hit-shaper reachability source-adaptive

- Ticket: `RIOTBOX-1425`
- Title: `P023: Make W-30 hit-shaper reachability source-adaptive`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1425/p023-make-w-30-hit-shaper-reachability-source-adaptive`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-07-27`
- Started: `2026-07-27`
- Finished: `2026-07-27`
- Branch: `feature/riotbox-1425-p023-make-w30-hit-shaper-reachability-source-adaptive`
- Linear branch: `feature/riotbox-1425-p023-make-w30-hit-shaper-reachability-source-adaptive`
- Assignee: `Owner`
- Labels: `Audio`, `Feature`, `review-followup`
- PR: `#1379 (https://github.com/marang/riotbox/pull/1379)`
- Merge commit: `e99a3e56bb8680402e5eac2f9492e9c8347cd2cc`
- Deleted from Linear: `2026-07-27`
- Verification: `focused selector regressions; eight-source development matrix; exact Beat03 non-listening preflight; cargo fmt; cargo test; cargo clippy; just ci; GitHub Rust CI`
- Docs touched: `docs/benchmarks/w30_hit_shaper_reachability_development_v1.json; docs/engineering/audio_numeric_values.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/fixture_corpus_spec.md; docs/research_decision_log.md; docs/reviews/riotbox_1425_w30_hit_shaper_reachability_review_2026-07-27.md`
- Follow-ups: `RIOTBOX-1422: consume frozen H22 exactly once through preflight and proceed to structured listening only if unchanged exact-path gates pass.`

## Why This Ticket Existed

Make exact W-30 source_hit_shaper_v3 reachability source-adaptive before spending another frozen holdout, without weakening source-ownership gates.

## What Shipped

- Replaced aggregate low-impact averaging with deterministic source-local transient and following-body evaluation.
- Carried typed transient-low-body role, decision, selected window, and gate evidence through product state, realtime snapshot, observer, and preflight.
- Froze an eight-source / seven-family development matrix while leaving H22 untouched and human_verdict unverified.

## Notes

- Contract enabler only; no candidate WAV or musical-quality claim. The transient-low-body role is explicitly not bass ownership.
