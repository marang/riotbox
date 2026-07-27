# Audio Numeric Values: Measurements, Thresholds, And Recipe Parameters

Status: Active engineering guide
Audience: musicians, contributors, reviewers, and coding agents

## Purpose

Riotbox contains many floating-point values. They are not all the same kind of
"magic number". A value may be:

- a measurement produced by one render;
- a runtime safety boundary;
- a QA acceptance threshold;
- a normalized musician control;
- a DSP coefficient;
- a versioned recipe parameter; or
- synthetic fixture input.

This guide explains how to tell those categories apart, how to read their
units, and how to change them without weakening the instrument or its evidence.
It is an index and interpretation guide, not a second source of truth. The
named Rust/Python owner, versioned manifest, and relevant specification remain
canonical.

## The RIOTBOX-1402 Example: `0.9161` Versus `0.92`

The final exact-path TR-909 Fill in the RIOTBOX-1402 four-source matrix reported:

| Value | Meaning | Kind | Canonical owner |
| --- | --- | --- | --- |
| `0.916060388...` | highest absolute pre-limiter sample measured in the rendered Fill | observed result | generated `gesture-manifest.json` |
| `0.92` | sample magnitude where the master-bus soft limiter begins changing samples | runtime threshold | `MASTER_BUS_LIMITER_THRESHOLD` |
| `0.985` | maximum output magnitude of that soft limiter | runtime ceiling | `MASTER_BUS_LIMITER_CEILING` |
| `0` | maximum permitted limited samples in this exact clean-path QA pack | QA policy | `MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT` |
| `0.765` | V2 Fill recipe-local output trim that helped produce the measured peak | versioned DSP/recipe parameter | `PHRASE_DRIVE_BREAK_CUT_STOMP_V2.output_gain` |

These values form a chain, but they are not interchangeable:

```text
V2 recipe output gain and all active lanes
                 |
                 v
rendered pre-limiter samples
                 |
                 +-- measured peak_abs = 0.916060...
                 |
                 v
soft-limiter knee starts at 0.92
                 |
                 +-- no sample crossed it
                 |
                 v
limited_sample_count = 0  -> exact clean-path gate passes
```

### What the values mean in dBFS

Riotbox render samples use linear floating-point amplitude, where `1.0` is
full scale. Convert positive peak amplitude to dBFS with:

```text
dBFS = 20 * log10(linear amplitude)
```

For this render:

- `0.916060388` is approximately `-0.7615 dBFS`;
- `0.92` is approximately `-0.7242 dBFS`;
- `0.985` is approximately `-0.1313 dBFS`;
- the distance from the measured peak to the limiter threshold is only
  `0.003939612` linear amplitude, or approximately `0.0373 dB`.

The Fill therefore passes the exact gate, but its peak margin is narrow. The
important evidence is not merely `0.9161 < 0.92`; all four real-source cases
produced zero clipped samples and zero limiter-modified samples, while the
musical escalation regression also passed.

### What happens above `0.92`

The master bus is not a hard clipper at `0.92`:

1. samples with magnitude at or below `0.92` pass unchanged;
2. samples above `0.92` enter a `tanh` soft knee;
3. the shaped result is capped at `0.985`;
4. a sample is counted in `limited_sample_count` whenever that process changes
   it;
5. ordinary clipping is separately counted at magnitude `>= 1.0`.

The limiter is valid runtime protection. The RIOTBOX-1402 exact-path gate is
stricter: it permits no limiter activity because otherwise a hot internal mix
could appear clean only after protection. That gate should drive a local mix or
recipe fix, not an automatic increase of the limiter threshold.

### Why the V2 Fill gain is `0.765`

`0.765` is not a universal Riotbox loudness target. It belongs only to the
typed `phrase_drive_break_cut_stomp_v2` primitive recipe.

During RIOTBOX-1402:

- the untrimmed combined Fill exceeded the exact clean-path limiter gate;
- `0.76` restored headroom but made the existing musical regression for a
  decisive late stomp fail narrowly;
- `0.765` kept that escalation regression green;
- the exact generated render and four real sources then peaked at
  `0.916060388...`, below the `0.92` knee, with zero limiter activity;
- V1 retained gain `1.0` and its historical focus, so V2 tuning did not rewrite
  the earlier review control.

