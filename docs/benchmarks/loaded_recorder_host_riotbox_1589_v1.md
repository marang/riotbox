# Current-runtime loaded-recorder regression — RIOTBOX-1589

Prospective preparation only. [Frozen phase values](loaded_recorder_host_riotbox_1589_v1.json).
No fresh approval, host observation or result is represented here. The consumed
RIOTBOX-1579 and RIOTBOX-1581 authorizations cannot authorize this new owner.
Use the existing phase V2 schema; `v1` names this ticket's first phase, not a
return to LoadedObservationVersion.V1.

## Purpose and unchanged limits

RIOTBOX-1581 passed its generated loaded-recording observation before
RIOTBOX-1582 corrected observed timing, RIOTBOX-1584 corrected control snapshot
coherence, and RIOTBOX-1587/1588 made history-capacity refusal safe. Their source-free
regressions and reviews do not demonstrate this changed current implementation
through the actual host callback and recording lifecycle. This new phase asks
only whether the repaired current runtime still completes that existing path.

Reuse the unchanged generated V1 fixture, explicit
[loaded V2](loaded_recorder_observation_v2.md), containment V3 and diagnostic V4.
One 250-ms startup check and at most one existing 2/8/16-bar recording at 132 BPM
use only isolated stereo virtual output; first failure stops without retry.
Recorded windows remain approximately 3.64/14.55/29.09 seconds plus existing
startup/stop overhead. All five composite contributors are named: TR-909, MC-202,
W-30 preview, W-30 resample tap and Source Monitor. This is not a new musical
mechanism, per-lane qualification or longer endurance workload. No new adapter,
harness, recorder, renderer, rate/buffer/priority, duration, threshold or artifact
gate. No original ALSA-cause, physical-device, latency, musical, hardness,
source-general, endurance or release claim. RIOTBOX-1041/1036 remain open.

All predecessor protocols and results remain immutable, including V1's failure
and V2's successful but older-build observation. The new JSON inherits every
1581 policy field unchanged; only predecessor identity, owner and path change.
The normal operator CLI remains blocked.

## Prospective ownership and order

Only `artifacts/audio_qa/local-loaded-recorder-riotbox-1589-v1-2026-10-16`
may own this phase. No alternate owner, old ignored attempt or reused fixture.
Preparation uses generated temporary tests only: no actual host metadata,
service/device access, audio child, phase directory or fixture is created.

After independent fence review and full exact-clean-revision local/native
source-free CI pass, build the existing default-feature `cpal_spike`,
`riotbox-app`, `dense_break_live_path_render` and V4 diagnostic library with
the unchanged V4 command. Exclusively create the phase directory (0700) and
prepare only its fresh `fixture-v1` using generated V1. Recheck exact producer,
report and generated source/Graph/capture identity. No real source/capture,
Holdout/commercial audio, source-directory discovery or old owner access.

Actual invocation also requires a new explicit affirmative reply naming this
RIOTBOX-1589 post-repair regression, not generic continuation or an older test.
Only then exclusively retain `fresh-authorization.json` with the unchanged
authorization V1 schema, owner_ticket RIOTBOX-1589, exact phase path/hash,
fresh_user_approval=true and an accurate approval_note identifying that reply
and its scope. Never fabricate, copy or reuse approval; the record is operational,
not cryptographic attestation.

Exclusively retain `prospective-binding.json` with the unchanged binding V1
schema and exact fields checked below: phase/loaded hashes, owner/fixture,
reviewed clean revision, successful exact native Rust CI run/attempt/head, four
binary hashes and raw report/authorization hashes. No build/tracked edit
between binding and invocation. Reject symlink ancestry, nonordinary/unowned
leaves and oversized records before no-follow bounded reads/streaming binary
hashes. This is not hostile concurrent-directory-writer or local-attestation
forgery protection.

The fence validates identities and CI before host metadata, explicitly selects
LoadedObservationVersion.V2, and exclusively publishes `invocation-consumed.json`
immediately before its sole existing operator call. Every subsequent failure
consumes this invocation even without an attempt/audio start. The operator owns
only `attempt-01`, the existing preflight and three cases. Preserve private
PipeWire-only ALSA containment, metadata/watchdog/terminal budgets, process-group,
stream/sink identity and unchanged default/mute/volume. Retain containment if
cleanup is uncertain. No repeated trial, physical/default-output fallback,
source access, DAW or human playback.

## Exact reviewed one-shot fence

The predecessor's executable fence is reused byte-for-byte except phase hash,
owner ticket/path and phase JSON filename. Generated adapters exercise the same
shared safety controls with fresh fake roots, non-executable pseudobinaries and
mocked commands. The new adapter also proves normalized fence identity and
rejects both consumed predecessors' authorization identities. Tests never run a
real host subprocess. This text itself is not approval.

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


PHASE_SHA256 = "a3f8a71e91a9a5f1892c39ab24b69666d8cd9a048cfcda97a920363617fc80c0"
phase = operator.ROOT / "artifacts/audio_qa/local-loaded-recorder-riotbox-1589-v1-2026-10-16"
owner = phase / "attempt-01"
fixture_path = phase / "fixture-v1/preparation.json"
contract = operator.ROOT / "docs/benchmarks/loaded_recorder_host_riotbox_1589_v1.json"
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
    "owner_ticket": "RIOTBOX-1589", "owner_path": str(owner),
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
    "schema": "riotbox.loaded_recorder_host_authorization.v1", "owner_ticket": "RIOTBOX-1589",
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
        "schema": "riotbox.loaded_recorder_invocation_consumed.v1", "owner_ticket": "RIOTBOX-1589",
        "binding_sha256": sha256(binding_bytes), "authorization_sha256": sha256(approval_bytes),
        "reviewed_git_revision": revision, "native_ci_run_id": run_id,
        "native_ci_run_attempt": run_attempt, "native_ci_response_sha256": sha256(ci_bytes),
    })
    operator.execute_attempt(interrupts, owner=owner, diagnostics=diagnostics, workload=workload)
```

## Outcome boundary

Retain all owned raw, partial and computed evidence. Independently correlate
route/query/marker timing, stdout/observer, actual WAV/proof/Session/action/receipt,
Core readiness, ALSA diagnostics and verified cleanup with unchanged defaults.
A stopped milestone or generated test cannot substitute for final recording proof.
Report the first failure honestly or only this bounded instrumented isolated
generated-composite current-runtime regression on success. Do not infer the
original ALSA cause or upgrade the prior frozen result. Normal PR/CI/main-sync
closeout cannot mark RIOTBOX-1041/1036 complete. Missing fresh approval means
preparation only; keep RIOTBOX-1589 open.
