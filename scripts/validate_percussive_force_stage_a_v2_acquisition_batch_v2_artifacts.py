#!/usr/bin/env python3
"""Validate RIOTBOX-1430 acquisition logs and manifests without audio access."""

from __future__ import annotations

import hashlib
import json
import os
import re
import stat
import sys
import uuid
from datetime import datetime
from pathlib import Path, PurePosixPath
from typing import Any

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_batch_v2_artifacts as artifacts
import percussive_force_stage_a_v2_acquisition_batch_v2_contract as batch_contract
import validate_percussive_force_stage_a_v2_acquisition_batch_v2 as batch_validator


SHA256 = re.compile(r"^[0-9a-f]{64}$")
SAFE_STAGE = re.compile(r"^[a-z0-9][a-z0-9_:.-]*$")
HEADER_KEYS = {
    "container",
    "riff_size",
    "fmt_offset",
    "fmt_size",
    "format_tag",
    "pcm_encoding",
    "channels",
    "sample_rate_hz",
    "byte_rate",
    "block_align",
    "sample_width_bits",
    "valid_bits",
    "data_offset",
    "data_size",
    "frame_count",
    "duration_numerator_frames",
    "duration_denominator_sample_rate",
    "container_overhead_bytes",
    "chunk_table",
    "header_validation_scope",
    "sample_decode_performed",
    "pcm_sample_iteration_performed",
}
CHUNK_KEYS = {
    "chunk_id",
    "header_offset",
    "payload_offset",
    "payload_size",
    "padding_bytes",
    "payload_read_for_header_validation",
}
PREDECESSOR_REJECTION_BINDING_KEY = (
    "predecessor_acquisition_rejection_binding"
)
PREDECESSOR_REJECTION_BINDING_FIELDS = {
    "batch_path",
    "batch_schema",
    "batch_raw_sha256",
    "batch_semantic_sha256",
    "rejection_report_path",
    "rejection_report_raw_sha256",
    "attempt_id",
    "access_log_path",
    "access_log_raw_sha256",
    "attempt_status",
    "rejection_stage",
    "request_count",
    "successful_request_count",
    "forbidden_observed_payload_sha256",
    "batch_v1_retry_allowed",
    "survivor_reuse_allowed",
    "payload_reuse_allowed",
}
REJECTED_BATCH_V1_OBSERVED_PAYLOAD_SHA256 = tuple(
    batch_contract.FORBIDDEN_OBSERVED_PAYLOAD_SHA256
)


class ContractError(ValueError):
    """Raised when a source-blind acquisition artifact fails closed."""


def _fail(path: str, message: str) -> None:
    raise ContractError(f"{path}: {message}")


def _strict_equal(actual: Any, expected: Any) -> bool:
    if type(actual) is not type(expected):
        return False
    if isinstance(expected, dict):
        return actual.keys() == expected.keys() and all(
            _strict_equal(actual[key], expected[key]) for key in expected
        )
    if isinstance(expected, list):
        return len(actual) == len(expected) and all(
            _strict_equal(left, right)
            for left, right in zip(actual, expected, strict=True)
        )
    return actual == expected


def _expect(path: str, actual: Any, expected: Any) -> None:
    if not _strict_equal(actual, expected):
        _fail(path, f"expected {expected!r}, got {actual!r}")


