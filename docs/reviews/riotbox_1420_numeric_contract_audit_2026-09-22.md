# RIOTBOX-1420 numeric-contract audit and branch review

Work class: maintenance/regression. Base: `f35ef3a8`.
Scope: Audio/Core numeric owners, exact RuntimeMix producer/validators and
professional-suite policy, as bounded in the
[inventory](../engineering/audio_numeric_inventory_2026-09-22.md).
No runtime DSP, recipe values, source registry or frozen Stage-A JSON changed.

## Results

- Corrected the shell Fill-exit predicate to the existing Rust three-way strict
  conjunction; a supported downbeat with step `.3`, local ratio `5`, attack
  ratio `2` was rejected by the previous two independent maxima and is accepted
  by the corrected predicate. All-three-exceeded still fails.
- Named/exported the unchanged Alpha correlation maximum and aligned inclusive
  equality, including negative polarity and old V1 manifests without the field.
  An explicit malformed field never takes the legacy compatibility route.
- Removed source-fixture format assumptions from existing-render validation;
  Graph/manifest format must agree, and window frames derive from stored rate.
- Unified exact-limiter evidence checks and twelve genuinely shared suite
  thresholds. Kept distinct metric definitions and source-modulation formula
  oracles; no global literal-deduplication or new runtime configuration system.
- Accepted inherited limiter values as explicitly provisional (RBX-379), with
  RIOTBOX-1501 calibration follow-up. RIOTBOX-1502 tracks the separate one-versus-
  two reverse-gesture suite/spec mismatch; no threshold retuning or historical
  verdict rewriting was hidden in this audit.

## Verification

- `just exact-mix-numeric-contract-fixtures`: 10 tests, including 27 Fill-boundary
  combinations, invalid/nonfinite metadata, stored-format variations, Alpha
  equality/legacy behavior, clean-path counts and suite producer/consumer
  inclusive-boundary agreement. Runs inside source-free PR QA.
- `cargo test -p riotbox-app --bin dense_break_live_path_render`: passed; added
  Rust f32 equality/adjacent-value checks for both waveform polarities.
- Fresh generated dense pack and `--validate-existing` on that exact pack:
  passed. No reused real-source artifacts were opened.
- Normal parallel `just ci`: passed (`/tmp/riotbox-1420-ci.log`), including 767
  App, 269 Audio, 470 Core and 14 Sidecar library tests; 19 dense-render binary
  tests; synthetic exact RuntimeMix/observer evidence; Python contracts;
  formatting and strict all-target/all-feature Clippy. Existing two informational
  benchmarks remained ignored. `git diff --check` and textual-include guard pass.

## Sequential solo review

The user-requested no-subagent constraint remains active. These are separate
review lenses applied by the implementing agent, not independent approvals.

- Maintainer / module ownership: small semantic jq evidence seam and Python
  suite-policy owner; no Rust textual shards or product-spine duplication.
- Product pragmatist: existing source/timing evidence, not fixture rate, owns
  reused artifacts. Diagnostic checks do not become sound-quality verdicts.
- Spec and evidence: explicit equality passport; corrected the inventory's
  held-mix comparator to inclusive `>=` after tracing its producer. Uncalibrated
  limiter and reverse-count mismatch remain visible follow-ups.
- Adversarial implementation: tests exposed jq `isfinite` accepting NaN;
  `finite_number` now also excludes `isnan`. Missing/typed-invalid thresholds
  cannot silently pass or use legacy defaults. Zero limiting remains mandatory.
- Rust / performance: only naming an unchanged f32 QA constant, additive
  reporting and a unit test; no callback allocation/lock, sample synthesis,
  Session, replay or timing changes.
- Risk assessor: validation consumes recorded metrics; it does not authenticate
  arbitrary manifests or establish musical approval. Historical Alpha metadata
  compatibility is explicitly scoped, not a general missing-evidence fallback.

No retained new branch-local blocker after corrections. Full production-suite
generation, real-source calibration, host-device testing and human listening
were deliberately not run. The audit closes numeric ownership/documentation
work, not RIOTBOX-1501/1502 or P023 audible acceptance.
