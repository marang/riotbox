# RIOTBOX-1420: bounded audio numeric-contract inventory

Baseline: `f35ef3a8` (2026-09-22), plus the corrections reviewed in this branch.
Work class: maintenance/regression. This is a risk-directed inventory of shared,
output-affecting and acceptance-bearing contracts, not a claim that every literal
in the repository was calibrated. The [numeric guide](audio_numeric_values.md)
owns the passport vocabulary. Code and the exact versioned contract remain the
owners; this document is not another executable threshold registry.

Method: trace producers, serialized evidence, consumers and equality behavior
across Audio runtime, Core performance policy, App exact-mix QA, Python suite
production and shell/Python validators. Inspect repository text and synthetic
tests only. No real source, active holdout, commercial reference, device, DAW or
human listening was used. Historical evidence below is cited, not rerun.

## Dispositions

| Finding | Disposition |
| --- | --- |
| Rust Fill-exit rejection is `step > max && local_ratio > max && attack_ratio > max`; shell rejected either of the first two and ignored attack support | Fixed shared jq predicate; all three strict comparisons, inclusive equality, finite metadata; 27 boundary combinations tested. Values unchanged. |
| Alpha changed-return Rust accepts absolute correlation equal to `0.985_f32`; shell required `< 0.985` | Named/exported the unchanged Rust maximum; validator consumes it with `<=`. Missing field in historical V1 uses only the exact old f32 value; explicit null/invalid values fail. |
| Existing-render path assumed 44.1-kHz source and a 480-frame window | Compare source format to stored Graph; derive window frames from output rate and recorded milliseconds. The generator's 48-kHz output remains unchanged. |
| Dense and first-playable wrappers duplicated different limiter checks | One `scripts/exact_mix_numeric.jq` predicate, preserving the categorical **zero limited samples** contract and rejecting malformed counts. |
| Twelve identical suite producer/validator thresholds had two owners | Move unchanged to `scripts/professional_output_numeric_policy.py`; explicit imports, synthetic metadata regression at inclusive boundaries. |
| Limiter knee/ceiling have no located calibration rationale | Explicit provisional acceptance in RBX-379 / audio core spec; **RIOTBOX-1501** owns bounded calibration. No retuning here. |
| Hook/Chop spec says two reverse gestures; suite's named floor is one | **RIOTBOX-1502** owns contract reconciliation. Values and historical verdicts unchanged; a current suite pass does not prove the stricter textual claim. |
| Other repeated numbers (`0.92`, `0.985`, `0.10`, `1e-5`) | Keep separate when units/semantics differ. Do not couple waveform similarity, safety, silence, window activity, normalized controls or numerical tolerance. |

## Passports

Every group below records name, kind, domain, scope, comparator, provenance,
owner, evidence and change rule. Grouping is by semantic owner, not by equal
spelling. “Regression constrained” is not “human calibrated”.

### 1. Runtime signal safety and measurement

Owner: [`public_api_shell.rs`](../../crates/riotbox-audio/src/runtime/public_api_shell.rs),
`master_bus_limited_sample` / `OfflineAudioMetrics`.

| Semantic name | Kind / unit / scope | Comparator and provenance |
| --- | --- | --- |
| `MASTER_BUS_LIMITER_THRESHOLD = 0.92` | Safety, linear absolute sample amplitude, shared master bus | `abs(sample) <= knee` unchanged; strictly above enters stateless tanh knee. Inherited provisional engineering choice, RBX-379. |
| `MASTER_BUS_LIMITER_CEILING = 0.985` | Safety, linear absolute sample amplitude, same bus | Shaped magnitude capped at ceiling; not a true-peak limit. Inherited provisional, not Alpha correlation. |
| `CLIP_THRESHOLD = 1.0` | Measurement classification, normalized full scale | `abs(sample) >= 1.0` counts a clip, equality included; signal-domain definition. |
| `NEAR_CLIP_THRESHOLD = 0.98` | Diagnostic count, normalized sample amplitude | `abs(sample) >= 0.98`, equality included; provisional warning band, not limiter onset. |
| `peak_abs`, `rms`, `limited_sample_count` | Observed artifact values, amplitude / sample count | Generated per exact render, never tuning targets. Historical `0.916060388...` is a RIOTBOX-1402 observation, not a constant. |

