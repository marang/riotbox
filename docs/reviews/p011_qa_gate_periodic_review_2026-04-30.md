# P011 QA Gate Periodic Codebase Review

Date: 2026-04-30
Scope:

- `crates/riotbox-app/src/bin/observer_audio_correlate.rs`
- `scripts/validate_observer_audio_summary_json.py`
- `scripts/validate_user_session_observer_ndjson.py`
- `scripts/validate_stage_style_jam_probe.sh`
- Observer/audio correlation fixtures under `crates/riotbox-app/tests/fixtures/`
- P011 QA docs under `docs/benchmarks/`, `docs/specs/audio_qa_workflow_spec.md`, and `docs/phase_definition_of_done.md`

## Verdict

No architecture blocker found.

The current P011 QA gates still preserve the intended boundary: observer files prove
control-path behavior, audio QA manifests prove output-path behavior, and the
correlation helper joins those two evidence streams without touching realtime audio
or inventing a second replay model.

## Findings

### 1. Observer/audio JSON contract lagged the generated control-path fields

- Location: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:24`
- Location: `crates/riotbox-app/src/bin/observer_audio_correlate.rs:348`
- Location: `scripts/validate_observer_audio_summary_json.py:43`
- Category: scope
- Severity: minor
- Status: fixed in this review slice

`observer_audio_correlate --json` emits `control_path.commit_count` and
`control_path.commit_boundaries`, and the repo validator requires both fields.
The documented stable v1 contract did not list them, which made automation rules
harder to reason about.

The contract now lists both fields and the current CI-smoke expectations include
typed commit-count and boundary-array coverage.

### 2. Stable metric keys could be omitted while still passing validation

- Location: `scripts/validate_observer_audio_summary_json.py:60`
- Location: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md:35`
- Location: `Justfile:111`
- Category: scope
- Severity: minor
- Status: fixed in this review slice

The contract describes a stable summary metric surface where missing evidence is
encoded as `null`, but the validator previously treated an absent metric key the
same as a present `null` value. That weakened machine-readable compatibility:
downstream automation could not distinguish "producer intentionally emitted no
evidence" from "producer silently stopped emitting the key".

The validator now requires every stable metric key to be present as a number or
`null`, the valid failure fixture includes all stable keys, and a negative fixture
proves a missing metric key is rejected.

## Positive Checks

- `scripts/validate_stage_style_jam_probe.sh:49` asserts the stage-style summary
  includes the expected key outcomes, `commit_count >= 4`, and `NextBar`,
  `NextBeat`, plus `NextPhrase` boundary coverage before accepting output
  evidence.
- `docs/specs/audio_qa_workflow_spec.md:487` correctly describes the current
  observer/audio helper as a correlation gate that extracts launch, runtime,
  commit-count, boundary, manifest, artifact, and output metric evidence.
- `docs/phase_definition_of_done.md:169` keeps P011 marked active and explicitly
  describes the current stage-style gate as a bounded CI probe, not a soak test or
  full product exit gate.
- `scripts/validate_user_session_observer_ndjson.py:52` gives the committed
  observer fixtures an explicit schema/shape validator, including recovery
  snapshot details when present.

## Residual Risks

- The P011 stage-style gate is still synthetic and bounded. It is useful CI
  coverage, not real host-session audio capture, a long-run live soak, or
  perceptual listening approval.
- The observer/audio JSON contract is still a repo-local validator, not a formal
  JSON Schema document. That is acceptable for the current phase, but should be
  revisited once external tooling or multiple consumers depend on it.
- Strict observer/audio correlation validates manifest envelope evidence today,
  but broader live observer-vs-audio correlation across documented recipes remains
  explicitly future work in the audio QA spec.

## Recommended Follow-Ups

- Keep expanding stage-style QA only when it catches a named P011 risk, such as a
  longer action sequence, a replay-family convergence gap, or a real user-session
  observer mismatch.
- Do not call P011 exit-ready until the current bounded gates are complemented by
  full replay execution, guided recovery, export reproducibility, and long-run
  stage behavior.
