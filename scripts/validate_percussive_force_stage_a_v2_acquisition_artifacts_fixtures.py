#!/usr/bin/env python3
"""Fail-closed synthetic fixtures for acquisition logs and sealed manifests."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any, Callable

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_artifacts as artifacts
import validate_percussive_force_stage_a_v2_acquisition_artifacts as validator
import validate_percussive_force_stage_a_v2_acquisition_batch as batch_validator


REPO = Path(__file__).resolve().parents[1]
ATTEMPT_ID = "11111111-1111-4111-8111-111111111111"


def timestamp(second: int) -> str:
    return f"2026-08-10T12:00:{second:02d}Z"


def transition(log: dict[str, Any], state: str, second: int) -> None:
    log["attempt_status"] = state
    log["transition_history"].append(
        {
            "sequence": len(log["transition_history"]) + 1,
            "state": state,
            "at_utc": timestamp(second),
        }
    )


def synthetic_header(declared_bytes: int) -> dict[str, Any]:
    data_size = declared_bytes - 44
    channels = 1
    width = 16
    align = 2
    rate = 192_000
    frames = data_size // align
    assert data_size % align == 0 and frames <= rate * 16
    return {
        "container": "RIFF/WAVE",
        "riff_size": declared_bytes - 8,
        "fmt_offset": 20,
        "fmt_size": 16,
        "format_tag": 1,
        "pcm_encoding": "signed_little_endian_integer",
        "channels": channels,
        "sample_rate_hz": rate,
        "byte_rate": rate * align,
        "block_align": align,
        "sample_width_bits": width,
        "valid_bits": width,
        "data_offset": 44,
        "data_size": data_size,
        "frame_count": frames,
        "duration_numerator_frames": frames,
        "duration_denominator_sample_rate": rate,
        "container_overhead_bytes": 44,
        "chunk_table": [
            {
                "chunk_id": "fmt ",
                "header_offset": 12,
                "payload_offset": 20,
                "payload_size": 16,
                "padding_bytes": 0,
                "payload_read_for_header_validation": True,
            },
            {
                "chunk_id": "data",
                "header_offset": 36,
                "payload_offset": 44,
                "payload_size": data_size,
                "padding_bytes": 0,
                "payload_read_for_header_validation": False,
            },
        ],
        "header_validation_scope": "container_headers_only_no_sample_payload_reads",
        "sample_decode_performed": False,
        "pcm_sample_iteration_performed": False,
    }


def build_documents() -> tuple[
    batch_validator.FrozenAcquisitionBatchV1,
    dict[str, Any],
    dict[str, Any],
    bytes,
]:
    frozen = batch_validator.validate_repository(REPO)
    snapshot = artifacts.build_implementation_snapshot(REPO)
    log = artifacts.build_initial_access_log(
        batch_document=frozen.document,
        batch_raw_sha256=frozen.raw_sha256,
        batch_semantic_sha256=frozen.semantic_sha256,
        implementation_snapshot=snapshot,
        attempt_id=ATTEMPT_ID,
        started_at_utc=timestamp(0),
    )
    log["filesystem"]["access_log_parent_fsync_completed"] = True
    log["filesystem"]["quarantine_created_exclusively"] = True
    log["filesystem"]["publication_probe_completed"] = True
    log["filesystem"]["available_free_bytes_before_network"] = (
        log["filesystem"]["required_free_bytes_before_network"] + 1
    )
    transition(log, "preflight_passed", 1)
    observed = artifacts.observed_bindings(
        frozen.raw_sha256,
        frozen.semantic_sha256,
        snapshot["aggregate_sha256"],
    )
    for index in range(3):
        checkpoint = log["revalidation_checkpoints"][index]
        checkpoint.update(
            {
                "status": "passed",
                "checked_at_utc": timestamp(2 + index * 3),
                "observed_bindings": copy.deepcopy(observed),
            }
        )
        if index == 0:
            transition(log, "acquiring", 3)
        record = log["entries"][index]
        record["state"] = "header_verified"
        record["request_count"] = 1
        record["request_started_at_utc"] = timestamp(3 + index * 3)
        record["verified_at_utc"] = timestamp(4 + index * 3)
        peer = ("1.1.1.1", "8.8.8.8", "9.9.9.9")[index]
        record["network"]["dns_answers"] = [peer]
        record["network"]["selected_peer_ip"] = peer
        record["network"]["connected_peer_ip"] = peer
        record["network"]["tls"]["negotiated_version"] = "TLSv1.3"
        record["network"]["tls"]["peer_certificate_sha256"] = hashlib.sha256(
            f"certificate-{index}".encode()
        ).hexdigest()
        record["network"]["response_gate"] = acquisition.validate_response_gate(
            status=200,
            headers={
                "content-length": [str(record["declared_attachment_byte_count"])],
                "content-type": ["audio/x-wav"],
            },
            expected_byte_count=record["declared_attachment_byte_count"],
        )
        record["stream"] = {
            "body_read_cap": record["declared_attachment_byte_count"] + 1,
            "body_bytes_consumed": record["declared_attachment_byte_count"],
            "actual_sha256": hashlib.sha256(
                f"synthetic-payload-{index}".encode()
            ).hexdigest(),
            "sample_decode_performed": False,
            "pcm_sample_iteration_performed": False,
        }
        record["quarantine_file_identity"] = {
            "device": 100,
            "inode": 1_000 + index,
            "link_count": 1,
            "byte_count": record["declared_attachment_byte_count"],
        }
        record["header_validation"] = synthetic_header(
            record["declared_attachment_byte_count"]
        )
        log["request_count"] += 1
        log["successful_request_count"] += 1
    transition(log, "all_headers_verified", 12)
    manifest = artifacts.build_sealed_manifest(log)
    manifest_payload = artifacts.render(manifest)
    for record in log["entries"]:
        record["state"] = "sealed"
    log["sealed_manifest"].update(
        {
            "state": "sealed_in_quarantine",
            "byte_count": len(manifest_payload),
            "raw_sha256": hashlib.sha256(manifest_payload).hexdigest(),
            "semantic_sha256": artifacts.semantic_sha256(manifest),
            "quarantine_file_identity": {
                "device": 100,
                "inode": 2_000,
                "link_count": 1,
                "byte_count": len(manifest_payload),
            },
        }
    )
    transition(log, "sealed_in_quarantine", 13)
    log["sealed_payload_revalidation"].update(
        {
            "state": "passed",
            "checked_at_utc": timestamp(14),
            "entry_count": 3,
            "raw_sha256_recomputed": True,
            "header_reinspection_performed": True,
        }
    )
    checkpoint = log["revalidation_checkpoints"][3]
    checkpoint.update(
        {
            "status": "passed",
            "checked_at_utc": timestamp(14),
            "observed_bindings": copy.deepcopy(observed),
        }
    )
    log["publication"].update(
        {
            "state": "pending",
            "prepared_directory_device": 100,
            "prepared_directory_inode": 200,
        }
    )
    transition(log, "publication_pending", 15)
    for record in log["entries"]:
        record["state"] = "published"
    log["sealed_manifest"]["state"] = "published"
    log["publication"].update(
        {
            "state": "completed",
            "rename_count": 1,
            "source_parent_fsync_completed": True,
            "destination_parent_fsync_completed": True,
            "published_at_utc": timestamp(16),
            "published_directory_device": 100,
            "published_directory_inode": 200,
        }
    )
    transition(log, "completed", 16)
    log["completed_at_utc"] = timestamp(16)
    return frozen, log, manifest, manifest_payload


def validate_log(
    frozen: batch_validator.FrozenAcquisitionBatchV1,
    log: dict[str, Any],
    manifest: dict[str, Any],
    manifest_payload: bytes,
) -> None:
    validator.validate_access_log_document(
        log,
        frozen_batch=frozen,
        repo_root=REPO,
        manifest_document=manifest,
        manifest_payload=manifest_payload,
    )


def expect_rejected(name: str, action: Callable[[], None]) -> None:
    try:
        action()
    except (validator.ContractError, ValueError, OSError):
        return
    raise AssertionError(f"fixture {name!r} failed open")


def expect_rejected_token(
    name: str, token: str, action: Callable[[], None]
) -> None:
    try:
        action()
    except (validator.ContractError, ValueError, OSError) as error:
        if token not in str(error):
            raise AssertionError(
                f"fixture {name!r} rejected for the wrong reason: {error}"
            ) from error
        return
    raise AssertionError(f"fixture {name!r} failed open")


def coherent_fmt_chunk_18(value: dict[str, Any]) -> None:
    header = value["entries"][0]["header_validation"]
    declared = value["entries"][0]["declared_attachment_byte_count"]
    data_size = declared - 46
    header["chunk_table"][0]["payload_size"] = 18
    header["chunk_table"][1].update(
        {
            "header_offset": 38,
            "payload_offset": 46,
            "payload_size": data_size,
        }
    )
    header["data_offset"] = 46
    header["data_size"] = data_size
    header["frame_count"] = data_size // 2
    header["duration_numerator_frames"] = data_size // 2
    header["container_overhead_bytes"] = 46


def mutate_log_case(
    name: str,
    frozen: batch_validator.FrozenAcquisitionBatchV1,
    baseline: dict[str, Any],
    manifest: dict[str, Any],
    manifest_payload: bytes,
    mutation: Callable[[dict[str, Any]], None],
) -> None:
    candidate = copy.deepcopy(baseline)
    mutation(candidate)
    expect_rejected(
        name,
        lambda: validate_log(frozen, candidate, manifest, manifest_payload),
    )


def mutate_manifest_case(
    name: str,
    frozen: batch_validator.FrozenAcquisitionBatchV1,
    log: dict[str, Any],
    baseline: dict[str, Any],
    mutation: Callable[[dict[str, Any]], None],
) -> None:
    candidate = copy.deepcopy(baseline)
    mutation(candidate)
    expect_rejected(
        name,
        lambda: validator.validate_manifest_document(
            candidate,
            access_log=log,
            frozen_batch=frozen,
            repo_root=REPO,
        ),
    )


def main() -> int:
    frozen, log, manifest, manifest_payload = build_documents()
    validate_log(frozen, log, manifest, manifest_payload)
    validator.validate_manifest_document(
        manifest,
        access_log=log,
        frozen_batch=frozen,
        repo_root=REPO,
    )
    cases: list[tuple[str, Callable[[dict[str, Any]], None]]] = [
        ("extra_top_level", lambda value: value.__setitem__("extra", False)),
        ("wrong_schema", lambda value: value.__setitem__("schema", "wrong")),
        ("bool_schema_version", lambda value: value.__setitem__("schema_version", True)),
        ("invalid_attempt_uuid", lambda value: value.__setitem__("attempt_id", "not-a-uuid")),
        ("noncanonical_timestamp", lambda value: value.__setitem__("started_at_utc", "2026-08-10T12:00:00.1Z")),
        ("quality_claim", lambda value: value.__setitem__("quality_proof", True)),
        ("source_qualification_claim", lambda value: value.__setitem__("source_qualification_claimed", True)),
        ("protocol_pin_drift", lambda value: value["contract_bindings"]["protocol_v2"].__setitem__("raw_sha256", "0" * 64)),
        ("batch_pin_drift", lambda value: value["contract_bindings"]["acquisition_batch_v1"].__setitem__("semantic_sha256", "0" * 64)),
        ("implementation_hash_drift", lambda value: value["implementation_snapshot"].__setitem__("aggregate_sha256", "0" * 64)),
        ("implementation_order_drift", lambda value: value["implementation_snapshot"]["files"].reverse()),
        ("holdout_access_claim", lambda value: value["scope_assertions"].__setitem__("holdout_audio_access_performed", True)),
        ("directory_listing_claim", lambda value: value["scope_assertions"].__setitem__("directory_discovery_performed", True)),
        ("playback_claim", lambda value: value["scope_assertions"].__setitem__("source_audio_playback_performed", True)),
        ("decode_claim", lambda value: value["scope_assertions"].__setitem__("audio_decode_performed", True)),
        ("final_path_drift", lambda value: value["filesystem"].__setitem__("final_batch_directory", "data/elsewhere")),
        ("overwrite_claim", lambda value: value["filesystem"].__setitem__("overwrite_performed", True)),
        ("checkpoint_reorder", lambda value: value["revalidation_checkpoints"].reverse()),
        ("checkpoint_pin_drift", lambda value: value["revalidation_checkpoints"][0]["observed_bindings"].__setitem__("implementation_aggregate_sha256", "0" * 64)),
        ("request_count_four", lambda value: value.__setitem__("request_count", 4)),
        ("success_count_two", lambda value: value.__setitem__("successful_request_count", 2)),
        ("entry_order_swap", lambda value: value["entries"].reverse()),
        ("entry_family_drift", lambda value: value["entries"][0].__setitem__("source_family", "electronic_drums")),
        ("entry_url_drift", lambda value: value["entries"][0].__setitem__("download_url", "https://opengameart.org/sites/default/files/other.wav")),
        ("entry_retry", lambda value: value["entries"][0].__setitem__("request_count", 2)),
        ("head_method", lambda value: value["entries"][0]["network"].__setitem__("method", "HEAD")),
        ("dns_search_suffix_eligible", lambda value: value["entries"][0]["network"].__setitem__("dns_query_name", "opengameart.org")),
        ("request_header_drift", lambda value: value["entries"][0]["network"]["request_headers"].append(["Cookie", "x=y"])),
        ("private_dns", lambda value: value["entries"][0]["network"].__setitem__("dns_answers", ["127.0.0.1"])),
        ("dns_unsorted", lambda value: value["entries"][0]["network"].__setitem__("dns_answers", ["9.9.9.9", "1.1.1.1"])),
        ("peer_mismatch", lambda value: value["entries"][0]["network"].__setitem__("connected_peer_ip", "8.8.8.8")),
        ("tls_downgrade", lambda value: value["entries"][0]["network"]["tls"].__setitem__("negotiated_version", "TLSv1.1")),
        ("redirect", lambda value: value["entries"][0]["network"].__setitem__("redirect_count", 1)),
        ("retry", lambda value: value["entries"][0]["network"].__setitem__("retry_count", 1)),
        ("proxy", lambda value: value["entries"][0]["network"].__setitem__("proxy_used", True)),
        ("wrong_status", lambda value: value["entries"][0]["network"]["response_gate"].__setitem__("response_status", 206)),
        ("stream_overread", lambda value: value["entries"][0]["stream"].__setitem__("body_read_cap", value["entries"][0]["stream"]["body_read_cap"] + 1)),
        ("stream_decode", lambda value: value["entries"][0]["stream"].__setitem__("sample_decode_performed", True)),
        ("duplicate_new_hash", lambda value: value["entries"][1]["stream"].__setitem__("actual_sha256", value["entries"][0]["stream"]["actual_sha256"])),
        ("header_data_read", lambda value: value["entries"][0]["header_validation"]["chunk_table"][1].__setitem__("payload_read_for_header_validation", True)),
        ("header_decode", lambda value: value["entries"][0]["header_validation"].__setitem__("sample_decode_performed", True)),
        ("header_duration", lambda value: value["entries"][0]["header_validation"].__setitem__("frame_count", 99_999_999)),
        ("manifest_hash_drift", lambda value: value["sealed_manifest"].__setitem__("raw_sha256", "0" * 64)),
        ("manifest_inode_drift", lambda value: value["sealed_manifest"]["quarantine_file_identity"].__setitem__("inode", 0)),
        ("rename_twice", lambda value: value["publication"].__setitem__("rename_count", 2)),
        ("parent_fsync_missing", lambda value: value["publication"].__setitem__("destination_parent_fsync_completed", False)),
        ("published_inode_drift", lambda value: value["publication"].__setitem__("published_directory_inode", 201)),
        ("illegal_transition", lambda value: value["transition_history"].pop(2)),
        ("forward_registry_pin", lambda value: value.__setitem__("future", "riotbox.source_holdout_rotation.v3")),
    ]
    for name, mutation in cases:
        mutate_log_case(
            name,
            frozen,
            log,
            manifest,
            manifest_payload,
            mutation,
        )

    registry = json.loads((REPO / "docs/benchmarks/source_holdout_rotation_v2.json").read_text())
    registry_hash = registry["entries"][0]["sha256"]
    mutate_log_case(
        "registry_v2_hash_collision",
        frozen,
        log,
        manifest,
        manifest_payload,
        lambda value: value["entries"][0]["stream"].__setitem__("actual_sha256", registry_hash),
    )

    manifest_cases: list[tuple[str, Callable[[dict[str, Any]], None]]] = [
        ("manifest_extra", lambda value: value.__setitem__("extra", False)),
        ("manifest_quality_claim", lambda value: value.__setitem__("quality_proof", True)),
        ("manifest_self_log_hash", lambda value: value["attempt_binding"].__setitem__("access_log_raw_sha256", "0" * 64)),
        ("manifest_entry_reorder", lambda value: value["entries"].reverse()),
        ("manifest_identity_drift", lambda value: value["entries"][0].__setitem__("case_id", "other")),
        ("manifest_byte_count_drift", lambda value: value["entries"][0].__setitem__("actual_attachment_byte_count", value["entries"][0]["actual_attachment_byte_count"] + 1)),
        ("manifest_header_read", lambda value: value["entries"][0]["header_validation"]["chunk_table"][1].__setitem__("payload_read_for_header_validation", True)),
        ("manifest_publication_claim", lambda value: value["publication_contract"].__setitem__("publication_success_claimed_by_manifest", True)),
        ("manifest_forward_matrix", lambda value: value.__setitem__("forward", "riotbox.percussive_force_development_matrix.v3")),
    ]
    for name, mutation in manifest_cases:
        mutate_manifest_case(name, frozen, log, manifest, mutation)

    targeted: list[tuple[str, str, Callable[[dict[str, Any]], None]]] = [
        ("coherent_fmt_chunk_size", "fmt_size", coherent_fmt_chunk_18),
        (
            "request_two_before_prior_verified",
            "later request started before prior header verification timestamp",
            lambda value: value["entries"][1].__setitem__(
                "request_started_at_utc", timestamp(3)
            ),
        ),
        (
            "verified_before_request",
            "precedes request start",
            lambda value: value["entries"][0].__setitem__(
                "verified_at_utc", timestamp(2)
            ),
        ),
        (
            "published_before_pending",
            "precedes publication_pending",
            lambda value: value["publication"].__setitem__(
                "published_at_utc", timestamp(14)
            ),
        ),
        (
            "completed_without_log_parent_fsync",
            "access_log_parent_fsync_completed",
            lambda value: value["filesystem"].__setitem__(
                "access_log_parent_fsync_completed", False
            ),
        ),
        (
            "completed_without_exclusive_quarantine",
            "quarantine_created_exclusively",
            lambda value: value["filesystem"].__setitem__(
                "quarantine_created_exclusively", False
            ),
        ),
        (
            "completed_with_incomplete_cleanup",
            "quarantine_cleanup_state",
            lambda value: value["filesystem"].__setitem__(
                "quarantine_cleanup_state", "cleanup_incomplete_fail_closed"
            ),
        ),
        (
            "completed_without_publication_probe",
            "publication_probe_completed",
            lambda value: value["filesystem"].__setitem__(
                "publication_probe_completed", False
            ),
        ),
        (
            "completed_without_free_space_evidence",
            "available_free_bytes_before_network",
            lambda value: value["filesystem"].__setitem__(
                "available_free_bytes_before_network", None
            ),
        ),
        (
            "completed_without_sealed_payload_revalidation",
            "sealed_payload_revalidation.state",
            lambda value: value.__setitem__(
                "sealed_payload_revalidation",
                copy.deepcopy(
                    artifacts.build_initial_access_log(
                        batch_document=frozen.document,
                        batch_raw_sha256=frozen.raw_sha256,
                        batch_semantic_sha256=frozen.semantic_sha256,
                        implementation_snapshot=value["implementation_snapshot"],
                        attempt_id=value["attempt_id"],
                        started_at_utc=value["started_at_utc"],
                    )["sealed_payload_revalidation"]
                ),
            ),
        ),
        (
            "atomic_update_path_drift",
            "access_log_update_path",
            lambda value: value["filesystem"].__setitem__(
                "access_log_update_path", "artifacts/elsewhere.next"
            ),
        ),
    ]
    for name, token, mutation in targeted:
        candidate = copy.deepcopy(log)
        mutation(candidate)
        expect_rejected_token(
            name,
            token,
            lambda candidate=candidate: validate_log(
                frozen, candidate, manifest, manifest_payload
            ),
        )

    expect_rejected(
        "duplicate_json_key",
        lambda: validator.parse_json(b'{"schema":1,"schema":2}', "fixture"),
    )
    expect_rejected(
        "nonfinite_json",
        lambda: validator.parse_json(b'{"value":NaN}', "fixture"),
    )
    print(
        f"PASS: {len(cases) + len(manifest_cases) + len(targeted) + 4} fail-closed acquisition-artifact fixtures"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
