# Master-bus limiter Development comparison v1

Owner: RIOTBOX-1501. Classification: bounded diagnostic calibration, not a new
musical mechanism. **Accepted before source access under RBX-418. Execution
requires the exact JSON pin and a clean, reviewed, committed implementation.**

The [JSON](master_bus_limiter_calibration_protocol_v1.json) owns exact identities,
numeric values, formats, windows and budgets. This document owns the preparation,
interpretation and operation rules. The previous
[design draft](master_bus_limiter_calibration_protocol_v1_draft.md) and
[baseline v1 / RBX-417](master_bus_limiter_baseline_protocol_v1.md) remain historical
unchanged controls. This successor does not amend Stage-A contracts.

## Question and claim boundary

Compare two directed parameter changes against the inherited sample limiter,
not a new limiter design. A/B/C share the production f32 operations and actual
write predicate; the production default remains A. The midpoint is arithmetic
in linear amplitude; both alternatives halve the derived knee width. Neither
is claimed optimal. No lookahead, oversampling, release envelope, channel link,
makeup gain, runtime control, new ActionCommand or Session model is introduced.

This removes P023's specific uncertainty about whether protection settings
preserve clean source-backed behavior and how they alter a declared overload.
It does not requalify full family journeys, live devices, restart/recall,
source-general quality, true peaks, hearing safety, hardness or release readiness.
Metrics and synthetic controls cannot issue a human verdict. Linux is the
execution platform; Windows compatibility work is deferred by user direction.

## Source admission and ownership

The three familiar examples are registered in the Sound Excellence corpus,
**not** the CC0 holdout registry. Their MusicRadar/SampleRadar local-use status
must not be relabeled CC0. Registry v3 supplies protected identity/path/hash
exclusions only. Compare all three selected identities against all nine
protected entries before the first original file open.

The Graph and historical Session JSONs are exact permitted metadata, not a
restore instruction. Their raw hashes and timing identities are bound before
audio access. Never hydrate historical captures or run an analyzer. Graph source
locators are the unchanged absolute paths under the pinned repository root.
The new Session is created through the existing fresh-ingest model; its real
output-only file set preserves ordinary artifact-required capture semantics.

Dense's original PCM24 format is known. Tonal/Sparse original bit depths were
not recorded in the inspected metadata. Their explicit admission permits only
the existing supported PCM16/PCM24 set; it does **not** pretend either header was
known. This opt-in is separate from, and cannot loosen, existing exact-width
Stage-A callers. Byte bounds use the larger permitted width; strict RIFF, SHA,
duration, channels, rate and integer-clipping checks still apply. Record the
actual format from the one admitted payload. No header reconnaissance or retries.

Reuse `run_development_access_session`: one fresh exclusive access log and one
no-follow bounded read per source. The access owner and Rust constructor each
verify SHA-256 over the **same admitted bytes**; this is two in-memory identity
checks, not a second file read. Only Rust decodes the source PCM, once. The
original path is never reopened by the renderer, metadata adapter or pack writer.
Normal capture commits may encode their one declared derived capture each and
hydrate that freshly written buffer. Do not count these as original-source
reopens or silently add other capture/render work.

## Exact committed preparation

Every case first uses the existing explicit-BPM validation and
`SourceTimingConfirmGrid` queue/Immediate commit at timestamp 10. Dense retains
its analysis hypothesis at approximately 130.285 BPM while confirming the
historical product tempo 130 BPM. Do not overwrite the hypothesis or substitute
automatic tempo confirmation. Tonal/Sparse retain their historical manual grids.
Require exactly one successful confirmation and the bound hypothesis/tempo.

Dense and Tonal reuse the existing six-action W-30 owner from semantic-hook V4:

| Action | Commit boundary / beat / bar / phrase | Queue → commit timestamp |
| --- | --- | --- |
| SourceTimingConfirmGrid | Immediate / unchanged initial transport | 10 → 10 |
| PresetActivate: FeralBreakAlphaV2 | Immediate / 0 / 1 / 1 | 90 → 95 |
| CaptureSetLength: OneBar | Immediate / 0 / 1 / 1 | 96 → 97 |
| CaptureBarGroup | Bar / 0 / 1 / 1 | 100 → 200 |
| PromoteCaptureToPad | Bar / 5 / 2 / 1 | 210 → 300 |
| W30TriggerPad | Beat / 6 / 2 / 1 | 310 → 400 |

Use the existing W-30 helper's commit semantics, which do not additionally
update the transport clock. Render its ordinary W-30 projection from beat 8;
other lane states are typed defaults and monitor mode is Riotbox. Do not invoke
an export or replay the six-gesture Dense qualification. The preset already
selects the transport-boundary hook policy; no direct policy mutation is needed.

