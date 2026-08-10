#!/usr/bin/env python3
"""Build the source-blind RIOTBOX-1430 Stage-A-v2 acquisition batch v2."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from percussive_force_stage_a_v2_acquisition import FORMAT_CONTRACT


BATCH_REL = Path(
    "docs/benchmarks/percussive_force_stage_a_v2_acquisition_batch_v2.json"
)
PROTOCOL_V2_REL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
REGISTRY_V2_REL = Path("docs/benchmarks/source_holdout_rotation_v2.json")
PREDECESSOR_BATCH_REL = Path(
    "docs/benchmarks/percussive_force_stage_a_v2_acquisition_batch_v1.json"
)
PREDECESSOR_REJECTION_REPORT_REL = Path(
    "docs/reviews/riotbox_1430_stage_a_v2_acquisition_batch_v1_rejection_2026-08-10.md"
)
PROTOCOL_V2_RAW_SHA256 = (
    "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
)
PROTOCOL_V2_SEMANTIC_SHA256 = (
    "6f8db5d1488168c11bbd13be6c8862b2ae9b70424ce9e3e4887fd87d311b74fb"
)
REGISTRY_V2_RAW_SHA256 = (
    "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812"
)
REGISTRY_V2_SEMANTIC_SHA256 = (
    "6cfe11cd10a5947427a09335fbd4795706c71530b6f6a7e5b9883259bcca8ce1"
)
PREDECESSOR_BATCH_RAW_SHA256 = (
    "ada49dc778bebe201c399413122765fce08d4476c445af30c2a1982bd524e6c9"
)
PREDECESSOR_BATCH_SEMANTIC_SHA256 = (
    "7c103a12b743e8c9406d66008527c0036dac472c78bee5512ee46beb7492b362"
)
PREDECESSOR_REJECTION_REPORT_RAW_SHA256 = (
    "2ab9d34888d5ba0a442a408f2e10e9f201fdbfa6291ffdd09d003908c93da619"
)
PREDECESSOR_ATTEMPT_ID = "1e6a1070-5df4-4813-8e07-dc30fae7c70a"
PREDECESSOR_ACCESS_LOG_PATH = (
    "artifacts/audio_qa/riotbox-1430/stage-a-v2-acquisition-access-v1.json"
)
PREDECESSOR_ACCESS_LOG_RAW_SHA256 = (
    "703806c0f6548f1af2e2f51408553e51f060ea51f4f47202079d63145540c174"
)
FORBIDDEN_OBSERVED_PAYLOAD_SHA256 = (
    "00d1ec0b442db60ade056fe24a72c18cc0f8deed23301f5ec961029f3eb810f9",
    "2212c182906ae1b7449e26c31b4c96f132c348a33fdd82c0b00f785f7a677e5f",
)
SOURCE_ROOT = "data/test_audio/external/RIOTBOX-1423/wav"
QUARANTINE_DIRECTORY = (
    "artifacts/audio_qa/riotbox-1430/stage-a-v2-acquisition-batch-v2.incomplete"
)
FINAL_BATCH_DIRECTORY = (
    "data/test_audio/external/RIOTBOX-1423/wav/"
    "riotbox-1430-stage-a-v2-acquisition-batch-v2"
)
ACCESS_LOG_PATH = (
    "artifacts/audio_qa/riotbox-1430/stage-a-v2-acquisition-access-v2.json"
)
SEALED_MANIFEST_NAME = "stage-a-v2-acquisition-sealed-manifest-v2.json"


def semantic_sha256(document: Any) -> str:
    canonical = json.dumps(
        document,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
    return hashlib.sha256(canonical).hexdigest()


def entry(
    *,
    ordinal: int,
    case_id: str,
    source_family: str,
    author: str,
    title: str,
    source_pack_id: str,
    page_url: str,
    published_on: str,
    download_url: str,
    attachment_filename: str,
    download_url_filename: str,
    attachment_byte_count: int,
    provider_attachment_id: int,
    destination_filename: str,
    metadata_family_basis: str,
) -> dict[str, Any]:
    return {
        "ordinal": ordinal,
        "case_id": case_id,
        "source_family": source_family,
        "family_assignment_state": "provisional_metadata_hypothesis_only",
        "metadata_family_basis": metadata_family_basis,
        "source_qualification_state": "unheard_uncomputed_pending_riotbox_1428",
        "author": author,
        "author_profile_url": f"https://opengameart.org/users/{author.casefold()}",
        "title": title,
        "source_pack_id": source_pack_id,
        "provider": "OpenGameArt",
        "page_url": page_url,
        "provider_published_on": published_on,
        "page_metadata_observed_on": "2026-08-10",
        "download_url": download_url,
        "attachment_filename": attachment_filename,
        "download_url_filename": download_url_filename,
        "attachment_mime_type": "audio/x-wav",
        "attachment_byte_count": attachment_byte_count,
        "provider_attachment_id": provider_attachment_id,
        "attachment_identity_evidence": (
            "exact_display_filename_direct_URL_MIME_byte_count_and_file_id_from_"
            "the_named_OpenGameArt_content_page_only"
        ),
        "license": "CC0-1.0",
        "license_evidence": "named_OpenGameArt_content_page_declares_CC0",
        "third_party_source_or_sample_pack_disclosed_on_page": False,
        "third_party_disclosure_interpretation": (
            "false_means_not_disclosed_on_the_named_page_not_proof_of_no_third_party_material"
        ),
        "partition": "development",
        "source_start_seconds": 0.0,
        "destination_path": f"{FINAL_BATCH_DIRECTORY}/{destination_filename}",
        "source_derivation": {
            "decoder": "none_identity_wav",
            "sample_rate_policy": "preserve_original",
            "sample_payload_transform": "none",
        },
        "commercial_reference": False,
    }


def build_document() -> dict[str, Any]:
    entries = [
        entry(
            ordinal=1,
            case_id="oga_farfadet46_loopable_beat_ludumdare",
            source_family="dense_break",
            author="farfadet46",
            title="Loopable beat for ludumdare game",
            source_pack_id="oga_loopable_beat_for_ludumdare_game",
            page_url=(
                "https://opengameart.org/content/loopable-beat-for-ludumdare-game"
            ),
            published_on="2014-08-26",
            download_url="https://opengameart.org/sites/default/files/test.wav",
            attachment_filename="test.wav",
            download_url_filename="test.wav",
            attachment_byte_count=1_695_934,
            provider_attachment_id=45_609,
            destination_filename="dense_oga_farfadet46_loopable_beat_ludumdare.wav",
            metadata_family_basis=(
                "page_title_identifies_a_loopable_beat; dense_break_role_separation_groove_"
                "and_positive_family_fitness_remain_unclaimed_until_qualification"
            ),
        ),
        entry(
            ordinal=2,
            case_id="oga_celestialghost8_cc0_scraps_slowdrum",
            source_family="sparse_drums",
            author="celestialghost8",
            title="CC0 Scraps",
            source_pack_id="oga_cc0_scraps",
            page_url="https://opengameart.org/content/cc0-scraps",
            published_on="2017-03-12",
            download_url=(
                "https://opengameart.org/sites/default/files/"
                "slowdrum%20-%20Track%2002%20%28New%20song%29.wav"
            ),
            attachment_filename="slowdrum - Track 02 (New song).wav",
            download_url_filename="slowdrum - Track 02 (New song).wav",
            attachment_byte_count=580_400,
            provider_attachment_id=93_984,
            destination_filename="sparse_oga_celestialghost8_cc0_scraps_slowdrum.wav",
            metadata_family_basis=(
                "attachment_display_filename_alone_contains_slowdrum; the_page_does_not_claim_"
                "looping_at_least_three_onsets_or_drums_only_content_and_sparse_event_separation_"
                "and_positive_family_fitness_remain_unclaimed_until_qualification"
            ),
        ),
        entry(
            ordinal=3,
            case_id="oga_cosmac_8_bit_disco_loop",
            source_family="electronic_drums",
            author="cosmac",
            title="8 Bit Disco Loop",
            source_pack_id="oga_8_bit_disco_loop",
            page_url="https://opengameart.org/content/8-bit-disco-loop",
            published_on="2025-03-27",
            download_url="https://opengameart.org/sites/default/files/title_1.wav",
            attachment_filename="title.wav",
            download_url_filename="title_1.wav",
            attachment_byte_count=676_908,
            provider_attachment_id=239_763,
            destination_filename="electronic_oga_cosmac_8_bit_disco_loop.wav",
            metadata_family_basis=(
                "page_title_identifies_an_8_bit_disco_loop; programmed_drum_identity_"
                "separable_events_and_positive_family_fitness_remain_unclaimed_until_qualification"
            ),
        ),
    ]
    return {
        "schema": "riotbox.percussive_force_stage_a_v2_acquisition_batch.v2",
        "schema_version": 2,
        "owner_ticket": "RIOTBOX-1430",
        "batch_id": "riotbox_1430_stage_a_v2_registry_v3_retry_batch_2",
        "work_class": "contract_enabler",
        "directly_enabled_followup": {
            "ticket": "RIOTBOX-1428",
            "outcome": (
                "Fresh development-only StageAQualificationSession v2 and, only after admission, "
                "the frozen three-family by four-source by two-event matrix."
            ),
        },
        "evidence_role": "network_acquisition_preregistration_only",
        "quality_proof": False,
        "source_qualification_claimed": False,
        "human_review_claimed": False,
        "protocol_binding": {
            "path": PROTOCOL_V2_REL.as_posix(),
            "schema": "riotbox.percussive_force_stage_a_protocol.v2",
            "raw_sha256": PROTOCOL_V2_RAW_SHA256,
            "semantic_sha256": PROTOCOL_V2_SEMANTIC_SHA256,
        },
        "predecessor_registry_binding": {
            "path": REGISTRY_V2_REL.as_posix(),
            "schema": "riotbox.source_holdout_rotation.v2",
            "raw_sha256": REGISTRY_V2_RAW_SHA256,
            "semantic_sha256": REGISTRY_V2_SEMANTIC_SHA256,
            "holdout_union_state": "immutable_unopened",
        },
        "predecessor_acquisition_rejection_binding": {
            "batch_path": PREDECESSOR_BATCH_REL.as_posix(),
            "batch_schema": "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1",
            "batch_raw_sha256": PREDECESSOR_BATCH_RAW_SHA256,
            "batch_semantic_sha256": PREDECESSOR_BATCH_SEMANTIC_SHA256,
            "rejection_report_path": PREDECESSOR_REJECTION_REPORT_REL.as_posix(),
            "rejection_report_raw_sha256": PREDECESSOR_REJECTION_REPORT_RAW_SHA256,
            "attempt_id": PREDECESSOR_ATTEMPT_ID,
            "access_log_path": PREDECESSOR_ACCESS_LOG_PATH,
            "access_log_raw_sha256": PREDECESSOR_ACCESS_LOG_RAW_SHA256,
            "attempt_status": "rejected",
            "rejection_stage": "request_2_header",
            "request_count": 2,
            "successful_request_count": 1,
            "forbidden_observed_payload_sha256": list(
                FORBIDDEN_OBSERVED_PAYLOAD_SHA256
            ),
            "batch_v1_retry_allowed": False,
            "survivor_reuse_allowed": False,
            "payload_reuse_allowed": False,
        },
        "metadata_basis": {
            "observed_on": "2026-08-10",
            "scope": "named_OpenGameArt_content_pages_and_author_links_only",
            "page_metadata_access_performed": True,
            "direct_attachment_request_performed_for_batch_v2": False,
            "attachment_body_bytes_accessed_for_batch_v2": False,
            "predecessor_payload_identity_used_only_as_forbidden_history": True,
            "source_directory_discovery_performed": False,
            "source_audio_playback_performed": False,
            "commercial_reference_access_performed": False,
            "holdout_audio_access_performed": False,
        },
        "license_policy": {
            "required_license": "CC0-1.0",
            "provider": "OpenGameArt",
            "provider_page_declaration_is_required": True,
            "known_third_party_source_or_sample_pack_disclosure_allowed": False,
            "false_disclosure_value_is_proof_of_absence": False,
            "redistribution_from_riotbox_repo": False,
        },
        "family_hypothesis_policy": {
            "required_exact_families": [
                "dense_break",
                "sparse_drums",
                "electronic_drums",
            ],
            "metadata_never_grants_family_fitness": True,
            "qualification_owner": "RIOTBOX-1428_StageAQualificationSession_v2",
            "failure": "typed_source_or_family_rejection_no_substitution_no_fallback",
        },
        "format_acceptance_contract": dict(FORMAT_CONTRACT),
        "network_contract": {
            "method": "GET",
            "exact_request_order": "entries_in_ascending_ordinal_order",
            "maximum_batch_attempts": 1,
            "head_or_probe_requests_allowed": False,
            "redirects_allowed": False,
            "automatic_retries_allowed": False,
            "proxy_environment_allowed": False,
            "cookies_auth_or_netrc_allowed": False,
            "automatic_content_decoding_allowed": False,
            "required_response_status": 200,
            "required_content_length": "exact_preregistered_attachment_byte_count",
            "transfer_encoding_allowed": False,
            "content_encoding_allowed": ["absent", "identity"],
            "body_read_cap": "attachment_byte_count_plus_one",
            "directory_discovery_wildcards_or_url_substitution_allowed": False,
        },
        "filesystem_contract": {
            "source_root": SOURCE_ROOT,
            "quarantine_directory": QUARANTINE_DIRECTORY,
            "final_batch_directory": FINAL_BATCH_DIRECTORY,
            "access_log_path": ACCESS_LOG_PATH,
            "sealed_manifest_name": SEALED_MANIFEST_NAME,
            "quarantine_and_final_parent_same_filesystem_required": True,
            "publication": (
                "one_atomic_renameat2_RENAME_NOREPLACE_of_the_complete_sealed_batch_directory"
            ),
            "individual_final_file_renames_allowed": False,
            "overwrite_allowed": False,
            "partial_survivors_allowed": False,
            "directory_listing_or_glob_allowed": False,
        },
        "execution_contract": {
            "exact_entry_count": 3,
            "exact_successful_request_count": 3,
            "one_exclusive_durable_attempt_log_before_network": True,
            "revalidate_all_pins_before_each_request_and_publication": True,
            "hash_raw_attachment_bytes": True,
            "header_validation_only_before_registry_v3": True,
            "audio_decode_or_pcm_sample_iteration_allowed": False,
            "source_feature_or_event_computation_allowed": False,
            "candidate_or_control_rendering_allowed": False,
            "source_preview_or_playback_allowed": False,
            "holdout_or_commercial_reference_access_allowed": False,
            "failure": (
                "reject_entire_batch_no_further_request_no_publication_"
                "new_versioned_metadata_decision_required"
            ),
        },
        "change_control": {
            "predecessor_failure_requires_complete_replacement_batch": True,
            "identity_disjointness": (
                "Registry_v2_and_batch_v1_author_case_pack_page_download_"
                "attachment_id_and_attachment_name"
            ),
            "single_rule": (
                "Any batch identity, order, author, family, page, URL, attachment byte count, path, "
                "license, format, access, or publication change after this v2 validates requires "
                "riotbox.percussive_force_stage_a_v2_acquisition_batch.v3 plus a Decision-Log entry "
                "before another network request."
            ),
            "result_driven_patch_allowed": False,
            "sequential_survivor_selection_allowed": False,
            "predecessor_identity_or_payload_reuse_allowed": False,
        },
        "entries": entries,
    }


def render(document: dict[str, Any]) -> bytes:
    return (
        json.dumps(document, indent=2, ensure_ascii=False, allow_nan=False) + "\n"
    ).encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    document = build_document()
    payload = render(document)
    output = repo / BATCH_REL
    if args.check:
        if not output.exists() or output.read_bytes() != payload:
            raise ValueError(f"{BATCH_REL} is missing or drifted")
    else:
        output.write_bytes(payload)
    print(f"acquisition_batch_v2_raw_sha256={hashlib.sha256(payload).hexdigest()}")
    print(f"acquisition_batch_v2_semantic_sha256={semantic_sha256(document)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