def _mapping(path: str, value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        _fail(path, "must be an object")
    return value


def _array(path: str, value: Any) -> list[Any]:
    if not isinstance(value, list):
        _fail(path, "must be an array")
    return value


def _integer(path: str, value: Any, *, minimum: int = 0) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < minimum:
        _fail(path, f"must be an integer >= {minimum}")
    return value


def _hash(path: str, value: Any) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        _fail(path, "must be a lowercase hexadecimal SHA-256")
    return value


def _timestamp(path: str, value: Any, *, nullable: bool = False) -> str | None:
    if value is None and nullable:
        return None
    if not isinstance(value, str) or not value.endswith("Z"):
        _fail(path, "must be a UTC timestamp ending in Z")
    try:
        parsed = datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError as error:
        raise ContractError(f"{path}: invalid timestamp") from error
    canonical = parsed.strftime("%Y-%m-%dT%H:%M:%SZ")
    if value != canonical:
        _fail(path, "must be canonical whole-second UTC")
    return value


def _uuid4(path: str, value: Any) -> str:
    if not isinstance(value, str):
        _fail(path, "must be a UUID string")
    try:
        parsed = uuid.UUID(value)
    except (ValueError, AttributeError) as error:
        raise ContractError(f"{path}: invalid UUID") from error
    if parsed.version != 4 or str(parsed) != value:
        _fail(path, "must be a canonical UUIDv4")
    return value


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        if key in value:
            _fail("JSON", f"duplicate object key {key!r}")
        value[key] = child
    return value


def _reject_nonfinite(token: str) -> None:
    _fail("JSON", f"nonfinite number {token!r} is forbidden")


def parse_json(payload: bytes, path: str) -> dict[str, Any]:
    if len(payload) > 2_097_152:
        _fail(path, "artifact exceeds the two-MiB JSON bound")
    try:
        document = json.loads(
            payload,
            object_pairs_hook=_reject_duplicate_keys,
            parse_constant=_reject_nonfinite,
        )
    except ContractError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ContractError(f"{path}: invalid UTF-8 JSON") from error
    return _mapping(path, document)


def _reject_forbidden_authority(value: Any, path: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            lowered = key.casefold()
            predecessor_access_log_pin = (
                lowered == "access_log_raw_sha256"
                and path
                in {
                    "access_log.contract_bindings.predecessor_acquisition_rejection_v1",
                    "manifest.contract_bindings.predecessor_acquisition_rejection_v1",
                }
            )
            if not predecessor_access_log_pin and lowered in {
                "access_log_raw_sha256",
                "access_log_semantic_sha256",
                "human_verdict",
                "hardness_verdict",
                "source_suitability_verdict",
                "family_fitness_verdict",
            }:
                _fail(f"{path}.{key}", "forbidden self or qualification authority")
            _reject_forbidden_authority(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _reject_forbidden_authority(child, f"{path}[{index}]")
    elif isinstance(value, str):
        lowered = value.casefold()
        if "source_holdout_rotation.v3" in lowered or "development_matrix.v3" in lowered:
            _fail(path, "forward Registry-v3 or Matrix-v3 authority is forbidden")


def validate_implementation_snapshot(
    value: Any, repo_root: Path, *, require_current: bool = True
) -> dict[str, Any]:
    snapshot = _mapping("implementation_snapshot", value)
    expected_keys = {
        "algorithm",
        "repository_head_commit",
        "files",
        "aggregate_sha256",
    }
    _expect("implementation_snapshot.keys", set(snapshot), expected_keys)
    _expect(
        "implementation_snapshot.algorithm",
        snapshot.get("algorithm"),
        artifacts.IMPLEMENTATION_AGGREGATE_ALGORITHM,
    )
    head = snapshot.get("repository_head_commit")
    if not isinstance(head, str) or re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", head) is None:
        _fail("implementation_snapshot.repository_head_commit", "invalid Git object ID")
    files = _array("implementation_snapshot.files", snapshot.get("files"))
    if len(files) != len(artifacts.IMPLEMENTATION_FILES):
        _fail("implementation_snapshot.files", "implementation file count changed")
    for index, (record_value, expected_path) in enumerate(
        zip(files, artifacts.IMPLEMENTATION_FILES, strict=True)
    ):
        record = _mapping(f"implementation_snapshot.files[{index}]", record_value)
        _expect(
            f"implementation_snapshot.files[{index}].keys",
            set(record),
            {"ordinal", "path", "raw_sha256"},
        )
        _expect(f"implementation_snapshot.files[{index}].ordinal", record["ordinal"], index + 1)
        _expect(f"implementation_snapshot.files[{index}].path", record["path"], expected_path)
        _hash(f"implementation_snapshot.files[{index}].raw_sha256", record["raw_sha256"])
    _hash("implementation_snapshot.aggregate_sha256", snapshot.get("aggregate_sha256"))
    if require_current:
        expected = artifacts.build_implementation_snapshot(repo_root)
        _expect(
            "implementation_snapshot current executable bytes",
            {
                "algorithm": snapshot["algorithm"],
                "repository_head_commit": snapshot["repository_head_commit"],
                "files": snapshot["files"],
                "aggregate_sha256": snapshot["aggregate_sha256"],
            },
            {
                "algorithm": expected["algorithm"],
                "repository_head_commit": expected["repository_head_commit"],
                "files": expected["files"],
                "aggregate_sha256": expected["aggregate_sha256"],
            },
        )
    return snapshot


def _validate_header_record(value: Any, declared_bytes: int, prefix: str) -> dict[str, Any]:
    header = _mapping(prefix, value)
    _expect(f"{prefix}.keys", set(header), HEADER_KEYS)
    _expect(f"{prefix}.container", header["container"], "RIFF/WAVE")
    _expect(f"{prefix}.riff_size", header["riff_size"], declared_bytes - 8)
    _expect(f"{prefix}.fmt_size", header["fmt_size"], 16)
    _expect(f"{prefix}.format_tag", header["format_tag"], 1)
    _expect(
        f"{prefix}.pcm_encoding",
        header["pcm_encoding"],
        "signed_little_endian_integer",
    )
    channels = _integer(f"{prefix}.channels", header["channels"], minimum=1)
    if channels not in acquisition.FORMAT_CONTRACT["channel_counts"]:
        _fail(f"{prefix}.channels", "outside frozen set")
    rate = _integer(f"{prefix}.sample_rate_hz", header["sample_rate_hz"], minimum=1)
    minimum_rate, maximum_rate = acquisition.FORMAT_CONTRACT[
        "sample_rate_hz_inclusive"
    ]
    if not minimum_rate <= rate <= maximum_rate:
        _fail(f"{prefix}.sample_rate_hz", "outside frozen range")
    width = _integer(f"{prefix}.sample_width_bits", header["sample_width_bits"], minimum=1)
    if width not in acquisition.FORMAT_CONTRACT["sample_width_bits"]:
        _fail(f"{prefix}.sample_width_bits", "outside frozen set")
    _expect(f"{prefix}.valid_bits", header["valid_bits"], width)
    align = channels * (width // 8)
    _expect(f"{prefix}.block_align", header["block_align"], align)
    _expect(f"{prefix}.byte_rate", header["byte_rate"], rate * align)
    data_size = _integer(f"{prefix}.data_size", header["data_size"], minimum=1)
    if data_size % align:
        _fail(f"{prefix}.data_size", "does not contain complete frames")
    frame_count = data_size // align
    _expect(f"{prefix}.frame_count", header["frame_count"], frame_count)
    _expect(f"{prefix}.duration_numerator_frames", header["duration_numerator_frames"], frame_count)
    _expect(f"{prefix}.duration_denominator_sample_rate", header["duration_denominator_sample_rate"], rate)
    if frame_count > rate * acquisition.FORMAT_CONTRACT["maximum_duration_seconds"]:
        _fail(f"{prefix}.frame_count", "exceeds frozen duration")
    overhead = declared_bytes - data_size
    _expect(f"{prefix}.container_overhead_bytes", header["container_overhead_bytes"], overhead)
    if overhead > acquisition.FORMAT_CONTRACT["maximum_container_overhead_bytes"]:
        _fail(f"{prefix}.container_overhead_bytes", "exceeds frozen bound")
    _expect(
        f"{prefix}.header_validation_scope",
        header["header_validation_scope"],
        "container_headers_only_no_sample_payload_reads",
    )
    _expect(f"{prefix}.sample_decode_performed", header["sample_decode_performed"], False)
    _expect(
        f"{prefix}.pcm_sample_iteration_performed",
        header["pcm_sample_iteration_performed"],
        False,
    )

    chunks = _array(f"{prefix}.chunk_table", header["chunk_table"])
    if not 2 <= len(chunks) <= acquisition.MAX_RIFF_CHUNKS:
        _fail(f"{prefix}.chunk_table", "chunk count outside frozen range")
    offset = 12
    fmt_chunks: list[dict[str, Any]] = []
    data_chunks: list[dict[str, Any]] = []
    for index, chunk_value in enumerate(chunks):
        chunk_prefix = f"{prefix}.chunk_table[{index}]"
        chunk = _mapping(chunk_prefix, chunk_value)
        _expect(f"{chunk_prefix}.keys", set(chunk), CHUNK_KEYS)
        chunk_id = chunk["chunk_id"]
        if not isinstance(chunk_id, str) or len(chunk_id) != 4 or any(
            not 32 <= ord(character) <= 126 for character in chunk_id
        ):
            _fail(f"{chunk_prefix}.chunk_id", "must be four printable ASCII bytes")
        _expect(f"{chunk_prefix}.header_offset", chunk["header_offset"], offset)
        _expect(f"{chunk_prefix}.payload_offset", chunk["payload_offset"], offset + 8)
        size = _integer(f"{chunk_prefix}.payload_size", chunk["payload_size"])
        padding = size & 1
        _expect(f"{chunk_prefix}.padding_bytes", chunk["padding_bytes"], padding)
        _expect(
            f"{chunk_prefix}.payload_read_for_header_validation",
            chunk["payload_read_for_header_validation"],
            chunk_id == "fmt ",
        )
        if chunk_id == "fmt ":
            fmt_chunks.append(chunk)
        if chunk_id == "data":
            data_chunks.append(chunk)
        offset += 8 + size + padding
    _expect(f"{prefix}.chunk_table.coverage", offset, declared_bytes)
    if len(fmt_chunks) != 1 or len(data_chunks) != 1:
        _fail(f"{prefix}.chunk_table", "must contain exactly one fmt and one data chunk")
    fmt_index = chunks.index(fmt_chunks[0])
    data_index = chunks.index(data_chunks[0])
    if fmt_index >= data_index:
        _fail(f"{prefix}.chunk_table", "fmt must precede data")
    _expect(f"{prefix}.fmt_offset", header["fmt_offset"], fmt_chunks[0]["payload_offset"])
    _expect(
        f"{prefix}.fmt_size",
        fmt_chunks[0]["payload_size"],
        header["fmt_size"],
    )
    _expect(f"{prefix}.data_offset", header["data_offset"], data_chunks[0]["payload_offset"])
    _expect(f"{prefix}.data_size", data_chunks[0]["payload_size"], data_size)
    return header


def _registry_v2_hashes(repo_root: Path) -> set[str]:
    path = repo_root / batch_contract.REGISTRY_V2_REL
    document = parse_json(
        artifacts._read_regular_file_no_follow(path),
        str(batch_contract.REGISTRY_V2_REL),
    )
    values: set[str] = set()
    for index, entry_value in enumerate(_array("registry_v2.entries", document.get("entries"))):
        entry = _mapping(f"registry_v2.entries[{index}]", entry_value)
        values.add(_hash(f"registry_v2.entries[{index}].sha256", entry.get("sha256")))
    return values


def _predecessor_rejection_payload_hashes(
    batch_document: dict[str, Any],
) -> set[str]:
    binding = _mapping(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}",
        batch_document.get(PREDECESSOR_REJECTION_BINDING_KEY),
    )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.keys",
        set(binding),
        PREDECESSOR_REJECTION_BINDING_FIELDS,
    )
    expected = _mapping(
        f"expected_batch.{PREDECESSOR_REJECTION_BINDING_KEY}",
        batch_contract.build_document().get(PREDECESSOR_REJECTION_BINDING_KEY),
    )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}",
        binding,
        expected,
    )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.attempt_status",
        binding["attempt_status"],
        "rejected",
    )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.rejection_stage",
        binding["rejection_stage"],
        "request_2_header",
    )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.request_count",
        binding["request_count"],
        2,
    )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.successful_request_count",
        binding["successful_request_count"],
        1,
    )
    for field in (
        "batch_v1_retry_allowed",
        "survivor_reuse_allowed",
        "payload_reuse_allowed",
    ):
        _expect(
            f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.{field}",
            binding[field],
            False,
        )
    values = _array(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.forbidden_observed_payload_sha256",
        binding["forbidden_observed_payload_sha256"],
    )
    for index, value in enumerate(values):
        _hash(
            f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.forbidden_observed_payload_sha256[{index}]",
            value,
        )
    if len(values) != 2 or len(set(values)) != 2:
        _fail(
            f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.forbidden_observed_payload_sha256",
            "must contain the two distinct rejected Batch-v1 observed payload hashes",
        )
    _expect(
        f"batch.{PREDECESSOR_REJECTION_BINDING_KEY}.forbidden_observed_payload_sha256",
        values,
        list(REJECTED_BATCH_V1_OBSERVED_PAYLOAD_SHA256),
    )
    return set(values)