Sparse reuses the existing `live_flow` prefix and transport-updating commit
helper, with timing-confirmed bar positions checked against the pinned grid:

| Action | Boundary / beat | Queue → commit timestamp |
| --- | --- | --- |
| CaptureSetLength: OneBar | Immediate / 0 | 90 → 95 |
| CaptureBarGroup | Bar / 0 | 100 → 200 |
| W30AuditionRawCapture | Bar / 4 | 210 → 220 |
| PromoteCaptureToPad | Bar / 8 | 230 → 300 |
| PresetActivate: FeralBreakAlphaV2 | Immediate / 8 | 305 → 306 |
| Mc202GenerateInstigator | Phrase / 16 | 311 → 400 |
| Monitor Source / Blend / Riotbox | Immediate / 16 | 420 → 421 / 430 → 431 / 440 → 441 |
| W30TriggerPad | Beat / 17 | 500 → 510 |

Hold the existing `after_w` projection at beat 17. This is not the old whole
Sparse journey or its backward-positioned beat-16 held stage. W-30 owns the
source transform, TR-909 the hardest transient and MC-202 punctuation; no bass
pressure claim. Do not add damage, gesture contrast, restart or recall. Check
all committed actions, capture window, roles and silent monitor/resample owners.

## Render and measurement

Freeze each prepared plan once. The three RuntimeMix passes are primary,
same-partition repeat, and alternate partition; require bit-identical pre/post
PCM and A reports. Reuse the retained primary pre-buffer for all policies and
both conditions. Do not feed A's limited output into B/C or reconstruct pre-PCM.
Six policy outputs per source stay in memory. No comparison WAVs are budgeted.

Clean requires A **and** both candidates to preserve every input sample with
zero writes, zero pre/post clips and the existing positive active-sample count.
Sparse additionally retains its existing whole-mix f32 RMS minimum; this does
not substitute for its unexecuted whole-journey or isolated-lane gates. A later knee cannot
rescue a baseline failure. Synthetic stress and exactly doubled source-backed
input are diagnostic challenges, never product recipes or gain compensation.
Record whether protection actually occurred; an unexercised case is unobserved,
not permission to raise gain or select another time window.

Retain Rust's original f32 aggregate report separately from explicitly labeled
f64 descriptive measurements. Validate finite, nonempty, equally sized,
frame-aligned stereo buffers first; never pad differences. Local windows start
at the fixed product-render origin, not an observed strongest event. Their
attack/body/recovery labels are descriptive: no event detector or automatic
musical-anatomy claim. Global delta uses the interleaved buffer; local deltas,
crest, correlation and spectral fractions are per channel. JSON defines the
spectral normalization and every zero-energy/null case. These are descriptions,
not newly inferred taste thresholds.

No sources are opened for synthetic tests. The baseline's fixed empty, quiet,
neighbor, overload, sparse-transient, sustained, partition and five-owner
controls remain immutable. Source-free alternative tests reuse the fixed
stateless catalog through that same limiter seam; the five-owner composite
remains an A-only baseline control, not a claimed full A/B/C composite catalog.
None chooses a policy from synthetic success.

## Execution and stopping

Build `dense_break_live_path_render` with the default-off `limiter-calibration`
feature. `python3 scripts/run_master_bus_limiter_calibration_v1.py` performs
metadata-only preflight. `--execute` additionally requires the accepted raw
protocol pin, clean committed implementation, unchanged binary identity and
one fresh fixed output directory. Before any source open, the executor performs
an explicit locked native-Linux Cargo build with the required feature and
records the returned compiler artifact, compiler version, command and SHA.
An old binary merely present in `target` is not accepted as build evidence.
The Rust `--limiter-calibration-v1` branch
receives only bounded JSON/admitted bytes on stdin, not input paths to open.
Record Git head, binary SHA, protocol SHA, access-session ID, action/commit
records, actual source format, capture identity and measurements.

Stop on the first violated identity, access, format, recipe, parity, finite,
ceiling or clean-path contract. Retain partial reports and the failed access
session; do not open remaining sources or delete the failed output directory to
retry. After any source evidence, changing this protocol, settings, preparation,
windows, analysis or admission requires a new version and Decision first.

No playback is authorized. The JSON preselects a possible later sparse stress
window only to prevent result-driven cherry-picking; it does not create a
listening artifact or grant readiness. If it does not exercise protection or
has no policy difference, record that as unobserved and avoid unnecessary
listening. Any later artifact generation/access and structured human comparison
needs its explicit bounded phase and the full listening-review gate. Until
then RIOTBOX-1501 remains incomplete and production policy stays provisional A.