Evidence: `runtime/tests/signal_metrics.rs`, `runtime/tests/render_parity.rs`,
source-monitor hot-sum tests; existing RIOTBOX-1402 evidence in the guide.
Change rule: shared runtime safety requires buffer/parity proof and the
RIOTBOX-1501 versioned calibration protocol before audible tuning. No inferred
speaker, hearing-safety, intersample-peak or device guarantee.

### 2. Callback capture bounds

Owner: [`live_master_capture.rs`](../../crates/riotbox-audio/src/runtime/live_master_capture.rs).
`LIVE_MASTER_MAX_INTERLEAVED_SAMPLE_COUNT = 16_777_216` is a resource-safety
count, not a frame/duration budget: `0 < frames * channels <= max` after checked
multiplication. Four-byte atomic slots bound the callback payload to 64 MiB;
finalization may allocate another copy on the control thread.
`LIVE_MASTER_CALLBACK_GAP_THRESHOLD_MICROS = 100_000` is a diagnostic time
boundary: gaps **strictly greater** increment a fault count. Equality passes.
Provenance: explicit bounded capture engineering contract, not proof of
acceptable musical latency. Evidence: adjacent allocation/gap/armed-capture
tests. Change rule: memory, callback/control lifecycle, fault reporting and
capture tests; real-device claims need real-device evidence.

### 3. Exact clean-path mix policy

Owner: [`dense_break_live_path_render/model.rs`](../../crates/riotbox-app/src/bin/dense_break_live_path_render/model.rs);
consumer: `scripts/exact_mix_numeric.jq` and the two exact-mix wrappers.

| Name / value | Kind, unit and scope | Acceptance |
| --- | --- | --- |
| `MIN_MIX_RMS = 0.01` | QA, linear RMS, candidate stages/modes | RMS strictly greater; equality fails. |
| `MIN_MONITOR_DELTA_RMS = 0.005` | QA, delta RMS, monitor/Alpha differences | Delta strictly greater; equality fails. |
| `MIN_ISOLATED_TR909_REGRESSION_RMS = 0.005` | QA, linear RMS, pre-live-gesture TR-909 stem | Strictly greater; not the later Fill state. |
| `MAX_SOURCE_MONITOR_SILENCE_RATIO = 0.05` | QA, sample fraction, Source monitor | Silence ratio at most maximum; equality passes. |
| `MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT = 0` | QA invariant, modified samples, exact clean path | No modified samples and zero pre/post clips. Not a tunable tolerance. |

Provenance: inherited regression floors and the explicit exact-clean-path
contract in automated QA. They reject weak/hidden-hot paths, not bad taste.
Evidence: source-free dense RuntimeMix smoke and first-playable correlation;
new metadata boundary tests. Change rule: producer owns and reports thresholds;
validators check domains and recorded values, not synthetic-source amplitudes.
Changing QA policy requires its own stated rationale, never a silent rescue of
a failed source result. Recorded thresholds do not authenticate a manifest.

### 4. Gesture difference and Fill boundary policy

Owner: [`manifest.rs`](../../crates/riotbox-app/src/bin/dense_break_live_path_render/manifest.rs),
`GestureQaPolicy`, perceptual and sequence-boundary metrics. All are diagnostic
QA, not musician control or a hardness pass.

| Scope | Units and acceptance |
| --- | --- |
| `w` trigger | Delta RMS `> .005`, delta peak `> .02`, relative RMS `> .05`, 1 beat. |
| `f` Fill | Delta RMS `> .005`, peak `> .05`, relative RMS `> .05`, 4 beats; relevant-window ratio `>= .15`, absolute correlation `<= .99`. |
| `s` Slam | Delta RMS `> .004`, peak `> .05`, relative RMS `> .05`, 1 beat; activity `>= .10`, absolute correlation `<= .99`. |
| Scene launch/return | Delta RMS `> .03`, peak `> .15`, relative RMS `> .20`, 4 beats. |
| Perceptual relevance | 10-ms windows; delta RMS must exceed both `.10 * candidate RMS` and `1e-5`. These define the metric; they are not silence or limiter constants. |
| Fill-exit click rejection | 10-ms local window; all of absolute step `> .20`, step/local-p99 `> 4`, step/attack-RMS `> 4` required together. Nonfinite evidence fails. Equality in any comparison is not rejected. |

Provenance: regression-constrained contrast / supported-transient discrimination;
not general human calibration. Evidence: manifest unit tests for waveform
polarity, localized spikes, supported downbeats/isolated clicks, nonfinite
samples; jq below/equal/above Cartesian boundary cases.
Change rule: preserve metric definition, export actual parameters and compare
at the same comparator. Threshold or metric recalibration needs a new bounded
QA decision and applicable source/listening gates.

