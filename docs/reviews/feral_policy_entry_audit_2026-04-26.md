# Feral Policy Entry Audit 2026-04-26

## Scope

This audit starts `P009 | Feral Policy Layer` after `P008 | Scene Brain` closed for bounded MVP.

Reviewed:

- `docs/execution_roadmap.md`
- `docs/phase_definition_of_done.md`
- `docs/specs/source_graph_spec.md`
- `docs/specs/preset_style_spec.md`
- `docs/specs/validation_benchmark_spec.md`
- `plan/riotbox_liam_howlett_feral_addendum.md`
- current Source Graph, Session, Scene, W-30, TR-909, and MC-202 seams

## Current Useful Seams

Riotbox already has enough structure to start Feral policy without a fork:

- Source Graph assets already include `HookFragment`, `DrumAnchor`, `PhraseFragment`, and `TextureFragment`.
- Source Graph candidates already include `KickAnchor`, `SnareAnchor`, `GhostHit`, `FillFragment`, `LoopCandidate`, `HookCandidate`, `AnswerCandidate`, and `CaptureCandidate`.
- Source Graph relationships already include `SupportsBreakRebuild`, `HighQuoteRiskWith`, and `GoodFollowupTo`.
- `AnalysisSummary` already exposes `break_rebuild_potential`.
- W-30 already has source-window capture artifacts, focused pad playback, and internal resample artifacts.
- TR-909 already has reinforcement, fills, slam, takeover, source-support profile, pattern adoption, and phrase variation.
- MC-202 already has contour, hook-response restraint, pressure, instigator, and mutation output proof.
- Scene Brain already has replay-safe movement state and bounded render movement.

## Architecture Boundary

The Feral layer must stay a policy/scoring/profile layer.

Allowed:

- derive scorecards from existing Source Graph objects
- add profile-aware view models and TUI diagnostics
- bias existing queue/commit actions
- add explicit session/profile state when behavior affects replay
- add output-path proof when policy changes audible behavior

Not allowed:

- second source graph
- second sampler
- second arranger
- callback-only feral heuristics
- Ghost autonomy as the first Feral implementation
- end-to-end style-transfer behavior hidden behind one command

## Key Constraint

The addendum states: new scores need consumers.

Therefore the first implementation should not add raw `feral_potential`, `bite_score`, or `quote_risk` fields in isolation. It should add a small view/projection that consumes existing graph evidence and makes the result visible to the musician.

## Recommended First Slice

Add a bounded `FeralScorecardView` projection from existing Source Graph data.

Minimum fields:

- `break_rebuild_potential`
- `hook_fragment_count`
- `break_support_count`
- `quote_risk_count`
- `capture_candidate_count`
- `top_reason`
- `warnings`

Consumer:

- Source screen or Jam/Source summary should show one compact Feral line, for example:
  - `feral break high | hooks 2 | quote risk 1 | use W-30 capture first`

Tests:

- core/view test with Source Graph assets, candidates, and relationships
- app/UI test proving the scorecard is visible
- no audio-output claim yet, because this first slice is a projection/consumer slice

## Why This Slice First

It creates visible product progress without pretending Feral rebuild exists already.

It also sets up the next audio-producing Feral slice: W-30 or TR-909 can consume the scorecard to bias slice-pool browsing, capture promotion, or break reinforcement with explicit output-path proof.

## Immediate Follow-Up Candidate

After the scorecard is visible, the next implementation should consume one score in one existing action path.

Best candidate:

- use `break_support_count` / `break_rebuild_potential` to bias a bounded TR-909 `reinforce_break` or W-30 slice-pool target choice
- prove the chosen target changes control state and audible output

## Decision

Start P009 with a Feral scorecard projection and visible TUI consumer.

Do not start with Ghost, full harvest manifests, quote-risk enforcement, or a new break-rebuild engine.