def _validate_response_gate(value: Any, expected_bytes: int, prefix: str) -> dict[str, Any]:
    gate = _mapping(prefix, value)
    expected_keys = {
        "response_status",
        "content_length_raw_values",
        "content_encoding_raw_values",
        "transfer_encoding_raw_values",
        "content_type_raw_values",
        "normalized_media_type",
    }
    _expect(f"{prefix}.keys", set(gate), expected_keys)
    headers = {
        "content-length": gate["content_length_raw_values"],
        "content-encoding": gate["content_encoding_raw_values"],
        "transfer-encoding": gate["transfer_encoding_raw_values"],
        "content-type": gate["content_type_raw_values"],
    }
    try:
        expected = acquisition.validate_response_gate(
            status=gate["response_status"],
            headers=headers,
            expected_byte_count=expected_bytes,
        )
    except acquisition.AcquisitionError as error:
        raise ContractError(f"{prefix}: {error}") from error
    _expect(prefix, gate, expected)
    return gate


def _validate_network(
    record: dict[str, Any],
    batch_entry: dict[str, Any],
    state_index: int,
    prefix: str,
) -> None:
    network = _mapping(f"{prefix}.network", record["network"])
    template = artifacts._entry_log_template(batch_entry)["network"]
    _expect(f"{prefix}.network.keys", set(network), set(template))
    for key in (
        "method",
        "host",
        "dns_query_name",
        "port",
        "request_target",
        "request_headers",
        "redirect_count",
        "retry_count",
        "proxy_used",
        "cookies_auth_or_netrc_used",
        "automatic_content_decoding_used",
    ):
        _expect(f"{prefix}.network.{key}", network[key], template[key])
    tls = _mapping(f"{prefix}.network.tls", network["tls"])
    _expect(f"{prefix}.network.tls.keys", set(tls), set(template["tls"]))
    for key in ("sni", "default_trust_and_hostname_validation", "minimum_version"):
        _expect(f"{prefix}.network.tls.{key}", tls[key], template["tls"][key])
    answers = _array(f"{prefix}.network.dns_answers", network["dns_answers"])
    normalized: list[str] = []
    for index, value in enumerate(answers):
        try:
            normalized.append(acquisition.validate_public_peer_ip(value))
        except acquisition.AcquisitionError as error:
            raise ContractError(f"{prefix}.network.dns_answers[{index}]: {error}") from error
    if answers != sorted(set(normalized)):
        _fail(f"{prefix}.network.dns_answers", "must be sorted unique canonical global IPs")
    selected = network["selected_peer_ip"]
    connected = network["connected_peer_ip"]
    if selected is not None:
        if selected not in answers:
            _fail(f"{prefix}.network.selected_peer_ip", "must be one validated DNS answer")
    if connected is not None:
        _expect(f"{prefix}.network.connected_peer_ip", connected, selected)
    negotiated = tls["negotiated_version"]
    certificate = tls["peer_certificate_sha256"]
    if negotiated is not None and negotiated not in {"TLSv1.2", "TLSv1.3"}:
        _fail(f"{prefix}.network.tls.negotiated_version", "must be TLSv1.2 or TLSv1.3")
    if certificate is not None:
        _hash(f"{prefix}.network.tls.peer_certificate_sha256", certificate)
    if state_index == 0:
        _expect(f"{prefix}.network.dns_answers", answers, [])
        for key in ("selected_peer_ip", "connected_peer_ip", "response_gate"):
            _expect(f"{prefix}.network.{key}", network[key], None)
        _expect(f"{prefix}.network.tls.negotiated_version", negotiated, None)
        _expect(f"{prefix}.network.tls.peer_certificate_sha256", certificate, None)
    if network["response_gate"] is not None:
        if connected is None or negotiated is None or certificate is None:
            _fail(f"{prefix}.network", "response metadata requires connected verified TLS peer")
        _validate_response_gate(
            network["response_gate"],
            record["declared_attachment_byte_count"],
            f"{prefix}.network.response_gate",
        )
    if state_index >= artifacts.ENTRY_STATES.index("response_metadata_verified"):
        if network["response_gate"] is None:
            _fail(f"{prefix}.network.response_gate", "required by entry state")


def _validate_entry(
    record_value: Any,
    batch_entry: dict[str, Any],
    registry_hashes: set[str],
    predecessor_rejection_hashes: set[str],
    observed_new_hashes: set[str],
    prefix: str,
) -> int:
    record = _mapping(prefix, record_value)
    template = artifacts._entry_log_template(batch_entry)
    _expect(f"{prefix}.keys", set(record), set(template))
    for key in template:
        if key not in {
            "state",
            "request_count",
            "request_started_at_utc",
            "verified_at_utc",
            "network",
            "stream",
            "header_validation",
            "quarantine_file_identity",
        }:
            _expect(f"{prefix}.{key}", record[key], template[key])
    state = record.get("state")
    if state not in artifacts.ENTRY_STATES:
        _fail(f"{prefix}.state", "unknown entry state")
    state_index = artifacts.ENTRY_STATES.index(state)
    expected_request_count = 0 if state_index == 0 else 1
    _expect(f"{prefix}.request_count", record.get("request_count"), expected_request_count)
    _timestamp(
        f"{prefix}.request_started_at_utc",
        record.get("request_started_at_utc"),
        nullable=state_index == 0,
    )
    if state_index == 0:
        _expect(f"{prefix}.request_started_at_utc", record["request_started_at_utc"], None)
    verified_required = state_index >= artifacts.ENTRY_STATES.index("header_verified")
    _timestamp(f"{prefix}.verified_at_utc", record.get("verified_at_utc"), nullable=not verified_required)
    if not verified_required:
        _expect(f"{prefix}.verified_at_utc", record["verified_at_utc"], None)
    _validate_network(record, batch_entry, state_index, prefix)

    stream = record.get("stream")
    if stream is not None:
        stream = _mapping(f"{prefix}.stream", stream)
        _expect(
            f"{prefix}.stream.keys",
            set(stream),
            {
                "body_read_cap",
                "body_bytes_consumed",
                "actual_sha256",
                "sample_decode_performed",
                "pcm_sample_iteration_performed",
            },
        )
        declared = record["declared_attachment_byte_count"]
        _expect(f"{prefix}.stream.body_read_cap", stream["body_read_cap"], declared + 1)
        _expect(f"{prefix}.stream.body_bytes_consumed", stream["body_bytes_consumed"], declared)
        digest = _hash(f"{prefix}.stream.actual_sha256", stream["actual_sha256"])
        if digest in registry_hashes:
            _fail(f"{prefix}.stream.actual_sha256", "collides with frozen Registry-v2 metadata")
        if digest in predecessor_rejection_hashes:
            _fail(
                f"{prefix}.stream.actual_sha256",
                "reuses a forbidden rejected Batch-v1 observed payload",
            )
        if digest in observed_new_hashes:
            _fail(f"{prefix}.stream.actual_sha256", "duplicates another new batch payload")
        observed_new_hashes.add(digest)
        _expect(f"{prefix}.stream.sample_decode_performed", stream["sample_decode_performed"], False)
        _expect(
            f"{prefix}.stream.pcm_sample_iteration_performed",
            stream["pcm_sample_iteration_performed"],
            False,
        )
    if state_index >= artifacts.ENTRY_STATES.index("body_verified") and stream is None:
        _fail(f"{prefix}.stream", "required by entry state")
    if state_index < artifacts.ENTRY_STATES.index("body_verified") and stream is not None:
        _fail(f"{prefix}.stream", "present before body_verified state")

    file_identity = record.get("quarantine_file_identity")
    if file_identity is not None:
        file_identity = _mapping(
            f"{prefix}.quarantine_file_identity", file_identity
        )
        _expect(
            f"{prefix}.quarantine_file_identity.keys",
            set(file_identity),
            {"device", "inode", "link_count", "byte_count"},
        )
        _integer(f"{prefix}.quarantine_file_identity.device", file_identity["device"])
        _integer(
            f"{prefix}.quarantine_file_identity.inode",
            file_identity["inode"],
            minimum=1,
        )
        _expect(
            f"{prefix}.quarantine_file_identity.link_count",
            file_identity["link_count"],
            1,
        )
        _expect(
            f"{prefix}.quarantine_file_identity.byte_count",
            file_identity["byte_count"],
            record["declared_attachment_byte_count"],
        )
    if state_index >= artifacts.ENTRY_STATES.index("body_verified") and file_identity is None:
        _fail(f"{prefix}.quarantine_file_identity", "required by entry state")
    if state_index < artifacts.ENTRY_STATES.index("body_verified") and file_identity is not None:
        _fail(f"{prefix}.quarantine_file_identity", "present before body_verified state")

    header = record.get("header_validation")
    if header is not None:
        _validate_header_record(
            header,
            record["declared_attachment_byte_count"],
            f"{prefix}.header_validation",
        )
    if verified_required and header is None:
        _fail(f"{prefix}.header_validation", "required by entry state")
    if not verified_required and header is not None:
        _fail(f"{prefix}.header_validation", "present before header_verified state")
    return state_index


