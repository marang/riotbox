# `RIOTBOX-1430` P023: Build qualified source pool and deliver Stage-A listening candidates

- Ticket: `RIOTBOX-1430`
- Title: `P023: Build qualified source pool and deliver Stage-A listening candidates`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1430/p023-build-qualified-source-pool-and-deliver-stage-a-listening`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-10`
- Started: `2026-08-10`
- Finished: `2026-08-11`
- Branch: `feature/riotbox-1430-qualified-source-pool-listening`
- Linear branch: `feature/riotbox-1430-p023-build-qualified-source-pool-and-deliver-stage-a`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `Spike`, `review-followup`
- PR: `#1387 (https://github.com/marang/riotbox/pull/1387)`
- Merge commit: `c74956b3e1bb5c2c7c2dac90620abcbb1059b3b3`
- Deleted from Linear: `2026-08-14`
- Verification: `GitHub Rust CI passed cargo fmt, cargo test, the audio-QA smoke gate, and strict all-target/all-feature Clippy. The focused 46-test Percussive-Force suite and every new v3/v4/v5/v2 contract, mutation, and source-blind preflight passed. Branch review found no remaining blocking or major issue.`
- Docs touched: `docs/benchmarks/percussive_force_stage_a_*; docs/benchmarks/percussive_force_development_matrix_*; docs/reviews/riotbox_1430_*; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1428 must continue Stage A with a newly versioned causal mechanism; Stage B remains blocked.`

## Why This Ticket Existed

The first four-source Protocol-v2 attempt closed honestly without a candidate,
but that bounded batch did not satisfy the larger product objective: Riotbox
still needed a substantial lawful Development pool, fresh source qualification,
the complete Stage-A mechanism matrix, and a real human hardness verdict.

RIOTBOX-1430 was reopened to correct that scope without retuning Protocol v2 or
reinterpreting its terminal evidence. Historical PR #1385 and its merge
`d00fc35cde6482c7ee5218b1e6a2462862e2f107` remain the immutable v2 result.

## What Shipped

- Preregistered fifteen new CC0 Freesound candidates across dense-break,
  sparse-drums, and electronic-drums families, with distinct authors, exact
  identities, deterministic order, reserves, and bounded stopping.
- Admitted thirteen exact Development files. Two incompatible container/format
  examples remained explicit negative evidence; no parser widening was used to
  force them through.
- Ran one fresh Development-only qualification with the unchanged frozen
  Detector, Anatomy, source-feature, and source-contrast algorithms. Nine
  sources qualified individually.
- Selected the first valid frozen combination: DABROmusic, Dr.Skitz, Garzul,
  and Aikighost, spanning all required families, four authors, and the unique
  three-cluster contrast partition.
- Executed all 24 conditions in Matrix v6: three mechanism families by four
  sources by two events. Nineteen conditions failed renderer/basic gates, four
  more failed advanced screens, and one F2 condition reached human review.
- Versioned the attenuation-view screen after it exposed a gain-dependent event
  identity contradiction. No threshold, renderer, candidate, or source result
  was tuned; source/candidate raw event identity now remains frozen across
  diagnostic gain views.
- Prepared and technically verified the exact local A/B artifact. The reviewer
  heard the same 10-second-A / 10-second-B presentation twice and judged B
  perceptually near-identical.
- Bound `human_verdict=reject`, kept source recognition clear, and froze F2
  `f2_exact_complementary_three_band_v1` against scalar retuning or further
  playback.

## Bounded Outcome

- RIOTBOX-1430 completed the corrected source-pool and structured-listening
  objective, but produced no positive `percussive_hard` pass.
- The human rejection overrides mechanical eligibility. F2 v1 is useful
  negative calibration evidence, not a product-ready mechanism.
- No RuntimeMix, realtime callback, TUI, ActionCommand, Session, replay,
  Source-Graph, Ghost, Feral, or product-output behavior changed.
- No holdout audio, commercial reference, source-directory discovery,
  credential leakage, reusable downloader subsystem, or fallback music was
  introduced.
- RIOTBOX-1428 Stage B remains blocked. A further Stage-A attempt requires a
  newly versioned causal hypothesis and Decision-Log entry; it may not adjust
  the frozen F2 recipe from this source result.

## Links

- [Reopened qualification and Matrix-v6 review](../../reviews/riotbox_1430_stage_a_v5_qualification_pass_matrix_v6_freeze_2026-08-11.md)
- [Protocol v5](../../benchmarks/percussive_force_stage_a_protocol_v5.json)
- [Bound source set v1](../../benchmarks/percussive_force_stage_a_bound_source_set_v1.json)
- [Development Matrix v6](../../benchmarks/percussive_force_development_matrix_v6.json)
- [Historical Protocol-v2 rejection](../../reviews/riotbox_1430_stage_a_v2_development_qualification_rejection_2026-08-11.md)
