# RIOTBOX-1550: source-free limiter baseline prerequisite

Date: 2026-10-01. Classification: contract_enabler for RIOTBOX-1501.
Implementation baseline: `2d4dbf5a3e73a3067e4469507ae79004b29d9370`.
Integrated docs-only predecessors: `ec2752bfcc6aeef33025515e817ba2e3a623be33`
and `7e4bf60ad2dfd08f742a69bc757037a4a7f64d65`.
Decision: RBX-417. [Frozen baseline protocol v1](../benchmarks/master_bus_limiter_baseline_protocol_v1.md).

## Purpose and unchanged product boundary

Remove the missing source-free preregistration/control prerequisite before
RIOTBOX-1501's future authorized Development comparison and human decision.
This is not another limiter, a new musical mechanism or completed calibration.
The inherited f32 knee 0.92 and ceiling 0.985 remain explicitly provisional.
Runtime DSP, callback, public APIs, Core/Session/replay, Cargo and frozen Stage-A
contracts are unchanged. Only two existing runtime regression owners gain tests.

The versioned protocol fixes finite in-memory controls, exact write/count and
headroom interpretation, contributor attribution and fail-closed access/stopping
rules. No executor, runtime parameter, alternative policy or validator framework
is added. The owning audio spec and numeric guide/passport link the protocol;
none changes a runtime or clean-path acceptance value.

## Technical interpretation and fixed observations

Magnitudes at/below the knee, including equality and signed zero, pass unchanged.
Above the knee, the existing f32 tanh shape is calculated, but writes/counting
require a computed delta strictly greater than f32::EPSILON. The nearest
above-knee f32 value remains bit-identical in the signed-neighbor control:
peak 0.9200001, zero writes. This does not define a new knee or perceptual onset.

The reports measure actual input/output buffers. Nominal ceiling headroom is
0.015 linear amplitude; its f32 report can be 0.014999986. Derived dBFS is not
dBTP, loudness or optimal calibration. Control observations from the completed
same-build serial Debug run:

| Control | Pre peak / clips | Actual writes | Post peak / clips |
| --- | --- | ---: | --- |
| Signed overload ladder | 4 / 6 | 10 | 0.985 / 0 |
| Sparse signed transient | 1.25 / 2 | 2 | 0.98499495 / 0 |
| Sustained alternating overload | 4 / 1,024 | 1,024 | 0.985 / 0 |
| Partition/repeat pattern | 4 / 772 | 1,158 | 0.985 / 0 |
| Five-owner exact composite | 1.5052301 / 1,364 | 1,652 | 0.985 / 0 |

Intentional overload passes only as protection evidence: the composite fails
the clean-product zero-pre-clips/zero-writes gate. Its constant monitor fixture
also makes DC large (pre 0.6742585, post 0.5927536); it is not a musical candidate,
safe listening artifact or a worst-case proof for all performer configurations.
No loudness/makeup gain rescues it. Quiet/zero signals are never boosted.

## Exact RuntimeMix contributors and counterfactuals

The fixed 44,100-Hz stereo/2,048-frame/128-callback composite names TR-909 Fill,
MC-202 Instigator with a synthetic phrase plan, W-30 raw preview, W-30 capture
resample and Source Monitor Blend. Existing synthetic window helpers and one
in-memory constant monitor cache are reused; no path is opened. The monitor is
explicitly anchored with Some(0.0) seconds at beat 32, as declared in the protocol.

Each owner sounds alone with other owners silent; removing each changes the
composite pre-limiter report. Monitor removal retains Blend generated gain and
removes only its PCM, avoiding a mode/gain confound. Independently sounding
post-RMS observations: TR-909 0.099055, MC-202 0.508038, W-30 preview 0.34705696,
W-30 resample 0.18745014, monitor 0.8615887. These are fixture measurements,
not source-informed decisions, isolated full-composite stems or taste scores.
All-default routes stay silent despite running transport; repeating the same
composite preserves its complete report/output. Whole-mixer partition evidence
retains existing regressions separately from the limiter-only stateless proof.

A draft test omitted the declared optional seconds anchor. The resulting source
position lay outside the short monitor buffer; pre peak 0.88523006, RMS 0.49759975,
zero clips/writes correctly failed the intended hot-composite assertion. Fixing
the fixture installs the already-declared anchor, not higher gain or different
limiter thresholds. No product defect or calibrated candidate is claimed.
An initial moved non-Copy test field is explicitly cloned. These diagnostic
attempts are not accepted final proof. Serial final metric collection avoids
the interleaved stderr of the initial parallel nocapture diagnostic.

## Verification and review

Pre-edit runtime baseline: 128 executed tests and one existing ignored benchmark.
Complete token/attribute comparison preserves all nine existing signal-metric
and both existing mix-safety functions; five new tests are added. Exact final
Debug/Release identity sets agree: 133 executed, same ignored benchmark, all
129 original names/statuses retained. No synthetic red is manufactured to call
this a product bug fix; the controls characterize existing behavior.

Debug identity proof: `/tmp/riotbox-1550-runtime-debug-final.log`.
Release identity proof: `/tmp/riotbox-1550-runtime-release-final.log`.
Complete serial computed reports: `/tmp/riotbox-1550-reports-final.log`.
The direct limiter proof covers bit identity/no boost/odd monotone response,
exact clip/write counts, sparse/sustained inputs and chunk sizes 1/127/128/257.
The current production source diff is empty; all crate changes are these two
test owners. The final serial run contains 19 complete computed reports, identical
to the reviewed serial controls. Final `just ci` exits 0:
`/tmp/riotbox-1550-ci-accepted.log`. This includes strict Clippy, formatting,
Rust/Python/contract checks and source-free audio smokes. The earlier Clippy
diagnostic log is not accepted proof. Branch/native workflow gates remain
separate obligations before feature completion.

Solo Rust/spec/evidence review checks complete retained bodies, unchanged
production operations/constants, access boundaries, actual owner routes,
honest pre/post reports and no new state/fallback/framework. No independent
panel/subagents or exhaustive audit claim. Metrics cannot award a human verdict.
The review catches one additional evidence-binding gap: neighbor tests originally
followed public getters without explicitly asserting v1's fixed values. They now
bind the unchanged 0.92/0.985 baseline, preventing silent protocol drift. Strict
Clippy's fixed-array grouping uses as_chunks, not a lint suppression or changed
control. The final Debug, Release, serial and full-CI runs include these assertions,
not an earlier build. Short final self-review finds no unresolved scoped finding;
it does not upgrade the bounded evidence to an independent or exhaustive review.

## Limits and named next step

Bounded finite controls only, not arbitrary huge or NaN/Inf admission stress,
allocation tracing, OS/device/hardware tests, true-peak or hearing safety.
No Development/Holdout/commercial audio, source discovery, DAW or playback.
No source-general, hardness, human, demo/release pass or P023 completion.
RIOTBOX-1501 remains incomplete: exact authorized Development inputs and bounded
alternative comparison must be preregistered before source results; artifact
preflight and fresh structured human review precede a calibration decision.
Do not chain another framework or retune a scalar from these control observations.
