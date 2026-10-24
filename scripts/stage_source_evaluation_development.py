#!/usr/bin/env python3
"""One-shot exact-file Development staging for RIOTBOX-1609, not Stage A.

This helper neither authorizes access nor runs ci-broad. Run it only after the
reviewed policy, metadata pins, candidate, and operation have been authorized.
It does not decode audio, discover directories, or inspect protected audio.
"""

from __future__ import annotations

import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
from typing import Any, Callable


POLICY_PATH = "docs/benchmarks/source_evaluation_alpha_development_access_v1.json"
REGISTRY_PATHS = tuple(
    f"docs/benchmarks/source_holdout_rotation_v{version}.json"
    for version in (1, 2, 3)
)
EXACT_SOURCES = (
    ("legacy_dense_beat03", "Beat03_130BPM(Full).wav", False),
    ("legacy_dense_beat08", "Beat08_128BPM(Full).wav", True),
    ("legacy_weak_beat20", "Beat20_128BPM(Full).wav", False),
    ("legacy_pressure_beatc", "DH_BeatC_120-01.wav", True),
    ("legacy_sparse_kicksnr", "DH_BeatC_KickSnr_120-01.wav", False),
    ("legacy_tonal_rusharp", "DH_RushArp_120_A.wav", False),
    ("legacy_pad_fadapad", "DH_Fadapad_120_A.wav", False),
)
MAXIMUM_SOURCE_BYTES = 67_108_864
MAXIMUM_METADATA_BYTES = 1_048_576
DIRECTORY_FLAGS = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
READ_FLAGS = os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW | os.O_NONBLOCK


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def valid_hash(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def relative_parts(value: str) -> tuple[str, ...]:
    require(isinstance(value, str), "relative path must be a string")
    parts = tuple(value.split("/"))
    require(bool(parts) and all(part not in {"", ".", ".."} for part in parts),
            "unsafe relative path")
    require(not value.startswith("/") and "\\" not in value and "\x00" not in value,
            "unsafe relative path")
    return parts


def absolute_path(value: Path) -> Path:
    text = str(value)
    require(value.is_absolute() and ".." not in text.split("/") and "\x00" not in text,
            "paths must be absolute without traversal")
    return value


def snapshot(value: os.stat_result) -> dict[str, int]:
    return {name: int(getattr(value, f"st_{name}")) for name in
            ("dev", "ino", "mode", "nlink", "uid", "size", "mtime_ns", "ctime_ns")}


def open_directory(path: Path, stack: ExitStack) -> int:
    """Walk named components only; never resolve/follow directory symlinks."""
    absolute_path(path)
    fd = os.open("/", DIRECTORY_FLAGS)
    stack.callback(os.close, fd)
    for part in path.parts[1:]:
        fd = os.open(part, DIRECTORY_FLAGS, dir_fd=fd)
        stack.callback(os.close, fd)
    return fd


def parent_directory(root_fd: int, relative: str, stack: ExitStack, *, create: bool = False) -> tuple[int, str]:
    parts = relative_parts(relative)
    fd = root_fd
    for part in parts[:-1]:
        if create:
            try:
                os.mkdir(part, mode=0o700, dir_fd=fd)
            except FileExistsError:
                pass
        fd = os.open(part, DIRECTORY_FLAGS, dir_fd=fd)
        stack.callback(os.close, fd)
    return fd, parts[-1]


def strict_object(payload: bytes) -> dict[str, Any]:
    def pairs(items: list[tuple[str, Any]]) -> dict[str, Any]:
        result: dict[str, Any] = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result

    def reject_constant(_: str) -> None:
        raise ValueError("nonfinite JSON")

    value = json.loads(payload, object_pairs_hook=pairs, parse_constant=reject_constant)
    require(isinstance(value, dict), "metadata must be a JSON object")
    return value


def read_once(root_fd: int, relative: str, maximum: int,
              event: Callable[[str, dict[str, Any]], None]) -> bytes:
    """One source open/read, with pre-byte bounds and descriptor/path stability."""
    with ExitStack() as stack:
        parent_fd, name = parent_directory(root_fd, relative, stack)
        event("stat_attempted", {})
        before = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        event("stat_observed", {"stat": snapshot(before)})
        require(stat.S_ISREG(before.st_mode), "input must be regular, not a symlink")
        require(before.st_nlink == 1, "input must have exactly one hard link")
        require(0 < before.st_size <= maximum, "input outside pre-open byte bound")
        event("open_attempted", {})
        fd = os.open(name, READ_FLAGS, dir_fd=parent_fd)
        stack.callback(os.close, fd)
        opened = os.fstat(fd)
        event("opened", {"stat": snapshot(opened)})
        require(snapshot(before) == snapshot(opened), "input changed during open")
        payload = os.read(fd, maximum + 1)
        event("bytes_read", {"bytes_read": len(payload)})
        after = os.fstat(fd)
        named_after = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        event("read_stat_observed", {"stat": snapshot(after), "named_stat": snapshot(named_after)})
        require(snapshot(opened) == snapshot(after) == snapshot(named_after),
                "input changed during bounded read")
        require(len(payload) == opened.st_size and len(payload) <= maximum,
                "input byte count changed during bounded read")
        return payload


def validate_policy(policy: dict[str, Any]) -> list[dict[str, Any]]:
    keys = {"schema", "owner_ticket", "authorization_date", "broad_invocation_limit",
            "original_reads_per_source_limit", "maximum_source_bytes", "quality_proof",
            "playback_allowed", "commercial_reference_material_allowed", "protected_registries",
            "rights_context", "sources"}
    require(set(policy) == keys, "unknown or missing policy fields")
    require(policy["schema"] == "riotbox.source_evaluation_alpha_development_access.v1"
            and policy["owner_ticket"] == "RIOTBOX-1609"
            and policy["authorization_date"] == "2026-10-24", "unsupported access policy")
    require(type(policy["broad_invocation_limit"]) is int and policy["broad_invocation_limit"] == 1
            and type(policy["original_reads_per_source_limit"]) is int
            and policy["original_reads_per_source_limit"] == 1, "policy limits changed")
    require(type(policy["maximum_source_bytes"]) is int
            and 0 < policy["maximum_source_bytes"] <= MAXIMUM_SOURCE_BYTES,
            "source byte limit loosened")
    require(all(policy[key] is False for key in
                ("quality_proof", "playback_allowed", "commercial_reference_material_allowed")),
            "policy claims or access broadened")
    require(policy["protected_registries"] == list(REGISTRY_PATHS), "protected registries changed")
    require(isinstance(policy["rights_context"], str) and bool(policy["rights_context"]),
            "missing rights context")
    sources = policy["sources"]
    require(isinstance(sources, list) and len(sources) == 7, "exactly seven sources required")
    for source, (case_id, name, enrollment) in zip(sources, EXACT_SOURCES):
        require(isinstance(source, dict) and set(source) ==
                {"case_id", "path", "historical_sha256", "allow_hash_enrollment", "identity_record"},
                "unknown or missing source fields")
        relative_parts(source["path"])
        require(source["case_id"] == case_id and source["path"] == f"data/test_audio/examples/{name}",
                "source identity/path differs from exact seven-file boundary")
        require(source["allow_hash_enrollment"] is enrollment, "enrollment boundary changed")
        require((source["historical_sha256"] is None) if enrollment
                else valid_hash(source["historical_sha256"]), "historical hash/enrollment confused")
        require(isinstance(source["identity_record"], str) and bool(source["identity_record"]),
                "source identity record missing")
    require(len({source["case_id"] for source in sources}) == 7
            and len({source["path"] for source in sources}) == 7, "duplicate source identity/path")
    return sources


def protected_identities(registries: list[dict[str, Any]]) -> tuple[set[str], set[str], set[str]]:
    require(len(registries) == 3, "all three protected registries required")
    ids: set[str] = set()
    paths: set[str] = set()
    hashes: set[str] = set()

    def collect_hashes(value: Any) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                if key == "sha256" or key.endswith("_sha256"):
                    require(child is None or valid_hash(child), "malformed protected SHA metadata")
                    if child is not None:
                        hashes.add(child)
                else:
                    collect_hashes(child)
        elif isinstance(value, list):
            for child in value:
                collect_hashes(child)

    for version, registry in zip((1, 2, 3), registries):
        require(registry.get("schema") == f"riotbox.source_holdout_rotation.v{version}"
                and registry.get("schema_version") == version, "unsupported protected registry")
        require(registry.get("commercial_reference_material_allowed") is False,
                "protected registry commercial policy changed")
        entries = registry.get("entries")
        require(isinstance(entries, list) and bool(entries), "protected entries missing")
        for entry in entries:
            require(isinstance(entry, dict) and isinstance(entry.get("case_id"), str)
                    and bool(entry["case_id"]) and isinstance(entry.get("partition"), str)
                    and bool(entry["partition"]) and isinstance(entry.get("source_path"), str)
                    and type(entry.get("commercial_reference")) is bool,
                    "malformed protected identity")
            relative_parts(entry["source_path"])
            require(entry.get("sha256") is None or valid_hash(entry["sha256"]),
                    "malformed protected SHA metadata")
            if entry["partition"] != "development" or entry["commercial_reference"]:
                ids.add(entry["case_id"])
                paths.add(entry["source_path"])
                collect_hashes(entry)
    return ids, paths, hashes


def stage(source_repo: Path, validation_worktree: Path, receipt_path: Path,
          policy_sha256: str, protected_pins: dict[str, str], *,
          acknowledge_new_hash_enrollment: bool) -> dict[str, Any]:
    """Stage all seven accepted byte strings; any error leaves a partial receipt."""
    source_repo, validation_worktree, receipt_path = map(
        absolute_path, (source_repo, validation_worktree, receipt_path))
    require(source_repo != validation_worktree
            and not source_repo.is_relative_to(validation_worktree)
            and not validation_worktree.is_relative_to(source_repo), "worktrees must be separate")
    require(not receipt_path.is_relative_to(source_repo)
            and not receipt_path.is_relative_to(validation_worktree), "receipt must be external")
    with ExitStack() as stack:
        receipt_parent = open_directory(receipt_path.parent, stack)
        receipt_fd = os.open(receipt_path.name, os.O_WRONLY | os.O_CREAT | os.O_EXCL
                             | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600, dir_fd=receipt_parent)
        receipt_file = stack.enter_context(os.fdopen(receipt_fd, "w", encoding="utf-8"))
        receipt: dict[str, Any] = {
            "schema": "riotbox.source_evaluation_alpha_development_receipt.v1",
            "owner_ticket": "RIOTBOX-1609", "status": "preflight_pending",
            "source_repo": str(source_repo), "validation_worktree": str(validation_worktree),
            "authority": "external reviewed user authorization, not the CLI acknowledgement",
            "hash_enrollment_acknowledged": acknowledge_new_hash_enrollment,
            "protected_registered_paths_opened": False,
            "protected_hash_collision_observed": False,
            "protected_hash_identity_assessment": "not_performed",
            "directory_discovery_performed": False,
            "audio_decoded": False, "broad_invocations": 0, "metadata": [], "sources": [],
            "frozen_identities": [], "staged_paths": [],
        }

        def persist() -> None:
            receipt_file.seek(0)
            json.dump(receipt, receipt_file, sort_keys=True, indent=2)
            receipt_file.write("\n")
            receipt_file.truncate()
            receipt_file.flush()
            os.fsync(receipt_file.fileno())

        phase = "metadata_preflight"
        persist()
        try:
            require(valid_hash(policy_sha256) and set(protected_pins) == set(REGISTRY_PATHS)
                    and all(valid_hash(value) for value in protected_pins.values()),
                    "exact reviewed metadata pins required")
            source_fd = open_directory(source_repo, stack)
            target_fd = open_directory(validation_worktree, stack)
            require(os.fstat(source_fd).st_uid == os.getuid()
                    and os.fstat(target_fd).st_uid == os.getuid(), "worktrees must be owned")
            marker = os.stat(".git", dir_fd=target_fd, follow_symlinks=False)
            require(stat.S_ISREG(marker.st_mode) and marker.st_nlink == 1
                    and marker.st_uid == os.getuid(), "target must be a fresh owned Git worktree")
            receipt["worktree_stat"] = {"source": snapshot(os.fstat(source_fd)),
                                        "validation": snapshot(os.fstat(target_fd)),
                                        "validation_git_marker": snapshot(marker)}
            persist()
            metadata: list[dict[str, Any]] = []
            for path, expected in [(POLICY_PATH, policy_sha256),
                                   *((path, protected_pins[path]) for path in REGISTRY_PATHS)]:
                record: dict[str, Any] = {"path": path, "expected_sha256": expected, "events": []}
                receipt["metadata"].append(record)

                def metadata_event(kind: str, details: dict[str, Any]) -> None:
                    record["events"].append({"event": kind, **details})
                    persist()

                payload = read_once(source_fd, path, MAXIMUM_METADATA_BYTES, metadata_event)
                record["observed_sha256"] = sha256(payload)
                persist()
                require(record["observed_sha256"] == expected, "reviewed metadata pin mismatch")
                metadata.append(strict_object(payload))
            sources = validate_policy(metadata[0])
            protected_ids, protected_paths, protected_hashes = protected_identities(metadata[1:])
            receipt["protected_metadata_comparison"] = {
                "protected_case_count": len(protected_ids), "protected_path_count": len(protected_paths),
                "available_protected_sha256_count": len(protected_hashes),
                "fields": ["case_id", "source_path", "available_sha256"],
            }
            require(acknowledge_new_hash_enrollment, "explicit hash enrollment acknowledgement required")
            for source in sources:
                require(source["case_id"] not in protected_ids, "protected case rejected before source access")
                require(source["path"] not in protected_paths, "protected path rejected before source access")
                require(source["historical_sha256"] not in protected_hashes,
                        "protected expected SHA rejected before source access")
            # Reject every existing destination before accessing any original.
            for source in sources:
                try:
                    with ExitStack() as check_stack:
                        parent_fd, name = parent_directory(target_fd, source["path"], check_stack)
                        os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
                except FileNotFoundError:
                    continue
                raise ValueError("staging destination already exists; no overwrite")
            receipt["status"] = "metadata_preflight_passed"
            receipt["protected_hash_identity_assessment"] = "incomplete"
            persist()
            accepted: list[tuple[dict[str, Any], bytes]] = []
            for source in sources:
                phase = f"source_access:{source['case_id']}"
                record = {"case_id": source["case_id"], "path": source["path"],
                          "historical_sha256": source["historical_sha256"],
                          "hash_basis": "authorized_new_enrollment" if source["allow_hash_enrollment"]
                          else "historical_expected_sha256", "events": [], "status": "pending"}
                receipt["sources"].append(record)

                def source_event(kind: str, details: dict[str, Any]) -> None:
                    record["events"].append({"event": kind, **details})
                    persist()

                payload = read_once(source_fd, source["path"], metadata[0]["maximum_source_bytes"], source_event)
                actual = sha256(payload)
                record["observed_sha256"] = actual
                record["events"].append({"event": "sha256_computed"})
                collision = actual in protected_hashes
                if collision:
                    receipt["protected_hash_collision_observed"] = True
                    receipt["protected_hash_identity_assessment"] = "collision_observed"
                persist()
                require(not collision, "observed protected SHA rejected before staging")
                require(source["allow_hash_enrollment"] or actual == source["historical_sha256"],
                        "historical expected SHA mismatch")
                record["status"] = "accepted"
                accepted.append((source, payload))
                persist()
            receipt["frozen_identities"] = [
                {key: record[key] for key in ("case_id", "path", "observed_sha256", "hash_basis")}
                for record in receipt["sources"]
            ]
            receipt["protected_hash_identity_assessment"] = "all_seven_no_collision"
            receipt["status"] = "seven_identities_frozen_before_staging"
            persist()
            for source, payload in accepted:
                phase = f"staging:{source['case_id']}"
                with ExitStack() as output_stack:
                    parent_fd, name = parent_directory(target_fd, source["path"], output_stack, create=True)
                    fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC
                                 | os.O_NOFOLLOW, 0o600, dir_fd=parent_fd)
                    output = output_stack.enter_context(os.fdopen(fd, "wb"))
                    receipt["staged_paths"].append({"path": source["path"], "status": "created"})
                    persist()
                    require(output.write(payload) == len(payload), "incomplete staging write")
                    output.flush()
                    os.fsync(output.fileno())
                    receipt["staged_paths"][-1]["status"] = "complete"
                    persist()
            receipt["status"] = "staged_ready_for_separately_authorized_single_broad_invocation"
            persist()
        except BaseException as error:
            receipt["status"] = "failed_stop_no_retry"
            receipt["failure"] = {"phase": phase, "type": type(error).__name__, "reason": str(error)}
            persist()
            raise
        return receipt


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-repo", type=Path, required=True)
    parser.add_argument("--validation-worktree", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--policy-sha256", required=True)
    parser.add_argument("--protected-registry-sha256", action="append", required=True, metavar="PATH=SHA256")
    parser.add_argument("--acknowledge-new-hash-enrollment", action="store_true",
                        help="acknowledgement only; does not grant access authority")
    args = parser.parse_args()
    try:
        pins: dict[str, str] = {}
        for item in args.protected_registry_sha256:
            path, digest = item.split("=", 1)
            require(path not in pins, "duplicate protected registry pin")
            pins[path] = digest
        stage(args.source_repo, args.validation_worktree, args.receipt, args.policy_sha256, pins,
              acknowledge_new_hash_enrollment=args.acknowledge_new_hash_enrollment)
    except (OSError, TypeError, ValueError) as error:
        print(f"Development staging refused: {error}", file=sys.stderr)
        return 1
    print("Seven Development identities frozen and staged; ci-broad has not run.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
