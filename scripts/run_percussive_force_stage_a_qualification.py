#!/usr/bin/env python3
"""Historical RIOTBOX-1428 Stage-A v1 qualification implementation.

RBX-254 permanently closed this v1/Matrix-v2/Registry-v2 route after its first
fail-closed execution. ``run_session`` now refuses before session validation,
contract validation, subprocess preflight, or safe-access callback. The helper
functions remain only for source-blind format/hash fixtures; the exact executed
implementation is reconstructible at Git commit
c60cbb392491950fdbb2edaf15a9f8926db51c71. A future source execution requires
the separately versioned Protocol-v2 runner.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import stat
import struct
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np

import percussive_force_stage_a_analysis as stage_a
from source_holdout_development_access import (
    SourceIdentity,
    parse_strict_pcm_wave,
)
from validate_percussive_force_stage_a_protocol import validate_repository
from validate_source_holdout_rotation import (
    SCHEMA_V2,
    STAGE_A_DEVELOPMENT_CASE_IDS,
    STAGE_A_REGISTRY_PATH,
    resolve_source_format,
    verify_development_source_files,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
PROTOCOL_PATH = Path("docs/benchmarks/percussive_force_stage_a_protocol_v1.json")
MATRIX_PATH = Path("docs/benchmarks/percussive_force_development_matrix_v2.json")
REGISTRY_PATH = Path("docs/benchmarks/source_holdout_rotation_v2.json")
EXPECTED_PROTOCOL_SHA256 = (
    "35091e697cacb3c187f9a33f4f41ac85aba26832a4214bf3251dfc703edad840"
)
EXPECTED_MATRIX_SHA256 = (
    "aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6"
)
EXPECTED_REGISTRY_SHA256 = (
    "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812"
)
PCM_F32LE_HASH_DOMAIN = "riotbox.percussive_force_pcm_f32le.v1"
SESSION_SCHEMA = "riotbox.percussive_force_stage_a_qualification_session.v1"
CATALOG_SCHEMA = "riotbox.percussive_force_stage_a_bound_event_catalog.v1"
REJECTION_SCHEMA = "riotbox.percussive_force_stage_a_qualification_rejection.v1"
COMMIT_SCHEMA = "riotbox.percussive_force_stage_a_qualification_commit.v1"
STAGE_A_V1_EXECUTION_CLOSED_CODE = "stage_a_v1_execution_closed_by_rbx_254"

IMPLEMENTATION_PATHS = (
    Path("Cargo.toml"),
    Path("Cargo.lock"),
    Path("Justfile"),
    Path("scripts/run_percussive_force_stage_a_qualification.py"),
    Path("scripts/percussive_force_stage_a_qualification_fixtures.py"),
    Path("scripts/percussive_force_stage_a_analysis.py"),
    Path("scripts/percussive_force_stage_a_analysis_fixtures.py"),
    Path("scripts/source_holdout_development_access.py"),
    Path("scripts/validate_source_holdout_rotation.py"),
    Path("scripts/validate_source_holdout_rotation_fixtures.py"),
    Path("scripts/validate_percussive_force_stage_a_protocol.py"),
    Path("scripts/validate_percussive_force_stage_a_protocol_fixtures.py"),
    Path("crates/riotbox-audio/Cargo.toml"),
    Path("crates/riotbox-audio/src/lib.rs"),
    Path("crates/riotbox-audio/src/percussive_force/mod.rs"),
    Path("crates/riotbox-audio/src/percussive_force/common.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f1.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f2.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f3.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f3_dynamic.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f3_dynamic/analysis.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f3_dynamic/preflight.rs"),
    Path("crates/riotbox-audio/src/percussive_force/f3_dynamic/preflight/identity.rs"),
    Path("crates/riotbox-audio/src/percussive_force/qualification_pcm.rs"),
    Path("crates/riotbox-audio/src/percussive_force/qualification_pcm/tests.rs"),
)

SOURCE_BLIND_PREFLIGHT_COMMANDS = (
    (sys.executable, "scripts/percussive_force_stage_a_analysis_fixtures.py"),
    (sys.executable, "scripts/percussive_force_stage_a_qualification_fixtures.py"),
    ("just", "source-holdout-rotation-fixtures"),
    ("just", "percussive-force-stage-a-protocol-validator"),
    ("just", "percussive-force-stage-a-protocol-validator-fixtures"),
    ("cargo", "test", "-p", "riotbox-audio", "percussive_force::", "--lib"),
    ("cargo", "fmt", "--check"),
    (
        "cargo",
        "clippy",
        "-p",
        "riotbox-audio",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ),
    ("git", "diff", "--check"),
)


class QualificationSessionError(RuntimeError):
    """A fail-closed session or binding error."""


class StageAV1ExecutionClosed(QualificationSessionError):
    """Typed refusal for every attempted post-RBX-254 v1 source execution."""

    code = STAGE_A_V1_EXECUTION_CLOSED_CODE

    def __init__(self) -> None:
        super().__init__(
            f"{self.code}: Protocol v1 is historical evidence only; "
            "freeze the RBX-254 Protocol-v2 boundary before source access"
        )


@dataclass(frozen=True)
class FrozenSourceSpec:
    case_id: str
    source_family: str
    author: str
    source_path: str
    source_sha256: str
    license: str
    partition: str
    source_format: dict[str, Any]


@dataclass(frozen=True)
class CapturedSource:
    identity: SourceIdentity
    payload: bytes
    access_record: dict[str, Any]


class SourceCaptureOwner:
    def __init__(self, specs: tuple[FrozenSourceSpec, ...]) -> None:
        self._specs = specs
        self.captured: list[CapturedSource] = []

    def __call__(
        self,
        identity: SourceIdentity,
        payload: bytes,
        access_record: dict[str, Any],
    ) -> None:
        index = len(self.captured)
        require(index < len(self._specs), "safe access delivered too many sources")
        expected = self._specs[index]
        require(identity.case_id == expected.case_id, "safe-access case order changed")
        require(identity.source_path == expected.source_path, "safe-access source path changed")
        require(
            identity.expected_sha256 == expected.source_sha256,
            "safe-access source SHA-256 changed",
        )
        require(identity.partition == "development", "non-development source delivered")
        require(
            identity.source_format == expected.source_format,
            "safe-access registered format changed",
        )
        require(isinstance(payload, bytes) and payload, "safe access delivered empty bytes")
        require(
            hashlib.sha256(payload).hexdigest() == expected.source_sha256,
            "delivered raw WAV bytes do not match the frozen source identity",
        )
        require(
            access_record.get("actual_sha256") == expected.source_sha256
            and access_record.get("access_verification_status") == "verified",
            "safe-access delivery record is not verified",
        )
        self.captured.append(CapturedSource(identity, payload, access_record))


class ExclusiveJsonSnapshot:
    def __init__(self, path: Path) -> None:
        self.path = path
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW
        descriptor = os.open(path, flags, 0o600)
        self._file = os.fdopen(descriptor, "w", encoding="utf-8", newline="\n")
        fsync_directory(path.parent)

    def persist(self, value: dict[str, Any]) -> None:
        payload = json.dumps(
            value,
            indent=2,
            sort_keys=True,
            allow_nan=False,
        ) + "\n"
        self._file.seek(0)
        self._file.write(payload)
        self._file.truncate()
        self._file.flush()
        os.fsync(self._file.fileno())

    def close(self) -> None:
        self._file.close()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise QualificationSessionError(message)


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace(
        "+00:00", "Z"
    )


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def read_pinned_json(path: Path, expected_sha256: str) -> tuple[dict[str, Any], bytes]:
    payload = path.read_bytes()
    require(
        sha256_bytes(payload) == expected_sha256,
        f"frozen contract raw SHA-256 changed: {path}",
    )
    value = json.loads(payload, object_pairs_hook=reject_duplicate_keys)
    require(isinstance(value, dict), f"frozen contract root must be an object: {path}")
    return value, payload


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        require(key not in value, f"duplicate JSON key in frozen contract: {key!r}")
        value[key] = child
    return value


def implementation_snapshot() -> dict[str, Any]:
    files: list[dict[str, str]] = []
    for relative in IMPLEMENTATION_PATHS:
        payload = (REPO_ROOT / relative).read_bytes()
        files.append({"path": relative.as_posix(), "sha256": sha256_bytes(payload)})
    encoded = json.dumps(files, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return {
        "schema": "riotbox.percussive_force_stage_a_implementation_snapshot.v1",
        "aggregate_sha256": sha256_bytes(encoded),
        "files": files,
    }


def validate_frozen_source_specs(
    matrix: dict[str, Any], registry: dict[str, Any]
) -> tuple[FrozenSourceSpec, ...]:
    matrix_rows = matrix.get("positive_sources")
    registry_rows = registry.get("entries")
    require(isinstance(matrix_rows, list), "matrix positive_sources must be an array")
    require(isinstance(registry_rows, list), "registry entries must be an array")
    require(
        tuple(row.get("case_id") for row in matrix_rows)
        == STAGE_A_DEVELOPMENT_CASE_IDS,
        "matrix positive-source order changed",
    )
    by_id = {
        str(row.get("case_id")): row
        for row in registry_rows
        if isinstance(row, dict)
    }
    specs: list[FrozenSourceSpec] = []
    for matrix_row in matrix_rows:
        require(isinstance(matrix_row, dict), "matrix positive source must be an object")
        case_id = str(matrix_row["case_id"])
        registry_row = by_id.get(case_id)
        require(isinstance(registry_row, dict), f"missing registry identity: {case_id}")
        for field in (
            "case_id",
            "source_family",
            "author",
            "source_path",
            "sha256",
            "license",
            "partition",
        ):
            require(
                registry_row.get(field) == matrix_row.get(field),
                f"matrix/registry {field} mismatch for {case_id}",
            )
        require(
            registry_row.get("partition") == "development",
            f"positive source is not development-owned: {case_id}",
        )
        source_format = resolve_source_format(
            registry_row,
            case_id,
            schema=SCHEMA_V2,
        )
        matrix_format = matrix_row.get("source_format")
        if matrix_format is not None:
            require(
                all(matrix_format.get(key) == source_format.get(key) for key in matrix_format),
                f"matrix/registry source format mismatch for {case_id}",
            )
        specs.append(
            FrozenSourceSpec(
                case_id=case_id,
                source_family=str(matrix_row["source_family"]),
                author=str(matrix_row["author"]),
                source_path=str(matrix_row["source_path"]),
                source_sha256=str(matrix_row["sha256"]),
                license=str(matrix_row["license"]),
                partition="development",
                source_format=source_format,
            )
        )
    return tuple(specs)


def run_source_blind_preflight() -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    for command in SOURCE_BLIND_PREFLIGHT_COMMANDS:
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            check=False,
            capture_output=True,
            timeout=300,
        )
        combined = completed.stdout + completed.stderr
        record = {
            "argv": list(command),
            "exit_code": completed.returncode,
            "output_sha256": sha256_bytes(combined),
        }
        results.append(record)
        if completed.returncode != 0:
            tail = combined.decode("utf-8", errors="replace")[-2_000:]
            raise QualificationSessionError(
                f"source-blind preflight failed: {' '.join(command)}\n{tail}"
            )
    return results


def validate_completed_access_result(
    access_result: dict[str, Any],
    specs: tuple[FrozenSourceSpec, ...],
    owner_id: str,
    requested_case_ids: tuple[str, ...] = STAGE_A_DEVELOPMENT_CASE_IDS,
) -> tuple[dict[str, Any], ...]:
    require(
        access_result.get("schema")
        == "riotbox.source_holdout_development_access_log.v3",
        "safe access returned an unexpected log schema",
    )
    require(
        access_result.get("session_kind") == "DevelopmentSourceAccessSession",
        "safe access returned an unexpected session kind",
    )
    require(access_result.get("access_status") == "completed", "safe access did not complete")
    require(
        access_result.get("qualification_status") == "not_evaluated_by_access_layer",
        "safe access claimed qualification ownership",
    )
    require(
        access_result.get("mode")
        == "explicit_development_cases_only_no_glob_or_directory_discovery"
        and access_result.get("directory_discovery_performed") is False,
        "safe access did not preserve the no-discovery mode",
    )
    holdout_record = access_result.get("holdout_metadata_comparison")
    require(
        isinstance(holdout_record, dict)
        and holdout_record.get("audio_files_opened") is False,
        "safe access reported holdout audio access",
    )
    require(
        tuple(access_result.get("requested_case_ids", ()))
        == requested_case_ids,
        "safe access request identity or order changed",
    )
    owner_record = access_result.get("qualification_owner")
    require(
        isinstance(owner_record, dict)
        and owner_record.get("owner_id") == owner_id
        and owner_record.get("in_process") is True
        and owner_record.get("delivery_status") == "completed"
        and owner_record.get("delivered_source_count") == len(specs),
        "safe-access owner delivery did not complete exactly",
    )
    opened = access_result.get("opened_development_files")
    require(
        isinstance(opened, list) and len(opened) == len(specs),
        "safe access did not finalize exactly four source records",
    )
    finalized: list[dict[str, Any]] = []
    for index, (record, spec) in enumerate(zip(opened, specs, strict=True)):
        require(isinstance(record, dict), f"safe-access record {index} is not an object")
        require(
            record.get("case_id") == spec.case_id
            and record.get("source_path") == spec.source_path
            and record.get("expected_sha256") == spec.source_sha256
            and record.get("actual_sha256") == spec.source_sha256,
            f"safe-access finalized identity mismatch for {spec.case_id}",
        )
        require(
            record.get("access_verification_status")
            == "verified_and_delivered_to_owner",
            f"safe-access delivery was not finalized for {spec.case_id}",
        )
        actual_format = record.get("actual_source_format")
        require(
            isinstance(actual_format, dict)
            and set(actual_format)
            == {
                "sample_rate_hz",
                "channels",
                "sample_width_bits",
                "compression_type",
                "format_tag",
                "block_align",
                "byte_rate",
                "frame_count",
                "data_bytes",
            },
            f"safe-access finalized PCM format shape mismatch for {spec.case_id}",
        )
        block_align = spec.source_format["channels"] * (
            spec.source_format["sample_width_bits"] // 8
        )
        frame_count = actual_format.get("frame_count")
        require(
            actual_format.get("sample_rate_hz") == spec.source_format["sample_rate_hz"]
            and actual_format.get("channels") == spec.source_format["channels"]
            and actual_format.get("sample_width_bits")
            == spec.source_format["sample_width_bits"]
            and actual_format.get("compression_type")
            == spec.source_format["compression_type"]
            and actual_format.get("format_tag") == 1
            and actual_format.get("block_align") == block_align
            and actual_format.get("byte_rate")
            == spec.source_format["sample_rate_hz"] * block_align
            and isinstance(frame_count, int)
            and not isinstance(frame_count, bool)
            and 0 < frame_count
            <= spec.source_format["sample_rate_hz"]
            * spec.source_format["maximum_duration_seconds"]
            + 1
            and actual_format.get("data_bytes") == frame_count * block_align,
            f"safe-access finalized PCM format mismatch for {spec.case_id}",
        )
        finalized.append(record)
    return tuple(finalized)


def read_and_match_access_log(
    access_log_path: Path,
    access_result: dict[str, Any],
) -> tuple[bytes, dict[str, Any]]:
    flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
    descriptor = os.open(access_log_path, flags)
    try:
        metadata = os.fstat(descriptor)
        require(stat.S_ISREG(metadata.st_mode), "access log is not a regular file")
        require(0 < metadata.st_size <= 1_048_576, "access log size is outside bounds")
        payload = os.read(descriptor, metadata.st_size + 1)
        require(len(payload) == metadata.st_size, "access log changed during bounded read")
    finally:
        os.close(descriptor)
    parsed = json.loads(payload, object_pairs_hook=reject_duplicate_keys)
    require(isinstance(parsed, dict), "access log root must be an object")
    require(parsed == access_result, "persisted access log differs from in-process result")
    return payload, parsed


def decode_captured_source(
    captured: CapturedSource,
    spec: FrozenSourceSpec,
) -> tuple[stage_a.SourceInput, dict[str, Any]]:
    parsed = parse_strict_pcm_wave(
        captured.payload,
        spec.source_format,
        f"StageAQualificationSession:{spec.case_id}",
    )
    sample_bytes = parsed.pop("sample_bytes")
    bits = int(parsed["sample_width_bits"])
    channels = int(parsed["channels"])
    if bits == 16:
        integers = np.frombuffer(sample_bytes, dtype="<i2").astype(np.int32)
        normalized = integers.astype(np.float64) / 32_768.0
        encoding = stage_a.PcmEncoding.PCM_S16LE
    elif bits == 24:
        octets = np.frombuffer(sample_bytes, dtype=np.uint8).reshape(-1, 3)
        unsigned = (
            octets[:, 0].astype(np.int32)
            | (octets[:, 1].astype(np.int32) << 8)
            | (octets[:, 2].astype(np.int32) << 16)
        )
        integers = np.where(unsigned & 0x80_0000, unsigned - 0x100_0000, unsigned)
        normalized = integers.astype(np.float64) / 8_388_608.0
        encoding = stage_a.PcmEncoding.PCM_S24LE
    else:
        raise QualificationSessionError(f"unsupported verified PCM width: {bits}")
    require(
        normalized.size % channels == 0,
        f"decoded PCM channel alignment changed for {spec.case_id}",
    )
    samples = normalized.reshape(-1, channels)
    input_lsb = math.ldexp(1.0, -(bits - 1))
    verified_format = stage_a.VerifiedPcmFormat(
        encoding=encoding,
        sample_rate_hz=int(parsed["sample_rate_hz"]),
        channel_count=channels,
        format_tag=int(parsed["format_tag"]),
        container_bits=bits,
        valid_bits=bits,
        block_align=int(parsed["block_align"]),
        compression_type=str(parsed["compression_type"]),
        input_lsb=input_lsb,
    )
    metadata = stage_a.SourceMetadata(
        case_id=spec.case_id,
        source_family=spec.source_family,
        author=spec.author,
        source_path=spec.source_path,
        source_sha256=spec.source_sha256,
        license=spec.license,
        verified_format=verified_format,
        partition=spec.partition,
    )
    source_input = stage_a.SourceInput(
        metadata=metadata,
        samples=samples,
        sample_rate_hz=verified_format.sample_rate_hz,
        input_lsb=input_lsb,
    )
    binding = {
        "case_id": spec.case_id,
        "source_path": spec.source_path,
        "raw_wav_sha256": spec.source_sha256,
        "pcm_f32le_sha256": pcm_f32le_sha256(
            samples,
            verified_format.sample_rate_hz,
            channels,
        ),
        "frame_count": int(samples.shape[0]),
        "verified_format": verified_format.to_dict(),
        "access_record": captured.access_record,
    }
    return source_input, binding


def pcm_f32le_sha256(
    samples: np.ndarray,
    sample_rate_hz: int,
    channel_count: int,
) -> str:
    frame_count = int(samples.shape[0])
    digest = hashlib.sha256()
    domain = PCM_F32LE_HASH_DOMAIN.encode("utf-8")
    digest.update(struct.pack("<I", len(domain)))
    digest.update(domain)
    digest.update(struct.pack("<I", sample_rate_hz))
    digest.update(struct.pack("<I", channel_count))
    digest.update(struct.pack("<Q", frame_count))
    digest.update(np.asarray(samples, dtype="<f4").tobytes(order="C"))
    return digest.hexdigest()


def create_exclusive_json(path: Path, value: dict[str, Any]) -> str:
    payload = (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode("utf-8")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW
    descriptor = os.open(path, flags, 0o600)
    try:
        with os.fdopen(descriptor, "wb") as output:
            descriptor = -1
            output.write(payload)
            output.flush()
            os.fsync(output.fileno())
        fsync_directory(path.parent)
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    return sha256_bytes(payload)


def fsync_directory(path: Path) -> None:
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    descriptor = os.open(path, flags)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def validate_session_directory(path: Path) -> None:
    require(path.is_absolute(), "session directory must be absolute")
    require(path == path.resolve(strict=True), "session directory must contain no symlink component")
    metadata = os.lstat(path)
    require(not stat.S_ISLNK(metadata.st_mode), "session directory must not be a symlink")
    require(stat.S_ISDIR(metadata.st_mode), "session path must be an existing directory")
    require(metadata.st_uid == os.geteuid(), "session directory must be owned by this user")
    require(
        stat.S_IMODE(metadata.st_mode) & 0o077 == 0,
        "session directory must not grant group or other access",
    )
    try:
        path.relative_to(REPO_ROOT.resolve(strict=True))
    except ValueError:
        pass
    else:
        raise QualificationSessionError("session directory must be outside the repository")
    with os.scandir(path) as entries:
        require(next(entries, None) is None, "session directory must be fresh and empty")


def run_session(session_directory: Path) -> tuple[int, dict[str, Any]]:
    del session_directory
    raise StageAV1ExecutionClosed()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--session-dir",
        required=True,
        type=Path,
        help="fresh absolute directory created for exactly one qualification session",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    os.chdir(REPO_ROOT)
    try:
        exit_code, session = run_session(args.session_dir)
    except Exception as error:
        print(f"FAIL: Stage-A qualification stopped fail-closed: {error}", file=sys.stderr)
        return 1
    artifact = session["qualification_artifact"]
    print(
        "PASS: frozen development-only Stage-A qualification"
        if artifact["passed"]
        else "REJECTED: frozen development-only Stage-A qualification"
    )
    print(f"session_id={session['session_id']}")
    print(f"artifact={artifact['path']}")
    print(f"artifact_sha256={artifact['sha256']}")
    commit_path = Path(session["qualification_commit"]["path"])
    print(f"qualification_commit={commit_path}")
    print(f"qualification_commit_sha256={sha256_bytes(commit_path.read_bytes())}")
    print("candidate_render_started=false")
    print("human_verdict=unverified")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
