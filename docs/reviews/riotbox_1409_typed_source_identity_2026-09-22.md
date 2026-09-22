# RIOTBOX-1409 — Typed source and Scene identity

Classification: maintenance/regression. Review base `7c1f9bdc` (including the
RIOTBOX-1414 archive). Scope: Core Graph/Session identity, their App/replay/lane
consumers, compatibility tests and the owning specs. This is not a new audible
mechanism, UI feature, source qualification or DAW handoff.

## Contract and ownership

- Session `SourceRef.decode_profile` shares the existing Graph `DecodeProfile`.
  A Session-only codec retains lower-case legacy standard strings and supported
  custom names. Colliding custom names use an explicit Custom representation.
  Profile disagreement makes source audio unavailable before opening its path;
  admitted audio retains the single-buffer hash/decode identity check. RBX-377
  records the ambiguity rule; Graph profile serialization is unchanged.
- Graph endpoints are `GraphNodeRef::{Source, Section, Asset, Candidate}`.
  A private deserialize-only wire boundary resolves original V1 string IDs from
  actual catalog membership, never prefixes. Unknown, malformed, ambiguous and
  wrong-kind endpoints fail instead of being guessed. Serialization validates
  in-memory edits and retains V1 field order/values/defaults. No second graph,
  external namespace or unowned Capture/Bar node is introduced.
- Core Session `SceneSourceBinding` owns explicit Scene/Source/Section identity.
  Labels and array order do not retarget it. Missing bindings enable only the
  centralized V1 compatibility adapter; explicit empty or invalid bindings do
  not. App construction/load/save and graph-aware replay materialize migration,
  including old snapshot payloads, stored action refs and supplied replay refs.
  Load does not rewrite disk. Source Monitor, TR-909, MC-202, Scene Brain and
  replay consume the same Core resolver. Existing display-only first-section
  energy summary fallback is not playback identity. RBX-378 owns these rules.

No new ActionCommand, queue/commit path, scheduler, replay history, dependency or
callback work was added. Existing performer/output behavior is the preservation
target. New modules own semantic identity/compatibility boundaries; no textual
include shard was added. Existing Rust APIs taking string endpoints or graph-only
scene context intentionally become typed; repository callers are migrated.

## Verification

The original unknown-profile restore regression was red: a conflicting legacy
profile admitted source audio. The corrected tests cover legacy standard/custom
roundtrips, reserved custom-name ambiguity, malformed profiles, metadata rejection
before file I/O, and unchanged Graph profile spelling.

Graph tests compare serialized bytes with the V1 raw-string wire shape and check
roundtrip stability, prefix-independent kind resolution, malformed/unknown IDs,
cross-kind and same-kind ambiguity, and wrong-kind/dangling in-memory save
rejection. Four synthetic fixture builders previously removed graph nodes while
retaining obsolete relationships; they now remove those obsolete links too,
without changing the asserted musical/render behavior.

Scene tests cover opaque IDs, legacy missing versus explicit empty bindings,
label/order changes, duplicate/mismatched bindings, public restore/save/reload
with identical Graph bytes and hash refs, failed save leaving disk unchanged,
invalid replay leaving the entire Session unchanged, and action-only historical
aliases. Existing snapshot-suffix, multi-lane and source-monitor timing/output
tests remain part of the integrated suite. UI fixtures that replace the cloned
scene catalog explicitly model an unmigrated legacy state before construction.

Full normal-parallel, source-free `just ci` passed in
`/tmp/riotbox-1409-ci-final.log`: 767 App library, 269 Audio library, 470 Core and
14 Sidecar tests, plus binary/doc/Python tests, generated-source live-path and
observer/audio gates, contract checks, formatting and strict Clippy. Two existing
informational benchmarks remain ignored. The full final-state rerun including
the review corrections also passes in `/tmp/riotbox-1409-ci-reviewed.log`.
`scripts/check_no_textual_includes.sh` and `git diff --check` pass.

No real source, Holdout, commercial reference, audio device, Bitwig or human
playback was accessed. New WAV tests create their own temporary synthetic files.
Rave-punk guidance is a preservation constraint, not a new taste verdict.

## Review findings and disposition

Review uses `code-review` / `code-review-rust` with sequential solo lenses,
not independent reviewers or subagents.

1. **P2 — Snapshot migration initially diverged from live construction.**
   Lens: Spec And Evidence Auditor. In
   `crates/riotbox-core/src/replay/scene_movement.rs:13`, restoring an old payload
   could keep missing bindings while the App-created equivalent had explicit
   bindings. Existing TR-909 and stage-style convergence assertions failed.
   **Fixed:** graph-aware Core replay migrates and validates its working Session
   before execution; the original parity assertions pass without weakening them.
2. **P2 — Action-only legacy aliases could lose their historical target.**
   Lens: Adversarial Implementation Reviewer. The initial migration in
   `crates/riotbox-core/src/session/scene_source_binding.rs` saw only current scene
   state; an old `scene-02-historical-label` appearing only in history/suffix
   would become unbound after migration. **Fixed:** include action target/params,
   commit-boundary refs and transport context, deduplicated before resolution.
   `legacy_replay_migrates_action_only_scene_alias_before_explicit_binding_gate`
   verifies both stored history and separately supplied replay entries.
3. **P3 — Typed W-30 matching should retain indexed lookup cost.**
   Lens: Performance And Operations. An intermediate Vec scan in
   `crates/riotbox-app/src/jam_app/w30_targets.rs` would compare every hook asset
   against every support relation. **Fixed:** retain a typed BTreeSet of Asset
   refs and a separate any-relation flag, preserving both lookup complexity and
   the original empty-relation behavior.

Serialization additionally validates typed graph edits before flattening their
IDs; negative tests prevent silently changing endpoint kind at save/load.
No further retained finding in the follow-up self-review; the final local CI
gate is green. No live-device, musical, source-general, hardness or
human-listening qualification is claimed.
