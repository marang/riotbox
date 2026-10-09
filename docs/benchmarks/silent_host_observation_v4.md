# Silent virtual-host ALSA measurements V4

Owner: RIOTBOX-1574 / RBX-441. Diagnostic loop construction for the accepted
P023 callback reliability blocker, not a guessed xrun fix or musical progress.
The phase-specific authorization below is separate from consumed V1/V2/V3.

## Frozen measurement and safety boundary

Reuse V3's operator, Rust binaries, existing transcript validator, route,
Node+serial/module ownership, private PipeWire-only ALSA configuration,
process/signal supervision, numeric budgets and stop/no-retry policy. The
ordinary CLI remains disabled. V4 adds only a diagnostic adapter. No CPAL fork,
Cargo dependency, renderer, scheduling, buffer/rate, error-ignore, Session,
Action, replay or product runtime change.

Use one reviewed QA-only C ABI interposer, dynamically preloaded exclusively
into each exact audio child via `env`, never into GNU timeout, host metadata,
the Python operator or unrelated processes. C is limited to this ALSA FFI
measurement leaf; the instrument and runtime remain Rust. Resolve real ALSA
symbols before audio startup; forward arguments, return values and errno.
Resolve ALSA_0.9 operations and ALSA_0.9.0rc4 modern parameter getters explicitly;
nested library calls are forwarded but excluded from top-level CPAL attribution.
Never inspect sample buffers, PCM names or source files. No logging, dynamic
allocation, mutex, file I/O or symbol resolution in measured avail/write calls.

At startup map one exclusively precreated, fixed-size owner file (0600),
prefault it and check the Linux x86-64/little-endian/lock-free-u64 ABI. Missing
symbols, invalid file, foreign PID or PCM capacity exhaustion terminates the
owned child rather than running without diagnostics. The parent reads only
after verified process-group cleanup, never concurrently with a writer.

ABI V1: eight 64-bit header words (magic `RBXALSA1`, version 1, PID, used PCM
slots, lost measurements, ready, clock failures, monotonic start ns), followed
by 64 slots of 29 words. Each slot owns one successful open/close lifetime;
pointer reuse cannot reuse an old slot. Capture last successfully applied
hardware parameters and read current buffer/period frames plus applied
channels/rate on the control path. Select exactly one writing playback PCM,
require valid captured geometry, close success and nonzero avail/write coverage.
This is ALSA plugin geometry, not the PipeWire graph quantum or physical latency.

Per-operation avail/write aggregates retain count, negative-return count,
first negative code, EPIPE count/first timestamp, last call-start, maximum
start-to-start gap, maximum underlying-call duration and maximum requested
frames (zero for avail). Also count prepare/recover and maximum latest
availability-return to next-write entry. These are raw monotonic measurements,
not render duration or scheduling-cause proof. EPIPE -32 distinguishes the two
normal pinned CPAL reporting sites if it occurs. No new performance pass
threshold or ignored error class. A generated error injection tests capture,
not the original ALSA trigger. Preserve original host/transcript failure even
when diagnostic validation also fails; retain partial evidence and notes.
Any top-level negative avail/write evidence forbids a successful V4 observation,
including an error racing the older pre-stop preflight snapshot. This preserves
the zero-error principle rather than adding a timing tolerance.

Instrumentation perturbs timing; no claim of zero observer cost, soundcard,
latency, loaded-runtime, endurance, musical, hardness or release qualification.
An all-green diagnostic run cannot explain or erase the earlier failed run.
No repeated trials, load/stress, buffer or priority tuning, physical fallback,
source/Holdout/commercial/capture audio, discovery, DAW or human playback.
RIOTBOX-1041 remains open. Local/native source-free CI and independent review
gate actual access. Diagnostic ABI/source/binary/protocol identities are pinned.

## Exactly one authorized prospective phase

The user explicitly approved on 2026-10-09 the new instrumented silent test:
250-ms preflight and at most three 60-second runs, first failure stops, no retry.
Only `artifacts/audio_qa/local-silent-host-riotbox-1574-v4-2026-10-09/attempt-01`
may own it. Preserve old ignored evidence unopened. A prospective binding
must name RIOTBOX-1574, exact owner, clean reviewed 40-character Git revision,
V4 protocol SHA256, default-feature `cpal_spike`/`silent_host_probe` and diagnostic
shared-library SHA256. Compare all before host metadata; exclusively create a
consumption marker before calling the existing operator. Any early failure
consumes this one invocation. No build/tracked edit between binding and call.
The exact reviewed executable fence below is the only launcher; this prose
alone grants no launch. Retain admission/result/diagnostic bytes and verify owned
cleanup and unchanged baseline independently afterward.

## Exact reviewed one-shot fence

Build the two default-feature Rust binaries and the C measurement library from
the reviewed clean revision after CI. C command:
`cc -std=c11 -Wall -Wextra -Werror -O2 -fPIC -shared scripts/native/silent_host_alsa_diagnostics.c -ldl -o target/debug/libriotbox_silent_alsa_diag.so`.
Exclusively create `prospective-binding.json` in the phase root using schema
`riotbox.silent_host_prospective_binding.v2`, owner_ticket, owner_path,
reviewed_git_revision, protocol_sha256 and binary_sha256 for exactly the three
names below. Concrete digests are retained before this sole invocation.

```python
import hashlib
import json
import subprocess

import run_silent_host_probe as operator
from silent_host_diagnostics import AlsaDiagnostics, V4_PROTOCOL_SHA256, load_protocol_v4
from silent_host_evidence import EvidenceError
from silent_host_signals import OperatorInterrupts

phase = operator.ROOT / "artifacts/audio_qa/local-silent-host-riotbox-1574-v4-2026-10-09"
binding_bytes = (phase / "prospective-binding.json").read_bytes()
binding = json.loads(binding_bytes)
owner = phase / "attempt-01"
names = {"cpal_spike", "silent_host_probe", "libriotbox_silent_alsa_diag.so"}
if (binding.get("schema") != "riotbox.silent_host_prospective_binding.v2"
        or binding.get("owner_ticket") != "RIOTBOX-1574"
        or binding.get("owner_path") != str(owner)
        or binding.get("protocol_sha256") != V4_PROTOCOL_SHA256
        or set(binding.get("binary_sha256", {})) != names
        or owner.exists()):
    raise EvidenceError("prospective V4 binding is invalid or consumed")
protocol = load_protocol_v4(operator.ROOT)
revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=operator.ROOT,
                                   text=True, timeout=protocol["metadata_timeout_seconds"]).strip()
dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=operator.ROOT,
                               text=True, timeout=protocol["metadata_timeout_seconds"])
if dirty or revision != binding.get("reviewed_git_revision"):
    raise EvidenceError("reviewed clean revision does not match prospective binding")
for name, expected in binding["binary_sha256"].items():
    if operator.digest(operator.ROOT / "target/debug" / name) != expected:
        raise EvidenceError("reviewed V4 binary does not match prospective binding")
diagnostics = AlsaDiagnostics(operator.ROOT,
                             operator.ROOT / "target/debug/libriotbox_silent_alsa_diag.so",
                             binding["binary_sha256"]["libriotbox_silent_alsa_diag.so"])
with OperatorInterrupts() as interrupts:
    operator.write_json(phase / "invocation-consumed.json", {
        "schema": "riotbox.silent_host_invocation_consumed.v1",
        "owner_ticket": "RIOTBOX-1574",
        "binding_sha256": hashlib.sha256(binding_bytes).hexdigest(),
        "reviewed_git_revision": revision,
    })
    operator.execute_attempt(interrupts, owner=owner, diagnostics=diagnostics)
```
