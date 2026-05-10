# RIOTBOX-690 Jam App Boundary Audit 2026-05-10

Context:

- ticket: `RIOTBOX-690`
- scope: `crates/riotbox-app/src/jam_app.rs` and `crates/riotbox-app/src/jam_app/`
- trigger: verify the current `jam_app` orchestration boundaries after the `RIOTBOX-140` module split
- review mode: current-state architecture audit, not diff review

## Summary

The old claim that `jam_app.rs` is still a monolith is stale. The current
`crates/riotbox-app/src/jam_app.rs` is a 104-line module root that declares and
wires focused submodules (`commit`, `persistence`, `projection`, `recovery`,
`side_effects`, lane queues, transport helpers, and test shards).

The remaining risk is not file size in the root. It is hidden coupling across the
module tree:

- broad `super::*` imports make module dependencies harder to see
- commit orchestration depends on an implicit side-effect order
- `runtime_view.rs` mixes musician-facing labels with invariant diagnostics for
  all lanes
- W-30 queueing remains a large orchestration seam because it owns target
  selection, action draft construction, conflict checks, and musician-facing
  explanation text

No broad rewrite is justified from this audit. The right follow-up is a small
set of targeted hardening slices.

## File-Size Snapshot

Largest files in the audited production scope:

```text
500 crates/riotbox-app/src/jam_app/runtime_view.rs
487 crates/riotbox-app/src/jam_app/w30_queue.rs
468 crates/riotbox-app/src/jam_app/recovery.rs
441 crates/riotbox-app/src/jam_app/projection/tr909_projection.rs
427 crates/riotbox-app/src/jam_app/w30_targets.rs
365 crates/riotbox-app/src/jam_app/persistence.rs
335 crates/riotbox-app/src/jam_app/capture_artifacts.rs
331 crates/riotbox-app/src/jam_app/capture_helpers.rs
246 crates/riotbox-app/src/jam_app/commit.rs
104 crates/riotbox-app/src/jam_app.rs
```

Largest test shards are below the soft 500-line budget, but the full test scope
is still large and held together by textual includes.

## Findings

### 1. Root module is now a wiring point, but imports still hide coupling

- Location: `crates/riotbox-app/src/jam_app.rs:1`
- Category: scope
- Severity: minor
- Title: module root still exports a broad implicit dependency surface
- Description: The root imports app, audio, core, sidecar, hashing, filesystem,
  and time concerns in one place, then many child modules pull from that root via
  `use super::*` (`commit.rs:1`, `persistence.rs:1`, `recovery.rs:1`,
  `w30_queue.rs:1`, `transport.rs:1`, `tr909_queue.rs:1`, and others). This
  keeps the split compact, but it makes per-module boundaries harder to review.
- Suggestion: Do not rewrite all imports mechanically. When touching a module
  for real behavior, replace `super::*` with explicit imports for that module or
  introduce a narrowly named internal prelude only for truly shared types.

### 2. Commit orchestration has a real implicit ordering contract

- Location: `crates/riotbox-app/src/jam_app/commit.rs:94`
- Category: scope
- Severity: major
- Title: committed action pipeline mixes log persistence, capture artifacts,
  lane side effects, scene side effects, ghost state, and transport sync
- Description: `commit_ready_actions` logs committed actions and records commit
  metadata, then `apply_committed_action_side_effects` persists captures,
  modifies action results, applies W-30, MC-202, TR-909, Scene, and Ghost side
  effects, and finally mirrors scene state back into runtime transport. This is
  a valid orchestration boundary, but the ordering is semantic and currently
  implicit.
- Suggestion: Add a bounded hardening slice that documents or encodes the commit
  pipeline order as named steps. The goal is not to move behavior out of
  `JamAppState`, but to make side-effect ordering testable and reviewable.

### 3. Runtime view mixes presentation strings with diagnostics

- Location: `crates/riotbox-app/src/jam_app/runtime_view.rs:65`
- Category: scope
- Severity: major
- Title: runtime view builder is the next review-cost hotspot
- Description: `JamRuntimeView::build` derives all lane summaries, audio/sidecar
  status, replay readiness labels, and warning lists. Later helpers in the same
  file derive TR-909, MC-202, W-30 preview, and W-30 resample warnings
  (`runtime_view.rs:352`, `runtime_view.rs:421`, `runtime_view.rs:443`,
  `runtime_view.rs:473`). This is cohesive as "runtime view", but it joins two
  responsibilities: musician-facing labels and internal invariant diagnostics.
- Suggestion: Split by semantic responsibility when the next runtime-view change
  lands: one path for lane summary labels, one path for runtime diagnostic
  warnings. Keep output types unchanged.

### 4. W-30 queueing is cohesive but overloaded

- Location: `crates/riotbox-app/src/jam_app/w30_queue.rs:3`
- Category: scope
- Severity: minor
- Title: W-30 queue methods combine target selection, conflict policy, draft
  construction, and user-facing explanation text
- Description: W-30 queueing is no longer a root-module problem, but the module
  owns many gestures end-to-end. It depends on selection helpers in
  `w30_targets.rs:27`, constants on `JamAppState` in `jam_app.rs:97`, and direct
  `ActionDraft` construction throughout `w30_queue.rs`.
- Suggestion: Avoid a premature split. If more W-30 gestures are added, extract
  a small W-30 action-draft helper around repeated `ActionDraft` construction
  and explanation formatting. Keep target selection in `w30_targets.rs`.

### 5. Test split improved token cost, but textual includes keep one namespace

- Location: `crates/riotbox-app/src/jam_app/tests.rs:1`
- Category: scope
- Severity: minor
- Title: test shards are file-sized well, but dependency boundaries remain
  implicit
- Description: The test tree avoids one giant 7k-line file, and current shards
  stay under the soft 500-line budget. However, `tests.rs` includes every shard
  textually, and `common_imports_fixtures.rs` imports the parent module broadly.
  This preserves historical test names, but it also means test dependencies are
  less explicit than normal `mod`-based test modules.
- Suggestion: Leave this alone unless tests become hard to navigate again. If
  future test churn grows, convert one behavior family at a time to normal
  nested modules with explicit fixture imports.

## Positive Boundary Notes

- `projection/` is already separated by lane-facing render-state derivation.
- `side_effects/` already separates lane and Ghost/Scene mutation functions.
- `recovery/` has moved specialized guidance into child modules.
- `capture_artifacts.rs` isolates artifact hydration and persistence concerns
  better than keeping them in commit code.
- The current split is aligned with the repo rule against mechanical file
  splitting: production files are close to, but not meaningfully over, the soft
  review budget.

## Recommended Follow-Ups

Create only bounded follow-ups, not another broad "refactor jam_app" ticket:

1. Commit pipeline hardening: make the side-effect ordering in
   `commit.rs` explicit through named steps and targeted tests.
2. Runtime view diagnostics split: separate invariant warning derivation from
   musician-facing summary label construction without changing UI output.
3. Import-boundary cleanup opportunistically: replace `super::*` with explicit
   imports only in modules already being touched for meaningful work.

## Conclusion

`RIOTBOX-140` achieved its main goal. `jam_app.rs` should no longer be treated
as a monolithic hotspot. The remaining architecture work is selective hardening
of orchestration seams, especially commit ordering and runtime-view diagnostics.