This is an example of bounded calibration: satisfy both musical behavior and
headroom evidence without moving the safety threshold or silently changing a
historical recipe.

## Numeric Categories

### 1. Observed measurements

Examples: `peak_abs`, `rms`, `silence_ratio`, waveform correlation, onset count.

Measurements describe one artifact or comparison. They should be generated,
stored in a manifest/report, and cited with the exact source, render path, and
artifact hash when used for a decision. Do not copy a measured value such as
`0.916060388` into product code as a target.

### 2. Runtime safety boundaries

Examples:

- `MASTER_BUS_LIMITER_THRESHOLD = 0.92`;
- `MASTER_BUS_LIMITER_CEILING = 0.985`;
- `CLIP_THRESHOLD = 1.0`;
- `NEAR_CLIP_THRESHOLD = 0.98`.

These values define runtime signal handling or diagnostics. They need a named
owner and buffer-level tests. Changing one can affect every lane and must not be
used as a shortcut to make a single candidate pass.

The current repository has a named and tested owner for these values, but no
decision-log entry was found that calibrates why the limiter knee is exactly
`0.92` rather than a nearby value. Treat that choice as an inherited,
provisional engineering contract until a dedicated calibration decision records
reference material, devices, source matrix, and intended headroom.

### 3. QA acceptance thresholds

Examples in the exact dense-break path:

- `MIN_MIX_RMS = 0.01`: reject silent or extremely weak candidate mixes;
- `MIN_MONITOR_DELTA_RMS = 0.005`: require monitor states to differ;
- `MAX_SOURCE_MONITOR_SILENCE_RATIO = 0.05`: reject a mostly silent source
  monitor;
- `MAX_EXACT_MIX_LIMITED_SAMPLE_COUNT = 0`: reject reliance on the limiter;
- gesture-specific minimum delta RMS, peak, relative delta, activity, and
  maximum correlation values.

QA thresholds answer a precise yes/no question. Each threshold must state:

- metric and unit;
- comparison direction (`>=`, `>`, `<=`, or `<`);
- scope and source family;
- whether equality passes;
- evidence used to calibrate it;
- whether it proves technical observability or musical quality.

Passing an automated threshold never replaces a human musical verdict.

### 4. Normalized controls

Examples: velocity, pressure, touch, grit, slam intensity, lane level.

These commonly use a nominal `0.0..1.0` domain, but their audible response is
not necessarily linear. A velocity of `0.92` has no semantic relationship to
the limiter threshold `0.92`. Controls should be read through the owning typed
state and its renderer mapping.

### 5. DSP coefficients

Examples: gain multipliers, decay factors, filter coefficients, saturation
drive, crossfade widths, or edge-memory values.

DSP coefficients belong close to the algorithm they shape. Name them when the
meaning matters across more than one expression, document their stable range,
and cover boundary behavior. Do not place all DSP values into one global
"constants" file; locality and typed ownership are more valuable than numerical
deduplication.

### 6. Versioned recipe parameters

Examples: TR-909 step velocities and `output_gain` inside a selected Fill
recipe.

These values are instrument vocabulary, not general thresholds. They may remain
local data when the recipe ID, selection inputs, activation action, affected
RuntimeMix paths, and candidate artifacts are explicit. Once a recipe has been
used as a historical listening control, later tuning must receive a new version
or separate parameter set rather than mutating the old sound.

### 7. Fixture values

Examples: synthetic source amplitude, test BPM confidence, or a deliberately
hot signal used to exercise the limiter.

Fixture values prove a test condition. They must not leak into product policy or
be imposed on reused real-source artifacts. In particular, a validation wrapper
must read the stored source/timing identity instead of assuming the fixture's
BPM, sample rate, source ID, or anchor.

## Why Repeated `0.92` Literals Are Dangerous

The repository currently uses the spelling `0.92` for unrelated concepts,
including:

- the runtime master-bus limiter knee;
- maximum accepted source-copy waveform correlation;
- normalized trigger velocity or pressure;
- synthetic source amplitude;
- confidence values in tests;
- drum/snare balance or step accents.

Numerical equality does not imply shared meaning. Replacing all occurrences
with one global `0.92` constant would create a false coupling: changing a
limiter policy could accidentally alter musical velocity or a correlation gate.

The correct cleanup is semantic naming and ownership, not value deduplication.

