# P014 Exit Candidate Review

Date: 2026-05-30
Status: exit candidate pending stacked PR CI / merge

## Scope Reviewed

P014 Arrangement / Scene System was reviewed as a bounded expansion of the
existing Scene Brain seam, not a full arranger.

The candidate stack proves:

- Arrangement Scene contract view rooted in Source Graph, Source Timing,
  Session scene state, Action Lexicon, queue / commit, replay, observer, and
  output QA
- manual scene-chain launch / launch / restore movement through Session,
  graph-aware replay, Jam view projection, and non-collapsed TR-909 / MC-202
  mix output
- section-aware Source Monitor repositioning from landed scene movement only
  when Source Timing is analyzer-locked or user-confirmed
- observer/audio correlation for landed scene movement, Source Monitor anchor
  evidence, and non-collapsed output metrics
- bounded extension contract that allows manual scene-chain expansion only when
  graph, scene material, and timing trust are ready, while automatic scene-chain
  scheduling remains out of scope for P014

## Evidence Commands

- `cargo test -p riotbox-app p014_scene_chain_launch_restore_replay_proves_transition_state_and_mix -- --nocapture`
- `cargo test -p riotbox-app source_monitor_scene_reposition -- --nocapture`
- `cargo test -p riotbox-core arrangement_scene_contract_preserves_timing_trust_matrix -- --nocapture`
- `cargo test -p riotbox-app --bin observer_audio_correlate scene_movement -- --nocapture`
- `cargo test -p riotbox-app --bin user_session_observer_probe p014_scene_movement -- --nocapture`
- `just p014-scene-movement-observer-probe`
- `just audio-qa-ci`
- `just ci`

## Exit Decision

P014 is an exit candidate, not yet formally closed in the roadmap.

Reason: implementation branches are pushed, represented as stacked PRs, and
locally green. Formal closure should happen after GitHub CI is inspected on the
remaining stacked P014 PRs and the stack is merged into `main`.

## Deferred Beyond P014

- automatic scene-chain scheduler
- full arranger / separate Scene Graph
- strip / build / slam scheduling
- source-derived MC-202 phrase planning
- final W-30 loop detection
- export arrangement packages

These require explicit Action Lexicon, Session/replay, observer, and output-QA
contracts before implementation.
