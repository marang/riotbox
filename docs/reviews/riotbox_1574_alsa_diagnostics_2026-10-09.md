# RIOTBOX-1574 — instrumented silent virtual-host observation

Date: 2026-10-09. P017 diagnostic work for the accepted P023 callback-reliability
blocker, not an audible slice or musical progress.

## Result and limits

**The single authorized V4 observation passed; cleanup independently checked.**
The 250-ms preflight and all three 60-second runs completed without reported
stream errors, scratch overflows or negative measured ALSA avail/write returns.
This is an instrumented idle virtual-endpoint observation, not a root-cause
fix, physical-device, endurance, loaded-runtime, latency or musical pass.

The [RIOTBOX-1572 failure](riotbox_1572_silent_host_v3_observation_2026-10-09.md)
remains failed and unexplained. No error occurred here to attribute to either
ALSA detecting operation. Instrumentation changes timing; a healthy instrumented
run cannot erase an earlier uninstrumented failure. RIOTBOX-1041 remains open.

## Prospective gates and identity

The user explicitly approved one new instrumented, isolated silent phase.
[V4 / RBX-441](../benchmarks/silent_host_observation_v4.md) was frozen, reviewed
and source-free CI checked before actual host access. Exactly one committed
fence invocation executed; no retry, replacement owner, buffer/priority/rate
tuning, threshold adjustment, physical fallback or opening of old ignored
attempts. Sources, Holdouts, commercial references, capture audio and DAWs
were not accessed; no human playback occurred.

Clean executed revision: `dc6bf5d3a02e5d628762b513de9410e542d53199`, based on
`6edf4ec9f31332dcb9495776907fc7b97bb4ce30`.

- V4 protocol: `36b23fc49def340150c0c63d54ab36b6fb0cc5272249523098d725fea4f7f740`.
- Default-feature `cpal_spike`: `58f6103c3f34ae3837a0b059c209052c36fa5398ded027a4b959dcba67993fc4`.
- Default-feature `silent_host_probe`: `59de339f9488a93958f240c26fea6456bce9e9f1ec872696d75efcd7a7fe778f`.
- `libriotbox_silent_alsa_diag.so`: `335f6d415b05b43749b35f7808660d56e46c63ce9322aae4c02ee4038ff71217`.
- Prospective binding: `dc7b78efec881654f5e9d7c785faad51a3ea8f6bd06933e585d3c40b8ebf3cf0`.
- Consumption marker: `9fa0f67f80804ba348e943445c580b96ae87b3132aaff0b18f47e59f0d65eaf4`.
- Result: `440d01dd2944fc33ff8d3816a4a1cc8eff7b976bd829e7015574a5dfc9e611dc`.
- Private PipeWire-only ALSA config: `87195f491f71dd2594ce1b7f54a1c7a216f4285d5fc93f955b80a1f2c893b789`.

Binding file mtime: `2026-10-09T21:02:22.306427Z`; consumption marker:
`21:02:33.008532Z`; result: `21:05:34.594096Z`. These are operational filesystem
timestamps, not monotonic run durations. Admission/result/marker/protocol and
three binary identities match. Final default-feature build followed exact-head
native CI; no build or tracked edit intervened between binding and execution.

