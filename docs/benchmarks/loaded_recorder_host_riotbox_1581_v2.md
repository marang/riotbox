# One isolated generated loaded-recorder observation — RIOTBOX-1581 V2

Prospective preparation only. [Frozen values](loaded_recorder_host_riotbox_1581_v2.json).
This is not evidence of fresh user approval or execution. The affirmative reply
for the failed/consumed RIOTBOX-1579 phase cannot authorize this new phase.
Actual access must await a new explicit affirmative reply for this exact V2 test,
after independent review and exact clean-revision local/native CI gates.

## Purpose and unchanged limits

Observe the existing recorder under the fixed generated composite through the
explicit [loaded V2 Client-reuse correction](loaded_recorder_observation_v2.md).
V1's negative result stays unchanged. No new adapter, recorder, launcher, DSP,
source policy, rate/buffer/priority, duration, threshold or artifact gate exists.
One 250-ms startup check and at most one existing 2/8/16-bar recording at 132 BPM
use only isolated stereo virtual output; first failure stops with no retry.
Recorded windows are about 3.64/14.55/29.09 seconds plus existing startup/stop
overhead. No physical speakers/output, DAW or human playback. Name all five
composite contributors: TR-909, MC-202, W-30 preview, W-30 resample tap and Source
Monitor. This does not qualify every lane, real sources, musical quality or
the original ALSA cause. RIOTBOX-1041/1036 remain open.

Reuse unchanged generated V1, containment V3 and diagnostic V4. The normal
operator CLI remains blocked. Only the sole exact fence below may enter the
existing operator once after fresh approval and all binding gates.

## Prospective ownership and order

Only `artifacts/audio_qa/local-loaded-recorder-riotbox-1581-v2-2026-10-16`
may own this phase. No alternate owner, old ignored attempt or reused fixture.
After the preparation fence is independently reviewed and full exact-revision
local/native source-free CI is green, build the existing default-feature
`cpal_spike`, `riotbox-app`, `dense_break_live_path_render` and V4 diagnostic
library with the unchanged V4 command. Exclusively create the phase directory
(0700) and prepare only its fresh `fixture-v1` using generated V1. Recheck exact
producer/report/generated source/Graph/capture identity; no real source/capture/
Holdout/commercial audio or directory discovery.

Only after a new actual affirmative reply, exclusively retain
`fresh-authorization.json` with the unchanged authorization V1 record schema,
owner_ticket RIOTBOX-1581, exact phase path/hash, fresh_user_approval=true and an
accurate approval_note identifying that new reply and scope. It is an operational
record, not cryptographic attestation. Never invent, copy or reuse approval.

Exclusively retain `prospective-binding.json` with the unchanged binding V1
record schema and exact fields checked below: phase/loaded hashes, owner/fixture,
clean reviewed revision, successful native Rust CI run/attempt/head, four binary
hashes, raw report/authorization hashes. No build/tracked edit between binding
and invocation. Reject symlink ancestry/nonordinary/unowned leaves before
no-follow bounded record reads/streaming binary hashes; no hostile concurrent
directory-writer or local attestation-forgery guarantee.

The fence validates every identity and CI response before host metadata. It
explicitly selects LoadedObservationVersion.V2, never V1 by default. Exclusively
publish `invocation-consumed.json` immediately before the single operator call;
every subsequent failure consumes the invocation even without an attempt/audio
start. The operator owns only `attempt-01`, existing preflight and three cases.
Preserve unchanged private PipeWire-only ALSA containment, metadata/watchdog/
terminal budgets, group/stream/sink identity and default/mute/volume. Retain
containment if cleanup is uncertain. No tuning or repeated trial.

## Exact reviewed one-shot fence

Executable review material; generated tests supply only temporary fake roots,
non-executable pseudobinaries and mocked commands. This text itself is not
approval. Both old V1 and new V2 phase adapters exercise the shared generated
safety controls; no real host subprocess is run by their tests.