## The Numeric Value Passport

Any new cross-module, replay-visible, manifest-visible, or release-gating value
should be explainable with this compact passport:

| Field | Question |
| --- | --- |
| Name | What semantic concept does it own? |
| Kind | Measurement, safety boundary, QA threshold, control, DSP, recipe, or fixture? |
| Unit/domain | Linear amplitude, dBFS, Hz, seconds, beats, samples, ratio, correlation, count, or normalized range? |
| Scope | Global runtime, one lane, one recipe version, one source family, or one fixture? |
| Comparator | How does pass/fail treat the value and equality? |
| Provenance | Derived formula, engineering safety choice, source matrix, human labels, or inherited provisional value? |
| Canonical owner | Which named constant, typed policy, recipe, manifest field, or spec owns it? |
| Evidence | Which tests, sources, artifacts, and human verdicts constrain it? |
| Change rule | What must be rerun or re-reviewed when it changes? |

If those questions cannot be answered, the literal is genuinely magic and
should not become a durable contract yet.

## Change Rules

When a numeric value needs adjustment:

1. identify its category and canonical owner;
2. state the musician-facing problem before changing the number;
3. change product behavior or the QA threshold, not both in the same tuning
   step unless a calibration task explicitly requires both;
4. preserve historical typed recipe versions;
5. run unit/buffer tests and the nearest exact product-path proof;
6. for shared audio tuning, run at least three contrasting real sources;
7. compare measurements before and after, including limiter activity and role-
   appropriate frequency/time behavior;
8. request structured human listening when musical usefulness changed;
9. record durable calibration rationale in the relevant spec or decision log.

Do not weaken a threshold merely because a candidate missed it. First determine
whether the candidate exposed a real defect, the metric observes the wrong
role, or the threshold lacks calibration.

## Current Ownership Map

| Concern | Current owner |
| --- | --- |
| master-bus limiter knee, ceiling, clip and near-clip thresholds | `crates/riotbox-audio/src/runtime/public_api_shell.rs` |
| exact dense-break pack-wide QA thresholds | `crates/riotbox-app/src/bin/dense_break_live_path_render/model.rs` |
| exact gesture-specific QA policies and boundary checks | `crates/riotbox-app/src/bin/dense_break_live_path_render/manifest.rs` |
| typed TR-909 Fill step and output parameters | `crates/riotbox-audio/src/runtime/tr909_fill_recipe.rs` |
| cross-source W-30 envelope-correlation gate | `scripts/validate_dense_break_live_source_matrix.py` |
| live source-character contrast margin and lane balance | `crates/riotbox-core/src/live_performance_policy.rs` |
| controlled dense/tonal/sparse exact-path gates | `crates/riotbox-app/src/bin/dense_break_live_path_render/controlled_source_manifest.rs` |
| controlled source stability/diversity gates | `scripts/validate_controlled_source_live_matrix.py` |
| general audio QA meaning and human-verdict boundary | `docs/specs/audio_qa_workflow_spec.md` |
| audio runtime behavior | `docs/specs/audio_core_spec.md` |
| accepted durable calibration decisions | `docs/research_decision_log.md` |

## Useful Inspection Commands

```bash
# Find named numeric contracts first.
rg -n 'const .*: (f32|f64|usize|u32)' crates scripts

# Find threshold-bearing manifests and validators.
rg -n 'threshold|ceiling|limited_sample_count|correlation' crates scripts docs/specs

# Inspect the exact generated values for one candidate.
jq '{thresholds, stages: [.performance_stages[] | {
  case_id,
  peak: .limiter.pre.peak_abs,
  limited: .limiter.limited_sample_count
}]}' path/to/gesture-manifest.json

# Convert a positive linear amplitude to dBFS.
awk 'BEGIN { value=0.916060388; print 20*log(value)/log(10) }'
```

The goal is not to eliminate every numeric literal. The goal is that every
number capable of changing sound, safety, evidence, or product behavior has a
clear semantic home and an honest explanation.

## Controlled Source Character Calibration

`LIVE_PERFORMANCE_CHARACTER_CONTRAST_MARGIN = 0.10` is a normalized
classification margin, not audio gain. It requires phrase evidence to clear the
neutral dense band in two independent relative comparisons. The trusted
RushArp source clears brightness-over-body and hook-restraint-over-body, while
the sparse BeatC source clears body-over-brightness and hook-space-over-offbeat
density. The accepted Beat03 dense source remains neutral. Equality passes.

