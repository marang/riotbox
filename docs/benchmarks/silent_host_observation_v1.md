# Silent virtual-host observation V1

Owner: RIOTBOX-1566, partial P017 / RIOTBOX-1041 operational evidence for the
accepted P023 callback path. Classification: maintenance / observation, not an
audible mechanism, physical-device qualification or a new telemetry authority.

The [frozen numeric protocol](silent_host_observation_v1.json) is shared by the
dev-only Rust driver and Linux operator. Freeze it before the first host probe;
any changed limit or acceptance condition requires a new version and decision.
Durations are bounded experiment budgets, not product performance thresholds.

## Attempt and observations

One attempt admits one existing 250 ms `cpal_spike` preflight, then three new
processes, each running `AudioRuntimeShell::start_default_output()` for 60
one-second observation intervals. Leave `cpal_spike` unchanged. All default
lanes are idle, transport stopped and source-free. No setters, Session/Graph
hydration, capture, source lookup or audible content is admitted.

At each interval health must remain Running, pass RBX-435, and callback
count must advance from the preceding observation. Preserve raw elapsed time,
negotiated output, counters and maximum-gap diagnostics. A final positive count
alone does not pass; no gap/latency/xrun threshold is introduced. Sampled progress
does not imply activity at every instant between samples. `stop()` and process/
stream removal are checked after each run before a subsequent run is allowed.

## Host and route admission

Verify active local Linux user-session context, local PipeWire, tools, clean
reviewed revision and exact binary/protocol hashes before launching audio.
Create one uniquely named stereo null sink, retain its module ID, node ID and
object serial, and preserve the original default output, mute and volume state.
Never change defaults/volumes, move unrelated streams, or use a physical fallback.

Before any CPAL open, the operator supplies a private, exclusive
`ALSA_CONFIG_PATH` containing only a stereo `pcm.!default` of `type pipewire`,
the local `pipewire-0` server and exact `playback_node` serial. No includes,
hooks, hardware PCM or fallback exist in that file; user/global ALSA defaults
are never modified. Missing PipeWire support must fail, not open hardware.
Metadata and child commands share the same local user runtime/core and Pulse
socket. Clear inherited ALSA, PipeWire, Pulse, SPA and dynamic-library injection
overrides, then explicitly set the route properties. Keep the private config
and its hash with the ignored attempt metadata.

This follows [ALSA's top-level configuration loading](https://raw.githubusercontent.com/alsa-project/alsa-lib/v1.2.14/src/conf.c),
[PipeWire's ALSA plugin](https://raw.githubusercontent.com/PipeWire/pipewire/1.6.9/pipewire-alsa/alsa-plugins/pcm_pipewire.c)
and [stream property precedence](https://raw.githubusercontent.com/PipeWire/pipewire/1.6.9/src/pipewire/stream.c).

Launch each process with `target.object` set to that exact serial and
`node.dont-fallback`, `node.dont-reconnect`, `node.dont-move` true. Bind the actual
CPAL PID through Client ID to Node ID, not the requested node name. Retain route
snapshots, including unmatched ones. Within the startup budget require two
active FL/FR links to the exact sink and all three properties. Reject any other
outgoing stream link, any incoming foreign stream or any outgoing sink link.
Incomplete but otherwise safe links may wait only within the startup budget.
After admission, loss or change of the route fails immediately. After all 60
successful intervals, natural teardown may drain those same safe links within
the teardown budget; complete stopped evidence and disappearance still gate the
result. The short preflight permits disappearance only at successful process
exit. Wrong targets, properties or foreign links never become acceptable.

The terminal deadline is conservatively anchored to operator prelaunch time
plus the final sample's validated driver elapsed time and the frozen teardown
budget. Driver timing starts later, so this can shorten but never extend the
budget. A slow metadata observation cannot restart the clock at detection time.

Property semantics are documented by the
[PipeWire developers](https://docs.pipewire.org/page_man_pipewire-props_7.html)
and [WirePlumber linking policy](https://pipewire.pages.freedesktop.org/wireplumber/policies/linking.html).
Requested properties are not proof of effective routing; observed links are
mandatory. Only local ignored metadata keeps full snapshots; committed reports
contain this experiment's attribution and summaries, not unrelated stream data.

## Supervision and failure

All metadata commands have explicit timeouts. GNU `timeout` independently bounds
each launched process, including a two-second forced-kill grace. The operator
owns a new process group and terminates it on failure, exception or interruption.
Keep the group leader unreaped until group cleanup, so its PID cannot be reused
as an unrelated kill target. Verify the process and attributed stream disappear
within the teardown budget. Only then unload the exact newly created sink and
verify it is gone; preserve unrelated nodes and every failed artifact.

The output owner must be newly created; never overwrite or retry a prior
attempt. The first failed preflight, route, progress, counter, supervision,
teardown or default-state check terminates the attempt. No replacement run,
physical fallback or result-driven tuning is authorized. Record a capability or
observation failure truthfully; lack of callback evidence alone does not prove
a broken backend. If cleanup cannot be verified, report the exact residual
process/module rather than claiming silence or deleting unrelated state.

TERM, HUP and INT unwind the normal operator path; repeated signals do not
interrupt its bounded cleanup. A first interruption during final publication
must not leave successful attempt evidence. Unverified process **or attributed
stream** removal retains containment, including when the Client object has
already disappeared. SIGKILL, host loss and deliberate child escape from its
owned group are not recoverable guarantees of this trusted-driver supervisor.

Only `python3 scripts/run_silent_host_probe.py --execute-reviewed-attempt`
executes the fixed attempt. It requires a clean reviewed revision and already
built `target/debug/cpal_spike` and `target/debug/silent_host_probe`; build both
from that revision before admission. The fixed ignored owner is
`artifacts/development/riotbox-1566/attempt-01`. Do not remove it to enable a
retry. `just silent-host-contract-fixtures` never invokes this entry point.

## Source-free tests and claim boundary

Test the driver's pure observation/reporting interface, generated route
admission fixtures and real generated child-process supervision before host
launch. Negative cases include PID/serial/property/link mismatches, malformed
or missing observations, stalled counters, command timeouts, early exit,
watchdog expiry and operator interruption. Tests must never start CPAL or
contact the host audio service. Independent review gates the actual attempt.

The result can establish only repeated silent virtual-endpoint callback and
process-lifetime observations in the recorded environment. It cannot establish
loaded musical playback, physical device or acoustic behavior, latency, xrun
absence, general endurance, source quality, human taste, release readiness or
RIOTBOX-1041 completion. No human playback is planned or authorized here.
