# Silent virtual-host metadata checks V3

Owner: RIOTBOX-1571 / RBX-440. Source-free repair of the concrete metadata
blocker found by [RIOTBOX-1570](../reviews/riotbox_1570_silent_host_v2_observation_2026-10-09.md).
**No real host invocation is authorized.** This is operational maintenance,
not musical progress or another prospective host phase.

## Version and execution boundary

Preserve V1/V2 JSON, contracts, consumed evidence and the exact 1570 invocation.
The executed V2 implementation is retained in Git at
`7201fe63001344dc1003df4d46f7c0d21c529991`, not copied into a parallel subsystem.
Reuse the current operator/modules and unchanged Rust driver. V3 changes only
module framing, the native/Pulse argument distinction, and pre-load named-Node
collision checking. All predecessor numeric values, process supervision,
typed Node ID+serial, signal handling, output containment and claim limits stay.
The ordinary CLI still rejects before files, host inspection or subprocesses.
A future real attempt requires fresh explicit consent and a new prospective
phase/owner/reviewed-revision/binary binding; 1571 cannot grant that permission.

Keep the V2 protocol loader/hash as historical compatibility for the generated
tests of the frozen consumed command. Current orchestration loads pinned V3
through a distinct named loader. This is a protocol leaf, not a second runtime
or launcher. Neither historical loader authorizes execution.

## Lossless, unambiguous paired framing

The [PulseAudio v17 module formatter](https://raw.githubusercontent.com/pulseaudio/pulseaudio/v17.0/src/utils/pactl.c)
provides name/argument strings in short JSON but no module index; short text
provides the actual index and emits argument contents directly. Tabs/newlines
inside native arguments are not record boundaries. Do not scan for apparent
headers, silently skip malformed lines, infer indices from JSON array positions
or invent a JSON `index` field.

Query explicit short JSON and short text once per read, without retries. Require
a JSON array of objects with nonempty, control-free module names and string
arguments (empty arguments allowed). Extra JSON fields have no authority.
Preserve exact argument strings; use their multiset, not JSON order. Text-mode
subprocess newline translation maps CRLF/CR to LF: project argument strings
only for framing while retaining the original JSON values for identity. Distinct
original payloads with indistinguishable projections remain ambiguous.

Consume text from offset zero. Parse each real canonical uint32 decimal index
(excluding UINT32_MAX) and its name, then match the complete JSON-known argument
projection and record terminator. Support the upstream trailing empty tab column
and the existing three-column controls, including the final newline stripped
by `HostCommands.text()`. Never trim argument whitespace. Require exactly one
distinct matching payload/framing per record, unique indices, exact JSON
multiset exhaustion and complete text consumption. Identical payload duplicates
retain their multiplicity, but cannot excuse duplicate indices or duplicate
owned modules. Reject competing prefix candidates immediately; no backtracking
or optimistic recovery even if one candidate might eventually consume the
stream. This conservative availability limit is intentional.

Paired snapshots are not atomic. Churn, mismatch or ambiguity fails closed, not
retry. Shared cleanup deadline checks surround both queries and run during
record/candidate validation. Production pre-load module admission gives the
pair and its cooperative parsing the existing three-second metadata budget;
cleanup passes the existing five-second deadline. Each command still has its
existing timeout. No success or unload is admitted after deadline; a blocking
command may finish its existing timeout before a late response is rejected.
Generated pure framing tests may omit a deadline; production callers may not.
Existing per-command response-size limits apply; this is not a new aggregate
RSS cap or protection against kernel stalls/malicious metadata servers.

## Argument domains and exact ownership

[PipeWire 1.6.9](https://raw.githubusercontent.com/PipeWire/pipewire/1.6.9/src/modules/module-protocol-pulse/pulse-server.c)
emits native module arguments separately from Pulse-compatible module arguments.
Validated names in the `libpipewire-module-*` family have opaque native bodies,
not Pulse shell-token declarations. Do not reject their comments/apostrophes,
quoted bodies or tabs/newlines through `shlex`. They cannot establish a Pulse
owned-null-sink match, regardless of owner-like text inside the body. Keep their
real indices in the complete table: a native record at the bound owned ID is
still an observed replacement and forbids unload/absence claims.

All other module candidates retain strict Pulse token parsing. Ownership needs
exact top-level `sink_name`, expected `module-null-sink` type, unique ownership
and the exact existing stereo argument set. Wrong type, extra/conflicting or
duplicate ownership arguments, malformed Pulse tokens, changed bound ID/type/
arguments and multiple matching modules still fail. Substrings and quoted
property contents never establish ownership. A bound index is checked against
the complete validated table independently of argument-domain interpretation.

Before loading any proposed owned null sink, additionally require its exact
named Node to be absent in validated PipeWire metadata. This protects admission
against existing native-created sink collisions without interpreting opaque
native bodies as Pulse declarations. Missing/malformed Node identity is not
absence. Do not mutate or unload an existing collision. Sampled checks and later
load/unload remain non-atomic; no adversarial inter-command race guarantee.

## Source-free verification and limits

Use generated JSON/text adapters and children only. Cover multiline/tab/CRLF
native bodies, apostrophe comments, empty arguments, fake-header contents,
JSON reordering, repeated payloads, ambiguous prefix/projection matches,
stale/partial tables, malformed fields, canonical/duplicate IDs, terminators,
native replacements at bound IDs, exact/wrong-type/conflicting ownership,
missing load acknowledgement, deadline exhaustion between queries/during
parsing/before unload, and pre-load Node collision. Preserve existing route,
process, signal, result-publication, containment and disabled-CLI tests. Assert
V1/V2 hash preservation and unchanged shared numeric/profile/access fields.

Independent code/contract review and full source-free local/native CI gate
merge. No actual host metadata/service query, audio process, source/Holdout/
commercial/capture access or discovery, physical/default output, listening,
DAW/TUI/Windows expansion or DSP/Session/action change. No callback, device,
endurance, musical, hardness or release pass. RIOTBOX-1041 remains open.
