#!/usr/bin/env python3
"""Fail-closed mutation fixtures for the Stage-A-v2 acquisition batch v2."""

from __future__ import annotations

import copy
import shutil
import tempfile
from pathlib import Path
from typing import Any, Callable

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_batch_v2_contract as batch_contract
import percussive_force_stage_a_v2_contract as protocol_v2_contract
import validate_percussive_force_stage_a_v2_acquisition_batch_v2 as validator


REPO_ROOT = Path(__file__).resolve().parents[1]


def expect_fail(name: str, operation: Callable[[], None], token: str) -> None:
    try:
        operation()
    except (validator.ContractError, acquisition.AcquisitionError, ValueError) as error:
        if token not in str(error):
            raise AssertionError(
                f"{name}: wrong failure {error!s}; expected token {token!r}"
            ) from error
        return
    raise AssertionError(f"{name}: mutation unexpectedly validated")


def _prepare_repository(root: Path) -> None:
    required = (
        protocol_v2_contract.PROTOCOL_V1_REL,
        protocol_v2_contract.PROTOCOL_V2_REL,
        Path("docs/benchmarks/percussive_force_development_matrix_v1.json"),
        Path("docs/benchmarks/percussive_force_development_matrix_v2.json"),
        Path("docs/benchmarks/source_holdout_rotation_v1.json"),
        batch_contract.REGISTRY_V2_REL,
        batch_contract.PREDECESSOR_BATCH_REL,
        batch_contract.BATCH_REL,
        batch_contract.PREDECESSOR_REJECTION_REPORT_REL,
    )
    for relative in required:
        destination = root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(REPO_ROOT / relative, destination)


def _tampered_repository_operation(
    relative: Path, transform: Callable[[bytes], bytes]
) -> Callable[[], None]:
    def operation() -> None:
        with tempfile.TemporaryDirectory(prefix="riotbox-stage-a-batch-v2-") as raw:
            root = Path(raw)
            _prepare_repository(root)
            path = root / relative
            path.write_bytes(transform(path.read_bytes()))
            validator.validate_repository(root)

    return operation