def _validate_transitions(log: dict[str, Any]) -> None:
    history = _array("access_log.transition_history", log["transition_history"])
    if not 1 <= len(history) <= len(artifacts.SUCCESS_TRANSITIONS) + 1:
        _fail("access_log.transition_history", "outside bounded transition count")
    states: list[str] = []
    previous_time: str | None = None
    for index, value in enumerate(history):
        record = _mapping(f"access_log.transition_history[{index}]", value)
        _expect(
            f"access_log.transition_history[{index}].keys",
            set(record),
            {"sequence", "state", "at_utc"},
        )
        _expect(f"access_log.transition_history[{index}].sequence", record["sequence"], index + 1)
        timestamp = _timestamp(f"access_log.transition_history[{index}].at_utc", record["at_utc"])
        if previous_time is not None and timestamp < previous_time:
            _fail("access_log.transition_history", "timestamps move backwards")
        previous_time = timestamp
        states.append(record["state"])
    status = log["attempt_status"]
    _expect(
        "access_log.transition_history initial timestamp",
        history[0]["at_utc"],
        log["started_at_utc"],
    )
    if status == "rejected":
        if states[-1] != "rejected":
            _fail("access_log.transition_history", "rejected status needs terminal transition")
        success_prefix = states[:-1]
        if success_prefix != list(artifacts.SUCCESS_TRANSITIONS[: len(success_prefix)]):
            _fail("access_log.transition_history", "illegal pre-rejection transition sequence")
        if "completed" in success_prefix:
            _fail("access_log.transition_history", "completed cannot transition to rejected")
    else:
        if status not in artifacts.SUCCESS_TRANSITIONS:
            _fail("access_log.attempt_status", "unknown status")
        expected = list(
            artifacts.SUCCESS_TRANSITIONS[
                : artifacts.SUCCESS_TRANSITIONS.index(status) + 1
            ]
        )
        _expect("access_log.transition_history states", states, expected)
    completed = log.get("completed_at_utc")
    if completed is not None:
        if history[-1]["at_utc"] != completed:
            _fail(
                "access_log.completed_at_utc",
                "must equal the terminal transition timestamp",
            )