The character-specific W-30, TR-909, and MC-202 values in the same typed policy
are normalized musician-control allocations. They are deliberately local to
the `live_performance_character.v1` decision and are not limiter, dBFS, or QA
thresholds. Tonal calibration reduces generated support and makes the W-30
source hook lead; sparse calibration preserves a quieter W-30 rhythm while the
TR-909 owns the highest isolated transient peak. The controlled exact-path
manifest reports the resolved values and role metrics. Any change must rerun
the dense regression plus `just controlled-source-live-matrix`, then receive
fresh human listening when the candidate sound changes.

`W30_DAMAGE_TRANSIENT_BITE_GATE_STEP_FRACTION = 0.44` is the maximum fraction
of one W-30 trigger step retained by the sparse destructive articulation. The
committed damage intensity scales it, so the current `0.82` action resolves to
`0.3608` of a step. It is not a gain: playback remains exactly `1.0x`, the
source-derived attack is retained, and the rest of the step is choked so source
kicks cannot drift between fixed-grid drums. `GATE_FADE_STEP_FRACTION = 0.10`
is the click-safe fade length within that trigger step. Zero disables the gate
and leaves existing clean and pitch-drag playback unchanged. Changes require
runtime gate tests, exact matrix determinism, and fresh sparse-destructive
listening because silence placement and groove are musician-facing behavior.

## W-30 Resample Cycle Alignment

`W30_RESAMPLE_SOURCE_WINDOW_LEN = 16_384` is the fixed full-duration mono proxy
capacity, currently shared with `W30_PAD_PLAYBACK_SAMPLE_WINDOW_LEN`. It was
increased from `4_096` so the resample path could span the complete capture
instead of one short grain. It is not a source sample rate, spectral-quality
target, or free realtime budget. The shared callback currently reloads all
`16_384` atomic samples per buffer; at `48 kHz` and `128` frames that is about
`6.14 million` atomic sample loads per second plus roughly `64 KiB` copied per
callback. This branch needs a bounded immutable handoff or real-session
worst-case callback/stack evidence before merge.

The current projector chooses uniformly spaced source frames without an
anti-alias filter. For the exact `81_237`-frame / `44.1 kHz` capture, `16_384`
points represent about `1.842` seconds: an effective proxy rate near
`8.89 kHz` and Nyquist near `4.45 kHz`. Any `6–20 kHz` output from that proxy is
not faithfully preserved source attack; it can come from decimation aliasing,
reconstruction/interpolation spectral images, cursor discontinuity, or later
nonlinear processing. Future attack/body plans must declare proxy bandwidth,
anti-alias and reconstruction behavior, and whether bounded full-rate onset
material is retained.

`MIN_GRID_ALIGNED_CYCLE_BEATS = 1.0` and
`MAX_GRID_ALIGNED_CYCLE_BEATS = 64.0` bound the whole-beat duration inferred
from a hydrated W-30 resample artifact. They are counts in beats, not gains,
playback rates, confidence scores, or QA thresholds. With valid transport
tempo, the callback calculates:

```text
source duration in seconds * transport BPM / 60
    -> nearest whole beat count, bounded to 1..64
    -> exact output-frame duration on the transport grid
```

This correction is intentionally small: committed capture material is already
quantized, while WAV/sample-rate rounding can leave its measured duration a
fraction away from an integer beat count. Aligning that cycle prevents the
source attack from wrapping a few milliseconds after the corresponding grid
attack. `64` is an MVP realtime-safety ceiling for the bounded proxy, not a
promise that the W-30 supports a 64-beat editing surface.

The source cursor uses `f64` so accumulated phase error cannot move an otherwise
correct cycle restart outside the one-frame timing regression. Generation depth
and grit still shape the source-backed signal, but they no longer alter Base
cycle rate and create a free-running loop. Any future pitched/dive behavior must
be a typed performer-owned variation with an explicit grid/retrigger contract,
not an incidental side effect of those controls.

## W-30 Hard Articulation

