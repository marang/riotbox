# P011 Evidence Gate Periodic Codebase Review

Date: 2026-05-07
Scope: P011 Pro Hardening evidence-gate layer after RIOTBOX-568 through
RIOTBOX-571.

Reviewed:

- `docs/benchmarks/p011_exit_evidence_manifest.json`
- `docs/benchmarks/p011_replay_family_manifest.json`
- `docs/reviews/p011_replay_recovery_exit_checklist_2026-04-30.md`
- `docs/phase_definition_of_done.md`
- `docs/specs/audio_qa_workflow_spec.md`
- `scripts/validate_p011_exit_evidence_manifest.py`
- `scripts/validate_p011_replay_family_manifest.py`
- `scripts/validate_product_export_reproducibility.py`
- `scripts/validate_stage_style_stability_proof.py`
- P011 replay, observer, and UI test hotspots over the soft review/context
  budget

## Verdict

No architecture blocker found.

The P011 evidence layer is still aligned with the intended boundary: evidence
indexes point at existing docs, validators, and CI-safe smokes; they do not
claim P011 is exit-ready; and they do not introduce a second replay, export, or
recovery architecture.

P011 should continue. The next work should tighten the evidence validator and
reduce review-cost hotspots before the next wave of replay and stage-style
hardening expands the same files further.

## Findings

### 1. Large P011 replay and observer shards are now review-cost hotspots

- Location: `crates/riotbox-core/src/replay/executor/tests/w30.rs:1`
- Location: `crates/riotbox-app/src/jam_app/tests/ghost_assist_queue.rs:1`
- Location: `crates/riotbox-app/src/jam_app/tests/w30_committed_preview_resample.rs:1`
- Location: `crates/riotbox-app/src/ui/tests/shell_state_fixture_snapshots.rs:1`
- Location: `crates/riotbox-app/src/bin/observer_audio_correlate.rs:1`
- Category: scope
- Severity: major
- Follow-up: `RIOTBOX-573`, `RIOTBOX-575`

Several P011-adjacent files are over the repo's soft 500-line review/context
budget:

- `crates/riotbox-core/src/replay/executor/tests/w30.rs`: 989 lines
- `crates/riotbox-app/src/jam_app/tests/ghost_assist_queue.rs`: 673 lines
- `crates/riotbox-app/src/jam_app/tests/w30_committed_preview_resample.rs`: 574 lines
- `crates/riotbox-app/src/ui/tests/shell_state_fixture_snapshots.rs`: 556 lines
- `crates/riotbox-app/src/bin/observer_audio_correlate.rs`: 516 lines

These files are not wrong because of size alone. The issue is that P011 replay,
recovery, and observer evidence keeps accumulating in the same places. Future
changes will be harder to review without loading broad, mixed-responsibility
test and helper shards.

Suggestion: split only by semantic responsibility. Good candidate boundaries
include replay executor W-30 behavior families, Ghost assist queue behavior
families, W-30 committed preview/resample cases, UI shell snapshot fixture
families, and observer/audio correlation parsing versus report emission. Do not
mechanically shard files just to chase a line count.

Follow-up status: `RIOTBOX-573` landed the core W-30 replay executor split.
`RIOTBOX-575` landed the app-level split for the Ghost assist queue, W-30
committed preview/resample cases, UI shell fixture snapshots, and observer/audio
correlation entrypoint.

### 2. P011 exit evidence validation does not prove referenced `just` recipes exist

- Location: `scripts/validate_p011_exit_evidence_manifest.py:102`
- Location: `scripts/validate_p011_exit_evidence_manifest.py:108`
- Location: `scripts/validate_p011_exit_evidence_manifest.py:110`
- Category: scope
- Severity: major
- Follow-up: `RIOTBOX-574`

The P011 exit evidence manifest validator checks that each proof path exists and
that each command starts with `just ` or `cargo test`. It does not validate that
the referenced `just` recipe exists in the current repo.

This leaves a drift hole: a typo such as `just product-export-reproducibility-smok`
could pass the manifest validator as long as the referenced proof path exists.
For an evidence index that is meant to keep P011 closeout machine-checkable, the
command reference should be checked too.

Suggestion: for `just ...` commands, validate the first recipe token against the
repo's available recipes, preferably through `just --summary` or another stable
recipe listing. Keep `cargo test ...` commands allowed without over-constraining
normal cargo arguments.

## Positive Checks

- `docs/benchmarks/p011_exit_evidence_manifest.json:5` explicitly says the
  manifest is not a declaration that P011 is complete.
- `docs/phase_definition_of_done.md:187` keeps P011 marked active and not
  exit-ready.
- `docs/specs/audio_qa_workflow_spec.md:678` describes
  `just p011-exit-evidence-manifest` as an evidence index, not an execution gate
  by itself.
- `docs/specs/audio_qa_workflow_spec.md:679` keeps product export
  reproducibility scoped to a deterministic generated-support pack, not full
  arrangement export.
- `docs/specs/audio_qa_workflow_spec.md:683` keeps stage-style stability scoped
  to a bounded CI-safe repeated-run smoke, not host-audio soak or long-run
  endurance validation.

## Residual Risks

- `scripts/validate_stage_style_stability_proof.py:63` builds a normalized proof
  object that still contains per-run artifact hashes at
  `scripts/validate_stage_style_stability_proof.py:101`. The stable output
  signal is the `stable_mix_sha256`, not necessarily the proof hash across
  independent working directories or regenerated observer artifacts. Treat the
  proof hash as run-local unless a later slice intentionally normalizes the
  per-run hashes out of the proof identity.
- The current P011 gate family remains CI-safe and bounded. It is useful
  hardening evidence, but it is still not a replacement for full replay
  execution, full product export, or live host-audio endurance validation.

## Recommended Follow-Ups

- `RIOTBOX-573`: split the largest replay/test hotspots by semantic
  responsibility before adding another broad replay family to those files. This
  landed for the core W-30 replay executor tests by separating cue/focus moves,
  promotion replay, capture replay, and artifact hydration planning.
- `RIOTBOX-574`: tighten the P011 exit evidence manifest validator so stale or
  misspelled `just` recipe references fail.
- `RIOTBOX-575`: continue with the remaining app-level Ghost/W-30/UI/observer
  hotspots after the core W-30 replay split lands. This landed by splitting
  Ghost queue/current/feed tests, W-30 committed preview versus resample tests,
  UI shell fixture/snapshot groups, and observer/audio correlation args, summary
  build, and summary render responsibilities.
- Continue P011 with the smallest next slice that converts smoke-level evidence
  into stronger replay, recovery, export, or stage-style proof without creating a
  shadow path.