def validate_access_log_document(
    document: dict[str, Any],
    *,
    frozen_batch: batch_validator.FrozenAcquisitionBatchV2,
    repo_root: Path,
    manifest_document: dict[str, Any] | None = None,
    manifest_payload: bytes | None = None,
    require_current_implementation: bool = True,
) -> None:
    frozen = batch_validator.revalidate_frozen_batch(frozen_batch)
    batch = frozen.document
    _reject_forbidden_authority(document, "access_log")
    snapshot = validate_implementation_snapshot(
        document.get("implementation_snapshot"),
        repo_root,
        require_current=require_current_implementation,
    )
    attempt_id = _uuid4("access_log.attempt_id", document.get("attempt_id"))
    started = _timestamp("access_log.started_at_utc", document.get("started_at_utc"))
    template = artifacts.build_initial_access_log(
        batch_document=batch,
        batch_raw_sha256=frozen.raw_sha256,
        batch_semantic_sha256=frozen.semantic_sha256,
        implementation_snapshot=snapshot,
        attempt_id=attempt_id,
        started_at_utc=started,
    )
    _expect("access_log.keys", set(document), set(template))
    for key in (
        "schema",
        "schema_version",
        "owner_ticket",
        "session_kind",
        "attempt_id",
        "batch_id",
        "started_at_utc",
        "evidence_role",
        "quality_proof",
        "source_qualification_claimed",
        "human_review_claimed",
        "contract_bindings",
        "implementation_snapshot",
        "scope_assertions",
    ):
        _expect(f"access_log.{key}", document[key], template[key])
    completed = _timestamp(
        "access_log.completed_at_utc",
        document.get("completed_at_utc"),
        nullable=True,
    )
    status = document.get("attempt_status")
    _validate_transitions(document)
    if status in {"completed", "rejected"}:
        if completed is None:
            _fail("access_log.completed_at_utc", "terminal state requires timestamp")
    else:
        _expect("access_log.completed_at_utc", completed, None)

    filesystem = _mapping("access_log.filesystem", document["filesystem"])
    _expect("access_log.filesystem.keys", set(filesystem), set(template["filesystem"]))
    for key in (
        "access_log_path",
        "access_log_update_path",
        "quarantine_directory",
        "final_batch_directory",
        "sealed_manifest_name",
        "access_log_created_exclusively",
        "final_destination_absent_before_network",
        "same_filesystem_verified",
        "publication_probe_source_path",
        "publication_probe_destination_path",
        "required_free_bytes_before_network",
        "directory_listing_or_glob_performed",
        "individual_final_file_renames_performed",
        "overwrite_performed",
    ):
        _expect(f"access_log.filesystem.{key}", filesystem[key], template["filesystem"][key])
    if not isinstance(filesystem["access_log_parent_fsync_completed"], bool):
        _fail("access_log.filesystem.access_log_parent_fsync_completed", "must be boolean")
    if not isinstance(filesystem["quarantine_created_exclusively"], bool):
        _fail("access_log.filesystem.quarantine_created_exclusively", "must be boolean")
    if not isinstance(filesystem["publication_probe_completed"], bool):
        _fail("access_log.filesystem.publication_probe_completed", "must be boolean")
    available_free = filesystem["available_free_bytes_before_network"]
    if available_free is not None:
        _integer(
            "access_log.filesystem.available_free_bytes_before_network",
            available_free,
        )
        if available_free < filesystem["required_free_bytes_before_network"]:
            _fail(
                "access_log.filesystem.available_free_bytes_before_network",
                "is below the frozen preflight requirement",
            )
    if filesystem["quarantine_cleanup_state"] not in {
        "not_required",
        "removed_exact_known_names",
        "cleanup_incomplete_fail_closed",
    }:
        _fail("access_log.filesystem.quarantine_cleanup_state", "unknown cleanup state")
    if any(
        transition["state"] == "preflight_passed"
        for transition in document["transition_history"]
    ):
        _expect(
            "access_log.filesystem.access_log_parent_fsync_completed",
            filesystem["access_log_parent_fsync_completed"],
            True,
        )
        _expect(
            "access_log.filesystem.quarantine_created_exclusively",
            filesystem["quarantine_created_exclusively"],
            True,
        )
        _expect(
            "access_log.filesystem.publication_probe_completed",
            filesystem["publication_probe_completed"],
            True,
        )
        if available_free is None:
            _fail(
                "access_log.filesystem.available_free_bytes_before_network",
                "preflight evidence is missing",
            )

    checkpoints = _array(
        "access_log.revalidation_checkpoints", document["revalidation_checkpoints"]
    )
    if len(checkpoints) != len(artifacts.REVALIDATION_CHECKPOINTS):
        _fail("access_log.revalidation_checkpoints", "checkpoint count changed")
    passed_count = 0
    expected_observed = artifacts.observed_bindings(
        frozen.raw_sha256,
        frozen.semantic_sha256,
        snapshot["aggregate_sha256"],
    )
    for index, (value, name) in enumerate(
        zip(checkpoints, artifacts.REVALIDATION_CHECKPOINTS, strict=True)
    ):
        checkpoint = _mapping(f"access_log.revalidation_checkpoints[{index}]", value)
        _expect(
            f"access_log.revalidation_checkpoints[{index}].keys",
            set(checkpoint),
            {"checkpoint", "status", "checked_at_utc", "observed_bindings"},
        )
        _expect(f"access_log.revalidation_checkpoints[{index}].checkpoint", checkpoint["checkpoint"], name)
        if checkpoint["status"] == "passed":
            passed_count += 1
            _timestamp(f"access_log.revalidation_checkpoints[{index}].checked_at_utc", checkpoint["checked_at_utc"])
            _expect(
                f"access_log.revalidation_checkpoints[{index}].observed_bindings",
                checkpoint["observed_bindings"],
                expected_observed,
            )
        elif checkpoint["status"] == "pending":
            _expect(f"access_log.revalidation_checkpoints[{index}].checked_at_utc", checkpoint["checked_at_utc"], None)
            _expect(f"access_log.revalidation_checkpoints[{index}].observed_bindings", checkpoint["observed_bindings"], None)
        else:
            _fail(f"access_log.revalidation_checkpoints[{index}].status", "must be pending or passed")
    if any(checkpoints[index]["status"] == "passed" and checkpoints[index - 1]["status"] != "passed" for index in range(1, len(checkpoints))):
        _fail("access_log.revalidation_checkpoints", "passed checkpoints must form a prefix")

    registry_hashes = _registry_v2_hashes(repo_root)
    predecessor_rejection_hashes = _predecessor_rejection_payload_hashes(batch)
    observed_new_hashes: set[str] = set()
    entries = _array("access_log.entries", document["entries"])
    if len(entries) != 3:
        _fail("access_log.entries", "must contain exactly three records")
    state_indices = [
        _validate_entry(
            record,
            batch_entry,
            registry_hashes,
            predecessor_rejection_hashes,
            observed_new_hashes,
            f"access_log.entries[{index}]",
        )
        for index, (record, batch_entry) in enumerate(
            zip(entries, batch["entries"], strict=True)
        )
    ]
    for index in range(1, len(state_indices)):
        if state_indices[index] > 0 and state_indices[index - 1] < artifacts.ENTRY_STATES.index("header_verified"):
            _fail("access_log.entries", "later request started before prior header verification")
        current_started = entries[index].get("request_started_at_utc")
        previous_verified = entries[index - 1].get("verified_at_utc")
        if current_started is not None and previous_verified is not None and current_started < previous_verified:
            _fail(
                f"access_log.entries[{index}].request_started_at_utc",
                "later request started before prior header verification timestamp",
            )
    for index, entry in enumerate(entries):
        request_started = entry.get("request_started_at_utc")
        verified_at = entry.get("verified_at_utc")
        if request_started is not None and request_started < started:
            _fail(
                f"access_log.entries[{index}].request_started_at_utc",
                "precedes attempt start",
            )
        if verified_at is not None and (
            request_started is None or verified_at < request_started
        ):
            _fail(
                f"access_log.entries[{index}].verified_at_utc",
                "precedes request start",
            )
    request_count = _integer("access_log.request_count", document["request_count"])
    successful_count = _integer(
        "access_log.successful_request_count", document["successful_request_count"]
    )
    _expect("access_log.request_count", request_count, sum(index > 0 for index in state_indices))
    _expect(
        "access_log.successful_request_count",
        successful_count,
        sum(index >= artifacts.ENTRY_STATES.index("header_verified") for index in state_indices),
    )
    if request_count > 3 or successful_count > request_count:
        _fail("access_log.request_count", "count exceeds frozen batch")
    if passed_count < request_count:
        _fail("access_log.revalidation_checkpoints", "request lacks passed pre-request checkpoint")
    for index in range(3):
        request_started = entries[index].get("request_started_at_utc")
        checkpoint_time = checkpoints[index].get("checked_at_utc")
        if request_started is not None and (
            checkpoint_time is None or checkpoint_time > request_started
        ):
            _fail(
                f"access_log.revalidation_checkpoints[{index}]",
                "must pass no later than its request reservation",
            )

    manifest_ref = _mapping("access_log.sealed_manifest", document["sealed_manifest"])
    _expect("access_log.sealed_manifest.keys", set(manifest_ref), set(template["sealed_manifest"]))
    for key in ("schema", "quarantine_path", "final_path"):
        _expect(f"access_log.sealed_manifest.{key}", manifest_ref[key], template["sealed_manifest"][key])
    payload_revalidation = _mapping(
        "access_log.sealed_payload_revalidation",
        document["sealed_payload_revalidation"],
    )
    _expect(
        "access_log.sealed_payload_revalidation.keys",
        set(payload_revalidation),
        set(template["sealed_payload_revalidation"]),
    )
    for key in (
        "sample_decode_performed",
        "pcm_sample_iteration_performed",
    ):
        _expect(
            f"access_log.sealed_payload_revalidation.{key}",
            payload_revalidation[key],
            False,
        )
    if payload_revalidation["state"] == "not_started":
        _expect(
            "access_log.sealed_payload_revalidation",
            payload_revalidation,
            template["sealed_payload_revalidation"],
        )
    elif payload_revalidation["state"] == "passed":
        _timestamp(
            "access_log.sealed_payload_revalidation.checked_at_utc",
            payload_revalidation["checked_at_utc"],
        )
        _expect(
            "access_log.sealed_payload_revalidation.entry_count",
            payload_revalidation["entry_count"],
            3,
        )
        _expect(
            "access_log.sealed_payload_revalidation.raw_sha256_recomputed",
            payload_revalidation["raw_sha256_recomputed"],
            True,
        )
        _expect(
            "access_log.sealed_payload_revalidation.header_reinspection_performed",
            payload_revalidation["header_reinspection_performed"],
            True,
        )
    else:
        _fail(
            "access_log.sealed_payload_revalidation.state",
            "must be not_started or passed",
        )
    publication = _mapping("access_log.publication", document["publication"])
    _expect("access_log.publication.keys", set(publication), set(template["publication"]))
    for key in ("method", "source_directory", "destination_directory"):
        _expect(f"access_log.publication.{key}", publication[key], template["publication"][key])
    if publication["state"] not in {"not_started", "pending", "completed"}:
        _fail("access_log.publication.state", "unknown publication state")
    rename_count = _integer("access_log.publication.rename_count", publication["rename_count"])
    if rename_count > 1:
        _fail("access_log.publication.rename_count", "more than one rename is forbidden")
    for key in ("source_parent_fsync_completed", "destination_parent_fsync_completed"):
        if not isinstance(publication[key], bool):
            _fail(f"access_log.publication.{key}", "must be boolean")
    for key in (
        "prepared_directory_device",
        "prepared_directory_inode",
        "published_directory_device",
        "published_directory_inode",
    ):
        value = publication[key]
        if value is not None:
            _integer(f"access_log.publication.{key}", value, minimum=1)
    published_at = _timestamp(
        "access_log.publication.published_at_utc",
        publication["published_at_utc"],
        nullable=True,
    )
    pending_transition_time = next(
        (
            record["at_utc"]
            for record in document["transition_history"]
            if record["state"] == "publication_pending"
        ),
        None,
    )
    if pending_transition_time is not None:
        checkpoint_time = checkpoints[3]["checked_at_utc"]
        if checkpoint_time is None or checkpoint_time > pending_transition_time:
            _fail(
                "access_log.revalidation_checkpoints[3]",
                "publication checkpoint must precede publication_pending",
            )
        if published_at is not None and published_at < pending_transition_time:
            _fail(
                "access_log.publication.published_at_utc",
                "precedes publication_pending",
            )

    manifest_required = manifest_ref["state"] in {"sealed_in_quarantine", "published"}
    if manifest_required:
        if successful_count != 3 or any(index < artifacts.ENTRY_STATES.index("header_verified") for index in state_indices):
            _fail("access_log.sealed_manifest", "manifest requires three header-verified entries")
        byte_count = _integer("access_log.sealed_manifest.byte_count", manifest_ref["byte_count"], minimum=1)
        raw_hash = _hash("access_log.sealed_manifest.raw_sha256", manifest_ref["raw_sha256"])
        semantic_hash = _hash("access_log.sealed_manifest.semantic_sha256", manifest_ref["semantic_sha256"])
        manifest_identity = _mapping(
            "access_log.sealed_manifest.quarantine_file_identity",
            manifest_ref["quarantine_file_identity"],
        )
        _expect(
            "access_log.sealed_manifest.quarantine_file_identity.keys",
            set(manifest_identity),
            {"device", "inode", "link_count", "byte_count"},
        )
        _integer(
            "access_log.sealed_manifest.quarantine_file_identity.device",
            manifest_identity["device"],
            minimum=1,
        )
        _integer(
            "access_log.sealed_manifest.quarantine_file_identity.inode",
            manifest_identity["inode"],
            minimum=1,
        )
        _expect(
            "access_log.sealed_manifest.quarantine_file_identity.link_count",
            manifest_identity["link_count"],
            1,
        )
        _expect(
            "access_log.sealed_manifest.quarantine_file_identity.byte_count",
            manifest_identity["byte_count"],
            byte_count,
        )
        if manifest_document is None or manifest_payload is None:
            _fail("access_log.sealed_manifest", "sealed state requires exact manifest payload")
        validate_manifest_document(
            manifest_document,
            access_log=document,
            frozen_batch=frozen,
            repo_root=repo_root,
            require_current_implementation=require_current_implementation,
        )
        _expect("access_log.sealed_manifest.byte_count", byte_count, len(manifest_payload))
        _expect("access_log.sealed_manifest.raw_sha256", raw_hash, hashlib.sha256(manifest_payload).hexdigest())
        _expect("access_log.sealed_manifest.semantic_sha256", semantic_hash, artifacts.semantic_sha256(manifest_document))
        _expect("access_log.sealed_manifest deterministic bytes", manifest_payload, artifacts.render(manifest_document))
    else:
        _expect("access_log.sealed_manifest.state", manifest_ref["state"], "not_written")
        for key in (
            "byte_count",
            "raw_sha256",
            "semantic_sha256",
            "quarantine_file_identity",
        ):
            _expect(f"access_log.sealed_manifest.{key}", manifest_ref[key], None)
        if manifest_document is not None or manifest_payload is not None:
            _fail("access_log.sealed_manifest", "unsealed log must not receive a manifest")

    if status == "completed":
        _expect("access_log.request_count", request_count, 3)
        _expect("access_log.successful_request_count", successful_count, 3)
        _expect("access_log.revalidation passed count", passed_count, 4)
        _expect("access_log.sealed_manifest.state", manifest_ref["state"], "published")
        _expect("access_log.publication.state", publication["state"], "completed")
        _expect("access_log.publication.rename_count", rename_count, 1)
        _expect("access_log.publication.source_parent_fsync_completed", publication["source_parent_fsync_completed"], True)
        _expect("access_log.publication.destination_parent_fsync_completed", publication["destination_parent_fsync_completed"], True)
        if publication["published_at_utc"] is None or publication["published_directory_device"] is None or publication["published_directory_inode"] is None:
            _fail("access_log.publication", "completed publication evidence is incomplete")
        _expect(
            "access_log.publication directory identity",
            (
                publication["published_directory_device"],
                publication["published_directory_inode"],
            ),
            (
                publication["prepared_directory_device"],
                publication["prepared_directory_inode"],
            ),
        )
        if any(index != artifacts.ENTRY_STATES.index("published") for index in state_indices):
            _fail("access_log.entries", "completed log requires all entries published")
        _expect("access_log.rejection", document["rejection"], None)
        _expect(
            "access_log.sealed_payload_revalidation.state",
            payload_revalidation["state"],
            "passed",
        )
        _expect(
            "access_log.filesystem.access_log_parent_fsync_completed",
            filesystem["access_log_parent_fsync_completed"],
            True,
        )
        _expect(
            "access_log.filesystem.quarantine_created_exclusively",
            filesystem["quarantine_created_exclusively"],
            True,
        )
        _expect(
            "access_log.filesystem.quarantine_cleanup_state",
            filesystem["quarantine_cleanup_state"],
            "not_required",
        )
        if pending_transition_time is None:
            _fail(
                "access_log.transition_history",
                "completed state requires publication_pending transition",
            )
        if completed is None or completed < started:
            _fail("access_log.completed_at_utc", "precedes attempt start")
        latest_verified = max(entry["verified_at_utc"] for entry in entries)
        if published_at is None or published_at < latest_verified:
            _fail(
                "access_log.publication.published_at_utc",
                "precedes final header verification",
            )
    elif status == "rejected":
        rejection = _mapping("access_log.rejection", document["rejection"])
        _expect(
            "access_log.rejection.keys",
            set(rejection),
            {
                "at_utc",
                "stage",
                "reason_code",
                "error_type",
                "error_message_sha256",
                "requests_started",
                "successful_requests",
                "further_requests_performed",
                "publication_authorized",
                "new_versioned_metadata_decision_required",
            },
        )
        _timestamp("access_log.rejection.at_utc", rejection["at_utc"])
        for key in ("stage", "reason_code", "error_type"):
            value = rejection[key]
            if not isinstance(value, str) or SAFE_STAGE.fullmatch(value) is None:
                _fail(f"access_log.rejection.{key}", "must be a bounded safe identifier")
        _hash("access_log.rejection.error_message_sha256", rejection["error_message_sha256"])
        _expect("access_log.rejection.requests_started", rejection["requests_started"], request_count)
        _expect("access_log.rejection.successful_requests", rejection["successful_requests"], successful_count)
        _expect("access_log.rejection.further_requests_performed", rejection["further_requests_performed"], False)
        if not isinstance(rejection["publication_authorized"], bool):
            _fail("access_log.rejection.publication_authorized", "must be boolean")
        _expect(
            "access_log.rejection.publication_authorized",
            rejection["publication_authorized"],
            False,
        )
        _expect("access_log.rejection.new_versioned_metadata_decision_required", rejection["new_versioned_metadata_decision_required"], True)
        if publication["state"] == "completed":
            _fail("access_log.publication", "rejected log cannot claim completed publication")
    else:
        _expect("access_log.rejection", document["rejection"], None)

    status_index = (
        artifacts.SUCCESS_TRANSITIONS.index(status)
        if status in artifacts.SUCCESS_TRANSITIONS
        else None
    )
    if status_index is not None and status_index >= artifacts.SUCCESS_TRANSITIONS.index("all_headers_verified"):
        _expect("access_log.successful_request_count", successful_count, 3)
    if status == "sealed_in_quarantine":
        _expect("access_log.sealed_manifest.state", manifest_ref["state"], "sealed_in_quarantine")
        _expect("access_log.publication.state", publication["state"], "not_started")
    if status == "publication_pending":
        _expect("access_log.sealed_manifest.state", manifest_ref["state"], "sealed_in_quarantine")
        _expect("access_log.publication.state", publication["state"], "pending")
        _expect("access_log.revalidation passed count", passed_count, 4)
        _expect(
            "access_log.sealed_payload_revalidation.state",
            payload_revalidation["state"],
            "passed",
        )
        if publication["prepared_directory_device"] is None or publication["prepared_directory_inode"] is None:
            _fail("access_log.publication", "pending publication needs prepared directory identity")