`HARD_TRANSIENT_CHOP_GATE_STEP_FRACTION = 0.55` means a selected transient-chop
hit reaches its declared gate end level after 55% of one eighth-note step.
`HARD_TRANSIENT_CHOP_GATE_END_LEVEL = 0.03` is linear envelope amplitude, about
`-30.5 dB` relative to the retriggered envelope. The callback derives a
sample-rate- and tempo-correct decay coefficient once per buffer. These values
are retained experimental recipe constants, not an accepted hardness target.
The standalone Hard sibling measured `45.75%` silence (`46.38%` in the exact
listened A/B Hard half under the documented near-silence floor), but the
structured review rejected the result as choppier and emptier rather than
harder. These values create source-backed attack/space contrast only; they are
not silence-ratio QA thresholds, output gains, a fixed BPM assumption, or
musical-quality evidence.

`HARD_TRANSIENT_PATH_GAIN = 1.12` compensates the complete gated
`source_transient_chop` Hard voice, not an isolated attack region. The earlier
`HARD_TRANSIENT_ATTACK_GAIN` name was misleading: this multiplier cannot alter
the attack/body relation or restore body removed by the envelope. It stays
behind the existing voice ceiling and must not be increased to disguise weak
articulation or missing body. Do not iterate this number again until a
different, research-backed mechanism establishes a source-adaptive attack/body
relation. Changes require zero-clipping proof across the real-source matrix,
native and loudness-matched comparison, and fresh structured listening.

The `HARD_SOURCE_*` drive, wet, body, and edge values shape both typed Hard
policies from their actual source samples. Continuous `source_texture_bite`
adds `HARD_TEXTURE_FOLD_DRIVE = 2.2`, eight amplitude-quantization steps, and a
`0.58` fold/crush mix. These are version-local DSP recipe values, not a fallback
voice: no source means digital silence, and texture material receives no
invented trigger grid. Changes require transient and texture policy tests,
same-source determinism, cross-source diversity, and role-specific listening.
The transient candidate using these values received `human_verdict: reject`;
the constants document current code, not a performer-approved recipe. Future
work follows
`docs/engineering/perceptual_hardness_and_musical_impact.md` and must not treat
another combination of fixed drive/wet/body/edge floats tuned to Beat03 as
generalized Hard behavior.

## W-30 H13 Reverse-Into-Impact Values

H13 is the frozen `source_reverse_into_impact_v1` experiment documented in
`docs/benchmarks/w30_resample_h13_characterization_v1.json`. Its values have
separate units and responsibilities:

- `0.02 s` and `0.10 s` delimit the source-analysis head and body windows. The
  callback articulates the corresponding `20..100 ms` output window.
- `0.90` is a dimensionless source-relative target: a weak selected body may
  approach 90% of that same impact's measured head RMS. It is not an output
  gain, wet mix, confidence, or QA threshold.
- `1.12..=1.40` bounds the derived body gain. `1.12` guarantees a meaningful
  local experiment; `1.40` prevents the source-relative target from becoming
  an unbounded loudness fix. The final value still varies by source.
- `0.75` is the lower bound for the independently derived impact-energy
  compensation. It prevents the body lift from passing by raising the entire
  impact.
- `0.12 s` is the reverse-pickup duration. `0.20` is the minimum share of
  whole-source RMS used when the preceding slot tail is nearly silent, and
  `0.10` is the pickup gain floor. These are source-normalization controls, not
  invented audio or a synthetic fallback.
- `10 ms` ramps the reverse pickup into the compensated impact boundary.
  `2.5 ms` smooths entry to and exit from the body articulation. Both are
  click-control durations, not rhythmic offsets.
- `1.05` is the frozen H12-to-H13 selected-body RMS gate; `0.20` is the frozen
  pickup relative-delta gate; `1.50` is the maximum local boundary-outlier
  ratio. These values validate distinct audible consequences and do not
  substitute for structured listening.

The body articulation is applied after the existing H12 callback voice has
been produced. An earlier implementation extended every selected attack replay
to 100 ms; that reduced body on short and textural sources before gain was
applied. Keeping H12's source-owned attack length and articulating the actual
output body fixes the causal error instead of compensating it with a larger
global scalar.

## W-30 H14 Exact Hit-Window Calibration Values

H14 supersedes the fixed H12 hit-shaper output calibration without rewriting
the frozen H13 evidence above:

- `0.94` remains the versioned H12 schema output gain and the retained target
  for the selected hit's `100..200 ms` following-body window. It is no longer
  applied blindly to the complete source.
