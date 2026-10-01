# Master-bus limiter baseline protocol v1

Owner: RIOTBOX-1550, bounded prerequisite for RIOTBOX-1501.
Decision: RBX-417.
Classification: contract_enabler / source-free engineering baseline.
Version: 1. No product calibration, musical verdict or source-access grant.

## Authority and permitted execution

Use the unchanged product `apply_master_bus_soft_limiter[_with_report]` and
`render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report`.
No alternate limiter, new runtime parameter, gain compensation or automatic
policy selection. The inherited f32 knee `0.92` and ceiling `0.985` remain
provisional (RBX-379). This protocol characterizes them; it does not establish
that either is perceptually optimal or change a clean-path QA threshold.

Only bounded, generated in-memory fixtures may execute. Synthetic cache paths
are metadata, never opened files. No Development/Holdout/commercial audio,
source-directory discovery, DAW, device, live playback or human review. Existing
source-free CI may run its already-authorized generated fixtures. A source or
audible-candidate comparison requires a separate exact frozen authorization and
applicable listening gates; this document does not enable that phase.

## Sample and report semantics

For finite f32 input x, t = the unchanged knee and c = the unchanged ceiling:

1. `abs(x) <= t` returns x, including signed zero and knee equality.
2. Above t, compute `signum(x) * min(t + (c-t) * tanh((abs(x)-t)/(c-t)), c)`
   using the existing f32 operations, not a replacement f64 reference renderer.
3. Write/count the shaped value only when `abs(shaped-x) > f32::EPSILON`.
   An above-knee neighbor can therefore pass unchanged; above-knee count is not
   `limited_sample_count`. That count measures actual writes, not onset events.
4. `applied` is count > zero. Pre/post reports use the actual input/output buffers.
   Clip equality is `abs(x) >= 1.0`; near-clip equality is `abs(x) >= 0.98`.
   Linear reported headroom is `1.0 - peak_abs`, possibly negative before limiting.

Derived dBFS is `20 * log10(positive amplitude)`, not LUFS or dBTP. The nominal
knee is approximately -0.7242 dBFS and ceiling -0.1313 dBFS; nominal peak headroom
at the ceiling is 0.015 linear amplitude. These are derived coordinates, not a
new gain target. Zero has no finite logarithmic value. Float rounding and the
write predicate remain visible; do not infer a portable cross-platform tanh hash.
The boundary test explicitly binds the public knee/ceiling getters to v1's
fixed values; following different getters silently is not a v1 baseline proof.

## Fixed source-free control catalog

Every fixture is non-product diagnostic material. Do not reinterpret its fixed
phrase or waveform as source intelligence, primitive promotion or musical proof.

| Control | Fixed input / seam | Required observation |
| --- | --- | --- |
| Empty/quiet/knee neighbors | Empty; signed zero, ±0.00005, and ±the immediate f32 neighbor below/equal/above t | Bit-preserved output, zero writes, unchanged pre/post report. |
| Signed finite overload ladder | Both signs of `[0, 0.25, t, 0.94, c, 1, 1.2, 4]` | Odd, monotone magnitude response; never boost; ten writes and six pre-clips; bounded post peak, no post-clips. |
| Sparse transient | 256 zeros with index 63 = 1.25 and 64 = -1.25 | Two writes/two pre-clips, no post-clips; all other samples stay zero. |
| Sustained overload | `[4, -4]` repeated 512 times | 1,024 writes/pre-clips, bounded finite post samples and zero post-clips; no makeup gain. |
| Partition/repeat | `[0, 0.25, -0.25, t, -t, 0.94, -0.94, 1.25, -1.25, 4, -4]` repeated 193 times | Same-build bit-identical output and equal total writes with whole buffer and chunks 1/127/128/257; report pre/post matches actual buffers. |
| Five-owner hot composite | Exact RuntimeMix, 44,100 Hz stereo, 2,048 frames, 128-frame callbacks, transport running at 128 BPM/beat 32 | Honest pre-overload and post-protection reports, all five independently sounding controls, bounded finite output; not an isolated lane or a device proof. |

The composite explicitly names TR-909 Fill/DrumBusSupport (level 4, slam 2.5),
MC-202 Instigator/InstigatorSpike with the existing synthetic parity source-plan
(level 4, touch 3), W-30 RawCaptureAudition with the existing synthetic parity
window (level 4, grit 1), W-30 CaptureLineageReady/InternalCaptureTap with the
existing synthetic resample window (level 4, grit 1, available promoted fixture,
two lineage refs/depth one), and Source Monitor Blend with generated constant
1.0 mono PCM at 44,100 Hz for 2,048 frames, anchored at beat 32/second zero.
The seconds anchor is explicitly `Some(0.0)`, not an absent legacy anchor.
Unmentioned fields keep their current typed defaults. These intentionally hot
fixture levels are not supported performer settings or a new recipe.

Check each owner separately with the same running transport and all other
owners in typed default/silent state; the monitor-only control uses Source mode.
Then check the named composite against each corresponding leave-one-owner-out
pre-limiter report. Record each computed report before enforcing its assertions;
post-limiter saturation alone cannot establish a contributor's absence/presence.
Removing the monitor keeps Blend mode/gain and removes only its PCM reference;
switching to Riotbox mode would also change the generated gain and confound it.
Retain the existing clean and three-lane hot regressions unchanged. Whole-mix
callback partition proof stays separate from limiter-only stateless partitioning.

## Interpretation, stopping and next phase

An intentional overload control passes only as a protection/control observation.
Its nonzero pre-clips/limited count explicitly FAIL the clean-product path even
when post samples have no clips. A clean path requires zero actual writes and
zero pre/post clips; do not move the knee or compensate gain to hide a weak mix.
Inspect complete computed reports, buffer length/finite values and the named
contributors; fail the control proof on mismatch. Keep failed metrics, not a
guessed pass. Same-build repeatability is not OS/device/true-peak qualification.

RIOTBOX-1501 remains the named next audible calibration work: preregister bounded
alternatives, exact authorized Development sources, render inputs/gestures,
pre/post and time-local transient measurements, comparison order and stopping
budget before new source results. Preserve the baseline and historical recipes.
Any selected alternative requires its versioned decision, owning spec/passport
update and structured human evidence; a retained baseline needs measured rationale
and limitations too. No source/candidate executor or validator framework is added
in this prerequisite. Holdouts stay closed until their exact phase permits them.

After acceptance, changes to this catalog, semantics or access boundary require
a new protocol version and Decision before recomputation. This protocol does not
change the independently frozen Stage-A contracts. It grants no true-peak,
speaker/hearing safety, source-general, hardness, human, demo or release pass.