def validate_manifest_document(
    document: dict[str, Any],
    *,
    access_log: dict[str, Any],
    frozen_batch: batch_validator.FrozenAcquisitionBatchV2,
    repo_root: Path,
    require_current_implementation: bool = True,
) -> None:
    frozen = batch_validator.revalidate_frozen_batch(frozen_batch)
    frozen_document = frozen.document
    _reject_forbidden_authority(document, "manifest")
    validate_implementation_snapshot(
        document.get("implementation_snapshot"),
        repo_root,
        require_current=require_current_implementation,
    )
    expected = artifacts.build_sealed_manifest(access_log)
    _expect("sealed_manifest", document, expected)
    _expect("sealed_manifest.schema", document.get("schema"), artifacts.MANIFEST_SCHEMA)
    _expect("sealed_manifest.schema_version", document.get("schema_version"), 2)
    expected_contract_bindings = {
        "protocol_v2": frozen_document["protocol_binding"],
        "acquisition_batch_v2": {
            "path": batch_contract.BATCH_REL.as_posix(),
            "schema": frozen_document["schema"],
            "raw_sha256": frozen.raw_sha256,
            "semantic_sha256": frozen.semantic_sha256,
        },
        "predecessor_registry_v2": frozen_document[
            "predecessor_registry_binding"
        ],
        "predecessor_acquisition_rejection_v1": frozen_document[
            PREDECESSOR_REJECTION_BINDING_KEY
        ],
    }
    access_log_bindings = _mapping(
        "access_log.contract_bindings", access_log.get("contract_bindings")
    )
    _expect(
        "access_log.contract_bindings",
        access_log_bindings,
        expected_contract_bindings,
    )
    _expect(
        "sealed_manifest.contract_bindings",
        document.get("contract_bindings"),
        expected_contract_bindings,
    )
    _expect(
        "access_log.schema",
        access_log.get("schema"),
        artifacts.ACCESS_LOG_SCHEMA,
    )
    access_log_filesystem = _mapping(
        "access_log.filesystem", access_log.get("filesystem")
    )
    _expect(
        "access_log.filesystem.access_log_path",
        access_log_filesystem.get("access_log_path"),
        batch_contract.ACCESS_LOG_PATH,
    )
    _expect("sealed_manifest.entry_count", document.get("entry_count"), 3)
    entries = _array("sealed_manifest.entries", document.get("entries"))
    registry_hashes = _registry_v2_hashes(repo_root)
    predecessor_rejection_hashes = _predecessor_rejection_payload_hashes(
        frozen_document
    )
    observed: set[str] = set()
    for index, (entry, batch_entry) in enumerate(
        zip(entries, frozen_document["entries"], strict=True)
    ):
        prefix = f"sealed_manifest.entries[{index}]"
        digest = _hash(f"{prefix}.actual_sha256", entry.get("actual_sha256"))
        if digest in registry_hashes or digest in observed:
            _fail(f"{prefix}.actual_sha256", "payload identity collision")
        if digest in predecessor_rejection_hashes:
            _fail(
                f"{prefix}.actual_sha256",
                "reuses a forbidden rejected Batch-v1 observed payload",
            )
        observed.add(digest)
        _expect(f"{prefix}.actual_attachment_byte_count", entry.get("actual_attachment_byte_count"), batch_entry["attachment_byte_count"])
        _validate_header_record(entry.get("header_validation"), batch_entry["attachment_byte_count"], f"{prefix}.header_validation")


