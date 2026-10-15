# RIOTBOX-1577 — incomplete preparation checkpoint

Date: 2026-10-13. Base: `8e0aeab5a269e78b42f46bcdea13a7e07d1775b0`.
Branch: `feature/riotbox-1577-prepare-a-versioned-generated-live-master-load-fixture`.
This is an unfinished, uncommitted prototype, not a final review or qualification.

## Implemented so far

- Shared private `ready_prefix` borrows the actual App facade and preserves the
  existing capture/audition/promotion/preset/MC-202 actions and helpers. Legacy
  diagnostic preparation retains its logging, family checks and full preflight.
- `--prepare-generated-live-master-load-v1 --output <absolute fresh directory>`
  admits no freeform source or host options. It exclusively creates its owner,
  generates only the known synthetic dense source, invokes normal App/Sidecar
  ingest, commits the existing prefix, saves and actually restores Session/Graph
  and capture identity, then checks recording plans and the shared offline mixer.
- The generator's optional `--only` selector leaves its default corpus unchanged.
- Proposed V1 JSON fixes one generated profile, 2/8/16 bars and 44.1/48-kHz stereo
  offline geometry. It is not yet accepted/final-pinned by a Decision Log entry.
- Reports keep `quality_proof=false`, `source_intelligence_qualified=false`,
  `host_execution_allowed=false`, and generated fixture provenance. All five
  callback contributors are named as a composite, not isolated lanes.
- Computed window metrics are retained if artifact publication fails; aggregate
  acceptance follows preparation-report publication. Further failure-path tests
  are still required.

## Evidence already executed

The extraction worker ran 19 existing default binary tests and 18 selected
calibration-feature tests, plus scoped strict Clippy and static body parity.
Logs: `/tmp/riotbox-1577-ready-prefix-{unit-green,calibration-regressions,clippy-green}.log`.
Those checks predate the completed prototype integration and are not full-branch
review or CI.

The integrated binary compiles and its selected binary tests pass
(`/tmp/riotbox-1577-check.log`, `/tmp/riotbox-1577-unit.log`). One preparation
run succeeds with all six 2/8/16-bar × 44.1/48-kHz cases:
`/tmp/riotbox-1577-prep-9TruYE/prepared/preparation.json`.
Run log: `/tmp/riotbox-1577-preparation-first.log`. These are exclusively generated
PCM and offline-control results, not actual callback/host or musical evidence.

Initial integration exposed four compile-time API/include assumptions, corrected
before that pass. Do not describe those as product regressions or RED/GREEN bug
proof. No debug or throwaway source was introduced.

## Work still required

- Audit input/provenance, contributor and identity claims against owning contracts.
- Add deterministic negative/occupied/stale/mutated/failure controls, including
  proof that computed metrics survive output errors without overwriting unowned
  artifacts. Review the offline artifact writer's ownership explicitly.
- Verify generator selection/default compatibility and generated-only integration
  reproducibility; keep logs and test attribution exact.
- Pin the final QA contract/semantic ownership in the Decision Log and owning docs
  only after review; no existing algorithm/threshold or recording-version retune.
- Independent full code/Rust/spec/adversarial review, short self-review, complete
  source-free local CI, PR/native CI, merge/main synchronization and Linear closeout.

RIOTBOX-1577 remains In Progress. No commit, push or PR has been made for this
slice. Preserve the entire dirty tree; neither reset nor discard the prototype.
Work pauses under the user's standing quality-stop instruction after repeated
avoidable path/API/integration mistakes; a fresh reassessment must precede more
implementation. RIOTBOX-1575 and RIOTBOX-1576 are already merged and Done.

## Access boundary

No actual audio runtime/device/service/host metadata, real source/capture/Holdout
or commercial audio, source-directory discovery, old ignored attempt, DAW,
playback, broad QA or Windows expansion was used. Only this fresh generated
owner and generated tests were opened. All previous host phases remain consumed.
Actual loaded-host execution needs a separate prospective contract and fresh
authorization; this preparation does not grant it. RIOTBOX-1041/1036 remain open
and the original 1572 ALSA trigger remains unexplained.

## Subsequent resumption

This file preserves the historical incomplete checkpoint, not current status.
The user explicitly resumed the work. The prior pause was overcautious, not a
demonstrated sustained quality or model failure. Corrections and current
verification are recorded in the
[2026-10-15 review](riotbox_1577_generated_load_preparation_2026-10-15.md).
