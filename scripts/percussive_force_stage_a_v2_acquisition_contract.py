#!/usr/bin/env python3
"""Build the metadata-only RIOTBOX-1430 Stage-A-v2 acquisition batch."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from percussive_force_stage_a_v2_acquisition import FORMAT_CONTRACT


BATCH_REL = Path(
    "docs/benchmarks/percussive_force_stage_a_v2_acquisition_batch_v1.json"
)
PROTOCOL_V2_REL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
REGISTRY_V2_REL = Path("docs/benchmarks/source_holdout_rotation_v2.json")
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
SOURCE_ROOT = "data/test_audio/external/RIOTBOX-1423/wav"
QUARANTINE_DIRECTORY = (
    "artifacts/audio_qa/riotbox-1430/stage-a-v2-acquisition-batch-v1.incomplete"
)
FINAL_BATCH_DIRECTORY = (
    "data/test_audio/external/RIOTBOX-1423/wav/"
    "riotbox-1430-stage-a-v2-acquisition-batch-v1"
)
ACCESS_LOG_PATH = (
    "artifacts/audio_qa/riotbox-1430/stage-a-v2-acquisition-access-v1.json"
)
SEALED_MANIFEST_NAME = "stage-a-v2-acquisition-sealed-manifest-v1.json"


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
    author_profile_url: str,
    title: str,
    source_pack_id: str,
    page_url: str,
    published_on: str,
    download_url: str,
    attachment_filename: str,
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
        "author_profile_url": author_profile_url,
        "title": title,
        "source_pack_id": source_pack_id,
        "provider": "OpenGameArt",
        "page_url": page_url,
        "provider_published_on": published_on,
        "page_metadata_observed_on": "2026-08-10",
        "download_url": download_url,
        "attachment_filename": attachment_filename,
        "attachment_mime_type": "audio/x-wav",
        "attachment_byte_count": attachment_byte_count,
        "provider_attachment_id": provider_attachment_id,
        "attachment_identity_evidence": (
            "exact_filename_mime_byte_count_and_file_id_from_the_named_OpenGameArt_content_page_only"
        ),
        "license": "CC0-1.0",
        "license_evidence": "named_OpenGameArt_content_page_declares_CC0",
        "third_party_source_or_sample_pack_disclosed_on_page": False,
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
            case_id="oga_eldritch_grim_pirates_incoming",
            source_family="dense_break",
            author="Eldritch Grim",
            author_profile_url="https://opengameart.org/users/eldritch-grim",
            title="Pirates Incoming",
            source_pack_id="oga_pirates_incoming",
            page_url="https://opengameart.org/content/pirates-incoming",
            published_on="2024-04-01",
            download_url=(
                "https://opengameart.org/sites/default/files/"
                "pirates_incoming_loop.wav"
            ),
            attachment_filename="pirates_incoming_loop.wav",
            attachment_byte_count=2_048_078,
            provider_attachment_id=224_760,
            destination_filename="dense_oga_eldritch_grim_pirates_incoming.wav",
            metadata_family_basis=(
                "page_calls_it_a_simple_epic_hard_drum_loop_and_tags_it_loop_epic_massive_hard_drums; "
                "distinct_roles_groove_and_positive_family_fitness_remain_unclaimed_until_qualification"
            ),
        ),
        entry(
            ordinal=2,
            case_id="oga_turnovus_simple_drumbeat_start",
            source_family="sparse_drums",
            author="Turnovus",
            author_profile_url="https://opengameart.org/users/turnovus",
            title="simple drumbeat - separate intro",
            source_pack_id="oga_simple_drumbeat",
            page_url="https://opengameart.org/content/simple-drumbeat",
            published_on="2016-08-20",
            download_url=(
                "https://opengameart.org/sites/default/files/"
                "drumbeat_start_wav.wav"
            ),
            attachment_filename="drumbeat_start_wav.wav",
            attachment_byte_count=3_629_150,
            provider_attachment_id=84_777,
            destination_filename="sparse_oga_turnovus_simple_drumbeat_start.wav",
            metadata_family_basis=(
                "page_calls_the_pack_a_very_simple_drumbeat_and_exposes_separate_intro_and_body_files; "
                "the_exact_start_attachment_is_an_intro_and_sparse_fitness_remains_unclaimed_until_qualification"
            ),
        ),
        entry(
            ordinal=3,
            case_id="oga_fupi_tense_bass_boost_drums",
            source_family="electronic_drums",
            author="Fupi",
            author_profile_url="https://opengameart.org/users/fupi",
            title="Tense Bass Boost Drum Loop 140 BPM",
            source_pack_id="oga_tense_bass_boost_drum_loop_140_bpm",
            page_url=(
                "https://opengameart.org/content/"
                "tense-bass-boost-drum-loop-140-bpm"
            ),
            published_on="2021-01-11",
            download_url=(
                "https://opengameart.org/sites/default/files/"
                "tensebassboostdrums.wav"
            ),
            attachment_filename="tensebassboostdrums.wav",
            attachment_byte_count=1_212_898,
            provider_attachment_id=170_631,
            destination_filename="electronic_oga_fupi_tense_bass_boost_drums.wav",
            metadata_family_basis=(
                "page_calls_it_a_drum_pattern_and_tags_it_edm_dance_drums_cymbals_and_140bpm; "
                "programmed_identity_separable_events_and_positive_family_fitness_remain_unclaimed_until_qualification"
            ),
        ),
    ]
    return {
        "schema": "riotbox.percussive_force_stage_a_v2_acquisition_batch.v1",
        "schema_version": 1,
        "owner_ticket": "RIOTBOX-1430",
        "batch_id": "riotbox_1430_stage_a_v2_registry_v3_retry_batch_1",
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
        "metadata_basis": {
            "observed_on": "2026-08-10",
            "scope": "named_OpenGameArt_content_pages_and_author_profiles_only",
            "page_metadata_access_performed": True,
            "direct_attachment_request_performed": False,
            "attachment_body_bytes_accessed": False,
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
            "publication": "one_atomic_renameat2_RENAME_NOREPLACE_of_the_complete_sealed_batch_directory",
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
            "failure": "reject_entire_batch_no_further_request_no_publication_new_versioned_metadata_decision_required",
        },
        "change_control": {
            "single_rule": (
                "Any batch identity, order, author, family, page, URL, attachment byte count, path, "
                "license, format, access, or publication change after this v1 validates requires "
                "riotbox.percussive_force_stage_a_v2_acquisition_batch.v2 plus a Decision-Log entry "
                "before another network request."
            ),
            "result_driven_patch_allowed": False,
            "sequential_survivor_selection_allowed": False,
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
    print(f"acquisition_batch_v1_raw_sha256={hashlib.sha256(payload).hexdigest()}")
    print(f"acquisition_batch_v1_semantic_sha256={semantic_sha256(document)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