All 165 generated source-free host tests and corrected full local `just ci`
passed (`/tmp/riotbox-1574-ci.log`). Native
[Rust CI 37990118126](https://github.com/marang/riotbox/actions/runs/37990118126)
passed on the exact executed revision before access. Twenty-five diagnostic,
operator and exact-fence controls use only generated evidence/fake providers.
They never open a device or load a real ALSA provider.

Independent Spec/Evidence and Adversarial reviewers found two real preparation
defects: non-writing PCM errors could escape the successful-verdict veto, and
a forked process could register a PCM into the parent's mapping. Both exact
counterexamples failed RED before repair and pass afterward. All-PCM veto and
retained per-PCM evidence, plus ownership checking before open forwarding,
repair them. Independent rechecks and self-review retain zero findings.
Pre-fix CI and failed preparation-only read-only checks are excluded from proof.

## Measurement seam and observed context

The QA-only C interposer reuses the existing Rust drivers and Python operator,
preloaded only into each supervised audio child, not timeout or metadata tools.
It forwards arguments, signed returns and errno, excludes nested ALSA calls,
and records fixed-capacity atomic counters into a prefaulted owner mapping.
The parent reads only after verified process-group cleanup. No Rust product,
Session, replay, Action, renderer or runtime dependency changed.

The parameter getters report the ALSA plugin's applied geometry, not physical
latency or PipeWire quantum. ALSA's
[PCM API](https://www.alsa-project.org/alsa-doc/alsa-lib/group___p_c_m.html)
defines these getters and signed availability/write results. The loader's
[versioned symbol lookup](https://man7.org/linux/man-pages/man3/dlsym.3.html)
is resolved at startup, not on measured calls.

Actual context was a non-container active local Linux user session: UID 1000,
`Active=yes`, `Remote=no`, PipeWire 1.6.9. Owned module `536870916`, sink Node
`84`, serial `2197`; the name intentionally retains the existing operator's
`riotbox_silent_host_v3_` prefix, while admission/result/protocol explicitly
identify V4. No physical/default output was selected by the child.

Driver output was F32 stereo, 44100 Hz. All four processes recorded three PCM
lifetimes, with exactly one writing lifetime (generation 3), closed successfully.
Its final applied ALSA plugin geometry was **1024 buffer frames / 512 period
frames**, with maximum requested write size 512. Supported buffer-range metadata
remained 32–2097152; it is not this applied geometry or a latency measurement.

| Observation | PID | Driver elapsed | Callbacks / avail / write | Admitted route observations |
| --- | --- | --- | --- | --- |
| Preflight | 2553057 | fixed 250-ms driver | 23 / 23 / 23 | 2 |
| Run 1 | 2553392 | 60007 ms | 5169 / 5169 / 5169 | 468 |
| Run 2 | 2558327 | 60007 ms | 5169 / 5169 / 5169 | 469 |
| Run 3 | 2562939 | 60009 ms | 5169 / 5169 / 5169 | 463 |

All three full transcripts contain 62 records, including 60 samples and terminal
`Stopped/ok`. All stream-error/overflow counters are zero, with no retained
error detail. Measured avail/write negative and EPIPE counters are zero across
**all** PCM lifetimes, not merely the writing lifetime. Each active PCM recorded
one top-level prepare and zero recover calls. Wrapper lifetimes were 0.443,
60.227, 60.276 and 60.207 seconds respectively.

Raw measurement SHA256, in preflight/run order:

- `3c6ac28ddf8866dfb8c14b0999f56216efcc1e1033eca735a0ad131f7a4cd57d`.
- `4ccdb51a186f9f1e0a6fb42a6a1dd306fe1e464f642af021cb34ac763132903f`.
- `233bc71628a30712fa0d03bc9ccb3d27d29a12846ba043e4630e4eecbd19aace`.
- `860512882cc3e675892659b87fd9973f054ce9c90022717797d47cdcdcbb6145`.

Run 1/2/3 maximum latest-avail-return→write-entry intervals were 6.151 / 2.307 /
1.585 ms; maximum underlying write durations were 28.907 / 29.282 / 31.635 µs.
Final callback-gap maxima were 21580 / 21629 / 24034 µs. These are instrumented
observations, not isolated render duration, a CPU budget or scheduling-cause
proof. No timing pass threshold was introduced.

## Separate post-run verification and closeout boundary

Only the new ignored owner
`artifacts/audio_qa/local-silent-host-riotbox-1574-v4-2026-10-09/attempt-01`
was opened. Independent post-run checks revalidated complete preflight/NDJSON,
all raw/decoded measurement identities, saved transcript/route digests and every
saved route snapshot. They confirmed the exact admitted route lifetimes and
all 1402 admitted observations above, with no sink-outgoing/foreign links.

Separate read-only real-session metadata checks confirmed unchanged default
sink/mute/volume, absence of the exact owned module/sink lifetime, absence of
all four owned PIDs, PID-associated streams and exact stream Node+serial
lifetimes. Operator result is `pass`, `cleanup_verified=true`, with no cleanup
error. No user service restart, default-output rewrite or uncertain-resource
unload was needed. The phase and authorization are consumed; no rerun is implied.

The result-bound `post-execution-checks.json` retains six exact read-only
metadata responses plus the coordinator's four process-absence observations:
SHA256 `441992094cfba539975955defe625b05d2ab4a7fdca41b4e73376a313f67b8ee`.
Reviewers can independently rederive metadata absence and baseline equality;
the `/proc` absence flags remain contemporaneous coordinator observations.

Final report-only CI/review and PR #1677 closeout follow the ordinary workflow;
exact closing results are recorded in Linear and the project journal.
The bounded task does not establish a fast deterministic reproducer for the
original ALSA trigger. Do not infer a fix or tune buffers/priority from this
positive result; causal work needs that feedback loop and its appropriate
prospective access boundary. RIOTBOX-1041 remains open.
