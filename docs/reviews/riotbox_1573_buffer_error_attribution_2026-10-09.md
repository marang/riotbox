# RIOTBOX-1573 — source-free buffer-error attribution

Date: 2026-10-09. P017 maintenance addressing the accepted P023 callback
reliability blocker recorded by [RIOTBOX-1572](riotbox_1572_silent_host_v3_observation_2026-10-09.md).
This locks down error/stop evidence, not a backend fix or a new host experiment.

## Established error path, not established trigger

`Cargo.lock` and `cargo metadata --locked --offline` resolve CPAL 0.17.3 and
alsa 0.11.0. The recorded string matches the display of
[`StreamError::BufferUnderrun`](https://raw.githubusercontent.com/RustAudio/cpal/v0.17.3/src/error.rs).
Raw health stores display text, not the enum, detecting function or errno.

The pinned [ALSA output backend](https://raw.githubusercontent.com/RustAudio/cpal/v0.17.3/src/host/alsa/mod.rs)
has two EPIPE paths that report this variant: availability polling enters the
XRun handler, while interleaved output writes report it directly. They attempt
prepare/recovery after reporting; possible recovery does not erase the event.
The recorded text cannot select between these sites. This is normal code-path
attribution, not a captured raw-errno or call-site measurement.

ALSA defines EPIPE as playback underrun or capture overrun. Because this driver
opens an output stream, playback underrun is the conventional interpretation
if the normal EPIPE mapping supplied this event; the original generic message
and frozen failed verdict stay unchanged. No particular scheduling/buffering
cause follows from this conditional inference.
[ALSA PCM documentation](https://www.alsa-project.org/alsa-doc/alsa-lib/pcm.html)
defines the direction and recovery semantics.

Riotbox's existing output error closure in `runtime/shared_transport_tr909.rs`
records device-error telemetry and marks an active capture's stream-error counter.
`RuntimeTelemetry::record_stream_error` increments the count and retains the
message. `health_snapshot` projects a running runtime with error evidence as
`Faulted`; `Stopped` remains stopped. The existing probe projection rejects
the error. The silent driver always explicitly stops before terminal reporting
and preserves the earlier failure. This agrees with the real sample-36 event
and sample-36 terminal stop; it is not an operator parsing false positive.

Expected normal path:

```text
ALSA EPIPE → CPAL error callback → sticky error telemetry → Faulted
          → explicit driver stop → failed terminal record → owned cleanup
```

No production driver/operator/runtime, error policy, buffer, scheduling,
threshold, schema, dependency or Session/replay state is changed.

## Fast feedback and regression proof

Replaying only the exact current-phase run-2 NDJSON through the unchanged
independent validator exits 1 for the exact retained stream error:
`/tmp/riotbox-1573-recorded-validator-red.log`. This is deterministic downstream
qualification rejection, **not reproduction of the original ALSA trigger**.
No source audio, old attempts, host services or audio processes were accessed/launched.

The existing generated driver tests passed 11 tests before adding one explicit
interval-backend-error case. Its generated health injection at intervals
1, 36 and 60 proves failed sample plus terminal error retention, exactly bounded
waits, explicit stop before terminal flush and no retry. All 12 driver tests
pass (`/tmp/riotbox-1573-interval-error-tests.log`). This is added policy coverage,
not a product fix or a regression-test RED/GREEN for the backend trigger.

The existing synthetic AudioRuntime fault-projection test passes one selected test
(`/tmp/riotbox-1573-runtime-fault-tests.log`). The corrected targeted telemetry
filter is `runtime::telemetry::telemetry_poison_tests`; an initial `::tests`
filter selected zero and is excluded from proof. Its four tests pass
(`/tmp/riotbox-1573-telemetry-corrected-tests.log`). Full local `just ci` passes
(`/tmp/riotbox-1573-ci.log`). Independent Rust/adversarial review has no retained
findings; the spec/evidence review's test-owner attribution correction is applied
above. Native CI gates PR closeout; exact results are recorded in Linear and
the project journal.

## Missing causal evidence and next safe step

The runtime reports supported buffer range, not negotiated ALSA period/ring
geometry. Callback-gap maxima neither measure render duration nor determine
the detecting EPIPE site. Recorded route snapshots establish sampled isolation,
not per-cycle scheduling or PipeWire xrun causality. One healthy 60-second run
and a later error do not establish a device/backend defect or justify changing
buffers, rates, priority, recovery policy or ignoring xruns.

Under the diagnosis workflow, there is no red-capable, repeatable original
backend trigger within this source-free authorization. Causal hypothesis,
instrumentation and fix phases therefore do not proceed. Source attribution
and generated stop-policy checks are the completed bounded work; the actual
trigger remains unresolved. No replacement host experiment is authorized by
this report, and the consumed 1572 evidence is not relabeled.

The next useful real experiment needs its own prospective authorization and
versioned measurement contract, reusing the existing runtime/containment seam.
It should distinguish the detecting backend operation and capture negotiated
buffering plus relevant timing, rather than repeat the unchanged probe or
guess a configuration fix. No source, DAW, physical-output, human-listening,
endurance, musical or release claim is added. RIOTBOX-1041 remains open.