def load_artifact(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.suffix != ".json":
        _fail(str(path), "must be a no-follow regular JSON file")
    try:
        payload = artifacts._read_regular_file_no_follow(path, 2_097_152)
    except (OSError, ValueError) as error:
        raise ContractError(
            f"{path}: must be a bounded no-follow single-link regular JSON file"
        ) from error
    return parse_json(payload, str(path)), payload


def _safe_relative_parts(value: str, prefix: str) -> tuple[str, ...]:
    if not isinstance(value, str) or not value or "\\" in value or "\x00" in value or "//" in value:
        _fail(prefix, "must be a non-empty safe repo-relative path")
    path = PurePosixPath(value)
    if path.is_absolute() or not path.parts or any(
        part in {"", ".", ".."} for part in path.parts
    ):
        _fail(prefix, "must be a safe repo-relative path")
    return path.parts


def _open_directory_chain_no_follow(
    repo_root: Path, relative_directory: str
) -> list[int]:
    parts = _safe_relative_parts(relative_directory, "artifact directory")
    root_before = os.lstat(repo_root)
    if not stat.S_ISDIR(root_before.st_mode) or stat.S_ISLNK(root_before.st_mode):
        _fail("repository root", "must be a no-follow directory")
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW
    descriptors: list[int] = []
    try:
        root_fd = os.open(repo_root, flags)
        descriptors.append(root_fd)
        root_opened = os.fstat(root_fd)
        if (root_before.st_dev, root_before.st_ino) != (
            root_opened.st_dev,
            root_opened.st_ino,
        ):
            _fail("repository root", "inode changed during open")
        for part in parts:
            parent_fd = descriptors[-1]
            before = os.stat(part, dir_fd=parent_fd, follow_symlinks=False)
            if not stat.S_ISDIR(before.st_mode) or stat.S_ISLNK(before.st_mode):
                _fail("artifact directory", f"unsafe ancestor {part!r}")
            child_fd = os.open(part, flags, dir_fd=parent_fd)
            descriptors.append(child_fd)
            opened = os.fstat(child_fd)
            if (before.st_dev, before.st_ino) != (opened.st_dev, opened.st_ino):
                _fail("artifact directory", f"ancestor changed during open: {part!r}")
        return descriptors
    except Exception:
        for descriptor in reversed(descriptors):
            os.close(descriptor)
        raise


def _read_exact_regular_at(
    parent_fd: int,
    name: str,
    *,
    expected_bytes: int,
    prefix: str,
) -> tuple[int, bytes]:
    if PurePosixPath(name).name != name or name in {"", ".", ".."}:
        _fail(prefix, "name must be one exact safe component")
    parent_before = os.fstat(parent_fd)
    if not stat.S_ISDIR(parent_before.st_mode):
        _fail(prefix, "parent descriptor is not a directory")
    before = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    if not stat.S_ISREG(before.st_mode) or stat.S_ISLNK(before.st_mode):
        _fail(prefix, "must be a no-follow regular file")
    if before.st_nlink != 1 or before.st_size != expected_bytes:
        _fail(prefix, "link count or byte count changed")
    descriptor = os.open(
        name,
        os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW,
        dir_fd=parent_fd,
    )
    try:
        opened = os.fstat(descriptor)
        if (
            before.st_dev,
            before.st_ino,
            before.st_mode,
            before.st_size,
            before.st_nlink,
            before.st_mtime_ns,
            before.st_ctime_ns,
        ) != (
            opened.st_dev,
            opened.st_ino,
            opened.st_mode,
            opened.st_size,
            opened.st_nlink,
            opened.st_mtime_ns,
            opened.st_ctime_ns,
        ):
            _fail(prefix, "inode changed during open")
        payload = bytearray()
        offset = 0
        while offset < expected_bytes:
            chunk = os.pread(
                descriptor,
                min(65_536, expected_bytes - offset),
                offset,
            )
            if not chunk:
                _fail(prefix, "file truncated during bounded raw-byte verification")
            payload.extend(chunk)
            offset += len(chunk)
        if os.pread(descriptor, 1, expected_bytes) != b"":
            _fail(prefix, "file grew beyond the exact byte boundary")
        final = os.fstat(descriptor)
        if (
            opened.st_dev,
            opened.st_ino,
            opened.st_mode,
            opened.st_size,
            opened.st_nlink,
            opened.st_mtime_ns,
            opened.st_ctime_ns,
        ) != (
            final.st_dev,
            final.st_ino,
            final.st_mode,
            final.st_size,
            final.st_nlink,
            final.st_mtime_ns,
            final.st_ctime_ns,
        ):
            _fail(prefix, "inode changed during bounded raw-byte verification")
        named_after = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        if not stat.S_ISREG(named_after.st_mode) or stat.S_ISLNK(
            named_after.st_mode
        ):
            _fail(prefix, "name no longer resolves to a no-follow regular file")
        if (
            named_after.st_dev,
            named_after.st_ino,
            named_after.st_mode,
            named_after.st_size,
            named_after.st_nlink,
            named_after.st_mtime_ns,
            named_after.st_ctime_ns,
        ) != (
            final.st_dev,
            final.st_ino,
            final.st_mode,
            final.st_size,
            final.st_nlink,
            final.st_mtime_ns,
            final.st_ctime_ns,
        ):
            _fail(prefix, "name no longer resolves to the verified inode")
        final_after_name = os.fstat(descriptor)
        if (
            final_after_name.st_dev,
            final_after_name.st_ino,
            final_after_name.st_mode,
            final_after_name.st_size,
            final_after_name.st_nlink,
            final_after_name.st_mtime_ns,
            final_after_name.st_ctime_ns,
        ) != (
            final.st_dev,
            final.st_ino,
            final.st_mode,
            final.st_size,
            final.st_nlink,
            final.st_mtime_ns,
            final.st_ctime_ns,
        ):
            _fail(prefix, "verified inode changed during final name binding")
        parent_after = os.fstat(parent_fd)
        if (
            parent_after.st_dev,
            parent_after.st_ino,
            parent_after.st_mode,
            parent_after.st_nlink,
            parent_after.st_mtime_ns,
            parent_after.st_ctime_ns,
        ) != (
            parent_before.st_dev,
            parent_before.st_ino,
            parent_before.st_mode,
            parent_before.st_nlink,
            parent_before.st_mtime_ns,
            parent_before.st_ctime_ns,
        ):
            _fail(prefix, "parent directory changed during bounded verification")
        return descriptor, bytes(payload)
    except Exception:
        os.close(descriptor)
        raise


def validate_payload_directory_fd(
    directory_fd: int,
    *,
    access_log: dict[str, Any],
    manifest_document: dict[str, Any],
) -> None:
    """Hash three exact registered payloads and recheck headers, never samples."""

    directory_stat = os.fstat(directory_fd)
    if not stat.S_ISDIR(directory_stat.st_mode):
        _fail("payload directory", "is not a directory")
    expected_identity = (
        access_log["publication"].get("prepared_directory_device"),
        access_log["publication"].get("prepared_directory_inode"),
    )
    if all(value is not None for value in expected_identity) and (
        directory_stat.st_dev,
        directory_stat.st_ino,
    ) != expected_identity:
        _fail("payload directory", "identity differs from publication intent")
    log_entries = _array("access_log.entries", access_log.get("entries"))
    manifest_entries = _array(
        "sealed_manifest.entries", manifest_document.get("entries")
    )
    if len(log_entries) != 3 or len(manifest_entries) != 3:
        _fail("payload directory", "requires exactly three registered entries")
    expected_parent = PurePosixPath(
        access_log["filesystem"]["final_batch_directory"]
    )
    for index, (log_entry_value, manifest_entry_value) in enumerate(
        zip(log_entries, manifest_entries, strict=True)
    ):
        prefix = f"payload_directory.entries[{index}]"
        log_entry = _mapping(f"{prefix}.log", log_entry_value)
        manifest_entry = _mapping(f"{prefix}.manifest", manifest_entry_value)
        destination = PurePosixPath(log_entry["destination_path"])
        if destination.parent != expected_parent:
            _fail(f"{prefix}.destination_path", "parent differs from final batch")
        expected_bytes = _integer(
            f"{prefix}.declared_attachment_byte_count",
            log_entry["declared_attachment_byte_count"],
            minimum=44,
        )
        descriptor, payload = _read_exact_regular_at(
            directory_fd,
            destination.name,
            expected_bytes=expected_bytes,
            prefix=prefix,
        )
        try:
            opened = os.fstat(descriptor)
            identity = _mapping(
                f"{prefix}.quarantine_file_identity",
                log_entry.get("quarantine_file_identity"),
            )
            _expect(
                f"{prefix}.file_identity",
                (opened.st_dev, opened.st_ino, opened.st_nlink, opened.st_size),
                (
                    identity["device"],
                    identity["inode"],
                    identity["link_count"],
                    identity["byte_count"],
                ),
            )
            digest = hashlib.sha256(payload).hexdigest()
            _expect(
                f"{prefix}.log_sha256",
                digest,
                log_entry["stream"]["actual_sha256"],
            )
            _expect(
                f"{prefix}.manifest_sha256",
                digest,
                manifest_entry["actual_sha256"],
            )
            header = acquisition.inspect_riff_pcm_header_only(
                descriptor, expected_bytes
            )
            _expect(
                f"{prefix}.log_header",
                header,
                log_entry["header_validation"],
            )
            _expect(
                f"{prefix}.manifest_header",
                header,
                manifest_entry["header_validation"],
            )
        finally:
            os.close(descriptor)


def validate_sealed_directory_fd(
    directory_fd: int,
    *,
    access_log: dict[str, Any],
    manifest_document: dict[str, Any],
    manifest_payload: bytes,
) -> None:
    """Reopen and bind the exact manifest plus every raw payload in one sealed dir."""

    manifest_ref = _mapping(
        "access_log.sealed_manifest", access_log.get("sealed_manifest")
    )
    expected_bytes = _integer(
        "access_log.sealed_manifest.byte_count",
        manifest_ref.get("byte_count"),
        minimum=1,
    )
    expected_identity = _mapping(
        "access_log.sealed_manifest.quarantine_file_identity",
        manifest_ref.get("quarantine_file_identity"),
    )
    manifest_name = access_log["filesystem"]["sealed_manifest_name"]
    descriptor, reopened_payload = _read_exact_regular_at(
        directory_fd,
        manifest_name,
        expected_bytes=expected_bytes,
        prefix="sealed manifest",
    )
    try:
        opened = os.fstat(descriptor)
        _expect(
            "sealed manifest file identity",
            (opened.st_dev, opened.st_ino, opened.st_nlink, opened.st_size),
            (
                expected_identity["device"],
                expected_identity["inode"],
                expected_identity["link_count"],
                expected_identity["byte_count"],
            ),
        )
        _expect("sealed manifest exact bytes", reopened_payload, manifest_payload)
        _expect(
            "sealed manifest parsed document",
            parse_json(reopened_payload, "sealed manifest"),
            manifest_document,
        )
    finally:
        os.close(descriptor)
    validate_payload_directory_fd(
        directory_fd,
        access_log=access_log,
        manifest_document=manifest_document,
    )


def _read_json_at(
    parent_fd: int, name: str, *, prefix: str
) -> tuple[dict[str, Any], bytes]:
    before = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
    if before.st_size > 2_097_152:
        _fail(prefix, "JSON exceeds the two-MiB bound")
    descriptor, payload = _read_exact_regular_at(
        parent_fd,
        name,
        expected_bytes=before.st_size,
        prefix=prefix,
    )
    os.close(descriptor)
    return parse_json(payload, prefix), payload


def validate_repository_terminal(repo_root: Path) -> tuple[str, str]:
    frozen = batch_validator.validate_repository(repo_root)
    log_path = PurePosixPath(batch_contract.ACCESS_LOG_PATH)
    log_chain = _open_directory_chain_no_follow(
        repo_root, log_path.parent.as_posix()
    )
    final_chain = _open_directory_chain_no_follow(
        repo_root, batch_contract.FINAL_BATCH_DIRECTORY
    )
    try:
        try:
            os.stat(
                log_path.name + artifacts.ACCESS_LOG_UPDATE_SUFFIX,
                dir_fd=log_chain[-1],
                follow_symlinks=False,
            )
        except FileNotFoundError:
            pass
        else:
            _fail(
                str(batch_contract.ACCESS_LOG_PATH),
                "stale atomic-update artifact requires no-network reconciliation",
            )
        log, log_payload = _read_json_at(
            log_chain[-1], log_path.name, prefix=str(batch_contract.ACCESS_LOG_PATH)
        )
        if log.get("attempt_status") != "completed":
            _fail(
                str(batch_contract.ACCESS_LOG_PATH),
                "terminal success requires completed status",
            )
        manifest, manifest_payload = _read_json_at(
            final_chain[-1],
            batch_contract.SEALED_MANIFEST_NAME,
            prefix="sealed manifest",
        )
        validate_access_log_document(
            log,
            frozen_batch=frozen,
            repo_root=repo_root,
            manifest_document=manifest,
            manifest_payload=manifest_payload,
        )
        if log_payload != artifacts.render(log):
            _fail(
                str(batch_contract.ACCESS_LOG_PATH),
                "access-log bytes are not deterministic",
            )
        final_stat = os.fstat(final_chain[-1])
        _expect(
            "terminal final directory identity",
            (final_stat.st_dev, final_stat.st_ino),
            (
                log["publication"]["published_directory_device"],
                log["publication"]["published_directory_inode"],
            ),
        )
        validate_sealed_directory_fd(
            final_chain[-1],
            access_log=log,
            manifest_document=manifest,
            manifest_payload=manifest_payload,
        )
        return (
            hashlib.sha256(log_payload).hexdigest(),
            hashlib.sha256(manifest_payload).hexdigest(),
        )
    finally:
        for descriptor in reversed(final_chain):
            os.close(descriptor)
        for descriptor in reversed(log_chain):
            os.close(descriptor)


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    try:
        log_hash, manifest_hash = validate_repository_terminal(repo_root)
    except (ContractError, batch_validator.ContractError, OSError, ValueError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: RIOTBOX-1430 acquisition access log and sealed manifest are terminal")
    print(f"acquisition_batch_v2_access_log_raw_sha256={log_hash}")
    print(f"acquisition_batch_v2_sealed_manifest_raw_sha256={manifest_hash}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