### 5. Alpha diagnostic arc

Owner: [`alpha_manifest.rs`](../../crates/riotbox-app/src/bin/dense_break_live_path_render/alpha_manifest.rs).
`MAX_HOOK_TO_CHANGED_RETURN_CORRELATION = .985_f32` is an absolute waveform
correlation maximum (`<=`, including negative polarity), unrelated to the
limiter ceiling. Negative-space RMS `<= .001`, silence fraction `>= .95`, hard
return RMS `>= .05` are arc-specific regression gates. Steps `[26,30)` and
return `[30,31)` at 8 steps/beat are versioned diagnostic window positions, not
source-derived intelligence. Provenance: existing scripted Alpha contrast
contract; unchanged here. Evidence: exact synthetic arc and explicit equality/
legacy-field tests. Change rule: preserve the historical arc, version a changed
musical recipe and perform its audible review; additive threshold reporting
does not rewrite an existing manifest or verdict.

### 6. Source-character and normalized controls

Owner: [`live_performance_policy.rs`](../../crates/riotbox-core/src/live_performance_policy.rs).
`LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN = .10` is a normalized
classification margin with inclusive comparisons, not linear output gain.
Historical dense/tonal/sparse evidence and limitations are linked in the guide.
The `.72` dense TR-909 lead transient boundary, `.70` tonal / `.38` sparse W-30
levels, sparse drum `.84 + transient * .15`, sparse Slam `.65 + transient * .15`
and dense `.68 + transient * .16` / `.54 + transient * .16` are local policy
allocation parameters. They are normalized controls, not dBFS thresholds;
clamping and typed intents own their use. Comparators are classification or
control mapping, not universal QA pass/fail.

Evidence: adjacent Core policy tests plus historical controlled matrix, not a
new source-general proof. The dense wrapper retains its independent formula
oracle for the explicitly named source-modulation policy; equality tolerances
are floating-point comparison tolerances, not alternate control values.
Change rule: update the typed/versioned policy, replay-visible consequences,
contrasting authorized Development cases and structured listening if audible.
Do not export a generic “.10” or “.82” to couple unrelated controls.

### 7. DSP coefficients and versioned instrument vocabulary

Owners: [`tr909_fill_voice.rs`](../../crates/riotbox-audio/src/runtime/tr909_fill_voice.rs),
[`tr909_fill_recipe.rs`](../../crates/riotbox-audio/src/runtime/tr909_fill_recipe.rs),
[`render_tr909_w30_preview.rs`](../../crates/riotbox-audio/src/runtime/render_tr909_w30_preview.rs).

- Fill kick profile fields explicitly separate Hz (50-Hz floor, 82/150-Hz
  spans), decay per second (34/70 pitch decay) and amplitude weights (`.82`
  fundamental). Retrigger retained tails and `.96` envelope ceiling are voice
  DSP, not master limiting. `VOICE_SILENCE_THRESHOLD = 1e-5` retires voices;
  it is not perceptual-delta relevance merely because the literal matches.
- `PHRASE_DRIVE_BREAK_CUT_STOMP_V2.output_gain = .765` is recipe-local amplitude
  trim. V1 remains `1.0`; 32-step hit/choke/silence data and velocities belong
  to their typed recipe ID. No threshold comparator: these generate sound.
- W-30 transient-bite `.44` maximum step fraction, intensity `.82` and `.10`
  fade fraction describe gated time and shaping, not gain; zero gate disables
  this articulation. Their concrete provenance is recorded in the guide.

Provenance: named voice/recipe engineering choices, bounded V2 historical
calibration described in RIOTBOX-1402, not transferable “optimal” coefficients.
Evidence: Audio voice/tail, W-30 gate and recipe escalation tests, exact callback
render. Change rule: preserve historical recipes; new audible parameters need
versioned decision, authorized source contrast and human review. No DSP values
changed in RIOTBOX-1420.

### 8. Controlled and professional source-family diagnostics

Owners: `controlled_source_manifest.rs`,
`scripts/validate_controlled_source_live_matrix.py`,
`scripts/professional_output_numeric_policy.py`,
`scripts/validate_professional_output_suite_contract.py`.