def main() -> int:
    frozen = validator.validate_repository(REPO_ROOT)
    document = frozen.document
    if (
        frozen.raw_sha256 != validator.EXPECTED_BATCH_RAW_SHA256
        or frozen.semantic_sha256 != validator.EXPECTED_BATCH_SEMANTIC_SHA256
    ):
        raise AssertionError("validated batch v2 does not expose its frozen pins")
    try:
        validator.FrozenAcquisitionBatchV2(raw_sha256=frozen.raw_sha256)
    except TypeError as error:
        if "must be created by validate_repository" not in str(error):
            raise AssertionError(
                f"sealed constructor raised the wrong error: {error}"
            ) from error
    else:
        raise AssertionError("FrozenAcquisitionBatchV2 constructor is forgeable")
    forged = object.__new__(validator.FrozenAcquisitionBatchV2)
    expect_fail(
        "forged_binding_without_root",
        lambda: validator.revalidate_frozen_batch(forged),
        "no repository root",
    )
    forged_payload = validator.FrozenAcquisitionBatchV2._from_validated(  # noqa: SLF001
        REPO_ROOT,
        b'{"schema":"forged"}\n',
    )
    if validator.revalidate_frozen_batch(forged_payload).raw_sha256 != frozen.raw_sha256:
        raise AssertionError("repository revalidation trusted a forged batch-v2 payload")

    cases: list[tuple[str, Callable[[], None], str]] = []

    def add_case(
        name: str,
        mutate: Callable[[dict[str, Any]], None],
        token: str,
    ) -> None:
        def operation() -> None:
            mutated = copy.deepcopy(document)
            mutate(mutated)
            validator.validate_document(mutated, repo_root=REPO_ROOT)

        cases.append((name, operation, token))

    add_case(
        "schema_version_bool",
        lambda value: value.__setitem__("schema_version", True),
        "batch.schema_version",
    )
    add_case(
        "wrong_schema",
        lambda value: value.__setitem__(
            "schema", "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1"
        ),
        "batch.schema",
    )
    add_case(
        "wrong_owner",
        lambda value: value.__setitem__("owner_ticket", "RIOTBOX-1428"),
        "batch.owner_ticket",
    )
    add_case(
        "quality_claim",
        lambda value: value.__setitem__("quality_proof", True),
        "batch.quality_proof",
    )
    add_case(
        "source_qualification_claim",
        lambda value: value.__setitem__("source_qualification_claimed", True),
        "source_qualification_claimed",
    )
    add_case(
        "human_review_claim",
        lambda value: value.__setitem__("human_review_claimed", True),
        "human_review_claimed",
    )
    add_case(
        "protocol_raw_pin",
        lambda value: value["protocol_binding"].__setitem__("raw_sha256", "0" * 64),
        "batch.protocol_binding",
    )
    add_case(
        "registry_semantic_pin",
        lambda value: value["predecessor_registry_binding"].__setitem__(
            "semantic_sha256", "0" * 64
        ),
        "batch.predecessor_registry_binding",
    )
    add_case(
        "holdout_union_reopened",
        lambda value: value["predecessor_registry_binding"].__setitem__(
            "holdout_union_state", "opened"
        ),
        "batch.predecessor_registry_binding",
    )

    rejection_mutations: tuple[
        tuple[str, str, Any], ...
    ] = (
        ("predecessor_batch_path", "batch_path", "wrong.json"),
        ("predecessor_batch_schema", "batch_schema", "wrong.v1"),
        ("predecessor_batch_raw", "batch_raw_sha256", "0" * 64),
        ("predecessor_batch_semantic", "batch_semantic_sha256", "0" * 64),
        ("rejection_report_path", "rejection_report_path", "wrong.md"),
        ("rejection_report_raw", "rejection_report_raw_sha256", "0" * 64),
        ("predecessor_attempt", "attempt_id", "00000000-0000-0000-0000-000000000000"),
        ("predecessor_access_log_path", "access_log_path", "wrong.json"),
        ("predecessor_access_log_raw", "access_log_raw_sha256", "0" * 64),
        ("predecessor_attempt_status", "attempt_status", "success"),
        ("predecessor_rejection_stage", "rejection_stage", "request_1_header"),
        ("predecessor_request_count", "request_count", 1),
        ("predecessor_request_count_bool", "request_count", True),
        ("predecessor_success_count", "successful_request_count", 2),
        ("predecessor_success_count_bool", "successful_request_count", True),
        ("v1_retry_enabled", "batch_v1_retry_allowed", True),
        ("survivor_reuse_enabled", "survivor_reuse_allowed", True),
        ("payload_reuse_enabled", "payload_reuse_allowed", True),
    )
    for name, key, replacement in rejection_mutations:
        add_case(
            name,
            lambda value, key=key, replacement=replacement: value[
                "predecessor_acquisition_rejection_binding"
            ].__setitem__(key, replacement),
            "batch.predecessor_acquisition_rejection_binding",
        )
    add_case(
        "forbidden_payload_removed",
        lambda value: value["predecessor_acquisition_rejection_binding"][
            "forbidden_observed_payload_sha256"
        ].pop(),
        "batch.predecessor_acquisition_rejection_binding",
    )
    add_case(
        "forbidden_payload_reordered",
        lambda value: value["predecessor_acquisition_rejection_binding"][
            "forbidden_observed_payload_sha256"
        ].reverse(),
        "batch.predecessor_acquisition_rejection_binding",
    )
    add_case(
        "forbidden_payload_rewritten",
        lambda value: value["predecessor_acquisition_rejection_binding"][
            "forbidden_observed_payload_sha256"
        ].__setitem__(0, "0" * 64),
        "batch.predecessor_acquisition_rejection_binding",
    )

    add_case(
        "profile_body_scope_claim",
        lambda value: value["metadata_basis"].__setitem__(
            "scope", "named_OpenGameArt_content_pages_and_author_profiles_only"
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "batch_v2_body_access_claim",
        lambda value: value["metadata_basis"].__setitem__(
            "attachment_body_bytes_accessed_for_batch_v2", True
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "false_disclosure_mislabeled_as_proof",
        lambda value: value["license_policy"].__setitem__(
            "false_disclosure_value_is_proof_of_absence", True
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "partial_two_entry_batch",
        lambda value: value["entries"].pop(),
        "exactly three",
    )
    add_case(
        "four_entry_batch",
        lambda value: value["entries"].append(copy.deepcopy(value["entries"][0])),
        "exactly three",
    )
    add_case(
        "entry_order_changed",
        lambda value: value["entries"].reverse(),
        "ordinals",
    )
    add_case(
        "family_collapse",
        lambda value: value["entries"][1].__setitem__("source_family", "dense_break"),
        "source_families",
    )
    add_case(
        "duplicate_v2_author",
        lambda value: value["entries"][1].__setitem__(
            "author", value["entries"][0]["author"]
        ),
        "duplicates another batch-v2 identity",
    )
    add_case(
        "v1_author_reuse",
        lambda value: value["entries"][0].__setitem__("author", "ELDRITCH GRIM"),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "registry_author_reuse",
        lambda value: value["entries"][0].__setitem__("author", "CINAMENG"),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "v1_case_reuse",
        lambda value: value["entries"][0].__setitem__(
            "case_id", "oga_eldritch_grim_pirates_incoming"
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "registry_case_reuse",
        lambda value: value["entries"][0].__setitem__(
            "case_id", "oga_cinameng_can_be_so_beautiful"
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "v1_pack_reuse",
        lambda value: value["entries"][0].__setitem__(
            "source_pack_id", "oga_pirates_incoming"
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "registry_pack_reuse",
        lambda value: value["entries"][0].__setitem__(
            "source_pack_id", "oga_can_be_so_beautiful"
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "v1_page_reuse",
        lambda value: value["entries"][0].__setitem__(
            "page_url", "https://opengameart.org/content/pirates-incoming"
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "registry_page_reuse",
        lambda value: value["entries"][0].__setitem__(
            "page_url", "https://opengameart.org/content/can-be-so-beautiful"
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "v1_download_reuse",
        lambda value: value["entries"][0].__setitem__(
            "download_url",
            "https://opengameart.org/sites/default/files/pirates_incoming_loop.wav",
        ),
        "download_url_filename",
    )
    add_case(
        "registry_download_reuse",
        lambda value: value["entries"][0].__setitem__(
            "download_url",
            "https://opengameart.org/sites/default/files/can_be_so_beautiful.wav",
        ),
        "download_url_filename",
    )
    add_case(
        "v1_attachment_id_reuse",
        lambda value: value["entries"][0].__setitem__(
            "provider_attachment_id", 224_760
        ),
        "reuses a Registry-v2 or batch-v1 identity",
    )
    add_case(
        "duplicate_v2_attachment_id",
        lambda value: value["entries"][1].__setitem__(
            "provider_attachment_id", value["entries"][0]["provider_attachment_id"]
        ),
        "duplicates another batch-v2 identity",
    )
    add_case(
        "v1_attachment_name_reuse",
        lambda value: value["entries"][0].__setitem__(
            "attachment_filename", "PIRATES_INCOMING_LOOP.WAV"
        ),
        "reuses a Registry-v2 or batch-v1 attachment name",
    )
    add_case(
        "registry_attachment_name_reuse",
        lambda value: value["entries"][0].__setitem__(
            "attachment_filename", "CAN_BE_SO_BEAUTIFUL.WAV"
        ),
        "reuses a Registry-v2 or batch-v1 attachment name",
    )
    add_case(
        "duplicate_v2_attachment_name",
        lambda value: value["entries"][1].__setitem__(
            "attachment_filename", value["entries"][0]["attachment_filename"]
        ),
        "duplicates another batch-v2 attachment name",
    )
    add_case(
        "decoded_url_name_drift",
        lambda value: value["entries"][1].__setitem__(
            "download_url_filename", "encoded-name-drift.wav"
        ),
        "download_url_filename",
    )
    add_case(
        "cosmac_display_and_storage_name_conflated",
        lambda value: value["entries"][2].__setitem__(
            "attachment_filename", "title_1.wav"
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "wrong_provider",
        lambda value: value["entries"][0].__setitem__("provider", "Other"),
        ".provider",
    )
    add_case(
        "wrong_license",
        lambda value: value["entries"][0].__setitem__("license", "CC-BY-3.0"),
        ".license",
    )
    add_case(
        "third_party_disclosure_changed",
        lambda value: value["entries"][0].__setitem__(
            "third_party_source_or_sample_pack_disclosed_on_page", True
        ),
        "third_party_source_or_sample_pack",
    )
    add_case(
        "third_party_false_claims_proof",
        lambda value: value["entries"][0].__setitem__(
            "third_party_disclosure_interpretation", "proof_of_no_third_party_material"
        ),
        "third_party_disclosure_interpretation",
    )
    add_case(
        "commercial_reference",
        lambda value: value["entries"][0].__setitem__("commercial_reference", True),
        "commercial_reference",
    )
    add_case(
        "holdout_partition",
        lambda value: value["entries"][0].__setitem__("partition", "holdout_a"),
        ".partition",
    )
    add_case(
        "source_start_integer_is_not_float_zero",
        lambda value: value["entries"][0].__setitem__("source_start_seconds", 0),
        "source_start_seconds",
    )
    add_case(
        "family_promoted_without_qualification",
        lambda value: value["entries"][0].__setitem__(
            "family_assignment_state", "human_confirmed"
        ),
        "family_assignment_state",
    )
    add_case(
        "sparse_metadata_overclaim",
        lambda value: value["entries"][1].__setitem__(
            "metadata_family_basis", "page_confirms_sparse_loop_with_three_onsets"
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "byte_count_bool",
        lambda value: value["entries"][0].__setitem__("attachment_byte_count", True),
        "attachment_byte_count",
    )
    add_case(
        "byte_count_exceeds_bound",
        lambda value: value["entries"][0].__setitem__(
            "attachment_byte_count", acquisition.maximum_declared_attachment_bytes() + 1
        ),
        "exceeds the frozen",
    )
    add_case(
        "page_wrong_host",
        lambda value: value["entries"][0].__setitem__(
            "page_url", "https://example.org/content/loopable-beat-for-ludumdare-game"
        ),
        "provider host",
    )
    add_case(
        "download_http",
        lambda value: value["entries"][0].__setitem__(
            "download_url", value["entries"][0]["download_url"].replace("https:", "http:")
        ),
        "HTTPS",
    )
    add_case(
        "parent_traversal_destination",
        lambda value: value["entries"][0].__setitem__(
            "destination_path", "data/test_audio/external/RIOTBOX-1423/wav/../escape.wav"
        ),
        "lexically safe",
    )
    add_case(
        "absolute_destination",
        lambda value: value["entries"][0].__setitem__(
            "destination_path", "/tmp/escape.wav"
        ),
        "repo-relative",
    )
    add_case(
        "nested_destination",
        lambda value: value["entries"][0].__setitem__(
            "destination_path", f"{batch_contract.FINAL_BATCH_DIRECTORY}/nested/escape.wav"
        ),
        "one atomic v2 directory",
    )
    add_case(
        "commercial_marker_destination",
        lambda value: value["entries"][0].__setitem__(
            "destination_path",
            f"{batch_contract.FINAL_BATCH_DIRECTORY}/commercial_reference.wav",
        ),
        "commercial/reference marker",
    )
    add_case(
        "format_rate_retuned",
        lambda value: value["format_acceptance_contract"].__setitem__(
            "sample_rate_hz_inclusive", [44_100, 192_000]
        ),
        "sample_rate_hz_inclusive",
    )
    add_case(
        "redirect_enabled",
        lambda value: value["network_contract"].__setitem__("redirects_allowed", True),
        "batch-v2 exact preregistration",
    )
    add_case(
        "retry_enabled",
        lambda value: value["network_contract"].__setitem__(
            "automatic_retries_allowed", True
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "access_log_reuses_v1_path",
        lambda value: value["filesystem_contract"].__setitem__(
            "access_log_path", batch_contract.PREDECESSOR_ACCESS_LOG_PATH
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "partial_survivors_enabled",
        lambda value: value["filesystem_contract"].__setitem__(
            "partial_survivors_allowed", True
        ),
        "batch-v2 exact preregistration",
    )
    add_case(
        "result_injection",
        lambda value: value["entries"][0].__setitem__("gate_result", {"passed": True}),
        "source-result fields",
    )
    add_case(
        "predecessor_payload_injected_into_entry",
        lambda value: value["entries"][0].__setitem__(
            "payload_sha256", batch_contract.FORBIDDEN_OBSERVED_PAYLOAD_SHA256[0]
        ),
        "source-result fields",
    )
    add_case(
        "self_raw_hash_cycle",
        lambda value: value.__setitem__(
            "batch_raw_sha256", validator.EXPECTED_BATCH_RAW_SHA256
        ),
        "own raw or semantic",
    )
    add_case(
        "registry_v3_forward_pin",
        lambda value: value.__setitem__(
            "future_registry", "riotbox.source_holdout_rotation.v3"
        ),
        "forward Registry-v3",
    )
    cases.extend(
        [
            (
                "duplicate_json_key",
                lambda: validator._reject_duplicate_keys(  # noqa: SLF001
                    [("schema", "first"), ("schema", "second")]
                ),
                "duplicate object key",
            ),
            (
                "raw_pin",
                lambda: validator.validate_pins(
                    "0" * 64, validator.EXPECTED_BATCH_SEMANTIC_SHA256
                ),
                "batch-v2 raw SHA-256",
            ),
            (
                "semantic_pin",
                lambda: validator.validate_pins(
                    validator.EXPECTED_BATCH_RAW_SHA256, "0" * 64
                ),
                "batch-v2 semantic SHA-256",
            ),
            (
                "rejection_report_tamper",
                _tampered_repository_operation(
                    batch_contract.PREDECESSOR_REJECTION_REPORT_REL,
                    lambda payload: payload + b"\nforged\n",
                ),
                "predecessor rejection report raw SHA-256",
            ),
            (
                "predecessor_batch_tamper",
                _tampered_repository_operation(
                    batch_contract.PREDECESSOR_BATCH_REL,
                    lambda payload: payload.replace(
                        b'"human_review_claimed": false',
                        b'"human_review_claimed": true ',
                        1,
                    ),
                ),
                "predecessor batch-v1 raw SHA-256",
            ),
        ]
    )

    for name, operation, token in cases:
        expect_fail(name, operation, token)
    print(f"PASS: {len(cases)} fail-closed acquisition-batch-v2 mutations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
