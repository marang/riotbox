# RIOTBOX-1582 — observed callback timing admission

Date: 2026-10-16. Base: `172adb9b1a470658ca6fbf3e13dc63bf605707b8`.
Classification: P017 maintenance/regression protecting the accepted P023
restore/play/recording journey, under RIOTBOX-1041. Contract:
[Audio Core §7.1.1](../specs/audio_core_spec.md#711-observed-callback-timing-availability),
RBX-448. No new sound or host qualification.

## Reproduced problem

Fresh RuntimeTelemetry previously returned false/0 BPM/0 beats before any
callback. A paused restored Session has no pending transport acknowledgment;
the real App consumer accepted that value, overwrote its nonzero runtime and
Session cursor, and the event loop could republish zero to audio control.
This is a scheduling-dependent startup defect, not an observed hardware event.
Runtime construction already seeds requested transport from the TR-909
projection; requested state must not instead be reported as observed timing.

The device-free App regression executed before repair and failed at the actual
timing consumer: expected `12.5`, got `0.0`. Log:
`/tmp/riotbox-1582-unobserved-red.log`. Its original location was
`transport_scene_select`; the final timing-admission family is separately owned.
The final test supplies explicit unavailable timing, while an independent
public-shell regression checks that the producer actually returns unavailable
before the first callback. Together they protect both ends of the real seam;
the consumer does not infer availability from numeric zeros.

## One observation owner, existing consumers

The existing public value fields and re-export stay unchanged. The runtime
method now returns `Option<AudioRuntimeTimingSnapshot>`. `None` is unavailable,
not observed stopped transport, a fault verdict, or a requested-state fallback.
A legitimate observed beat zero remains `Some` and is accepted normally.

One private `CallbackTimingPublication` owns first-publication identity and
running/tempo/completed-position coherence. The unique output callback is its
only writer. Sequentially consistent revision/payload operations publish a
complete value; readers try at most three times and withhold an in-progress or
changed observation. No unchecked last read, callback mutex, allocation, I/O,
logging, sleep or unbounded retry is added. The fixed atomic work is not a
zero-cost or real-time-deadline claim. Separate health counters are unchanged
and are not promoted to one atomic aggregate snapshot.

The existing App consumer takes the optional value and skips absence before
changing Session/runtime clock, pending Play/Pause acknowledgment, queue or
commit state. The existing event-loop forwarding therefore inherits the fix
without a second transport or new App-owned truth. Recorder confirmation only
accepts available observed timing and preserves the existing health checks,
two-second confirmation deadline, ten-millisecond wait and all later capture,
stop, publication, receipt and replay behavior. It cannot arm or publish merely
because timing is unavailable.

## Executed evidence

| Generated check | Passed | Scope |
| --- | ---: | --- |
| App timing admission | 3 | Paused restore, pending Play/Pause and queued-action preservation, actual beat zero |
| Audio lifecycle | 9 | Public shell absent before callback, first observed timing and existing lifecycle |
| Exact production callback heap controls | 5 | Cold formats/active mix/overflow, lifecycle and control churn; isolated child subcases retained |
| Diagnostic-lock controls | 4 | Timing remains independent of the error-buffer mutex; poison behavior unchanged |
| Actual-source timing Loom models | 2 | First publication and one subsequent update versus a reader |
| Recorder/controller after semantic test placement | 20 | Existing cases plus delayed/permanent-unavailable/fault admission |
| Capture/audition regressions | 17 | Existing App behavior with explicit observed input |
| MC-202 timing regressions | 13 | Existing quantized timing consumer |
| Recording App regressions | 21 | Existing recording plans, windows, persistence and queue behavior |

These selections may overlap and are not summed as unique coverage. Logs:
`/tmp/riotbox-1582-app-admission-green.log`,
`/tmp/riotbox-1582-audio-lifecycle.log`,
`/tmp/riotbox-1582-callback-heap.log`,
`/tmp/riotbox-1582-timing-lock.log`,
`/tmp/riotbox-1582-timing-loom-green.log`,
`/tmp/riotbox-1582-recorder-post-move.log`, and
`/tmp/riotbox-1582-{capture-audition,mc202-timing,recording-app}-regressions.log`.

Loom compiles the actual production publication source with only its atomic
primitives substituted, with no time/preemption/permutation pruning. It does
not model the complete runtime, hardware or arbitrary numbers of publications.
Removing the final revision validation fails the update model with the concrete
impossible combination false/128 BPM/64.25 beats; restoring validation passes.
Mutation log: `/tmp/riotbox-1582-coherence-mutation-red.log`. No mutation remains.

New App and recorder timing-admission tests have ordinary semantic modules;
existing fixture ownership and old test bodies remain. This avoids growing
unrelated scene/controller families for the new observation contract. No new
dependency, public test constructor or generic test/launch framework is added.

## Review, gates and limits

Independent code-review/code-review-rust retained no P0-P3 findings under
Maintainer, Adversarial Implementation and Concurrency lenses. That reviewer
independently reran timing Loom 2, App admission 3, recorder availability 2,
confirmation 1 and Audio lifecycle 9 checks: all passed. Logs:
`/tmp/riotbox-1582-review-{callback-loom,app-admission,recorder-availability,recorder-confirmation,audio-lifecycle}.log`.
Independent Spec/Evidence/Rust review also retained no findings and reran Loom
2, App admission 3 and complete recorder/controller 20 checks: all passed.
Logs: `/tmp/riotbox-1582-spec-{loom,app-admission,recorder}.log`.
Both checked the actual RED/mutation records, production callers, contract and
report. Neither claims an independent host or full-runtime model test.

Root Product/Risk self-review retains no finding: no unavailable-as-zero or
requested-as-observed shortcut, no new persistent truth, callback blocking or
publication fallback, and unchanged Core/Session/recording owners. Public Rust
consumers explicitly handle the changed optional result; full workspace CI
remains its integration gate. Full local source-free CI and native PR
CI/merge/main-sync/Linear closeout remain gates. Current full CI log:
`/tmp/riotbox-1582-full-ci.log`.

No new ActionCommand, persistent App state, Session/replay schema, DSP/render
policy, recording/observer/artifact version, rate/buffer/priority or numeric
audio threshold changes. Frozen host phases and their evidence are unchanged;
RIOTBOX-1581 is still consumed and cannot qualify this new implementation.
No host/service/device/audio metadata or real runtime launch, source/capture/
Holdout/commercial audio, ignored attempt, playback, DAW or Windows work occurs.
There is no fresh audible/musical/human/hardness/source-general, latency,
physical-device, endurance, original ALSA-cause or release pass.
RIOTBOX-1041/1036 remain open.
