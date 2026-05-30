# Arrangement / Scene System Spec

Version: 0.1
Status: Draft
Audience: core, app, TUI, audio QA, replay

---

## 1. Purpose

P014 turns the existing Scene Brain seam into a bounded Arrangement / Scene
System without adding a second arranger, timing model, replay model, mix truth,
or Ghost/Feral-only path.

The first contract is intentionally small:

- Source Graph remains the source-structure authority.
- Source Timing summary remains the timing trust authority.
- Session `scene_state` remains the durable scene / restore / landed-movement
  state.
- Action Lexicon `scene.launch` and `scene.restore` remain the user and replay
  control surface.
- Queue / commit records remain the musical-boundary authority.
- Existing TR-909, MC-202, W-30, observer/audio, and QA seams remain the output
  proof surfaces.

## 2. Contract View

The shared Jam view exposes an `ArrangementSceneContractView`. It is a
read-only contract surface, not new product state.

It records:

- readiness for arrangement / scene expansion
- whether a Source Graph is present
- Source Timing consumer readiness
- whether source-locked scene movement may use source-window timing
- scene material state: scene count, active scene, next scene, restore scene
- whether a scene transition is pending
- whether a landed movement already exists in Session state
- the controlling truth source: Source Graph + Session + Action Lexicon +
  queue / commit
- proof obligations for P014 slices

Readiness values:

- `ready`: graph, scene material, and locked or user-confirmed timing are
  available
- `missing_source_graph`: no Source Graph is available
- `needs_scene_material`: not enough queueable scene material exists yet
- `needs_timing_evidence`: a graph exists but timing is unavailable
- `needs_timing_confirmation`: timing exists but needs explicit user trust
- `fallback_timing_only`: only fallback timing is available

`ready` does not mean a finished arranger exists. It means a later P014 slice may
attempt source-locked scene behavior without bypassing the product spine.

Source-locked scene movement uses a stricter timing gate than generic scene
readiness:

| Source Timing state | Contract readiness | `can_use_source_locked_scene_movement` | Source Monitor scene anchor |
| --- | --- | --- | --- |
| Analyzer locked | `ready` | `true` | may reposition to the landed scene section |
| User confirmed | `ready` | `true` | may reposition to the landed scene section |
| Needs user confirmation | `needs_timing_confirmation` | `false` | keep transport-position playback |
| Fallback grid | `fallback_timing_only` | `false` | keep transport-position playback |
| Unavailable / disabled / missing BPM | `needs_timing_evidence` | `false` | keep transport-position playback |
| Missing Source Graph | `missing_source_graph` | `false` | keep transport-position playback |

The movement flag is a timing-trust gate, not a second arranger state. Session
scene state, queue / commit records, and replay remain the durable truth.

## 3. P014 Rules

Every P014 arrangement or scene slice must:

- use existing `scene.launch` / `scene.restore` or document any new
  `ActionCommand` against the ActionCommand rule before implementation
- persist replay-, restore-, or product-contract state in core/session models,
  not in app-local state
- preserve P012 source-grid timing gates
- preserve P013 representative musical-quality gates
- prove control path through Session, queue, commit, replay, Jam view, or
  observer state
- prove output path for any audible arrangement or scene behavior
- label fallback, manual-confirm, explicit-BPM, and locked-grid behavior
  honestly instead of treating every source as grid-locked

## 4. Out Of Scope For The First Contract

The first P014 contract does not implement:

- a full arranger
- a separate Scene Graph
- untrusted or automatic source playback repositioning beyond landed scene
  sections
- strip / build / slam scheduling
- automatic scene chains
- source-derived MC-202 phrase planning
- final W-30 loop detection
- export arrangement packages

Those may land as later P014+ or P016+ slices only by extending this contract.
Manual scene-chain proofs may still use repeated `scene.launch` plus
`scene.restore` actions when they stay inside the existing Action Lexicon,
Session scene state, queue / commit records, replay executor, and output QA
surfaces. That is transition proof, not an automatic chain scheduler.
The first section-aware source playback seam may reposition Source Monitor
playback to the landed scene section only when Source Timing is analyzer-locked
or user-confirmed. Fallback and manual-confirm-only timing must keep transport
position playback until the user trust boundary is explicit.

## 5. Proof Baselines

Keep these green while P014 lands:

- `just p012-all-lane-source-grid-output-proof`
- representative source showcase musical-quality gates from P013
- `just audio-qa-ci`
- `just ci`

For a contract-only slice that changes no audible output, targeted core/app tests
plus docs/spec updates are sufficient. The first audible P014 slice must add
non-collapsed output evidence.

Current P014 transition proof:

- `cargo test -p riotbox-app p014_scene_chain_launch_restore_replay_proves_transition_state_and_mix -- --nocapture`
  proves a three-step manual scene chain (`scene.launch`, `scene.launch`,
  `scene.restore`) through Session scene state, queue / commit records,
  graph-aware replay, Jam view movement projection, and non-collapsed mixed
  TR-909 / MC-202 output deltas.

Current P014 source playback reposition proof:

- `cargo test -p riotbox-app source_monitor_scene_reposition -- --nocapture`
  proves section-aware Source Monitor repositioning from landed Scene movement
  and trusted Source Timing, including replay equivalence and a fallback-timing
  guardrail.

Current P014 observer / audio correlation proof:

- `just p014-scene-movement-observer-probe` proves a headless `scene.launch`
  user path through queue, Bar commit, Session/Jam landed movement projection,
  Source Monitor scene anchor evidence, observer NDJSON shape, and strict
  observer/audio JSON correlation with non-collapsed output metrics.
