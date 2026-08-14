# `RIOTBOX-1428` P023: Reconstruct source impact events and gate local Hard takeover

- Ticket: `RIOTBOX-1428`
- Title: `P023: Reconstruct source impact events and gate local Hard takeover`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1428/p023-reconstruct-source-impact-events-and-gate-local-hard-takeover`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-02`
- Started: `2026-08-10`
- Finished: `2026-08-11`
- Branch: `feature/riotbox-1428-stage-a-percussive-force-gate`
- Linear branch: `feature/riotbox-1428-p023-reconstruct-source-impact-events-and-gate-local-hard`
- Assignee: `Markus`
- Labels: `review-followup`, `Spike`, `Audio`, `Feature`
- PR: `#1384 (https://github.com/marang/riotbox/pull/1384); #1388 (https://github.com/marang/riotbox/pull/1388)`
- Merge commit: `5491b146ad4d411c95a9c38fc245ea07b76d044d`
- Deleted from Linear: `2026-08-14`
- Verification: `The focused percussive-force suite, Matrix-v7 binary target, Protocol-v6/v7 mutation fixtures, Matrix-v7 validate-only contracts, cargo fmt, strict riotbox-audio Clippy, branch review, and GitHub Rust CI passed.`
- Docs touched: `docs/benchmarks/percussive_force_stage_a_protocol_v1.json; docs/benchmarks/percussive_force_stage_a_protocol_v6.json; docs/benchmarks/percussive_force_stage_a_protocol_v7.json; docs/benchmarks/percussive_force_development_matrix_v2.json; docs/benchmarks/percussive_force_development_matrix_v7.json; docs/benchmarks/source_holdout_rotation_v2.json; docs/reviews/riotbox_1428_stage_a_development_qualification_rejection_2026-08-10.md; docs/reviews/riotbox_1430_stage_a_v5_qualification_pass_matrix_v6_freeze_2026-08-11.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1434 terminated fail-closed; RIOTBOX-1435 canceled because its mandatory handoff was absent.`

## Why This Ticket Existed

Riotbox needed to execute the frozen source-blind Stage-A percussive-force gate
against a legally registered Development corpus, preserve every negative
result, and permit product integration only after an isolated candidate was
both mechanically valid and clearly harder in human review.

## What Shipped

- Frozen and validated the initial Protocol-v1, Matrix-v2, and Registry-v2
  contracts plus source-blind F1, F2, and F3-v2 implementations.
- Preserved the first bounded Development qualification rejection, then used
  RIOTBOX-1430's separately versioned qualified pool to execute the complete
  F1/F2/F3 matrix without changing frozen mechanisms from source results.
- Implemented and synthetically proved the structurally distinct F4
  source-native body-sustain hypothesis, froze Protocol v6/v7 and Matrix v7,
  and executed the complete declared Development matrix.
- Bound the sole surviving level-controlled F4 candidate to an exact technical
  preflight and structured human review. The listener reported perceptual
  near-identity, so F4 was frozen as terminal negative evidence.
- Routed the remaining natural-velocity research question to RIOTBOX-1434;
  that bounded control study also terminated fail-closed, leaving no valid
  implementation handoff for RIOTBOX-1435.

## Bounded Outcome

- Stage A ended without a human `percussive_hard` pass. F1, F2, F3-v2, and F4
  are negative research evidence, not product-ready mechanisms.
- No holdout audio or commercial reference was accessed.
- No Stage-B RuntimeMix, realtime callback, ActionCommand, Session, replay,
  Source-Graph, TUI, Ghost, or Feral integration was authorized.
- All pinned algorithms, thresholds, JSON contracts, exact candidates, and
  human verdicts remain immutable historical evidence. A future attempt needs
  a genuinely new source-blind causal hypothesis and versioned decision.

## Links

- [Stage-A rejection record](../../reviews/riotbox_1428_stage_a_development_qualification_rejection_2026-08-10.md)
- [Source pre-admission record](../../reviews/riotbox_1428_stage_a_source_pre_admission_2026-08-10.md)
- [Protocol v1](../../benchmarks/percussive_force_stage_a_protocol_v1.json)
- [Development matrix v2](../../benchmarks/percussive_force_development_matrix_v2.json)
- [Source registry v2](../../benchmarks/source_holdout_rotation_v2.json)
- [Protocol v7](../../benchmarks/percussive_force_stage_a_protocol_v7.json)
- [Development matrix v7](../../benchmarks/percussive_force_development_matrix_v7.json)
- [Final F4 matrix and listening record](../../reviews/riotbox_1430_stage_a_v5_qualification_pass_matrix_v6_freeze_2026-08-11.md)
- [Natural-velocity control closeout](../../reviews/riotbox_1434_natural_velocity_control_qualification_2026-08-11.md)
