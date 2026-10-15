# Loaded recorder observation adapter V1

RIOTBOX-1578: source-free maintenance of the accepted P023 recording path under
RIOTBOX-1041. Values: [frozen JSON](loaded_recorder_observation_v1.json).
This contract permits implementation and generated tests only, not host access.
It completes one adapter around the existing recorder, not another recorder,
runtime, persistence model or chain of implementation-enabler tickets.

## Existing owners and scope

Use the existing live-master CLI with explicit 2/8/16 bars, a fresh run-Session,
owned destination and optional observer. Keep Core/App queue, tap, timing,
writer, receipt, Session and replay authoritative. Reuse the existing operator's
private PipeWire-only ALSA child environment, sink/module/node identities,
ManagedProcess, independent watchdog, route inspection, interruption and verified
cleanup. Ordinary execution CLI remains disabled. A closed known loaded workload
may vary command/result interpretation; arbitrary launcher/profile injection is
not an interface. Silent V3/V4 contracts and their behavior stay unchanged.

The new protocol inherits the exact existing containment numeric values, not
the silent driver's 60-second minimum lifetime or interval transcript. Three
loaded cases mean one existing two-, eight- and sixteen-bar recording, not
three silent 60-second runs. Preflight stays the existing 250-ms source-free
CPAL sanity command with its existing process budget. No rates, buffer sizes,
priorities, DSP, recording versions, production thresholds or health gates change.
Reuse the opt-in ALSA measurement leaf and its exact ABI/library identity; adapt
only geometry attribution to the recorder's final rate/channel fields. Negative
ALSA operations remain failures. Instrumented results are not zero-overhead proof.

## Fixed fixture and mutable output ownership

Only an explicitly bound fresh
[generated V1 preparation](generated_live_master_load_fixture_v1.md) is eligible.
Admit its exact report, contract, producer, generated source, Graph and capture
identities; constrain every declared path to that known generated owner without
directory discovery. No arbitrary Session or source ingress. Baseline files
remain unchanged and hash-checked before/after cases. Exclusively allocate each
mutable run-Session next to the baseline so relative Graph/capture references
retain their ordinary meaning. Copy exact Session bytes, never rewrite product
JSON or re-ingest a source. The existing recorder saves only the run-local Session
without rewriting the shared Graph. All new filenames and output ownership are
declared before launch; occupied paths reject, no retry or cleanup of unowned data.
Immediately before CLI hydration, recheck the exact registered case identity,
baseline-equal run-Session bytes and still-absent outputs, then consume the case
once. Postflight mutation is expected; stale pre-launch mutation is not. This
does not claim protection against a hostile concurrent writer after admission.

The baseline file SHA256 and Core's actual in-memory pre-capture Session hash
are different identities. Transport confirmation/advance legitimately changes
the latter. Correlate the milestone's actual action/plan hash with proof/receipt,
not with the untouched baseline file hash. A readiness report is Core metadata
validation, never independent WAV/route proof.

## Post-stop milestone, not a success event

The existing final CLI JSON and observer event follow stop, finalization,
publication and Session save. Do not interpret silent-driver NDJSON as recorder
output or permit early route loss merely because the process eventually exits.

Add typed optional `live_master_recording_window_stopped` under the existing
non-interactive observer schema, with observation schema
`riotbox.live_master_recording_window_stopped.v1`. Capture the completed-window
instant in memory; preserve the immediate existing stop and observed-Stopped
guard. Emit/flush the milestone only AFTER verified stop, BEFORE finalization,
WAV/proof or Session I/O. No pre-stop file I/O, periodic logging, callback I/O,
new action or persistent state. Optional observer errors stay latched and visible
in the final observer status without converting a valid product recording into
observer success or prolonging output. Clock failure/non-Linux absence cannot
create qualified timestamps; never fall back to wall time.

Required payload: process PID, `clock_domain=linux_clock_monotonic`,
`window_complete_monotonic_ns`, `runtime_stopped_monotonic_ns`, observed
`window_complete`/`runtime_stopped`/`fault_count`, actual `action_id`, `session_id`,
`session_pre_capture_sha256`, `duration_bars`, `duration_beats`, requested start,
frame count, rate/channels and the existing typed graph/timing/capture lineage
references from the plan. It carries no success/ready verdict. Linux uses explicit
CLOCK_MONOTONIC nanoseconds; generated controller tests inject that same logical
clock domain. Completion must not follow the stopped timestamp.

## Route and terminal evidence

Record every route query's start/end in explicit Linux CLOCK_MONOTONIC ns.
After an admitted route disappears, bounded-pend ONLY otherwise-safe absence
until the milestone arrives, using the existing teardown budget measured from
that first absence. Never defer foreign links, changed route/node identity,
default changes or other unsafe metadata errors. For the first absent query:
end before completion is premature; start at/after completion is eligible;
overlap is ambiguous and rejects. Missing, duplicate, malformed, foreign-PID,
wrong-plan or inconsistent milestones reject. Do not invent interval samples.
The ordinary independent process watchdog still bounds the whole child, and
finalization/drain use the unchanged terminal budget. Retain partial evidence.

The milestone authorizes only bounded terminal drain. Success additionally
requires zero exit, exact final ready/stopped JSON, recorded observer status,
matching new Core action/receipt and persisted run Session, actual proof/WAV
identity/geometry/technical gates, matching diagnostics and verified owned
process/stream/sink cleanup with unchanged default/mute/volume. No marker alone,
metadata-only readiness, template or synthetic verdict is host evidence.
Verify process and attributed-stream removal within the unchanged terminal
budget before independently measuring published artifacts. Postflight sample
analysis does not extend that deadline or the live runtime lifetime. Persist
computed artifact metrics before reporting a later failure, including when
diagnostics or final Core readiness reject the run.

## Verification and execution fence

Exercise real shared-controller milestone/stop/publication ordering with generated
runtime/time/observer controls, preserving default behavior and optional-observer
failure semantics. Generated process/metadata controls cover delayed/missing/
duplicate/wrong marker, premature/ambiguous route loss, finalization hang,
malformed/blocked/nonzero output, foreign/replaced route, uncertain cleanup,
stale/mutated fixture, occupied outputs, unchanged baseline and mismatched proof/
receipt/diagnostic geometry/errors. Independent Spec/Rust/adversarial review,
self-review and full local/native source-free CI gate merge.

No actual audio runtime/host/device/service/audio-metadata operation, real source/
capture/Holdout/commercial audio, source-directory discovery, old ignored attempts,
DAW, playback, broad QA or Windows expansion. Fresh generated files, fake children
and metadata only. Fixture origin and false quality/host-execution claims remain.
Prior host phases are consumed and cannot be substituted or retried. Actual loaded
execution needs a separate prospective versioned phase with exact clean reviewed
revision/native CI/binary/protocol/fixture bindings and fresh explicit approval.
It grants no musical/source-general/hardness/latency/physical-device/endurance/
release claim; RIOTBOX-1041/1036 stay open and the original ALSA trigger remains
unexplained.
