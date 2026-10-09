# One prospective silent-host V3 attempt — RIOTBOX-1572

Date: 2026-10-09. P017 operational observation of the accepted P023 callback
path. The user explicitly authorized this new once-only silent attempt after
the V3 repair: temporary virtual output, 250-ms preflight and three 60-second
runs, no sources, DAW or physical output. RIOTBOX-1571 is merged at
`58a4eccddf2ff35c0d2e54a0cd578d2da44b30ff`. No attempt has occurred at preparation.

## Frozen boundaries and owner

Reuse [V3](silent_host_observation_v3.md) / RBX-440 without changing its JSON,
source-free default policy, disabled ordinary CLI, operator, Rust driver,
budgets or numeric gates. This separate authorization grants only the exact
invocation below, not a generally enabled launcher. Preserve consumed V1/V2
contracts and evidence; do not open their ignored attempt directories, retry
or reclassify them. This is a reversible operational authorization, not a new
algorithm/access-boundary decision or an additional Decision Log entry.

Exactly one ignored phase root:
`/home/markus/Dev/riotbox/artifacts/audio_qa/local-silent-host-riotbox-1572-v3-2026-10-09`.
Exactly one fresh attempt owner: `attempt-01`. No alternate owner, overwrite,
repair-and-retry or threshold change. Missing/invalid binding, existing owner
or consumption marker fails closed.

Independently review this exact invocation and the existing operator/containment
contracts. Pass source-free local/native CI on the exact reviewed clean commit
before host metadata. Build default-feature `cpal_spike` and `silent_host_probe`
from that revision after CI; exclusively create `prospective-binding.json`:

- `schema`: `riotbox.silent_host_prospective_binding.v1`;
- `owner_ticket`: `RIOTBOX-1572`;
- `owner_path`: the absolute `attempt-01` path above;
- `reviewed_git_revision`: exact 40-character reviewed commit;
- `protocol_sha256`: `a0934097be7b2f160ff698b3d69c1dd7180e86defcdc0974c5e23606297ef85b`;
- `binary_sha256`: exact digests for those two default-feature binaries.

Record concrete identities prospectively and compare them before invoking the
operator. No build or tracked edit between binding and execution. Later
admission records cannot substitute for these comparisons.

## Exact one-shot invocation

Run this reviewed code once from the repository root in the real non-root user
session with `scripts` on Python's import path. The ordinary CLI stays blocked.

```python
import hashlib
import json
from pathlib import Path
import subprocess

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError, V3_PROTOCOL_SHA256, load_protocol_v3
from silent_host_signals import OperatorInterrupts

phase = operator.ROOT / "artifacts/audio_qa/local-silent-host-riotbox-1572-v3-2026-10-09"
binding_path = phase / "prospective-binding.json"
binding_bytes = binding_path.read_bytes()
binding = json.loads(binding_bytes)
owner = phase / "attempt-01"
if (binding.get("schema") != "riotbox.silent_host_prospective_binding.v1"
        or binding.get("owner_ticket") != "RIOTBOX-1572"
        or binding.get("owner_path") != str(owner)
        or binding.get("protocol_sha256") != V3_PROTOCOL_SHA256
        or set(binding.get("binary_sha256", {})) != {"cpal_spike", "silent_host_probe"}
        or owner.exists()):
    raise EvidenceError("prospective phase/owner binding is invalid or consumed")
protocol = load_protocol_v3(operator.ROOT / "docs/benchmarks/silent_host_observation_v3.json")
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
        "owner_ticket": "RIOTBOX-1572",
        "binding_sha256": hashlib.sha256(binding_bytes).hexdigest(),
        "reviewed_git_revision": revision,
    })
    operator.execute_attempt(interrupts, owner=owner)
```

The exclusive marker consumes this authorization before admission, even if
the operator fails before creating `attempt-01`. Never rerun or replace it.
Retain partial/failure evidence and report it.

## Execution, cleanup and evidence limits

Exactly the existing 250-ms preflight and three 60-second silent-default runs,
one-second samples, route polling and independent watchdogs. Only the owned
stereo null sink and private exclusive PipeWire-only child ALSA configuration.
No source/Holdout/commercial/capture audio, directory discovery, DAW, playback,
physical/default fallback, reconnect, stream move, or default/mute/volume/
unrelated resource mutation.

Retain exact process-group, Node ID+serial, PID/target-stream, module ID/type/
name/arguments and baseline checks. Uncertainty retains containment instead of
unsafe unload. Cleanup only established owned resources; do not restore guessed
user state. Independently verify cleanup disposition and compare completed
transcripts with `silent_host_evidence` validators; never tune after results.

Keep bulky evidence ignored; commit a bounded dated report with prospective
and admission identities, actual context, geometry, callback progress/errors,
result and cleanup. RIOTBOX-1041 stays open. A pass proves sampled silent virtual
callback lifetime only, not physical-device, loaded-runtime, latency/xrun,
endurance, source, musical, hardness or release qualification.
