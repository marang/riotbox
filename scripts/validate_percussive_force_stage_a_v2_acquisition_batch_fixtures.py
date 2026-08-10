#!/usr/bin/env python3
"""Fail-closed mutation fixtures for the Stage-A-v2 acquisition batch."""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any, Callable

import percussive_force_stage_a_v2_acquisition as acquisition
import percussive_force_stage_a_v2_acquisition_contract as batch_contract
import validate_percussive_force_stage_a_v2_acquisition_batch as validator


REPO_ROOT = Path(__file__).resolve().parents[1]


def expect_fail(name: str, operation: Callable[[], None], token: str) -> None:
    try:
        operation()
    except (validator.ContractError, acquisition.AcquisitionError) as error:
        if token not in str(error):
            raise AssertionError(
                f"{name}: wrong failure {error!s}; expected token {token!r}"
            ) from error
        return
    raise AssertionError(f"{name}: mutation unexpectedly validated")


def main() -> int:
    frozen = validator.validate_repository(REPO_ROOT)
    document = frozen.document
    if (
        frozen.raw_sha256 != validator.EXPECTED_BATCH_RAW_SHA256
        or frozen.semantic_sha256 != validator.EXPECTED_BATCH_SEMANTIC_SHA256
    ):
        raise AssertionError("validated batch does not expose its frozen pins")
    try:
        validator.FrozenAcquisitionBatchV1(raw_sha256=frozen.raw_sha256)
    except TypeError as error:
        if "must be created by validate_repository" not in str(error):
            raise AssertionError(f"sealed constructor raised the wrong error: {error}") from error
    else:
        raise AssertionError("FrozenAcquisitionBatchV1 constructor is forgeable")
    forged = object.__new__(validator.FrozenAcquisitionBatchV1)
    expect_fail(
        "forged_binding_without_root",
        lambda: validator.revalidate_frozen_batch(forged),
        "no repository root",
    )
    forged_payload = validator.FrozenAcquisitionBatchV1._from_validated(  # noqa: SLF001
        REPO_ROOT,
        b'{"schema":"forged"}\n',
    )
    if validator.revalidate_frozen_batch(forged_payload).raw_sha256 != frozen.raw_sha256:
        raise AssertionError("repository revalidation trusted a forged batch payload")

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
        "protocol_binding.raw_sha256",
    )
    add_case(
        "registry_semantic_pin",
        lambda value: value["predecessor_registry_binding"].__setitem__(
            "semantic_sha256", "0" * 64
        ),
        "predecessor_registry_binding.semantic_sha256",
    )
    add_case(
        "holdout_union_reopened",
        lambda value: value["predecessor_registry_binding"].__setitem__(
            "holdout_union_state", "opened"
        ),
        "holdout_union_state",
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
        "author_collapse",
        lambda value: value["entries"][1].__setitem__(
            "author", value["entries"][0]["author"]
        ),
        "authors must be distinct",
    )
    add_case(
        "cinameng_carryover",
        lambda value: value["entries"][0].__setitem__("author", "Cinameng"),
        "different from Cinameng",
    )
    add_case(
        "known_case_collision",
        lambda value: value["entries"][0].__setitem__(
            "case_id", "oga_cinameng_can_be_so_beautiful"
        ),
        "Registry v2 metadata",
    )
    add_case(
        "known_pack_collision",
        lambda value: value["entries"][0].__setitem__(
            "source_pack_id", "oga_can_be_so_beautiful"
        ),
        "Registry v2 metadata",
    )
    add_case(
        "known_holdout_page_collision",
        lambda value: value["entries"][0].__setitem__(
            "page_url", "https://opengameart.org/content/get-equipped-8-bit-drum-loop"
        ),
        "Registry v2 metadata",
    )
    add_case(
        "duplicate_batch_download",
        lambda value: value["entries"][1].__setitem__(
            "download_url", value["entries"][0]["download_url"]
        ),
        "attachment_filename",
    )
    add_case(
        "duplicate_batch_destination",
        lambda value: value["entries"][1].__setitem__(
            "destination_path", value["entries"][0]["destination_path"]
        ),
        "duplicates another batch identity",
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
            "destination_path",
            f"{batch_contract.FINAL_BATCH_DIRECTORY}/nested/escape.wav",
        ),
        "one atomic final batch directory",
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
        "wrong_provider",
        lambda value: value["entries"][0].__setitem__("provider", "Other"),
        ".provider",
    )
    add_case(
        "wrong_license",
        lambda value: value["entries"][0].__setitem__("license", "CC-BY-SA-3.0"),
        ".license",
    )
    add_case(
        "third_party_disclosure",
        lambda value: value["entries"][0].__setitem__(
            "third_party_source_or_sample_pack_disclosed_on_page", True
        ),
        "third_party_source_or_sample_pack",
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
        "attachment_filename_url_mismatch",
        lambda value: value["entries"][0].__setitem__(
            "attachment_filename", "different.wav"
        ),
        "URL basename",
    )
    add_case(
        "page_wrong_host",
        lambda value: value["entries"][0].__setitem__(
            "page_url", "https://example.org/content/pirates-incoming"
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
        "family_promoted_without_qualification",
        lambda value: value["entries"][0].__setitem__(
            "family_assignment_state", "human_confirmed"
        ),
        "family_assignment_state",
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
        "batch exact preregistration",
    )
    add_case(
        "retry_enabled",
        lambda value: value["network_contract"].__setitem__(
            "automatic_retries_allowed", True
        ),
        "batch exact preregistration",
    )
    add_case(
        "partial_survivors_enabled",
        lambda value: value["filesystem_contract"].__setitem__(
            "partial_survivors_allowed", True
        ),
        "batch exact preregistration",
    )
    add_case(
        "result_injection",
        lambda value: value.__setitem__("gate_result", {"passed": True}),
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
                "batch raw SHA-256",
            ),
            (
                "semantic_pin",
                lambda: validator.validate_pins(
                    validator.EXPECTED_BATCH_RAW_SHA256, "0" * 64
                ),
                "batch semantic SHA-256",
            ),
        ]
    )

    for name, operation, token in cases:
        expect_fail(name, operation, token)
    print(f"PASS: {len(cases)} fail-closed acquisition-batch mutations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
