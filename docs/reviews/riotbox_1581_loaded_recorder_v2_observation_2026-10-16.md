# RIOTBOX-1581 — one isolated generated loaded-recorder V2 observation

Date: 2026-10-16. Classification: maintenance/regression of the accepted P023
recording path, RIOTBOX-1041/P017. **PASS within this bounded instrumented
generated-composite observation; invocation consumed, no further trial.**
The 250-ms startup check and all three existing 2/8/16-bar cases completed with
actual recording artifacts and verified cleanup. This is not a musical,
physical-device, latency, endurance, release or original ALSA-cause pass.

## Fresh authority and exact implementation

After the new phase's independent review and exact local/native CI gates passed,
the user explicitly selected “Ja, diesen einmaligen V2-Test ausführen” on
2026-10-16. The question named RIOTBOX-1581 V2, one 250-ms startup check and one
each 2/8/16-bar recording of the synthetic composite, isolated virtual output,
no speakers/real sources/DAW, first failure stops without repetition.
This is fresh approval, not the consumed RIOTBOX-1579 authorization.

Only the sole exact executable fence in
[phase V2](../benchmarks/loaded_recorder_host_riotbox_1581_v2.md) ran, once, on
clean reviewed `0a8bad975a72079936e54938039ca9a19fc5683d`. Full local source-free
CI passed at that unchanged clean head:
`/tmp/riotbox-1581-exact-head-ci.log`. Exact native
[Rust CI 38859803889, attempt 1](https://github.com/marang/riotbox/actions/runs/38859803889)
also passed. Preparation [PR #1685](https://github.com/marang/riotbox/pull/1685)
merged as `2f0a8e146c81a32920ca310c2ecdd6bd47cde7cf` with an identical tree.
Execution used its reviewed head; the result branch moved to that identical
main only after the completed invocation. No build or tracked edit occurred
between binding and call. No protocol, algorithm, threshold, rate, buffer,
priority, recording version or other product policy changed from results.

The explicit workload was LoadedObservationVersion.V2, never the V1 default.
Phase JSON SHA256 is `197cbc4fb478c7d7bca2495909ba6acaeb846e26a262721a8d933df6a221f141`;
loaded V2 is `4cf9514cf33057d67e268f4ee6d234e083f4703d25a2f3d8537de2b3c99564ed`.
V1 predecessor/fixture/V3/V4 contracts were separately checked by the fence/loaders.
The ordinary execution CLI remains disabled.

Default-feature binaries and diagnostic library, built after the exact gates:

| Binary | SHA256 |
| --- | --- |
| `cpal_spike` | `4b50d6b5d491e60cadc2fbb4f39d42e212c52a2e6793e453c5a0a4d3d5939d5a` |
| `riotbox-app` | `6973e4831253a628c92050f656e73113d0b5f2bb776fb3147facf24eb2b6c923` |
| `dense_break_live_path_render` | `af560dcb31d3956a571eeb270db4fe81a06e24601847b354b8ee4246eb56adf2` |
| `libriotbox_silent_alsa_diag.so` | `335f6d415b05b43749b35f7808660d56e46c63ce9322aae4c02ee4038ff71217` |

## Owned original evidence

Raw evidence remains local/ignored under exactly
`artifacts/audio_qa/local-loaded-recorder-riotbox-1581-v2-2026-10-16`.
No old ignored phase was opened or substituted. Paths below are relative to this
owner; SHA256 values identify original bytes, not JSON reserialization.

| Record | SHA256 |
| --- | --- |
| `fixture-v1/preparation.json` | `d2b67555df7fe7eac30e5da242f2c8944c1a9160efbcd9db928d8a701174d807` |
| `fresh-authorization.json` | `d70c29e75e750e68f630f84fa026bd676468475860b96702fc08bee836e32bf5` |
| `prospective-binding.json` | `27ddae1073b21508440210344accbea7614951a6aced219e02a7f21a3f1dd03b` |
| `invocation-consumed.json` | `3c1ecf386a4d42ba4e6ca78e426f19bbcd2111d74bb3f347574e6921fe25830d` |
| `attempt-01/result.json` | `981638921376157af3976eb433aee3e162537b50e41943fab2e24f9e3fac082e` |
| `postflight-verification.json` | `715696d6e594fe8d0bf222fe52984b812098ab3c15dec50a93ca2976d7c7d074` |

The exclusive consumed marker binds approval/binding bytes, revision, CI run/
attempt and the actual CI response digest before entering the operator. Exact
invocation log: `/tmp/riotbox-1581-actual-host-v2.log`, exit 0, three runs and
verified cleanup. No alternate owner, repeated case or tuning was used.

## Actual recordings and technical results

Real non-root local Linux user session, UID 1000, active/nonremote, PulseAudio
on PipeWire 1.6.9; not a sandbox-only audio result. Only the private PipeWire-only
ALSA child configuration and the exact isolated stereo virtual sink were used.
The composite names all five contributors: TR-909, MC-202, W-30 preview, W-30
resample tap and Source Monitor. Naming does not independently qualify every lane.

Preflight PID 2706722: 23 callbacks, zero stream errors and scratch overflows,
successful stream result. All three recordings are IEEE float32 stereo WAVs at
44,100 Hz. Peak/RMS below are sample-domain amplitude, not true peak, latency or
musical thresholds. The existing frozen technical gates were not changed.

| Bars | Child PID | Frames | Window seconds (approx.) | Peak | RMS | Nonfinite / clipped samples |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | 2707019 | 160364 | 3.6364 | 0.621003 | 0.121921 | 0 / 0 |
| 8 | 2707934 | 641455 | 14.5455 | 0.621003 | 0.120810 | 0 / 0 |
| 16 | 2709967 | 1282909 | 29.0909 | 0.621003 | 0.120356 | 0 / 0 |

All three cases retain the post-stop milestone and complete final observer/JSON
with `ready=true`, `status=ready`, `runtime_stopped=true`, observer recorded,
fault count zero. Each fresh mutable run-Session starts from the same immutable
baseline and owns its own action 8 / `export-receipt-a-0008`; those IDs are local
to the separate copies, not three append operations on one Session. Exact Core
readiness, proof/receipt/Session/action/Graph/timing/capture references matched.
The active source capture is `cap-01`; its lineage list is legitimately empty.
The independent audit correlated the retained Core readiness records; it did
not launch a second Core or recorder execution.

| Bars | Published WAV SHA256 | Raw proof SHA256 | Persisted run-Session SHA256 |
| --- | --- | --- | --- |
| 2 | `02aba51a54b09433194191213d190be706b2ab2091ea628df24991269a39c5aa` | `251ff95687e0c0ab0b8c572e90569f790a16397011a3c8e2ea96e53350d59b78` | `e43dc3c45597f2e9253c4bdd2a7df14f9dc0a6fa4cafdaa297e872a41d0ad1a1` |
| 8 | `f2012efc1c6859fbba394bba7ea8babe4d179ffe967c795f01be31e133d17641` | `80be6097f81b368609801bc3a2a8b5db6cd332b2c8d26ca32d3a3f2179c553e9` | `d6e11c5d89413e551f592d24c30f9b6748fc7edce56feaaf4bc728be4ddd2db4` |
| 16 | `2d93e92d59566cd63224ece498c08010da5ff0eee269d9e0aaa0df70d24883f1` | `410657d4e3f3bc49d3e60c266cac45f0ee42ae3476b2114562b18d670f2890fc` | `47ae0d6d8cb2baf30f447196f678fed1bd20b3abe05f50c7272ddc99058bd492` |

Measured ALSA playback PCM geometry is 44,100 Hz/stereo, 1024 buffer frames,
512 period frames, successful close. Preflight/2/8/16-bar avail/write call counts
are respectively 23/471/1410/2664 each. All four measurements contain zero negative
avail/write returns and zero EPIPE. This is instrumented plugin geometry/coverage,
not physical latency, PipeWire quantum or zero-observer-cost evidence.

Retained route observations also exercise the actual V2 correction: in the
16-bar case the original Node 88 / serial 5357 disappeared, after which 22
snapshots showed ID 88 occupied only by structurally valid, distinct-serial
Clients, never a replacement audio Node. First absent queries started after
both window completion and stop in all three cases. Final teardown observations
had no original Node, child-PID stream or sink-attached stream, with respectively
4306.896 / 3096.131 / 1371.842 ms remaining under the unchanged five-second
terminal deadlines. This is retained observation evidence, not continuous
physical-route monitoring.

## Cleanup, limits and closeout

Operator supervision verified owned process-group/stream cleanup within the
unchanged budgets and removed the exact sink/module. A separate read-only
postflight verified all four child PIDs absent, no attributed streams, no owned
sink Node/module, exact original default/mute/volume and unchanged declared
generated inputs. It did not reconstruct historical process-group membership;
that proof remains the supervised cleanup path. No physical/default setting was
changed and no human playback occurred. Technical artifact analysis follows
verified audio cleanup, outside the live drain budget, not an extended runtime.

Independent Spec/Evidence review re-admitted all 13 immutable generated inputs,
decoded all 4,169,456 float samples across the three actual WAVs and correlated
original-byte/payload identities, proofs, separate Sessions/actions/receipts,
retained Core/observer records, route intervals, deadlines, raw ALSA diagnostics,
preflight and approval/binding/consumption. No findings. Native CI success was
checked separately by the coordinator; the offline raw audit does not re-attest
the live GitHub response from its recorded digest alone. Independent
Maintainer/Spec review of this report and environment routing also found no
findings. Root Product/Risk self-review found no unsupported musical, physical,
latency, endurance or original-cause claim and no contract/product drift.

Full source-free result CI and normal PR/main-sync/Linear closeout remain merge
gates. No production code or frozen contract is changed in the result diff.
The prior V1 failure remains negative/consumed; a separate successful V2
observation cannot erase it.

This closes only the bounded generated loaded-recording observation. It does
not close RIOTBOX-1041/1036, explain the earlier ALSA trigger, establish real-source
diversity, prove physical devices or long endurance, or award a musical/human/
hardness/source-general/latency/release pass. No real source/capture/Holdout/
commercial audio, source-directory discovery, DAW, broad QA or Windows work was
used. Any future actual trial needs its own prospective scope and fresh approval.
