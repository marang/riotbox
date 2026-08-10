#!/usr/bin/env python3
"""Frozen source-blind access-log and manifest shapes for RIOTBOX-1430.

The builders in this module describe evidence from the single authorized
Stage-A-v2 acquisition.  They deliberately cannot grant source suitability,
family fitness, event fitness, hardness, or human-review status.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
from copy import deepcopy
from pathlib import Path, PurePosixPath
from typing import Any
from urllib.parse import urlsplit

import percussive_force_stage_a_v2_acquisition_contract as batch_contract


ACCESS_LOG_SCHEMA = "riotbox.percussive_force_stage_a_v2_acquisition_access_log.v1"
ACCESS_LOG_UPDATE_SUFFIX = ".next"
MANIFEST_SCHEMA = (
    "riotbox.percussive_force_stage_a_v2_acquisition_sealed_manifest.v1"
)
SESSION_KIND = "DevelopmentOnlyStageAV2Acquisition"
IMPLEMENTATION_AGGREGATE_ALGORITHM = (
    "sha256_domain_riotbox_stage_a_v2_acquisition_implementation_v1_"
    "then_u32be_path_length_path_utf8_and_raw_sha256_bytes_in_ordinal_order"
)
IMPLEMENTATION_DOMAIN = (
    b"riotbox.stage_a_v2_acquisition.implementation.v1\x00"
)
IMPLEMENTATION_FILES = (
    "scripts/percussive_force_stage_a_v2_contract.py",
    "scripts/validate_percussive_force_stage_a_protocol.py",
    "scripts/validate_percussive_force_stage_a_protocol_v2.py",
    "scripts/percussive_force_stage_a_v2_acquisition.py",
    "scripts/percussive_force_stage_a_v2_acquisition_contract.py",
    "scripts/validate_percussive_force_stage_a_v2_acquisition_batch.py",
    "scripts/percussive_force_stage_a_v2_acquisition_artifacts.py",
    "scripts/validate_percussive_force_stage_a_v2_acquisition_artifacts.py",
    "scripts/run_percussive_force_stage_a_v2_acquisition.py",
)
REVALIDATION_CHECKPOINTS = (
    "before_request_1",
    "before_request_2",
    "before_request_3",
    "before_publication",
)
SUCCESS_TRANSITIONS = (
    "attempt_created",
    "preflight_passed",
    "acquiring",
    "all_headers_verified",
    "sealed_in_quarantine",
    "publication_pending",
    "completed",
)
ENTRY_STATES = (
    "not_requested",
    "request_started",
    "response_metadata_verified",
    "body_verified",
    "header_verified",
    "sealed",
    "published",
)
REQUEST_HEADERS = (
    ("Host", "opengameart.org"),
    ("Accept", "audio/wav, audio/x-wav, application/octet-stream"),
    ("Accept-Encoding", "identity"),
    ("Connection", "close"),
    ("User-Agent", "Riotbox-RIOTBOX-1430-source-acquisition/1.0"),
)
SHA256 = re.compile(r"^[0-9a-f]{64}$")


def semantic_sha256(document: Any) -> str:
    canonical = json.dumps(
        document,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
    return hashlib.sha256(canonical).hexdigest()


def render(document: dict[str, Any]) -> bytes:
    return (
        json.dumps(document, indent=2, ensure_ascii=False, allow_nan=False) + "\n"
    ).encode("utf-8")


def _read_regular_file_no_follow(path: Path, maximum_bytes: int = 4_194_304) -> bytes:
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode) or stat.S_ISLNK(before.st_mode):
        raise ValueError(f"implementation file is not a no-follow regular file: {path}")
    if before.st_nlink != 1 or before.st_size > maximum_bytes:
        raise ValueError(f"implementation file has unsafe link count or size: {path}")
    descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        opened = os.fstat(descriptor)
        if (
            opened.st_dev,
            opened.st_ino,
            opened.st_size,
            opened.st_nlink,
        ) != (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_nlink,
        ):
            raise ValueError(f"implementation file changed during open: {path}")
        chunks: list[bytes] = []
        consumed = 0
        while consumed <= maximum_bytes:
            chunk = os.read(descriptor, min(65_536, maximum_bytes + 1 - consumed))
            if not chunk:
                break
            chunks.append(chunk)
            consumed += len(chunk)
        payload = b"".join(chunks)
        final = os.fstat(descriptor)
        if len(payload) != opened.st_size or (
            final.st_dev,
            final.st_ino,
            final.st_size,
            final.st_nlink,
        ) != (
            opened.st_dev,
            opened.st_ino,
            opened.st_size,
            opened.st_nlink,
        ):
            raise ValueError(f"implementation file changed during bounded read: {path}")
        return payload
    finally:
        os.close(descriptor)


def _repository_head(repo_root: Path) -> str:
    git_directory = repo_root / ".git"
    git_stat = git_directory.lstat()
    if not stat.S_ISDIR(git_stat.st_mode) or stat.S_ISLNK(git_stat.st_mode):
        raise ValueError("repository .git must be a no-follow directory")
    head_payload = _read_regular_file_no_follow(git_directory / "HEAD", 4096)
    try:
        head = head_payload.decode("ascii").strip()
    except UnicodeDecodeError as error:
        raise ValueError("repository HEAD is not ASCII") from error
    if head.startswith("ref: "):
        ref = head.removeprefix("ref: ")
        if (
            not ref.startswith("refs/heads/")
            or "\\" in ref
            or any(part in {"", ".", ".."} for part in PurePosixPath(ref).parts)
        ):
            raise ValueError("repository HEAD ref is not a safe local branch ref")
        ref_path = git_directory.joinpath(*PurePosixPath(ref).parts)
        if ref_path.exists():
            head = _read_regular_file_no_follow(ref_path, 4096).decode("ascii").strip()
        else:
            packed_payload = _read_regular_file_no_follow(
                git_directory / "packed-refs", 4_194_304
            )
            matches = []
            for line in packed_payload.decode("ascii").splitlines():
                if not line or line.startswith(("#", "^")):
                    continue
                fields = line.split(" ", 1)
                if len(fields) == 2 and fields[1] == ref:
                    matches.append(fields[0])
            if len(matches) != 1:
                raise ValueError("repository HEAD ref is missing or ambiguous")
            head = matches[0]
    if SHA256.fullmatch(head) is None and re.fullmatch(r"[0-9a-f]{40}", head) is None:
        raise ValueError("repository HEAD is not a lowercase Git object ID")
    return head


def build_implementation_snapshot(repo_root: Path) -> dict[str, Any]:
    aggregate = hashlib.sha256()
    aggregate.update(IMPLEMENTATION_DOMAIN)
    files: list[dict[str, Any]] = []
    for ordinal, relative in enumerate(IMPLEMENTATION_FILES, start=1):
        payload = _read_regular_file_no_follow(repo_root / relative)
        raw_digest = hashlib.sha256(payload).digest()
        encoded_path = relative.encode("utf-8")
        aggregate.update(len(encoded_path).to_bytes(4, "big"))
        aggregate.update(encoded_path)
        aggregate.update(raw_digest)
        files.append(
            {
                "ordinal": ordinal,
                "path": relative,
                "raw_sha256": raw_digest.hex(),
            }
        )
    return {
        "algorithm": IMPLEMENTATION_AGGREGATE_ALGORITHM,
        "repository_head_commit": _repository_head(repo_root),
        "files": files,
        "aggregate_sha256": aggregate.hexdigest(),
    }


def observed_bindings(
    batch_raw_sha256: str,
    batch_semantic_sha256: str,
    implementation_aggregate_sha256: str,
) -> dict[str, Any]:
    return {
        "protocol_v2_raw_sha256": batch_contract.PROTOCOL_V2_RAW_SHA256,
        "protocol_v2_semantic_sha256": (
            batch_contract.PROTOCOL_V2_SEMANTIC_SHA256
        ),
        "acquisition_batch_v1_raw_sha256": batch_raw_sha256,
        "acquisition_batch_v1_semantic_sha256": batch_semantic_sha256,
        "predecessor_registry_v2_raw_sha256": (
            batch_contract.REGISTRY_V2_RAW_SHA256
        ),
        "predecessor_registry_v2_semantic_sha256": (
            batch_contract.REGISTRY_V2_SEMANTIC_SHA256
        ),
        "implementation_aggregate_sha256": implementation_aggregate_sha256,
    }


def _entry_log_template(entry: dict[str, Any]) -> dict[str, Any]:
    request_target = urlsplit(entry["download_url"]).path
    return {
        "ordinal": entry["ordinal"],
        "case_id": entry["case_id"],
        "source_family": entry["source_family"],
        "family_assignment_state": entry["family_assignment_state"],
        "author": entry["author"],
        "source_pack_id": entry["source_pack_id"],
        "provider_attachment_id": entry["provider_attachment_id"],
        "page_url": entry["page_url"],
        "download_url": entry["download_url"],
        "attachment_filename": entry["attachment_filename"],
        "page_declared_attachment_mime_type": entry["attachment_mime_type"],
        "declared_attachment_byte_count": entry["attachment_byte_count"],
        "destination_path": entry["destination_path"],
        "source_qualification_state": entry["source_qualification_state"],
        "state": "not_requested",
        "request_count": 0,
        "request_started_at_utc": None,
        "verified_at_utc": None,
        "network": {
            "method": "GET",
            "host": "opengameart.org",
            "dns_query_name": "opengameart.org.",
            "port": 443,
            "request_target": request_target,
            "request_headers": [list(item) for item in REQUEST_HEADERS],
            "dns_answers": [],
            "selected_peer_ip": None,
            "connected_peer_ip": None,
            "tls": {
                "sni": "opengameart.org",
                "default_trust_and_hostname_validation": True,
                "minimum_version": "TLSv1.2",
                "negotiated_version": None,
                "peer_certificate_sha256": None,
            },
            "redirect_count": 0,
            "retry_count": 0,
            "proxy_used": False,
            "cookies_auth_or_netrc_used": False,
            "automatic_content_decoding_used": False,
            "response_gate": None,
        },
        "stream": None,
        "header_validation": None,
        "quarantine_file_identity": None,
    }


def build_initial_access_log(
    *,
    batch_document: dict[str, Any],
    batch_raw_sha256: str,
    batch_semantic_sha256: str,
    implementation_snapshot: dict[str, Any],
    attempt_id: str,
    started_at_utc: str,
) -> dict[str, Any]:
    entries = [_entry_log_template(entry) for entry in batch_document["entries"]]
    log_parent = PurePosixPath(
        batch_document["filesystem_contract"]["access_log_path"]
    ).parent
    final_parent = PurePosixPath(
        batch_document["filesystem_contract"]["final_batch_directory"]
    ).parent
    required_free_bytes = sum(
        entry["attachment_byte_count"] for entry in batch_document["entries"]
    ) + 8_388_608
    return {
        "schema": ACCESS_LOG_SCHEMA,
        "schema_version": 1,
        "owner_ticket": "RIOTBOX-1430",
        "session_kind": SESSION_KIND,
        "attempt_id": attempt_id,
        "batch_id": batch_document["batch_id"],
        "started_at_utc": started_at_utc,
        "completed_at_utc": None,
        "attempt_status": "attempt_created",
        "evidence_role": "identity_hash_and_header_format_only",
        "quality_proof": False,
        "source_qualification_claimed": False,
        "human_review_claimed": False,
        "contract_bindings": {
            "protocol_v2": deepcopy(batch_document["protocol_binding"]),
            "acquisition_batch_v1": {
                "path": batch_contract.BATCH_REL.as_posix(),
                "schema": batch_document["schema"],
                "raw_sha256": batch_raw_sha256,
                "semantic_sha256": batch_semantic_sha256,
            },
            "predecessor_registry_v2": deepcopy(
                batch_document["predecessor_registry_binding"]
            ),
        },
        "implementation_snapshot": deepcopy(implementation_snapshot),
        "scope_assertions": {
            "partition": "development",
            "exact_entry_count": 3,
            "directory_discovery_performed": False,
            "holdout_audio_access_performed": False,
            "commercial_reference_access_performed": False,
            "source_audio_playback_performed": False,
            "audio_decode_performed": False,
            "pcm_sample_iteration_performed": False,
            "source_feature_or_event_computation_performed": False,
            "candidate_or_control_rendering_performed": False,
        },
        "filesystem": {
            "access_log_path": batch_document["filesystem_contract"]["access_log_path"],
            "access_log_update_path": (
                batch_document["filesystem_contract"]["access_log_path"]
                + ACCESS_LOG_UPDATE_SUFFIX
            ),
            "quarantine_directory": batch_document["filesystem_contract"][
                "quarantine_directory"
            ],
            "final_batch_directory": batch_document["filesystem_contract"][
                "final_batch_directory"
            ],
            "sealed_manifest_name": batch_document["filesystem_contract"][
                "sealed_manifest_name"
            ],
            "access_log_created_exclusively": True,
            "access_log_parent_fsync_completed": False,
            "quarantine_created_exclusively": False,
            "final_destination_absent_before_network": True,
            "same_filesystem_verified": True,
            "publication_probe_source_path": (
                f"{log_parent.as_posix()}/riotbox-1430-publication-probe-source"
            ),
            "publication_probe_destination_path": (
                f"{final_parent.as_posix()}/riotbox-1430-publication-probe-destination"
            ),
            "publication_probe_completed": False,
            "required_free_bytes_before_network": required_free_bytes,
            "available_free_bytes_before_network": None,
            "directory_listing_or_glob_performed": False,
            "individual_final_file_renames_performed": False,
            "overwrite_performed": False,
            "quarantine_cleanup_state": "not_required",
        },
        "revalidation_checkpoints": [
            {
                "checkpoint": checkpoint,
                "status": "pending",
                "checked_at_utc": None,
                "observed_bindings": None,
            }
            for checkpoint in REVALIDATION_CHECKPOINTS
        ],
        "request_count": 0,
        "successful_request_count": 0,
        "entries": entries,
        "sealed_manifest": {
            "schema": MANIFEST_SCHEMA,
            "quarantine_path": (
                f"{batch_document['filesystem_contract']['quarantine_directory']}/"
                f"{batch_document['filesystem_contract']['sealed_manifest_name']}"
            ),
            "final_path": (
                f"{batch_document['filesystem_contract']['final_batch_directory']}/"
                f"{batch_document['filesystem_contract']['sealed_manifest_name']}"
            ),
            "state": "not_written",
            "byte_count": None,
            "raw_sha256": None,
            "semantic_sha256": None,
            "quarantine_file_identity": None,
        },
        "sealed_payload_revalidation": {
            "state": "not_started",
            "checked_at_utc": None,
            "entry_count": 0,
            "raw_sha256_recomputed": False,
            "header_reinspection_performed": False,
            "sample_decode_performed": False,
            "pcm_sample_iteration_performed": False,
        },
        "publication": {
            "method": "renameat2_RENAME_NOREPLACE",
            "source_directory": batch_document["filesystem_contract"][
                "quarantine_directory"
            ],
            "destination_directory": batch_document["filesystem_contract"][
                "final_batch_directory"
            ],
            "state": "not_started",
            "rename_count": 0,
            "prepared_directory_device": None,
            "prepared_directory_inode": None,
            "source_parent_fsync_completed": False,
            "destination_parent_fsync_completed": False,
            "published_at_utc": None,
            "published_directory_device": None,
            "published_directory_inode": None,
        },
        "transition_history": [
            {"sequence": 1, "state": "attempt_created", "at_utc": started_at_utc}
        ],
        "rejection": None,
    }


def build_sealed_manifest(access_log: dict[str, Any]) -> dict[str, Any]:
    entries: list[dict[str, Any]] = []
    for record in access_log["entries"]:
        entries.append(
            {
                "ordinal": record["ordinal"],
                "case_id": record["case_id"],
                "source_family": record["source_family"],
                "family_assignment_state": record["family_assignment_state"],
                "author": record["author"],
                "source_pack_id": record["source_pack_id"],
                "provider_attachment_id": record["provider_attachment_id"],
                "page_url": record["page_url"],
                "download_url": record["download_url"],
                "attachment_filename": record["attachment_filename"],
                "page_declared_attachment_mime_type": record[
                    "page_declared_attachment_mime_type"
                ],
                "declared_attachment_byte_count": record[
                    "declared_attachment_byte_count"
                ],
                "destination_path": record["destination_path"],
                "actual_attachment_byte_count": record["stream"][
                    "body_bytes_consumed"
                ],
                "actual_sha256": record["stream"]["actual_sha256"],
                "header_validation": deepcopy(record["header_validation"]),
                "source_qualification_state": record[
                    "source_qualification_state"
                ],
            }
        )
    return {
        "schema": MANIFEST_SCHEMA,
        "schema_version": 1,
        "owner_ticket": "RIOTBOX-1430",
        "batch_id": access_log["batch_id"],
        "attempt_binding": {
            "attempt_id": access_log["attempt_id"],
            "access_log_schema": ACCESS_LOG_SCHEMA,
            "access_log_path": access_log["filesystem"]["access_log_path"],
        },
        "evidence_role": "identity_hash_and_header_format_only",
        "quality_proof": False,
        "source_qualification_claimed": False,
        "human_review_claimed": False,
        "contract_bindings": deepcopy(access_log["contract_bindings"]),
        "implementation_snapshot": deepcopy(access_log["implementation_snapshot"]),
        "access_scope": deepcopy(access_log["scope_assertions"]),
        "entry_count": 3,
        "entries": entries,
        "publication_contract": {
            "quarantine_directory": access_log["filesystem"][
                "quarantine_directory"
            ],
            "final_batch_directory": access_log["filesystem"][
                "final_batch_directory"
            ],
            "sealed_manifest_name": access_log["filesystem"][
                "sealed_manifest_name"
            ],
            "method": "one_atomic_renameat2_RENAME_NOREPLACE_of_complete_directory",
            "overwrite_allowed": False,
            "individual_final_file_renames_allowed": False,
            "publication_success_claimed_by_manifest": False,
        },
    }


def clone(value: dict[str, Any]) -> dict[str, Any]:
    """Return a deep copy for state-machine updates without aliasing evidence."""

    return deepcopy(value)