```python
from contextlib import contextmanager
import hashlib
import os
from pathlib import Path
import re
import stat
import subprocess

import run_silent_host_probe as operator
from loaded_recorder_evidence import (
    V2_PROTOCOL_SHA256 as LOADED_SHA256, LoadedObservationVersion, integer, load_json_bytes,
    load_protocol_v2 as load_protocol, require, sha256,
)
from loaded_recorder_fixture import GeneratedFixture
from loaded_recorder_workload import LoadedRecorderWorkload
from silent_host_diagnostics import AlsaDiagnostics
from silent_host_evidence import EvidenceError
from silent_host_signals import OperatorInterrupts


@contextmanager
def known_regular(path):
    path = Path(path)
    if not path.is_absolute() or ".." in path.parts:
        raise EvidenceError("prospective file path must be explicit and absolute")
    current = Path(path.anchor)
    for part in path.parts[1:-1]:
        current /= part
        if not stat.S_ISDIR(current.lstat().st_mode):
            raise EvidenceError("prospective file parent is not an ordinary directory")
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or before.st_uid != os.geteuid():
        raise EvidenceError("prospective file is not an owned ordinary file")
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), "rb") as source:
        opened = os.fstat(source.fileno())
        if (not stat.S_ISREG(opened.st_mode)
                or (opened.st_dev, opened.st_ino) != (before.st_dev, before.st_ino)):
            raise EvidenceError("prospective file identity changed before reading")
        yield source


def read_record(path):
    with known_regular(path) as source:
        if os.fstat(source.fileno()).st_size > 262144:
            raise EvidenceError("prospective record exceeds existing record budget")
        data = source.read(262145)
    if len(data) > 262144:
        raise EvidenceError("prospective record grew past existing record budget")
    return data


def digest_regular(path):
    with known_regular(path) as source:
        result = hashlib.sha256()
        for chunk in iter(lambda: source.read(65536), b""):
            result.update(chunk)
    return result.hexdigest()


PHASE_SHA256 = "197cbc4fb478c7d7bca2495909ba6acaeb846e26a262721a8d933df6a221f141"
phase = operator.ROOT / "artifacts/audio_qa/local-loaded-recorder-riotbox-1581-v2-2026-10-16"
owner = phase / "attempt-01"
fixture_path = phase / "fixture-v1/preparation.json"
contract = operator.ROOT / "docs/benchmarks/loaded_recorder_host_riotbox_1581_v2.json"
require(digest_regular(contract), PHASE_SHA256, "prospective phase protocol")
for name in ("loaded_recorder_observation_v2.json", "loaded_recorder_observation_v1.json",
             "generated_live_master_load_fixture_v1.json",
             "silent_host_observation_v3.json", "silent_host_observation_v4.json"):
    read_record(operator.ROOT / "docs/benchmarks" / name)
protocol = load_protocol(operator.ROOT)
binding_bytes = read_record(phase / "prospective-binding.json")
binding = load_json_bytes(binding_bytes)
for key, expected in {
    "schema": "riotbox.loaded_recorder_prospective_binding.v1",
    "owner_ticket": "RIOTBOX-1581", "owner_path": str(owner),
    "fixture_report_path": str(fixture_path), "phase_protocol_sha256": PHASE_SHA256,
    "loaded_protocol_sha256": LOADED_SHA256,
}.items():
    require(binding.get(key), expected, "prospective " + key)
if owner.exists() or owner.is_symlink() or (phase / "invocation-consumed.json").exists():
    raise EvidenceError("prospective loaded phase is already occupied or consumed")
approval_bytes = read_record(phase / "fresh-authorization.json")
require(sha256(approval_bytes), binding.get("authorization_sha256"), "authorization bytes")
approval = load_json_bytes(approval_bytes)
for key, expected in {
    "schema": "riotbox.loaded_recorder_host_authorization.v1", "owner_ticket": "RIOTBOX-1581",
    "phase_path": str(phase), "phase_protocol_sha256": PHASE_SHA256,
    "fresh_user_approval": True,
}.items():
    require(approval.get(key), expected, "fresh authorization " + key)
if not isinstance(approval.get("approval_note"), str) or not approval["approval_note"].strip():
    raise EvidenceError("fresh explicit user approval is not recorded")
revision = binding.get("reviewed_git_revision")
if not isinstance(revision, str) or re.fullmatch("[0-9a-f]{40}", revision) is None:
    raise EvidenceError("invalid reviewed Git revision")
timeout = protocol["metadata_timeout_seconds"]
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=operator.ROOT,
                               text=True, timeout=timeout).strip()
dirty = subprocess.check_output(["git", "status", "--porcelain"], cwd=operator.ROOT,
                                text=True, timeout=timeout)
if dirty or head != revision:
    raise EvidenceError("prospective phase requires its exact clean reviewed revision")
run_id = integer(binding.get("native_ci_run_id"), "native CI run ID")
run_attempt = integer(binding.get("native_ci_run_attempt"), "native CI attempt")
if not run_id or not run_attempt:
    raise EvidenceError("native CI identity is missing")
ci_bytes = subprocess.check_output(["gh", "api", f"repos/marang/riotbox/actions/runs/{run_id}"],
                                   cwd=operator.ROOT, text=True, timeout=timeout).encode("utf-8")
if len(ci_bytes) > 262144:
    raise EvidenceError("native CI response exceeds existing record budget")
ci = load_json_bytes(ci_bytes)
for key, expected in {
    "id": run_id, "run_attempt": run_attempt, "head_sha": revision,
    "status": "completed", "conclusion": "success", "name": "Rust CI",
}.items():
    require(ci.get(key), expected, "native CI " + key)
require(ci.get("repository", {}).get("full_name"), "marang/riotbox", "native CI repository")
names = {"cpal_spike", "riotbox-app", "dense_break_live_path_render",
         "libriotbox_silent_alsa_diag.so"}
hashes = binding.get("binary_sha256")
if not isinstance(hashes, dict) or set(hashes) != names:
    raise EvidenceError("prospective binary set is not exact")
for name, expected in hashes.items():
    require(digest_regular(operator.ROOT / "target/debug" / name), expected, "prospective binary")
fixture = GeneratedFixture.admit(
    fixture_path, expected_report_sha256=binding.get("fixture_report_sha256"),
    expected_producer_sha256=hashes["dense_break_live_path_render"],
)
diagnostics = AlsaDiagnostics(operator.ROOT,
                             operator.ROOT / "target/debug/libriotbox_silent_alsa_diag.so",
                             hashes["libriotbox_silent_alsa_diag.so"])
workload = LoadedRecorderWorkload(fixture, hashes["riotbox-app"],
                                  observation_version=LoadedObservationVersion.V2)
with OperatorInterrupts() as interrupts:
    operator.write_json(phase / "invocation-consumed.json", {
        "schema": "riotbox.loaded_recorder_invocation_consumed.v1", "owner_ticket": "RIOTBOX-1581",
        "binding_sha256": sha256(binding_bytes), "authorization_sha256": sha256(approval_bytes),
        "reviewed_git_revision": revision, "native_ci_run_id": run_id,
        "native_ci_run_attempt": run_attempt, "native_ci_response_sha256": sha256(ci_bytes),
    })
    operator.execute_attempt(interrupts, owner=owner, diagnostics=diagnostics, workload=workload)
```

## Outcome boundary

Retain all owned raw/partial/computed evidence. Independently correlate exact
route/query/marker timing, stdout/observer, actual WAV/proof/Session/action/receipt,
Core readiness, ALSA diagnostics and cleanup with unchanged default/mute/volume.
A stopped marker or generated pass cannot substitute for final recording proof.
Report the first failure honestly, or only the bounded instrumented isolated
generated-composite observation on success. No original-cause, human/musical/
hardness/source-general/physical-device/latency/endurance/release claim.
Complete normal PR/CI/main-sync/Linear closeout without marking RIOTBOX-1041/1036
complete. Missing new approval means preparation only; keep this issue open.
