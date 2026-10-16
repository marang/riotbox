# Loaded recorder observation — typed Client reuse V2

RIOTBOX-1580 / RBX-446. Source-free maintenance of the accepted P023 recorder
path under RIOTBOX-1041/P017. [Frozen V2 values](loaded_recorder_observation_v2.json).
This grants no actual host access, invocation or retry.

## Narrow correction and historical boundary

The [consumed RIOTBOX-1579 result](../reviews/riotbox_1579_loaded_recorder_observation_2026-10-16.md)
failed after the stopped-window marker: a global ID formerly owned by the audio
Node was reused by the metadata command's non-audio Client. The caller rejected
any occupied ID after the original typed Node lifetime disappeared. The shared
`node_present()` helper already distinguished Node lifetime from global occupancy.
This is an observer false positive, not demonstrated unsafe audio routing or a
production recorder/DSP fault. It cannot retroactively grant a recording pass.

Preserve all [loaded V1](loaded_recorder_observation_v1.md) MD/JSON bytes/hashes,
the 1579 fence/consumption/result and silent V3/V4 behavior. The actual V1 code is
retained at `024274a2c34d6e3b284709d373cabf3b97c73db7`. Keep the existing V1 hash,
loader and two-argument workload constructor semantics; select V2 explicitly
through one closed typed version of the existing workload, never an arbitrary
command/profile or another launcher. V2 admission records identify V2 and its
protocol hash. Historical V1 generated behavior must retain rejection.

## Typed lifetime rule

Run all existing route/default checks first. If the exact original Node ID and
serial remain, V2 changes nothing. If that ID is unoccupied, retain ordinary
bounded absence. If occupied, only a structurally valid
`PipeWire:Interface:Client` with ordinary map info/properties and a positive
uint64 `object.serial` distinct from the original Node serial can mean original
Node absence. Accept the existing supported decimal serial forms, never bool,
negative, missing, overflow or reused original serial. Other occupied interface
types, including unknown/malformed types and an unrelated Node with a different
serial, remain fail-closed. The Client's name/PID does not authorize a route or
change ownership; no special case for the literal `pw-dump` name.

Client reuse is not evidence of completion. Apply the unchanged first-absence
query/completion comparison, delayed-marker bound, stop-based terminal deadline,
independent watchdog, route-reappearance rejection, foreign-link/default/node
checks and cleanup. Premature or overlapping first absence still rejects;
missing/bad/late markers cannot turn it into success or extend any budget.
Verified process/stream cleanup still precedes diagnostics and artifact analysis;
final JSON/observer/Session/action/receipt/proof/WAV/Core/diagnostic requirements
are unchanged. Do not modify the shared lifetime/cleanup semantics to implement
this caller-specific correction. No additional interface vocabulary is inferred
from a prefix; broader non-Node reuse support needs a new explicit decision.

## Verification and limits

Use generated fake process/clock/metadata controls through the real
`run_recording()` call site, not a mocked lifetime predicate: active route,
post-completion absence, recycled Client, process exit, cleanup and final gates.
Require the actual pre-fix failure, V2 success, V1 rejection, delayed/missing/
malformed markers, early/overlapping absence, wrong-serial Nodes, unsupported/
malformed occupied objects, foreign routes, reappearance and unchanged deadlines.
Verify explicit typed workload/record attribution, exact V2/V1 hashes and complete
inheritance of predecessor budgets/access/geometry. Keep all existing suites.
Independent review and full local/native source-free CI gate merge.

No host/device/service/audio metadata, sink/runtime, real source/capture/Holdout/
commercial audio, discovery, old ignored attempts, playback, DAW, broad QA or
Windows work. No new product action, Session/replay model, recorder, DSP,
algorithm/threshold, recording version, rate/buffer/priority or dependency.
Generated control success is not actual recording, original ALSA cause, musical/
human/hardness/source-general/physical-device/latency/endurance/release proof.
RIOTBOX-1041/1036 remain open. Any future actual observation requires a new
prospectively reviewed phase, exact clean revision/native CI/binary/fixture/
protocol bindings and fresh explicit user approval; no actual owner/fence or
approval is created by RIOTBOX-1580.
