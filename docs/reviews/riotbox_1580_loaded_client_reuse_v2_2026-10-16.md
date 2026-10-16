# RIOTBOX-1580 — explicit loaded-observer Client reuse V2

Date: 2026-10-16. Classification: source-free maintenance/regression for the
accepted P023 recording path, RIOTBOX-1041/P017. Decision: RBX-446;
[frozen V2 contract](../benchmarks/loaded_recorder_observation_v2.md).

## Problem and scope

The [consumed RIOTBOX-1579 observation](riotbox_1579_loaded_recorder_observation_2026-10-16.md)
shows the original audio Node disappearing after the stopped-window milestone,
then its global ID being reused by an ordinary metadata Client. The loaded
supervisor's caller-side occupancy check classified that as changed Node
identity. The actual result remains failed; there is no completed recording
proof to promote. This slice reads that canonical report, not old ignored host
attempts, and does not execute another host test.

The generated feedback loop runs the actual `run_recording()` supervisor with
existing fake process/clock/metadata adapters: admitted route, post-completion
absence, Client reuse, child exit and final validation. It does not mock route,
typed lifetime or cleanup functions. Before implementation:

`PYTHONPATH=scripts python3 -m unittest test_loaded_recorder_process.ProcessTests.test_terminal_client_id_reuse_preserves_original_node_absence`

failed in 0.008 seconds with the exact `EvidenceError: admitted loaded node
identity changed` at the real caller. Log:
`/tmp/riotbox-1580-client-reuse-red.log`. This directly reproduces the failure;
no additional host probing, speculative audio cause or instrumentation is needed.

## Versioned correction, not a relaxed historical verdict

V1 contract/fence bytes, hash/loader, two-argument workload default and original
rejection are retained. V2 must be selected explicitly with
`LoadedObservationVersion.V2` in the same closed workload. Non-enum values reject
before fixture/binary/host work. V2 admission identifies its own schema, protocol
hash and source-free evidence scope. Its loader checks both exact bytes and the
complete type-sensitive V1-derived field/key set. SHA256:
`4cf9514cf33057d67e268f4ee6d234e083f4703d25a2f3d8537de2b3c99564ed`.

Only V2 treats a structurally valid Client at the old Node ID, with a distinct
positive uint64 serial, as original-Node absence. No literal client name/PID
exception, prefix-inferred interface registry, arbitrary launcher or profile
exists. Wrong-serial Nodes, unsupported/malformed interfaces and invalid Client
serials/maps reject. Shared `node_present()`, routing, process/group/stream/sink
cleanup and operator remain unchanged.

All ordinary absence gates still apply: this never establishes completion,
extends a first-absence or stop deadline, authorizes reappearance or skips
foreign/default checks. Artifact, observer, Session/action/receipt, diagnostics
and Core readiness remain mandatory. No product/Rust/recorder/DSP/action/Session/
replay, source, rate/buffer/priority, recording-version or numeric-gate change.

## Evidence and review

The same named regression now passes with explicit V2; an additional V1 control
retains the original rejection. Process plus Client-reuse controls pass 34 tests
(`/tmp/riotbox-1580-client-reuse-controls.log`). Extra cases cover decimal/boundary
serials, missing/bool/negative/zero/overflow/original serial, unknown/malformed/
non-Client interfaces/maps, delayed/missing/bad markers, premature/overlapping
absence, finalization hang, unsafe foreign/default change, route reappearance,
teardown reuse and non-typed version rejection before process/fixture work.

Loader/workload controls verify full V2 inheritance independently of its byte
gate, exact V1/V2 identities, missing/mutated contracts, closed version admission,
explicit matching loader/record attribution and forwarding without skipping Core
validation. Together with the unchanged historical fake fence, 43 focused tests
pass: `/tmp/riotbox-1580-loader-workload-compat.log`.

Independent Maintainer/Adversarial review reports zero retained findings:
56 focused generated tests and all 21 unchanged historical fence tests pass.
Logs: `/tmp/riotbox-1580-independent-maintainer-python.log`,
`/tmp/riotbox-1580-independent-v1-fence.log`. Independent Spec/Evidence review
also reports zero retained findings after 77 focused test executions, including
the historical fence (`/tmp/riotbox-1580-spec-*.log`); its documentation refresh
found no inaccurate claim. Root self-review traced version selection, typed
lifetime admission and unchanged failure/cleanup/final gates and found no
additional issue. No finding is deferred or suppressed as a fixture pass.
Full local/native source-free CI still gate closeout.
Full local log: `/tmp/riotbox-1580-full-ci.log`. Only the installed `just` runner's
temporary directory is relocated to a fresh writable `/tmp` directory because
the sandbox's user-runtime directory is read-only; no audio endpoint is changed.

## Actual evidence still missing

Generated supervisor controls use mocked final artifact/observer evidence seams;
their passing result is not a new WAV/receipt or an actual host recording pass.
The unchanged artifact/observer validators have their own source-free suites.
No actual host/device/service/audio metadata, sink/runtime, source/capture/
Holdout/commercial audio, discovery, DAW, playback or broader QA was accessed.
No human/musical/source-general/hardness/latency/physical-device/endurance/release
claim or explanation of the original ALSA trigger is added.

RIOTBOX-1041/1036 stay open. A future actual observation needs its own prospective
reviewed phase, exact clean revision/native CI/binary/fixture/protocol bindings
and fresh explicit user approval. RIOTBOX-1580 creates neither that owner/fence
nor approval and grants no retry of RIOTBOX-1579.
