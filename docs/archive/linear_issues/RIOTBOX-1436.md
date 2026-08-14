# `RIOTBOX-1436` P023: Make live TR-909 Slam land harder with collision-local impact pockets

- Ticket: `RIOTBOX-1436`
- Title: `P023: Make live TR-909 Slam land harder with collision-local impact pockets`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1436/p023-make-live-tr-909-slam-land-harder-with-collision-local-impact`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-14`
- Started: `2026-08-14`
- Finished: `2026-08-14`
- Branch: `feature/riotbox-1436-p023-make-live-tr-909-slam-land-harder-with-collision-local`
- Linear branch: `feature/riotbox-1436-p023-make-live-tr-909-slam-land-harder-with-collision-local`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `timing`
- PR: `#1392 (https://github.com/marang/riotbox/pull/1392)`
- Merge commit: `2339475f31642d8a031bcabdde291b0f2970da9c`
- Deleted from Linear: `2026-08-14`
- Verification: `targeted core/app/audio tests passed; cargo check --workspace --all-targets passed; just ci passed; GitHub rust-ci passed`
- Docs touched: `docs/benchmarks/tr909_impact_pocket_development_v1.json; docs/reviews/riotbox_1436_tr909_impact_pocket_development_2026-08-14.md; docs/research_decision_log.md`
- Follow-ups: `Any follow-up requires a genuinely new source-blind versioned causal mechanism and Linear-first slice; do not scalar-retune tr909_impact_pocket_v1.`

## Why This Ticket Existed

Test whether the RIOTBOX-1429 H-LAYER-1 collision-local-space hypothesis could make the existing performer-owned TR-909 Slam land clearly harder in the exact full mix.

## What Shipped

- Preserved the frozen v1 Development contract, three-source technical qualification, structured human weak verdict, and RBX-288 fail-closed decision.
- Reverted the inaudible DSP implementation before merge; no runtime behavior, quality claim, hardness claim, or Holdout access shipped.

## Notes

- All three Development sources passed technical gates, but the repeated dense full-mix A/B produced no perceptible contrast.
