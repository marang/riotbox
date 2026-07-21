# `RIOTBOX-1404` P023: Expand the human-passed live alpha to tonal hook and sparse pressure

- Ticket: `RIOTBOX-1404`
- Title: `P023: Expand the human-passed live alpha to tonal hook and sparse pressure`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1404/p023-expand-the-human-passed-live-alpha-to-tonal-hook-and-sparse`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-21`
- Finished: `2026-07-21`
- Branch: `feature/riotbox-1404-p023-controlled-source-expansion`
- Linear branch: `feature/riotbox-1404-p023-expand-the-human-passed-live-alpha-to-tonal-hook-and`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`
- PR: `#1372 (https://github.com/marang/riotbox/pull/1372)`
- Merge commit: `86eee84a552e879635b485fc86ba4125a4c208b0`
- Deleted from Linear: `2026-07-21`
- Verification: `just ci; cargo test; cargo clippy --all-targets --all-features -- -D warnings; just controlled-source-live-matrix artifacts/audio_qa/local-riotbox-1404-gated-controlled-matrix; GitHub rust-ci`
- Docs touched: `docs/specs/audio_core_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/preset_style_spec.md; docs/engineering/audio_numeric_values.md; docs/execution_roadmap.md; docs/jam_recipes.md; docs/research_decision_log.md`
- Follow-ups: `Continue with the next bounded P023 slice; RIOTBOX-1403 and RIOTBOX-1405 remain roadmap candidates.`

## Why This Ticket Existed

Prove that the human-passed dense live alpha generalizes to tonal-hook and sparse-pressure sources through the same exact product spine without copying one scripted arrangement.

## What Shipped

- Typed dense_break, tonal_hook, and sparse_pressure live character policy derived from trusted persisted phrase-audio evidence.
- Exact callback-path controlled-source matrix with same-source determinism, cross-source diversity, zero clipping, and zero limiter concealment.
- Human-kept tonal and sparse held loops plus character-specific destructive variants; sparse destruction uses a grid-locked 1.0x source chop/choke that preserves rhythm and removes confusing intermediate kicks.

## Notes

- Generated manifests remain diagnostic and human_verdict unverified by design; separate listening-review records carry the human verdicts.
