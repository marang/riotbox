#!/usr/bin/env python3
"""Holdout-safe, exact-path development source access for Riotbox Stage A."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import stat
import struct
import sys
import uuid
from array import array
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Iterable, TextIO


LEGACY_SOURCE_FORMAT = {
    "sample_rate_hz": 48_000,
    "channels": 2,
    "sample_width_bits": 16,
    "compression_type": "NONE",
    "maximum_duration_seconds": 16,
}
V2_SOURCE_FORMATS = {
    "oga_william_hector_horde_war_drums": {
        "sample_rate_hz": 44_100,
        "channels": 2,
        "sample_width_bits": 24,
        "compression_type": "NONE",
        "maximum_duration_seconds": 16,
    },
    "oga_frosty_ham_osdrums": {
        "sample_rate_hz": 44_100,
        "channels": 2,
        "sample_width_bits": 16,
        "compression_type": "NONE",
        "maximum_duration_seconds": 16,
    },
}
MAX_RIFF_CONTAINER_OVERHEAD_BYTES = 1_048_576
STAGE_A_REGISTRY_SCHEMA = "riotbox.source_holdout_rotation.v2"
STAGE_A_REGISTRY_RAW_SHA256 = (
    "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812"
)


@dataclass(frozen=True)
class SourceIdentity:
    case_id: str
    source_path: str
    expected_sha256: str
    partition: str
    source_format: dict[str, Any]


@dataclass(frozen=True)
class DevelopmentAccessPlan:
    selected: tuple[SourceIdentity, ...]
    active_holdout_count: int


@dataclass(frozen=True)
class PinnedStageARegistry:
    path: Path
    schema: str
    raw_sha256: str


def load_pinned_stage_a_registry(
    manifest_path: Path,
    caller_manifest: dict[str, Any],
) -> tuple[PinnedStageARegistry, dict[str, Any]]:
    """Read, hash, and strictly parse one immutable Stage-A registry snapshot."""

    payload = manifest_path.read_bytes()
    actual_sha256 = hashlib.sha256(payload).hexdigest()
    require(
        actual_sha256 == STAGE_A_REGISTRY_RAW_SHA256,
        f"{manifest_path}: Stage-A registry raw SHA-256 does not match the frozen pin",
    )
    parsed_manifest = parse_strict_json_object(payload, str(manifest_path))
    require(
        parsed_manifest.get("schema") == STAGE_A_REGISTRY_SCHEMA,
        f"{manifest_path}: Stage-A registry schema does not match the frozen pin",
    )
    require(
        parsed_manifest == caller_manifest,
        f"{manifest_path}: caller manifest does not match the pinned raw registry bytes",
    )
    return (
        PinnedStageARegistry(
            path=manifest_path,
            schema=STAGE_A_REGISTRY_SCHEMA,
            raw_sha256=STAGE_A_REGISTRY_RAW_SHA256,
        ),
        parsed_manifest,
    )


def preflight_development_identities(
    identities: Iterable[SourceIdentity],
    requested_case_ids: list[str],
    *,
    prefix: str,
) -> DevelopmentAccessPlan:
    """Pure metadata preflight; this function performs no filesystem access."""

    identity_list = list(identities)
    by_id = {identity.case_id: identity for identity in identity_list}
    require(
        len(by_id) == len(identity_list),
        f"{prefix}: source identity list contains duplicate case IDs",
    )
    require(requested_case_ids, f"{prefix}: at least one exact development case is required")
    require(
        len(requested_case_ids) == len(set(requested_case_ids)),
        f"{prefix}: duplicate development verification case",
    )
    holdouts = [
        identity
        for identity in identity_list
        if identity.partition.startswith("holdout_")
    ]
    holdout_ids = {identity.case_id for identity in holdouts}
    holdout_paths = {identity.source_path for identity in holdouts}
    holdout_hashes = {identity.expected_sha256 for identity in holdouts}

    selected: list[SourceIdentity] = []
    for case_id in requested_case_ids:
        require(case_id in by_id, f"{prefix}: unknown exact case: {case_id}")
        identity = by_id[case_id]
        require(
            identity.case_id not in holdout_ids,
            f"{prefix}: rejected active holdout case before file open: {case_id}",
        )
        require(
            identity.source_path not in holdout_paths,
            f"{prefix}: rejected active holdout path before file open: "
            f"{identity.source_path}",
        )
        require(
            identity.expected_sha256 not in holdout_hashes,
            f"{prefix}: rejected active holdout SHA-256 before file open: "
            f"{identity.expected_sha256}",
        )
        require(
            identity.partition == "development",
            f"{prefix}: exact case is not development-owned: {case_id}",
        )
        selected.append(identity)

    return DevelopmentAccessPlan(
        selected=tuple(selected),
        active_holdout_count=len(holdouts),
    )


def run_development_access_session(
    identities: Iterable[SourceIdentity],
    requested_case_ids: list[str],
    *,
    repo: Path,
    registry: PinnedStageARegistry,
    access_log_path: Path,
    qualification_owner_id: str,
    qualification_owner: Callable[[SourceIdentity, bytes, dict[str, Any]], None],
    on_file_open: Callable[[Path], None] | None = None,
) -> dict[str, Any]:
    """Preflight identities and deliver exact verified bytes to an in-process owner.

    This generic helper creates only a development-access session. It cannot mint a
    StageAQualificationSession or record qualification success. The mandatory owner
    receives verified bytes from the same no-follow file open; payload bytes are
    never added to the access log.
    """

    require(
        isinstance(registry, PinnedStageARegistry)
        and registry.schema == STAGE_A_REGISTRY_SCHEMA
        and registry.raw_sha256 == STAGE_A_REGISTRY_RAW_SHA256,
        "development access requires the frozen Stage-A registry snapshot",
    )
    owner_id = validate_safe_token(
        qualification_owner_id,
        "qualification_owner_id",
    )
    require(
        callable(qualification_owner),
        "development access requires an in-process qualification owner",
    )
    identity_list = list(identities)
    holdout_count = sum(
        identity.partition.startswith("holdout_") for identity in identity_list
    )
    access_session_id = str(uuid.uuid4())
    access_log: dict[str, Any] = {
        "schema": "riotbox.source_holdout_development_access_log.v3",
        "session_kind": "DevelopmentSourceAccessSession",
        "access_session_id": access_session_id,
        "started_at_utc": utc_now(),
        "manifest_path": registry.path.as_posix(),
        "manifest_schema": registry.schema,
        "registry_sha256": registry.raw_sha256,
        "mode": "explicit_development_cases_only_no_glob_or_directory_discovery",
        "directory_discovery_performed": False,
        "holdout_metadata_comparison": {
            "fields": ["case_id", "source_path", "sha256"],
            "active_holdout_count": holdout_count,
            "audio_files_opened": False,
        },
        "requested_case_ids": list(requested_case_ids),
        "opened_development_files": [],
        "access_status": "preflight_pending",
        "access_preflight_status": "pending",
        "qualification_status": "not_evaluated_by_access_layer",
        "qualification_owner": {
            "owner_id": owner_id,
            "in_process": True,
            "delivery_status": "pending",
            "delivered_source_count": 0,
        },
    }
    with create_exclusive_access_log(access_log_path) as access_log_file:
        persist_access_log(access_log_file, access_log)
        failure_stage = "metadata_preflight"
        try:
            plan = preflight_development_identities(
                identity_list,
                requested_case_ids,
                prefix=str(registry.path),
            )
            require(
                plan.active_holdout_count == holdout_count,
                f"{registry.path}: holdout metadata count changed during preflight",
            )
            access_log["access_preflight_status"] = "passed"
            access_log["access_status"] = "preflight_passed"
            persist_access_log(access_log_file, access_log)
            for identity in plan.selected:
                failure_stage = f"registry_pin_pre_open:{identity.case_id}"
                assert_registry_pin_current(registry)
                failure_stage = f"source_access:{identity.case_id}"
                opened_record: dict[str, Any] | None = None

                def record_open(
                    path: Path,
                    *,
                    selected_identity: SourceIdentity = identity,
                ) -> None:
                    nonlocal opened_record
                    opened_record = {
                        "case_id": selected_identity.case_id,
                        "source_path": selected_identity.source_path,
                        "expected_sha256": selected_identity.expected_sha256,
                        "access_verification_status": "opened",
                    }
                    access_log["opened_development_files"].append(opened_record)
                    access_log["access_status"] = "reading_development_sources"
                    persist_access_log(access_log_file, access_log)
                    if on_file_open is not None:
                        on_file_open(path)

                payload, result = validate_contained_source_file(
                    repo,
                    Path(identity.source_path),
                    identity.expected_sha256,
                    identity.source_format,
                    f"{registry.path}: {identity.case_id}",
                    on_open=record_open,
                    return_payload=True,
                )
                require(
                    opened_record is not None,
                    f"{registry.path}: verified file was not recorded as opened",
                )
                verified_record = {
                    **opened_record,
                    **result,
                    "access_verification_status": "verified",
                }
                failure_stage = f"qualification_owner_delivery:{identity.case_id}"
                qualification_owner(
                    identity,
                    payload,
                    copy.deepcopy(verified_record),
                )
                failure_stage = f"source_access:{identity.case_id}"
                opened_record.update(result)
                opened_record["access_verification_status"] = (
                    "verified_and_delivered_to_owner"
                )
                access_log["qualification_owner"]["delivered_source_count"] += 1
                persist_access_log(access_log_file, access_log)
            access_log["access_status"] = "completed"
            access_log["qualification_owner"]["delivery_status"] = "completed"
            access_log["completed_at_utc"] = utc_now()
            persist_access_log(access_log_file, access_log)
        except Exception as error:
            access_log["access_status"] = (
                "aborted"
                if failure_stage.startswith("qualification_owner_delivery:")
                else "rejected"
            )
            access_log["qualification_owner"]["delivery_status"] = (
                "failed"
                if failure_stage.startswith("qualification_owner_delivery:")
                else "incomplete"
            )
            access_log["rejection_type"] = type(error).__name__
            access_log["rejection_stage"] = failure_stage
            access_log["rejection"] = (
                "in-process qualification owner rejected verified source delivery"
                if failure_stage.startswith("qualification_owner_delivery:")
                else str(error)
            )
            access_log["completed_at_utc"] = utc_now()
            persist_access_log(access_log_file, access_log)
            raise
    return access_log


def assert_registry_pin_current(registry: PinnedStageARegistry) -> None:
    """Fail closed if the frozen registry bytes changed after session setup."""

    current_sha256 = hashlib.sha256(registry.path.read_bytes()).hexdigest()
    require(
        current_sha256 == registry.raw_sha256 == STAGE_A_REGISTRY_RAW_SHA256,
        f"{registry.path}: Stage-A registry changed before development source open",
    )


def validate_contained_source_file(
    repo: Path,
    relative_path: Path,
    expected_sha256: str,
    source_format: dict[str, Any],
    prefix: str,
    *,
    on_open: Callable[[Path], None] | None = None,
    return_payload: bool = False,
) -> dict[str, Any] | tuple[bytes, dict[str, Any]]:
    maximum_bytes = maximum_source_file_bytes(source_format)
    payload = read_contained_regular_file(
        repo,
        relative_path,
        prefix,
        on_open=on_open,
        maximum_bytes=maximum_bytes,
    )
    result = validate_wav_payload(payload, expected_sha256, source_format, prefix)
    if return_payload:
        return payload, result
    return result


def read_contained_regular_file(
    repo: Path,
    relative_path: Path,
    prefix: str,
    *,
    on_open: Callable[[Path], None] | None = None,
    maximum_bytes: int | None = None,
) -> bytes:
    """Open one named file using no-follow directory descriptors; never enumerate."""

    require(
        not relative_path.is_absolute()
        and relative_path.parts
        and all(part not in {"", ".", ".."} for part in relative_path.parts),
        f"{prefix}: selected source path must be safe and repo-relative",
    )
    root_stat = os.lstat(repo)
    require(not stat.S_ISLNK(root_stat.st_mode), f"{prefix}: repository root is a symlink")
    require(stat.S_ISDIR(root_stat.st_mode), f"{prefix}: repository root is not a directory")

    directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    file_flags = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW
    directory_fds: list[int] = []
    file_fd: int | None = None
    try:
        root_fd = os.open(repo, directory_flags)
        directory_fds.append(root_fd)
        opened_root = os.fstat(root_fd)
        require(
            stat.S_ISDIR(opened_root.st_mode),
            f"{prefix}: opened repository root is not a directory",
        )
        require(
            (root_stat.st_dev, root_stat.st_ino)
            == (opened_root.st_dev, opened_root.st_ino),
            f"{prefix}: repository root inode changed during open",
        )
        for part in relative_path.parts[:-1]:
            current_fd = directory_fds[-1]
            ancestor_stat = os.stat(part, dir_fd=current_fd, follow_symlinks=False)
            require(
                not stat.S_ISLNK(ancestor_stat.st_mode),
                f"{prefix}: selected source ancestor is a symlink: {part}",
            )
            require(
                stat.S_ISDIR(ancestor_stat.st_mode),
                f"{prefix}: selected source ancestor is not a directory: {part}",
            )
            ancestor_fd = os.open(part, directory_flags, dir_fd=current_fd)
            directory_fds.append(ancestor_fd)
            opened_ancestor = os.fstat(ancestor_fd)
            require(
                stat.S_ISDIR(opened_ancestor.st_mode),
                f"{prefix}: opened source ancestor is not a directory: {part}",
            )
            require(
                (ancestor_stat.st_dev, ancestor_stat.st_ino)
                == (opened_ancestor.st_dev, opened_ancestor.st_ino),
                f"{prefix}: selected source ancestor inode changed during open: {part}",
            )

        final_name = relative_path.parts[-1]
        parent_fd = directory_fds[-1]
        before = os.stat(final_name, dir_fd=parent_fd, follow_symlinks=False)
        require(
            not stat.S_ISLNK(before.st_mode),
            f"{prefix}: selected source file is a symlink",
        )
        require(stat.S_ISREG(before.st_mode), f"{prefix}: selected source is not a regular file")
        require(
            before.st_nlink == 1,
            f"{prefix}: selected source file must have exactly one hard link",
        )
        file_fd = os.open(final_name, file_flags, dir_fd=parent_fd)
        after = os.fstat(file_fd)
        require(
            stat.S_ISREG(after.st_mode),
            f"{prefix}: opened source is not a regular file",
        )
        require(
            after.st_nlink == 1,
            f"{prefix}: opened source file must have exactly one hard link",
        )
        require(
            (before.st_dev, before.st_ino) == (after.st_dev, after.st_ino),
            f"{prefix}: selected source inode changed during open",
        )
        if on_open is not None:
            on_open(repo / relative_path)
        with os.fdopen(file_fd, "rb") as source_file:
            file_fd = None
            if maximum_bytes is not None:
                require(
                    after.st_size <= maximum_bytes,
                    f"{prefix}: selected source exceeds its pre-read byte boundary",
                )
                payload = source_file.read(maximum_bytes + 1)
            else:
                payload = source_file.read()
            final = os.fstat(source_file.fileno())
            require(
                (after.st_dev, after.st_ino, after.st_size)
                == (final.st_dev, final.st_ino, final.st_size),
                f"{prefix}: selected source changed during bounded read",
            )
            require(
                len(payload) == after.st_size,
                f"{prefix}: selected source byte count changed during bounded read",
            )
            return payload
    finally:
        if file_fd is not None:
            os.close(file_fd)
        for directory_fd in reversed(directory_fds):
            os.close(directory_fd)


def validate_wav_file(
    path: Path,
    expected_sha256: str,
    source_format: dict[str, Any],
    prefix: str,
    *,
    on_open: Callable[[Path], None] | None = None,
) -> dict[str, Any]:
    require(path.is_file(), f"{prefix}: missing source file: {path}")
    maximum_bytes = maximum_source_file_bytes(source_format)
    with path.open("rb") as source_file:
        if on_open is not None:
            on_open(path)
        source_stat = os.fstat(source_file.fileno())
        require(
            source_stat.st_size <= maximum_bytes,
            f"{prefix}: source exceeds its pre-read byte boundary",
        )
        payload = source_file.read(maximum_bytes + 1)
        require(
            len(payload) == source_stat.st_size,
            f"{prefix}: source byte count changed during bounded read",
        )
    return validate_wav_payload(payload, expected_sha256, source_format, prefix)


def validate_wav_payload(
    payload: bytes,
    expected_sha256: str,
    source_format: dict[str, Any],
    prefix: str,
) -> dict[str, Any]:
    validate_source_format(source_format, f"{prefix}.source_format")
    actual_hash = hashlib.sha256(payload).hexdigest()
    require(actual_hash == expected_sha256, f"{prefix}: source SHA-256 mismatch")
    parsed = parse_strict_pcm_wave(payload, source_format, prefix)
    sample_width_bits = parsed["sample_width_bits"]
    max_absolute = maximum_pcm_absolute(
        parsed.pop("sample_bytes"), sample_width_bits // 8
    )
    full_scale_positive = (1 << (sample_width_bits - 1)) - 1
    require(
        max_absolute < full_scale_positive,
        f"{prefix}: source WAV contains clipped integer samples",
    )
    return {
        "actual_sha256": actual_hash,
        "actual_source_format": parsed,
    }


def parse_strict_pcm_wave(
    payload: bytes,
    source_format: dict[str, Any],
    prefix: str,
) -> dict[str, Any]:
    require(len(payload) >= 12, f"{prefix}: source WAV header is truncated")
    require(
        payload[:4] == b"RIFF",
        f"{prefix}: source WAV must use RIFF little-endian container",
    )
    require(payload[8:12] == b"WAVE", f"{prefix}: source is not RIFF/WAVE")
    riff_size = struct.unpack_from("<I", payload, 4)[0]
    require(
        riff_size == len(payload) - 8,
        f"{prefix}: RIFF size must exactly match the bounded file payload",
    )

    fmt_fields: tuple[int, int, int, int, int, int] | None = None
    sample_bytes: bytes | None = None
    offset = 12
    while offset < len(payload):
        require(
            offset + 8 <= len(payload),
            f"{prefix}: source WAV chunk header is truncated",
        )
        chunk_id = payload[offset : offset + 4]
        chunk_size = struct.unpack_from("<I", payload, offset + 4)[0]
        chunk_start = offset + 8
        chunk_end = chunk_start + chunk_size
        padded_end = chunk_end + (chunk_size & 1)
        require(
            chunk_end <= len(payload) and padded_end <= len(payload),
            f"{prefix}: source WAV chunk exceeds RIFF boundary",
        )
        if chunk_id == b"fmt ":
            require(fmt_fields is None, f"{prefix}: source WAV must contain one fmt chunk")
            require(
                chunk_size == 16,
                f"{prefix}: PCM fmt chunk must be exactly 16 bytes",
            )
            fmt_fields = struct.unpack_from("<HHIIHH", payload, chunk_start)
        elif chunk_id == b"data":
            require(fmt_fields is not None, f"{prefix}: data chunk must follow fmt chunk")
            require(sample_bytes is None, f"{prefix}: source WAV must contain one data chunk")
            sample_bytes = payload[chunk_start:chunk_end]
        offset = padded_end
    require(offset == len(payload), f"{prefix}: source WAV chunk alignment is invalid")
    require(fmt_fields is not None, f"{prefix}: source WAV is missing fmt chunk")
    require(sample_bytes is not None, f"{prefix}: source WAV is missing data chunk")

    format_tag, channels, sample_rate_hz, byte_rate, block_align, bits = fmt_fields
    require(
        format_tag == 1,
        f"{prefix}: source WAV format_tag must be 1 PCM (not extensible or float)",
    )
    require(channels == source_format["channels"], f"{prefix}: source WAV channel-count mismatch")
    require(
        sample_rate_hz == source_format["sample_rate_hz"],
        f"{prefix}: source WAV sample-rate mismatch",
    )
    require(
        bits == source_format["sample_width_bits"],
        f"{prefix}: source WAV sample-width mismatch",
    )
    expected_block_align = channels * (bits // 8)
    require(
        block_align == expected_block_align,
        f"{prefix}: source WAV block_align mismatch",
    )
    require(
        byte_rate == sample_rate_hz * block_align,
        f"{prefix}: source WAV byte_rate mismatch",
    )
    require(
        len(sample_bytes) % block_align == 0,
        f"{prefix}: source WAV data chunk is not block-aligned",
    )
    frame_count = len(sample_bytes) // block_align
    require(frame_count > 0, f"{prefix}: source WAV must not be empty")
    require(
        frame_count
        <= source_format["sample_rate_hz"]
        * source_format["maximum_duration_seconds"]
        + 1,
        f"{prefix}: source WAV exceeds its registered duration boundary",
    )
    return {
        "sample_rate_hz": sample_rate_hz,
        "channels": channels,
        "sample_width_bits": bits,
        "compression_type": "NONE",
        "format_tag": format_tag,
        "block_align": block_align,
        "byte_rate": byte_rate,
        "frame_count": frame_count,
        "data_bytes": len(sample_bytes),
        "sample_bytes": sample_bytes,
    }


def validate_source_format(source_format: dict[str, Any], prefix: str) -> None:
    require(
        isinstance(source_format.get("sample_rate_hz"), int)
        and not isinstance(source_format.get("sample_rate_hz"), bool)
        and source_format["sample_rate_hz"] > 0,
        f"{prefix}.sample_rate_hz must be a positive integer",
    )
    require(
        source_format.get("channels") in {1, 2},
        f"{prefix}.channels must be 1 or 2",
    )
    require(
        source_format.get("sample_width_bits") in {16, 24},
        f"{prefix}.sample_width_bits must be 16 or 24",
    )
    require(
        source_format.get("compression_type") == "NONE",
        f"{prefix}.compression_type must be NONE",
    )
    duration = source_format.get("maximum_duration_seconds")
    require(
        isinstance(duration, int)
        and not isinstance(duration, bool)
        and duration > 0,
        f"{prefix}.maximum_duration_seconds must be a positive integer",
    )


def maximum_source_file_bytes(source_format: dict[str, Any]) -> int:
    validate_source_format(source_format, "source_format")
    maximum_frames = (
        source_format["sample_rate_hz"]
        * source_format["maximum_duration_seconds"]
        + 1
    )
    maximum_data_bytes = (
        maximum_frames
        * source_format["channels"]
        * (source_format["sample_width_bits"] // 8)
    )
    return maximum_data_bytes + MAX_RIFF_CONTAINER_OVERHEAD_BYTES


def maximum_pcm_absolute(payload: bytes, sample_width: int) -> int:
    if sample_width == 2:
        samples = array("h")
        samples.frombytes(payload)
        if sys.byteorder != "little":
            samples.byteswap()
        return max((abs(sample) for sample in samples), default=0)
    if sample_width == 3:
        require(len(payload) % 3 == 0, "PCM24 payload must contain complete samples")
        return max(
            (
                abs(int.from_bytes(payload[offset : offset + 3], "little", signed=True))
                for offset in range(0, len(payload), 3)
            ),
            default=0,
        )
    raise ValueError(f"unsupported PCM sample width: {sample_width}")


def create_exclusive_access_log(path: Path) -> TextIO:
    path.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC
    file_descriptor = os.open(path, flags, 0o600)
    try:
        fsync_directory(path.parent)
        return os.fdopen(file_descriptor, "w", encoding="utf-8", newline="\n")
    except Exception:
        os.close(file_descriptor)
        raise


def fsync_directory(path: Path) -> None:
    """Persist an exclusively created log's directory entry before snapshots."""

    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    directory_fd = os.open(path, flags)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)


def persist_access_log(access_log_file: TextIO, access_log: dict[str, Any]) -> None:
    access_log_file.seek(0)
    access_log_file.write(json.dumps(access_log, indent=2, sort_keys=True) + "\n")
    access_log_file.truncate()
    access_log_file.flush()
    os.fsync(access_log_file.fileno())


def validate_safe_token(value: str, field: str) -> str:
    require(
        isinstance(value, str)
        and value.strip() == value
        and 1 <= len(value) <= 128
        and all(character.isalnum() or character in "-_.:" for character in value),
        f"{field} must be a non-empty safe token",
    )
    return value


def parse_strict_json_object(payload: bytes, prefix: str) -> dict[str, Any]:
    try:
        decoded = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError(f"{prefix}: registry must be UTF-8 JSON") from error
    value = json.loads(decoded, object_pairs_hook=reject_duplicate_json_keys)
    require(isinstance(value, dict), f"{prefix}: registry JSON root must be an object")
    return value


def reject_duplicate_json_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        require(
            key not in value,
            f"duplicate registry JSON object key: {key!r}",
        )
        value[key] = child
    return value


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds").replace(
        "+00:00", "Z"
    )


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)
