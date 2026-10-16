# RIOTBOX-1589 — current-runtime generated loaded-recording regression

Date: 2026-10-16. Classification: maintenance/regression of the accepted P023
callback/recording path, under RIOTBOX-1041/P017. **The one approved observation
passed its existing gates; invocation consumed, no further trial authorized.**
The 250-ms startup check and one each 2/8/16-bar recording completed through the
changed current runtime. This is instrumented isolated generated-composite
recording evidence, not musical, physical-device, latency, long-endurance,
release or original ALSA-cause qualification.

## Fresh approval and exact implementation

After independent preparation review and exact-clean-head full local/native CI,
the user answered affirmatively to the new RIOTBOX-1589 request on 2026-10-16.
The request named one 250-ms startup check, then one each 2/8/16-bar synthetic
recording at 132 BPM, isolated silent virtual output, no speakers/real sources/
DAW, first failure stops without retry. The reply was retained accurately in
this phase's fresh authorization record. It is not an older approval or generic
authority to repeat tests.

Only the unchanged executable fence in the
[1589 prospective phase](../benchmarks/loaded_recorder_host_riotbox_1589_v1.md)
ran, once, on clean reviewed `6c65cc8cb6c0379bde61f9615ddb2970b4d29767`, tree
`77cbf2118e598d0063c0b83abac6bf696ef7ce6e`.
Full exact-head source-free local CI passed:
`/tmp/riotbox-1589-exact-head-ci.log`. The command used the installed
`JUST_TEMPDIR` option with a fresh private `/tmp` directory because the first
invocation reached a sandbox-read-only runtime temp parent. That earlier
non-green log remains `/tmp/riotbox-1589-full-ci.log`; no code or test policy
changed. Exact native [Rust CI 38909243358, attempt 2](https://github.com/marang/riotbox/actions/runs/38909243358)
passed both existing jobs before access. Linux passed first; four unchanged
Windows Sidecar Control timeouts passed on the sole failed-job retry. This is
not an RIOTBOX-1509 cause diagnosis, transport fix or Windows-product expansion.

The existing default-feature Rust binaries and unchanged V4 diagnostic library
were built after those gates and bound before invocation. No build or tracked
edit intervened between binding and execution. The changed product code is the
already merged 1582/1584/1587/1588 repairs; this phase tunes no implementation,
protocol, duration, threshold, rate, buffer, priority or sound policy from results.

| Binary | SHA256 |
| --- | --- |
| `cpal_spike` | `51904600cfdf2497890417e043e5a025dc38e04b183fc341248d50465f7d7b3c` |
| `riotbox-app` | `4a164a68de760b028e58ff3e78fbe46695c324049f0b588b59cca093c87f29e9` |
| `dense_break_live_path_render` | `6057295250135fc1014c8297d2c4101b818a5f904cf3b82e972b3da39beb1205` |
| `libriotbox_silent_alsa_diag.so` | `335f6d415b05b43749b35f7808660d56e46c63ce9322aae4c02ee4038ff71217` |

The explicit workload is LoadedObservationVersion.V2. The phase uses the
existing phase V2 schema, not a return to the rejected V1 observer. Phase JSON
SHA256: `a3f8a71e91a9a5f1892c39ab24b69666d8cd9a048cfcda97a920363617fc80c0`.
Loaded V2: `4cf9514cf33057d67e268f4ee6d234e083f4703d25a2f3d8537de2b3c99564ed`.
All inherited fixture/containment/diagnostic contracts remain unchanged. The
normal execution CLI stays disabled.

## Owned original evidence

Raw evidence stays local/ignored under exactly
`artifacts/audio_qa/local-loaded-recorder-riotbox-1589-v1-2026-10-16`.
The generator exclusively created a fresh `fixture-v1`; admission verified its
declared generated source/Graph/capture/Session/artifact identities and producer.
No older ignored attempt, real source or capture was opened or substituted.
The actual one-shot log `/tmp/riotbox-1589-actual-host-once.log` returns exit 0,
three runs, `result=pass` and `cleanup_verified=true`.

Paths below are relative to this owner; hashes identify original bytes, not
JSON reserialization. The consumed marker binds the exact authorization,
prospective binding, reviewed revision and live native-CI response digest before
the sole operator call. No failed case was repeated or replaced.

| Record | SHA256 |
| --- | --- |
| `fixture-v1/preparation.json` | `5d4d34a1d87dec415afb451d1c830500dfba0bad16c47e4ec63e49912af40040` |
| `fresh-authorization.json` | `17a3bda749015c677b9e9b4d553d1117a0ec152c8981e6b8e30b1053173d2b58` |
| `prospective-binding.json` | `ccbbec59849217fa1cfa13fa3f0e752cb12249be296f4546452412829699648c` |
| `invocation-consumed.json` | `688f623ccc0e6ddb97f4e6ad070f8f2d2e4fc973f63c77248385844a390633a7` |
| `attempt-01/result.json` | `ffa01c01564837c7f8e57b4a5fc32243d3498ca8b07f0a1b54fe2ca55747ecdd` |
| `postflight-verification.json` | `3c3f697de0a47b555c5bb2add8cb4098e270486d344b04df9c0a8b3a731d58f7` |

## Actual technical result

Non-root active/nonremote local Linux user session, UID 1000, PulseAudio on
PipeWire 1.6.9; not sandbox-only audio evidence. The child used only its private
PipeWire-only ALSA configuration and exact isolated stereo virtual sink, never
the user's potentially physical default. All five composite contributors stay
named: TR-909, MC-202, W-30 preview, W-30 resample tap and Source Monitor. This
does not separately qualify every lane or real-source behavior.

Startup PID 1447921: 23 callbacks, zero reported stream errors/scratch overflows,
successful stream result. The three actual recordings are IEEE float32 stereo
WAVs at 44,100 Hz. The retained product peak/RMS values below are sample-domain
amplitudes, not true peak, latency or new musical thresholds. No gate was changed
from these values.

| Bars | Child PID | Frames | Recorded seconds, approx. | Peak | RMS | Nonfinite / clipped samples |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | 1448520 | 160364 | 3.6364 | 0.621003 | 0.121921 | 0 / 0 |
| 8 | 1450033 | 641455 | 14.5455 | 0.621003 | 0.120810 | 0 / 0 |
| 16 | 1453609 | 1282909 | 29.0909 | 0.621003 | 0.120386 | 0 / 0 |

Published original WAV SHA256, in 2/8/16-bar order:

- `02aba51a54b09433194191213d190be706b2ab2091ea628df24991269a39c5aa`
- `f2012efc1c6859fbba394bba7ea8babe4d179ffe967c795f01be31e133d17641`
- `cdb02e758a24ee0b3a4bd5149148a683348f6c14f42bd7d5e232cc797c6a6ada`

Each case retains its completed/stopped marker, final observer/JSON and Core
readiness with `ready=true`, `status=ready`, `runtime_stopped=true`, observer
recorded and zero faults. Exact proof/receipt/Session/action/Graph/timing/capture
bindings pass. Each mutable run-Session is a separate copy of one unchanged
baseline: action 8 and `export-receipt-a-0008` are local to each copy, not three
append operations on one Session. Active source capture `cap-01` legitimately
has an empty lineage list. The original result binds each raw proof, persisted
Session and retained Core-readiness digest.

All four writing ALSA PCM lifetimes closed successfully and measured stereo
44,100 Hz, 1024 buffer frames / 512 period frames. Availability and write call
counts are respectively 23, 472, 1411 and 2664 in startup/case order. Every PCM
lifetime has zero negative avail/write returns and zero EPIPE. This is
instrumented plugin geometry/coverage, not physical latency, PipeWire quantum,
zero-observer-cost evidence or explanation of the earlier ALSA failure.
Retained maximum callback gaps are 22.597 / 32.213 / 83.885 ms in recording
order. These instrumented observations are not a physical latency measurement,
CPU budget, causal attribution or a new timing pass/fail threshold. A successful
bounded observation does not establish general scheduling or dropout immunity.

## Cleanup, independent audit and bounds

The operator supervised process-group/stream cleanup before diagnostic/artifact
analysis and removed its exact virtual sink/module within unchanged budgets.
Separate read-only real-session postflight at `2026-10-16T15:31:50.312306+00:00`
verified all four PIDs absent, no attributed/original-node or sink-attached
streams, absent original sink/module, exact original default/mute/volume and
unchanged generated inputs. Six raw metadata responses and their digests remain
in `postflight-metadata/` and the bound postflight record. This contemporaneous
absence check is not historical process-group reconstruction; that evidence
belongs to the supervised operator. No service restart, setting change or
uncertain-resource unload occurred. It is absence of this test's owned output,
not a claim that unrelated user applications are globally silent.

Independent Spec And Evidence / Test Evidence / Adversarial offline audit
retains zero P0–P3 findings. It admitted all 13 fixture identities, correlated
all three complete artifact/observer/Session/action/receipt/readiness chains,
decoded all 4,169,456 float samples and all four original ALSA buffers, and
checked all 383 retained route snapshots (376 explicitly timed loaded queries).
Thirteen observe rows exercise V2's distinct-serial Client reuse without treating
it as completion. Earliest absence follows completion in every case; final
teardown-query margins under the unchanged five-second deadline are
4068.010 / 2987.753 / 1178.896 ms in 2/8/16-bar order. These are retained
observations, not continuous physical routing or new timing pass thresholds.
The audit explicitly does not re-attest a live CI response from its saved digest,
query current processes or replace the separate live postflight. Its scratch
reader `/tmp/riotbox_1589_offline_audit.py` performs offline calculations only.
The coordinator inspected that reader and reproduced its 5,260 passing offline
assertions, `/tmp/riotbox-1589-root-raw-audit.log`. This rerun is not a separately
authored third audit or a second host observation.

The separately independent Spec And Evidence / Maintainer / Product Pragmatist /
Devil's Advocate result-report review retains zero P0–P3 findings. It checked
the exact two-file result diff, record/protocol/binary/WAV/proof/Session/observer/
readiness identities, frozen sequential-f32 metrics, ALSA summaries and six
postflight raw-response digests without querying the host, CI or older owners.
The coordinator's follow-up self-review retains zero additional P0–P3 findings
in approval/consumption scope, raw identities, complete output proof, cleanup,
claim limits or environment routing. No production/Rust/module/action state or
frozen numeric/access contract changes; no routine Decision Log entry is needed.
Full source-free result CI, exact-head native PR CI and normal main-sync/issue/
branch closeout remain merge gates. The result diff changes no production code
or frozen contract. Prior failed/successful phases and their consumed approvals
stay unchanged and unopened.

This closes only the bounded current-runtime generated-recording regression.
No further actual trial, real source/capture/Holdout/commercial access, source
discovery, human playback or musical/hardness/source-general, physical-device,
latency/endurance, original-cause, DAW, Windows-product, broad-QA, release or
RIOTBOX-1041/1036 completion claim follows.
