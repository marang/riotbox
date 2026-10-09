# One prospective silent-host V2 attempt — RIOTBOX-1570

Date: 2026-10-09. Owner: RIOTBOX-1570 / RBX-439, a separately authorized P017
observation of the accepted P023 callback path. The user explicitly authorized
one isolated silent V2 attempt after the export fix. RIOTBOX-1569 is merged at
`c3f28432f0abe66e5d0b690eaf0a46e85b92588a`, with clean synchronized main and
completed branch closeout. No attempt has occurred at contract preparation.

## Frozen boundaries and owner

Reuse [V2 lifecycle checks](silent_host_observation_v2.md) without changing its
JSON, `source_free_validation_only` policy, disabled legacy CLI, Rust driver,
budgets or numeric gates. This separate phase grants only the invocation below,
not a generally enabled launcher. V1's consumed failed evidence stays unchanged;
do not open its ignored attempt directory or retry/reclassify it.

Exactly one ignored phase root:
`/home/markus/Dev/riotbox/artifacts/audio_qa/local-silent-host-riotbox-1570-v2-2026-10-09`.
Exactly one fresh attempt owner beneath it: `attempt-01`. Do not substitute an
alternate owner or overwrite any existing phase evidence. A missing/invalid
prospective binding, pre-existing attempt owner or consumption marker fails closed.

Before host metadata, independently review this invocation and the existing
operator/containment contracts. Pass source-free local/native CI on the exact
reviewed commit. Build default-feature `cpal_spike` and `silent_host_probe` from
that clean revision, then create the exclusive `prospective-binding.json`:

- `schema`: `riotbox.silent_host_prospective_binding.v1`;
- `owner_ticket`: `RIOTBOX-1570`;
- `owner_path`: the absolute `attempt-01` path above;
- `reviewed_git_revision`: the exact 40-character reviewed commit;
- `protocol_sha256`: `9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea`;
- `binary_sha256`: exact SHA-256 values for those two default-feature binaries.

Those concrete values must be recorded prospectively, compared again before
invocation and retained with the evidence. Operator admission recording values
afterward is not a substitute for these comparisons. No build, tracked edit or
threshold change is permitted between binding and execution.

## Exact one-shot invocation

Run this reviewed code once from the repository root in the real non-root user
session, with the existing `scripts` directory on Python's import path. The
ordinary CLI remains blocked. This is phase-specific operator orchestration,
not a new product API or persisted Session model.

```python
import hashlib
import json
from pathlib import Path
import subprocess

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError, PROTOCOL_SHA256, load_protocol
from silent_host_signals import OperatorInterrupts

phase = operator.ROOT / "artifacts/audio_qa/local-silent-host-riotbox-1570-v2-2026-10-09"
binding_path = phase / "prospective-binding.json"
binding_bytes = binding_path.read_bytes()
binding = json.loads(binding_bytes)
owner = phase / "attempt-01"
if (binding.get("schema") != "riotbox.silent_host_prospective_binding.v1"
        or binding.get("owner_ticket") != "RIOTBOX-1570"
        or binding.get("owner_path") != str(owner)
        or binding.get("protocol_sha256") != PROTOCOL_SHA256
        or set(binding.get("binary_sha256", {})) != {"cpal_spike", "silent_host_probe"}
        or owner.exists()):
    raise EvidenceError("prospective phase/owner binding is invalid or consumed")
protocol = load_protocol(operator.ROOT / "docs/benchmarks/silent_host_observation_v2.json")
revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=operator.ROOT,
                                   text=True, timeout=protocol["metadata_timeout_seconds"]).strip()
dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=operator.ROOT,
                               text=True, timeout=protocol["metadata_timeout_seconds"])
if dirty or revision != binding.get("reviewed_git_revision"):
    raise EvidenceError("reviewed clean revision does not match prospective binding")
for name, expected in binding["binary_sha256"].items():
    if operator.digest(operator.ROOT / "target/debug" / name) != expected:
        raise EvidenceError("reviewed binary does not match prospective binding")
with OperatorInterrupts() as interrupts:
    operator.write_json(phase / "invocation-consumed.json", {
        "schema": "riotbox.silent_host_invocation_consumed.v1",
        "owner_ticket": "RIOTBOX-1570",
        "binding_sha256": hashlib.sha256(binding_bytes).hexdigest(),
        "reviewed_git_revision": revision,
    })
    operator.execute_attempt(interrupts, owner=owner)
```

`write_json` exclusively creates the consumption marker before calling the
operator. The authorization is consumed at that call, including early admission
failure before the operator creates `attempt-01`. Never rerun this invocation,
replace the marker, select another owner, tune gates or repair-and-retry against
this authorization. Retain partial/failure evidence and report it.

## Execution, cleanup and evidence limits

Retain the 250-ms admission probe plus exactly three 60-second silent-default
runs, existing one-second samples, route polling and independent watchdogs.
Use only the exact owned stereo null sink and private PipeWire-only child ALSA
configuration. No physical/default fallback, reconnect, move, source/Holdout/
commercial/capture WAV access, source-directory discovery, DAW, playback or
default/mute/volume/unrelated stream/module mutation.

The existing operator verifies process groups, original Node ID+serial, PID-
attributed and target streams, module ID/type/name/arguments and baseline output.
On uncertainty retain containment instead of unsafe unload. Record an early
error even if no result file was created. Do not restore a changed user default
by guessing or overwrite another user's state. Clean up only established owned
resources and independently verify the operator's cleanup disposition.

Keep bulky command/route transcripts ignored. Commit a bounded dated report
with prospective/admission identities, actual user-session context, negotiated
geometry, callback progress/errors, result and cleanup. Compare any completed
transcript with the existing independent validator. No thresholds are learned
from results. RIOTBOX-1041 remains open: this is sampled silent virtual-endpoint
evidence, not physical-device, loaded-runtime, latency/xrun, endurance, musical,
hardness or release qualification.
