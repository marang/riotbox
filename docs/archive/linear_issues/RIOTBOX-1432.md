# `RIOTBOX-1432` P023: Replace W-30 Golden Path hook-selection heuristics with a source-evidence algorithm

- Ticket: `RIOTBOX-1432`
- Title: `P023: Replace W-30 Golden Path hook-selection heuristics with a source-evidence algorithm`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1432/p023-replace-w-30-golden-path-hook-selection-heuristics-with-a-source`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M2 | Live Dense-Break Golden Path`
- Status: `Done`
- Created: `2026-08-11`
- Started: `2026-08-14`
- Finished: `2026-08-14`
- Branch: `feature/riotbox-1432-w30-source-hook-selection`
- Linear branch: `feature/riotbox-1432-p023-replace-w-30-golden-path-hook-selection-heuristics-with`
- Assignee: `Markus`
- Labels: `review-followup`, `benchmark`, `Analysis`, `Audio`, `Feature`, `Improvement`
- PR: `#1391 (https://github.com/marang/riotbox/pull/1391)`
- Merge commit: `e23aadf3d520f784bc20c3f113222f297a5493a7`
- Deleted from Linear: `Pending fresh token-authenticated cleanup; retained as Done`
- Verification: `cargo test -p riotbox-core -p riotbox-audio -p riotbox-app`; `cargo clippy -p riotbox-core -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings`; `just ci`; `GitHub rust-ci`; `branch-level code review and self-review`
- Docs touched: `docs/README.md`, `docs/execution_roadmap.md`, `docs/jam_recipes.md`, `docs/phase_definition_of_done.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1432_w30_source_hook_selection_2026-08-14.md`, `docs/specs/audio_core_spec.md`, `docs/specs/session_file_spec.md`, `docs/specs/source_graph_spec.md`
- Follow-ups: `None; a future hook-selection retry requires a genuinely new research basis and a separately versioned decision`

## Why This Ticket Existed

The Golden Path could replay and perform committed W-30 capture audio, but the
hook window still came from the musician and transport boundary rather than a
typed source-evidence decision. This ticket tested whether two bounded,
explainable policies could select a stronger keeper without creating another
selection, persistence, or validation system.

## What Shipped

- Added typed per-bar hook evidence and two source-blind, versioned selection
  policies while preserving the existing Source Graph, capture, Session,
  replay, and RuntimeMix path.
- Persisted policy, selected source range, evidence values, scores, lift, and a
  typed decision reason in capture lineage.
- Ran the frozen baseline and both policies across exactly the registered
  dense-break, tonal-riff, and sparse-drums Development sources.
- Recorded the fail-closed `no_winner_fail_closed` result: dense and sparse
  retained baseline, while both policies collapsed to the same tonal decision.
- Retained `FeralBreakAlphaV2` on `transport_boundary_v1`; no unqualified
  candidate, fallback music, or competing product truth was promoted.

## Notes

- All nine exact RuntimeMix renders were non-silent and unclipped, but the
  frozen cross-source diversity gate failed.
- The failed technical prerequisite correctly prevented human playback and
  Holdout access; no commercial reference informed the result.
- The Linear issue is retained as `Done` until authenticated issue deletion is
  available after this archive handoff.