| Name / scope | Kind and units | Acceptance and provenance |
| --- | --- | --- |
| Controlled held mix / audible lane / intentional stay-out | QA RMS | Mix `>= .01`, audible lane `>= .005`, stay-out `<= 1e-5`; distinct intent, not “all lanes must sound”. Existing controlled policy. |
| Controlled cross-source envelope | QA, signed correlation magnitude and linear RMS-envelope delta | `abs(corr) <= .95`, mean absolute delta `>= .01`; 20-ms envelopes. Regression diversity, not waveform or taste identity. |
| Suite source-first/support balance | QA, generated/source RMS ratios | Source-first `<= .08` plus masking headroom `>= .04`; support in `[.145,.46]`. Twelve shared suite values now have one named owner. |
| Rendered TR-909 contribution/body | QA, ratio / linear low-band RMS | Contribution `>= .050`; named default low-band floor `.0030` with profile-specific handling in producer. Evidence remains diagnostic. |
| Dense/tonal Hook/Chop | QA, RMS ratio / normalized margin / correlation | W-30/source `>= .22`, headroom margin `>= .10`, response delta `>= .35`, correlation `<= .92`, transient retention `>= .58`. Not the runtime `.92`. |
| Hook riff diversity | QA, counts / normalized velocity | Source offsets `>= 6`, hits `>= 10`, velocity span `>= .25`, reverse count currently `>= 1`; two-reverse textual discrepancy tracked by RIOTBOX-1502. |
| Sparse bass pressure | QA, Hz / energy ratios / share | Static distance `>= 1.75 Hz`, span `>= 17 Hz`, low-band lift `>= 2.70`, share `>= .36`, low/mid `>= 2.45`, dominance margin `>= .20`. |
| Destructive contrast | QA, RMS / transient ratios | Dropout/stutter `<= .0065`; stutter/hook `>= 1.55`, restore/hook `>= 1.60`, restore/pressure `>= 1.36`; documented source-relative alternative remains separate. |
| Selection retention / search evidence | QA, RMS ratio / case counts | Retention `.98` for character-window search versus `.60` for separate policy selection; searched `>= 3`, promoted `>= 1`, policy candidates `>= 3`, score lift `>= 0`. Distinct denominators/scopes. |

Provenance: inherited diagnostic policy and source-family regression contracts
in automated QA; not a fresh calibrated musical judgement. Evidence: existing
suite mutation fixtures and source-free producer/validator boundary tests here.
Change rule: a source-family floor requires a specific role rationale and
versioned evidence, not generic limiter deduplication. Production suite generation
is guarded source work and was **not** run by this audit.

### 9. Automated fitness, fixtures and frozen qualification

Owner: `scripts/validate_automated_musical_fitness.py`. `MIN_FULL_RMS = .015`,
`MIN_LOW_BAND_RMS = .006` and event density `[30,700]` are reject-only QA;
peak `>= .995` and identity correlation `>= .999` **fail**, unlike the inclusive
maxima above. Provenance: provisional deterministic rejection rules, not human
label calibration. Evidence: committed positive/negative fitness fixtures;
RIOTBOX-1208 owns judge calibration. Change rule: test bad-output rejection and
coverage, preserve `human_verdict: unverified` and no musical-pass inference.

Exact-mix source generator 44.1-kHz input / 132 BPM / 32 seconds and renderer
48-kHz stereo / 128-frame blocks are fixture/harness configuration, not source
identity rules. Artifact sample rate, Graph sample rate, confirmed timing and
source anchor must agree with the stored evidence instead of those defaults.
First-playable observer fixture timestamps remain deliberate fixture assertions;
observer and exact mix only share an action-contract comparison.

Frozen Stage-A protocol/matrix values remain owned by their exact versioned JSON
and owning decision. This audit neither rereads audio nor changes any detector,
anatomy, source contrast, mechanism, normalization, holdout registry or frozen
threshold. Historical rejected versions are not defaults for another run.
Change rule: new version and decision **before** accessing new evidence under
the authorized phase; numeric naming never authorizes qualification or access.

## Evidence and limits

`just exact-mix-numeric-contract-fixtures` is now part of source-free PR QA.
It checks producer/consumer value preservation, inclusive/exclusive boundaries,
all Fill predicate combinations, malformed/nonfinite data, legacy Alpha metadata,
stored source formats and categorical clean-path limiting. The exact synthetic
pack and normal `just ci` exercise the real wrappers; the branch review records
their results. No fixture result counts as listening, hardness, source-general
qualification, or a resolution of RIOTBOX-1501/1502.