- `1.0` is unity, not a taste boost: the selected hit's `0..100 ms` primary
  window is kept at the unattenuated callback level while material between
  selected hits receives the source/tempo-specific whole-path gain.
- `20 ms` of source-backed material immediately before a selected hit is also
  kept at unity. A `2.5 ms` smooth lead-in starts before that window, so the
  gain change does not introduce a new click while the complete 20 ms context
  still reaches unity. This primes the declared 900–3600 Hz head filter from
  the actual pre-hit context instead of the attenuated between-hit level.
- `200 ms` is the end of the retained following-body window; `10 ms` is its
  smooth transition back to the calibrated between-hit gain. These windows
  correspond to the existing role-specific QA regions, not invented notes or
  a new rhythm.
- `1.20` is the preferred exact-callback whole-path RMS ratio; `1.30` remains
  the hard release ceiling. Three exact endpoint callback renders establish
  source-specific energy curves, a ten-step in-memory search proposes the
  between-hit gain, and at most four exact callback refinements resolve model
  error or the role floors below. Every exact render covers four complete
  eight-step cycles at 48 kHz stereo, and every measured window covers all
  four cycles. The result is cached by source revision, trusted tempo,
  variation, grit, policy, trigger/onset plan, low-impact recipe, and H13
  gesture plan.
- `0.25` is the minimum between-hit output gain. Its inverse establishes the
  `4.0` maximum local compensation needed to retain a selected primary hit at
  unity. The compensation never makes that hit hotter than the unattenuated
  callback path.
- `1.10` is the calibration search's anti-collapse floor for the filtered
  0–20 ms head and `1.15` is its 20–100 ms body floor. If the preferred
  `1.20` whole-path target and these role floors cannot all be met, the
  calibration chooses the lowest exact gain that retains both roles, but only
  while the exact result remains below `1.30`. The authoritative product-path
  QA still independently requires its own `1.15` filtered-head gate. A source
  that misses any authoritative gate is still rejected.
- `0.95` is H14's updated source-relative selected-body target and
  `1.12..=1.45` its gain range. The earlier H13 values (`0.90`,
  `1.12..=1.40`) remain frozen historical evidence. H14 raises only the
  source-selected 20–100 ms impact body; it does not assign bass or raise the
  whole path.

The derived output gain, selected-hit compensation, exact-calibration flag,
evaluated flag, and predicted level/body ratios are visible in runtime and
observer evidence. Matching evaluated rejections are cached as well as
successful calibrations, so an unsuitable hit-shaper source cannot trigger the
same offline render search on every view refresh. Missing source PCM, untrusted
tempo, non-transient policy, and non-hit-shaper recipes do not run this
calibration.

## W-30 H18 Runtime-Synchronous Body Preservation Values

H18 supersedes the active H14/H13 callback values after a fresh holdout exposed
two hidden mismatches. The historical values above remain the record of those
frozen generations.

- The exact calibration measures H12 without H13, chooses the H12 between-hit
  gain, then verifies the same gain through the complete H13 callback. H13
  remains part of the cache key. This prevents an H13 level reduction from
  hiding an over-limit H12 counterfactual.
- `120..200 ms` is the independently gated late-body window. The callback
  carries the already typed `hit_window_compensation_gain` into realtime and
  may lift only that window above unity when exact source evidence requires
  it. `4.0` remains the absolute compensation cap. The `0..120 ms` head/body
  stays at unity, and the existing `10 ms` fade returns to the between-hit
  gain.
- `0.95` is the unchanged minimum source-relative late-body ratio. H18 derives
  the smallest late-body target that passes it with at most four exact callback
  refinements; it does not add a fixed taste boost.
- `1.0` is now the minimum H13 impact-level compensation: the chosen impact is
  not attenuated after source-local articulation. `1.75` is the bounded maximum
  derived body gain. Exact H12 and full-H13 level checks still enforce the
  unchanged `1.30` ceiling for `source_hit_shaper_v3`.
- QA windows use the projected runtime tempo, not merely the requested BPM.
  For example, Beat03 is measured at the product-owned `130.28494 BPM` grid
  even when the confirmation request was `130 BPM`.

These values belong to drum/transient and midrange/hook impact. Bass ownership
remains `unassigned`; none of them claims bass pressure.
