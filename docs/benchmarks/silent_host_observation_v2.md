# Silent virtual-host lifecycle checks V2

Owner: RIOTBOX-1567 / RBX-437, P017 maintenance of the RIOTBOX-1041 observation
seam on the accepted P023 callback path. **Source-free repair only; no host
attempt is authorized.** This is not musical or instrument progress.

## Version and execution boundary

The [V1 protocol](silent_host_observation_v1.md), numeric JSON and
[failed-attempt report](../reviews/riotbox_1566_silent_host_observation_2026-10-03.md)
remain unchanged. The executed V1 implementation is retained in Git at
`cf4f76664a50ba8f054e5e2a9fc19016092c89dd`; no parallel copy is maintained.
Neither V1's result nor its cleanup evidence may be rewritten or reclassified.

The [V2 JSON](silent_host_observation_v2.json) changes only version, lifetime
identity and module parsing. All V1 operational budgets, source exclusions,
watchdogs, stop rules and claim limits remain. The unchanged Rust driver still
embeds V1's identical schedule and emits `riotbox.silent_host_sample.v1`;
V2 versions the operator acceptance, not the driver transcript or DSP.

The normal operator entry point rejects execution, including the old explicit
flag, before creating files, inspecting the host or starting any subprocess.
The existing attempt orchestration remains a generated-test seam with an
explicit caller-supplied temporary owner, not a new fixed real-attempt path.
A future host phase must separately authorize and bind its prospective owner,
reviewed revision and binaries before enabling a launch path. This ticket does
not create that phase or a chain of further prerequisites.

## Node lifetime identity

An admitted route owns a typed Node identity: global ID and `object.serial`.
Record it alongside Client and stereo-link attribution. Reject absent or
malformed serials before admission, and reject a serial change in an active
route even when all integer IDs and links remain equal.

During teardown, reuse of the old ID by a Client or a Node with a different
valid serial is not persistence of the old Node. A surviving original Node
still blocks removal after its Client, links or target property disappear.
Missing/corrupt identity is inconclusive and fails closed, not absence. Keep
the independent PID-attributed-stream, targeting-stream, foreign-link and exact
sink-identity checks; ID reuse never excuses those conditions. Preserve the
same total deadline and typed uncertain-cleanup propagation.

PipeWire documents [object IDs and incrementing 64-bit object serials](https://docs.pipewire.org/group__pw__keys.html).
The V1 trace demonstrated why a recyclable ID alone cannot identify a lifetime.
These are sampled checks in one local server context, not atomic observation
or protection against a malicious server replacing metadata.

## Pulse module identity

Use explicitly text-formatted `pactl list short modules`, with tab-separated
index, module name and argument columns. Accept the optional trailing empty
column emitted by the upstream utility, but reject malformed/duplicate IDs,
ambiguous owned modules, conflicting sink-name arguments, unexpected owned
module type or changed ID. Require exact `sink_name`, stereo channel count and
channel map for this operator's module; do not infer identity from a substring.
Never unload an observed replacement at an already-bound module ID.

The [PulseAudio 17 utility implementation](https://raw.githubusercontent.com/pulseaudio/pulseaudio/v17.0/src/utils/pactl.c)
emits the index in short text output but not in module JSON. Do not read an
invented JSON `index` field or infer indices from array position. A missing
load acknowledgement may resolve only the unique exact owned module, not an
arbitrary null sink. Absence must still be verified against module and sink
metadata within the unchanged cleanup deadline.

Module queries and unload are separate commands, not an atomic compare-and-
unload transaction. Reject observed identity changes; do not claim adversarial
protection against a replacement between the final query and unload. No
defaults, volume, unrelated streams or modules may be intentionally changed.

## Verification and non-claims

Use generated snapshots, transcripts, command adapters and child processes only.
Regressions cover recycled Client/Node IDs, active serial replacement, genuine
orphan Nodes, incomplete identity, lingering targets/foreign links, real
short-module text shapes, absent JSON indices, ID/type/name/argument mismatch,
ambiguity, and late cleanup. Existing watchdog, signal, log/publication failure
and attempt-containment checks remain required. Assert V1 JSON identity and
budget compatibility, and prove that the CLI cannot launch an attempt.

Independent review and source-free local/native CI gate merge. No new host,
source, Holdout, commercial-reference, capture, physical-device or listening
access; no directory discovery, DAW, TUI, Windows or DSP/Session change. Generated
tests cannot establish host, endurance, latency/xrun, musical or release passes.
RIOTBOX-1041 remains open.
