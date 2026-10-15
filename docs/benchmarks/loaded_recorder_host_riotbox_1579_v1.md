# One isolated generated loaded-recorder observation — RIOTBOX-1579 V1

Prospective phase only. [Frozen values](loaded_recorder_host_riotbox_1579_v1.json).
This document is not evidence of user approval or execution. The explicit
question presented on 2026-10-15 requests one 250-ms admission probe and one
2-, 8- and 16-bar recording of the freshly generated composite, solely through
an isolated virtual output, with first-error stop and no retry. Until a fresh
affirmative answer actually arrives, only source-free preparation is permitted.
No old authorization or consumed phase can substitute.

## Purpose and non-goals

Exercise the actual existing recorder under the fixed generated instrument
load and verify its persisted results. This is the next real observation after
the complete RIOTBOX-1578 adapter, not another adapter-enabler ticket. Name all
five composite contributors. The generated signal is nonzero, but nothing is
played to the listener, physical speakers or DAW. No new sound policy, source
intelligence, musical verdict, physical-device, latency, endurance or release
qualification. The original ALSA cause may remain unexplained. RIOTBOX-1041/1036
remain open independently of this observation's outcome.

Reuse unchanged [loaded V1](loaded_recorder_observation_v1.md),
[generated preparation V1](generated_live_master_load_fixture_v1.md), containment
V3 and diagnostic V4 contracts. Only this phase-specific fence can authorize
one new call after all its gates and actual user approval. The ordinary operator
CLI stays blocked. No durations, output rates, buffer sizes, priorities, DSP,
action/Session/replay truth, algorithms or thresholds change from results.

## Exact ownership and admission order

Only `artifacts/audio_qa/local-loaded-recorder-riotbox-1579-v1-2026-10-15`
may own this phase. Exclusively create that directory (0700); do not search for
owners or inspect any old ignored attempt. Its fresh `fixture-v1` is prepared
through the existing default-feature generated V1 CLI. No real source/capture/
Holdout/commercial audio, discovery or arbitrary fixture is eligible. Only exact
declared generated files and new owned outputs can be read or hashed.
Before opening binding, approval, protocol or binary files, reject symlink
ancestry and nonregular leaves. Bounded JSON reads and streaming binary hashes
use no-follow descriptors. No hostile concurrent directory-writer or local
attestation-forgery guarantee is claimed.

Before any actual host/service/audio metadata or sink operation:

1. Independently review this exact fence and its generated tests; pass full
   local/native source-free CI at the exact clean revision.
2. Build default-feature `cpal_spike`, `riotbox-app`,
   `dense_break_live_path_render` and the existing diagnostic library at that
   revision. Use the unchanged V4 library build command. Prepare the fresh
   generated fixture and record the actual producer/report identities.
3. Only after an actual affirmative user reply, exclusively retain
   `fresh-authorization.json` with schema
   `riotbox.loaded_recorder_host_authorization.v1`, this owner ticket, exact
   phase path, this phase protocol hash, `fresh_user_approval=true`, and a concise
   `approval_note` identifying that fresh reply and its exact scope. This is an
   operational record, not a cryptographic human-attestation mechanism.
4. Exclusively retain `prospective-binding.json` with the fields checked below.
   Bind the successful native Rust CI run ID/attempt and exact head, all four
   binary hashes, this phase and loaded protocol hashes, fixture report and
   authorization bytes. No build/tracked edit between binding and invocation.
5. Run only the exact reviewed fence below, once. It validates all bindings and
   native CI before creating `invocation-consumed.json` exclusively, immediately
   before entering the existing operator. Any subsequent failure consumes the
   invocation even if no attempt directory or audio runtime was created.

The operator owns `attempt-01`, one existing 250-ms preflight and at most one
recording of each existing 2/8/16-bar duration. At 132 BPM the recorded windows
are approximately 3.64/14.55/29.09 seconds, plus existing bar/start/stop overhead.
All existing watchdog, route, metadata and teardown budgets remain unchanged.
Keep default output, mute, volume and unrelated streams unchanged. Verify
owned process/stream/sink removal; retain containment if uncertain. Stop on the
first failure; no tuning, retries, alternate owner or wider measurement.

## Exact reviewed one-shot fence

The following is executable review material, not a generally enabled launcher.
Tests extract and execute it only against generated adapters and metadata.

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
    PROTOCOL_SHA256 as LOADED_SHA256, integer, load_json_bytes,
    load_protocol, require, sha256,
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


PHASE_SHA256 = "9a2a924fa0eab54d592d9d349ca72bd22eec6abba59bcb53b1e62a19be1272c4"
phase = operator.ROOT / "artifacts/audio_qa/local-loaded-recorder-riotbox-1579-v1-2026-10-15"
owner = phase / "attempt-01"
fixture_path = phase / "fixture-v1/preparation.json"
contract = operator.ROOT / "docs/benchmarks/loaded_recorder_host_riotbox_1579_v1.json"
require(digest_regular(contract), PHASE_SHA256, "prospective phase protocol")
for name in ("loaded_recorder_observation_v1.json", "generated_live_master_load_fixture_v1.json",
             "silent_host_observation_v3.json", "silent_host_observation_v4.json"):
    read_record(operator.ROOT / "docs/benchmarks" / name)
protocol = load_protocol(operator.ROOT)
binding_bytes = read_record(phase / "prospective-binding.json")
binding = load_json_bytes(binding_bytes)
for key, expected in {
    "schema": "riotbox.loaded_recorder_prospective_binding.v1",
    "owner_ticket": "RIOTBOX-1579", "owner_path": str(owner),
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
    "schema": "riotbox.loaded_recorder_host_authorization.v1", "owner_ticket": "RIOTBOX-1579",
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
workload = LoadedRecorderWorkload(fixture, hashes["riotbox-app"])
with OperatorInterrupts() as interrupts:
    operator.write_json(phase / "invocation-consumed.json", {
        "schema": "riotbox.loaded_recorder_invocation_consumed.v1", "owner_ticket": "RIOTBOX-1579",
        "binding_sha256": sha256(binding_bytes), "authorization_sha256": sha256(approval_bytes),
        "reviewed_git_revision": revision, "native_ci_run_id": run_id,
        "native_ci_run_attempt": run_attempt, "native_ci_response_sha256": sha256(ci_bytes),
    })
    operator.execute_attempt(interrupts, owner=owner, diagnostics=diagnostics, workload=workload)
```

## Outcome and closeout

Retain the prospective/authorization/consumption records and all owned attempt
files. Independently correlate route, milestone, final observer/stdout, published
WAV/proof/Session/action/receipt, Core readiness and ALSA measurements. Report
first failure honestly without changing frozen gates. A successful result is
only an instrumented isolated generated-composite recording observation, never
causal, source-general, musical, physical-device or endurance qualification.
Record cleanup and unchanged defaults even on failure. Complete normal PR/CI/
main-sync/Linear closeout; do not mark broader parents complete by implication.
