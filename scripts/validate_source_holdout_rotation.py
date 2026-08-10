#!/usr/bin/env python3
"""Validate the P023 legal source-corpus and rotating-holdout contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import subprocess
import sys
import wave
from datetime import date
from pathlib import Path
from typing import Any, Callable
from urllib.parse import urlparse

from source_holdout_development_access import (
    LEGACY_SOURCE_FORMAT,
    STAGE_A_REGISTRY_RAW_SHA256,
    V2_SOURCE_FORMATS,
    SourceIdentity,
    load_pinned_stage_a_registry,
    run_development_access_session,
    validate_source_format,
    validate_wav_file,
)


SCHEMA_V1 = "riotbox.source_holdout_rotation.v1"
SCHEMA_V2 = "riotbox.source_holdout_rotation.v2"
V1_OWNER_TICKET = "RIOTBOX-1423"
V1_FOLLOWUP_TICKET = "RIOTBOX-1422"
V2_OWNER_TICKET = "RIOTBOX-1428"
V2_FOLLOWUP_TICKET = "RIOTBOX-1428"
V2_FOLLOWUP_OUTCOME = (
    "Exact audible F1-F3 source-backed render plus bounded human directional comparison."
)
V2_PREDECESSOR_PATH = Path("docs/benchmarks/source_holdout_rotation_v1.json")
V2_PREDECESSOR_SHA256 = "dd017080f311dcb2a8eda2fac63d8da372a356f0fc2cc33d5c97d3fd2ea34cfc"
V2_REVIEW_RECORD = Path(
    "docs/reviews/riotbox_1428_stage_a_source_pre_admission_2026-08-10.md"
)
V2_ADMITTED_CASE_IDS = (
    "oga_william_hector_horde_war_drums",
    "oga_frosty_ham_osdrums",
)
STAGE_A_DEVELOPMENT_CASE_IDS = (
    "oga_cinameng_can_be_so_beautiful",
    "oga_marwan_cinematic_percussion",
    "oga_william_hector_horde_war_drums",
    "oga_frosty_ham_osdrums",
)
STAGE_A_REGISTRY_PATH = Path("docs/benchmarks/source_holdout_rotation_v2.json")
SOURCE_ROOT = Path("data/test_audio/external/RIOTBOX-1423/wav")
V1_CORE_FAMILIES = {
    "bad_timing",
    "dense_break",
    "pad_noise",
    "sparse_drums",
    "tonal_riff",
    "weak_source",
}
V2_CORE_FAMILIES = V1_CORE_FAMILIES | {"electronic_drums"}
STRESS_FAMILIES = {"dense_full_mix"}
MINIMUMS = {
    "eligible_source_count": 12,
    "eligible_source_pack_count": 12,
    "candidate_matrix_source_count": 5,
    "candidate_matrix_family_count": 4,
    "holdout_source_count": 2,
    "holdout_family_count": 2,
    "distinct_author_count": 10,
}
PARTITIONS = {"development", "holdout_a", "holdout_b", "retired"}
SHA256 = re.compile(r"^[0-9a-f]{64}$")
SAFE_ID = re.compile(r"^[a-z0-9][a-z0-9_]*$")
TICKET = re.compile(r"^RIOTBOX-[0-9]+$")
REFERENCE_PATH_MARKERS = {
    ".download-examples",
    "the prodigy",
    "tidal",
    "commercial-reference",
    "commercial_reference",
}
V2_ADMISSION_IDENTITIES = {
    "oga_william_hector_horde_war_drums": {
        "source_family": "dense_break",
        "source_path": (
            "data/test_audio/external/RIOTBOX-1423/wav/"
            "sparse_oga_william_hector_horde_war_drums.wav"
        ),
        "sha256": "a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417",
    },
    "oga_frosty_ham_osdrums": {
        "source_family": "electronic_drums",
        "source_path": (
            "data/test_audio/external/RIOTBOX-1423/wav/"
            "sparse_oga_frosty_ham_osdrums.wav"
        ),
        "sha256": "7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2",
    },
}
V2_SOURCE_DERIVATIONS = {
    "oga_william_hector_horde_war_drums": {
        "original_url": (
            "https://opengameart.org/sites/default/files/"
            "horde_war_drums_by_william_hector.wav"
        ),
        "original_sha256": (
            "a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417"
        ),
        "derived_sha256": (
            "a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417"
        ),
        "decoded_format": "pcm_s24le_wav",
        "decoder": "none_identity_wav",
        "sample_rate_policy": "preserve_original",
    },
    "oga_frosty_ham_osdrums": {
        "original_url": "https://opengameart.org/sites/default/files/osdrums.ogg",
        "original_sha256": (
            "297121379bd972c775bb18d66c4077c28db7ad3da935f9dc43768da55ebc0e40"
        ),
        "derived_sha256": (
            "7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2"
        ),
        "decoded_format": "pcm_s16le_wav",
        "decoder": "ffmpeg",
        "sample_rate_policy": "preserve_original",
    },
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--require-existing-source-files", action="store_true")
    args = parser.parse_args()

    try:
        manifest = read_json_object(args.manifest)
        summary = validate_manifest(
            manifest,
            args.manifest,
            require_existing_source_files=args.require_existing_source_files,
        )
    except (OSError, TypeError, ValueError, json.JSONDecodeError, wave.Error) as error:
        print(f"invalid source holdout rotation: {error}", file=sys.stderr)
        return 1

    print(
        "valid source holdout rotation: "
        f"{args.manifest} "
        f"({summary['eligible_source_count']} eligible sources, "
        f"{summary['eligible_source_pack_count']} source packs, "
        f"{summary['candidate_family_count']} development families, "
        f"{summary['holdout_count']} holdout sets)"
    )
    return 0


def schema_contract(manifest: dict[str, Any], prefix: str) -> dict[str, Any]:
    schema = manifest.get("schema")
    if schema == SCHEMA_V1:
        contract = {
            "schema": SCHEMA_V1,
            "schema_version": 1,
            "owner_ticket": V1_OWNER_TICKET,
            "followup_ticket": V1_FOLLOWUP_TICKET,
            "core_families": V1_CORE_FAMILIES,
        }
    elif schema == SCHEMA_V2:
        contract = {
            "schema": SCHEMA_V2,
            "schema_version": 2,
            "owner_ticket": V2_OWNER_TICKET,
            "followup_ticket": V2_FOLLOWUP_TICKET,
            "core_families": V2_CORE_FAMILIES,
        }
    else:
        raise ValueError(
            f"{prefix}: schema must be {SCHEMA_V1} or {SCHEMA_V2}"
        )
    require(
        manifest.get("schema_version") == contract["schema_version"],
        f"{prefix}: schema_version must be {contract['schema_version']}",
    )
    return contract


def validate_manifest(
    manifest: dict[str, Any],
    manifest_path: Path,
    *,
    require_existing_source_files: bool,
    repo: Path | None = None,
) -> dict[str, int]:
    prefix = str(manifest_path)
    contract = schema_contract(manifest, prefix)
    schema = str(contract["schema"])
    require(
        not (schema == SCHEMA_V2 and require_existing_source_files),
        f"{prefix}: schema v2 forbids full local-file validation during Stage A; "
        "use explicit development-only verification",
    )
    require(
        manifest.get("owner_ticket") == contract["owner_ticket"],
        f"{prefix}: owner_ticket must be {contract['owner_ticket']}",
    )
    followup = object_field(manifest, "directly_enabled_followup", prefix)
    require(
        followup.get("ticket") == contract["followup_ticket"],
        f"{prefix}: directly_enabled_followup.ticket must be {contract['followup_ticket']}",
    )
    non_empty_string(followup.get("outcome"), f"{prefix}: followup outcome")
    if schema == SCHEMA_V2:
        require(
            followup.get("outcome") == V2_FOLLOWUP_OUTCOME,
            f"{prefix}: schema v2 followup outcome must name the exact audible F1-F3 "
            "render and bounded human directional comparison",
        )
    require(
        manifest.get("evidence_role") == "source_corpus_and_holdout_contract",
        f"{prefix}: invalid evidence_role",
    )
    require(manifest.get("quality_proof") is False, f"{prefix}: quality_proof must be false")
    require(
        manifest.get("source_audio_presence") == "local_ignored",
        f"{prefix}: source_audio_presence must be local_ignored",
    )
    require(
        manifest.get("source_root") == SOURCE_ROOT.as_posix(),
        f"{prefix}: source_root must be {SOURCE_ROOT}",
    )
    require(
        manifest.get("commercial_reference_material_allowed") is False,
        f"{prefix}: commercial reference material must be forbidden",
    )

    license_policy = object_field(manifest, "license_policy", prefix)
    require(
        license_policy.get("required_license") == "CC0-1.0",
        f"{prefix}: required license must be CC0-1.0",
    )
    require(
        license_policy.get("provider") == "OpenGameArt",
        f"{prefix}: provider must be OpenGameArt",
    )
    require(
        license_policy.get("redistribution_from_riotbox_repo") is False,
        f"{prefix}: source redistribution must remain false",
    )

    core_families = set(string_list(manifest, "required_core_families", prefix))
    stress_families = set(string_list(manifest, "stress_families", prefix))
    require(
        core_families == contract["core_families"],
        f"{prefix}: required_core_families must be exactly "
        f"{sorted(contract['core_families'])}",
    )
    require(
        stress_families == STRESS_FAMILIES,
        f"{prefix}: stress_families must be exactly {sorted(STRESS_FAMILIES)}",
    )
    validate_family_contracts(manifest, prefix, schema=schema)
    if schema == SCHEMA_V2:
        require(
            manifest.get("source_format_default") == LEGACY_SOURCE_FORMAT,
            f"{prefix}: source_format_default must preserve the v1 48 kHz PCM16 contract",
        )

    minimums = object_field(manifest, "minimums", prefix)
    for field, floor in MINIMUMS.items():
        value = minimums.get(field)
        require(
            isinstance(value, int) and not isinstance(value, bool) and value >= floor,
            f"{prefix}: minimums.{field} must be an integer >= {floor}",
        )

    entries = list_field(manifest, "entries", prefix)
    require(entries, f"{prefix}: entries must not be empty")
    validated_entries: dict[str, dict[str, Any]] = {}
    seen_hashes: set[str] = set()
    resolved_repo = repo
    if require_existing_source_files and resolved_repo is None:
        resolved_repo = repo_root()

    for index, raw_entry in enumerate(entries):
        require(isinstance(raw_entry, dict), f"{prefix}: entries[{index}] must be an object")
        entry = raw_entry
        case_id = validate_entry(
            entry,
            f"{prefix}: entries[{index}]",
            schema=schema,
            core_families=contract["core_families"],
            require_existing_source_files=require_existing_source_files,
            repo=resolved_repo,
        )
        require(case_id not in validated_entries, f"{prefix}: duplicate case_id {case_id}")
        source_hash = str(entry["sha256"])
        require(source_hash not in seen_hashes, f"{prefix}: duplicate source sha256 {source_hash}")
        validated_entries[case_id] = entry
        seen_hashes.add(source_hash)

    eligible = [entry for entry in entries if entry["corpus_eligible"]]
    eligible_families = {str(entry["source_family"]) for entry in eligible}
    eligible_packs = {str(entry["source_pack_id"]) for entry in eligible}
    authors = {str(entry["author"]).strip().casefold() for entry in entries}
    require(
        len(eligible) >= minimums["eligible_source_count"],
        f"{prefix}: eligible source count collapsed below {minimums['eligible_source_count']}",
    )
    require(
        len(eligible_packs) >= minimums["eligible_source_pack_count"],
        f"{prefix}: eligible source-pack count collapsed below "
        f"{minimums['eligible_source_pack_count']}",
    )
    require(
        eligible_families == contract["core_families"],
        f"{prefix}: eligible core-family coverage must be exactly "
        f"{sorted(contract['core_families'])}",
    )
    require(
        len(authors) >= minimums["distinct_author_count"],
        f"{prefix}: distinct author count collapsed below {minimums['distinct_author_count']}",
    )

    candidate_ids = validate_candidate_matrix(
        manifest,
        validated_entries,
        minimums,
        prefix,
    )
    holdout_membership = validate_holdout_sets(
        manifest,
        validated_entries,
        minimums,
        prefix,
    )
    validate_partitions(validated_entries, candidate_ids, holdout_membership, prefix)
    validate_rotation_history(
        manifest,
        validated_entries,
        holdout_membership,
        prefix,
    )
    validate_pack_independence(entries, candidate_ids, holdout_membership, prefix)

    if schema == SCHEMA_V2:
        validate_v2_transition(manifest, prefix)

    candidate_families = {
        str(validated_entries[case_id]["source_family"])
        for case_id in candidate_ids
        if validated_entries[case_id]["corpus_eligible"]
    }
    return {
        "eligible_source_count": len(eligible),
        "eligible_source_pack_count": len(eligible_packs),
        "candidate_family_count": len(candidate_families),
        "holdout_count": len(holdout_membership),
    }


def validate_family_contracts(
    manifest: dict[str, Any], prefix: str, *, schema: str
) -> None:
    contracts = object_field(manifest, "family_contracts", prefix)
    dense_break = object_field(contracts, "dense_break", f"{prefix}: family_contracts")
    require(
        dense_break.get("positive_coverage") is True,
        f"{prefix}: dense_break must be positive coverage",
    )
    non_empty_string(dense_break.get("required_character"), f"{prefix}: dense_break character")
    non_empty_string(dense_break.get("not_sufficient"), f"{prefix}: dense_break exclusion")

    dense_full_mix = object_field(
        contracts,
        "dense_full_mix",
        f"{prefix}: family_contracts",
    )
    require(
        dense_full_mix.get("positive_coverage") is False,
        f"{prefix}: dense_full_mix must not count as positive coverage",
    )
    non_empty_string(dense_full_mix.get("purpose"), f"{prefix}: dense_full_mix purpose")

    weak = object_field(contracts, "weak_source", f"{prefix}: family_contracts")
    require(weak.get("positive_coverage") is True, f"{prefix}: weak_source coverage")
    require(
        "raw source level" in non_empty_string(
            weak.get("source_truth"),
            f"{prefix}: weak_source source_truth",
        ).lower(),
        f"{prefix}: weak_source must preserve raw source-level truth",
    )

    bad_timing = object_field(contracts, "bad_timing", f"{prefix}: family_contracts")
    require(bad_timing.get("positive_coverage") is True, f"{prefix}: bad_timing coverage")
    outcomes = set(string_list(bad_timing, "allowed_outcomes", f"{prefix}: bad_timing"))
    require(
        outcomes == {"transformed", "degraded", "reject"},
        f"{prefix}: bad_timing allowed outcomes changed",
    )
    if schema == SCHEMA_V2:
        electronic = object_field(
            contracts,
            "electronic_drums",
            f"{prefix}: family_contracts",
        )
        require(
            electronic.get("positive_coverage") is True,
            f"{prefix}: electronic_drums must be positive coverage",
        )
        non_empty_string(
            electronic.get("required_character"),
            f"{prefix}: electronic_drums required_character",
        )
        non_empty_string(
            electronic.get("not_sufficient"),
            f"{prefix}: electronic_drums not_sufficient",
        )


def validate_entry(
    entry: dict[str, Any],
    prefix: str,
    *,
    schema: str,
    core_families: set[str],
    require_existing_source_files: bool,
    repo: Path | None,
) -> str:
    case_id = non_empty_string(entry.get("case_id"), f"{prefix}.case_id")
    require(SAFE_ID.fullmatch(case_id) is not None, f"{prefix}.case_id is not safe")
    family = non_empty_string(entry.get("source_family"), f"{prefix}.source_family")
    require(
        family in core_families | STRESS_FAMILIES,
        f"{prefix}.source_family is unsupported: {family}",
    )
    eligible = entry.get("corpus_eligible")
    require(isinstance(eligible, bool), f"{prefix}.corpus_eligible must be boolean")
    require(
        eligible is (family in core_families),
        f"{prefix}: only core-family entries may count as corpus eligible",
    )

    partition = non_empty_string(entry.get("partition"), f"{prefix}.partition")
    require(partition in PARTITIONS, f"{prefix}.partition is unsupported: {partition}")
    classification = non_empty_string(
        entry.get("classification_status"),
        f"{prefix}.classification_status",
    )
    verdict = non_empty_string(
        entry.get("source_suitability_verdict"),
        f"{prefix}.source_suitability_verdict",
    )
    if partition.startswith("holdout_"):
        require(
            classification == "provisional_unheard_holdout",
            f"{prefix}: active holdouts must remain provisional and unheard",
        )
        require(
            verdict == "unverified",
            f"{prefix}: active holdout suitability must be unverified",
        )
        for review_field in ("reviewed_on", "reviewer_role", "family_verdict_owner"):
            require(
                review_field not in entry,
                f"{prefix}: active holdout must not carry {review_field}",
            )
    elif partition == "development" and eligible:
        require(
            classification == "human_confirmed",
            f"{prefix}: eligible development sources must be human_confirmed",
        )
        require(
            verdict == "usable",
            f"{prefix}: eligible development sources must be marked usable",
        )
        if schema == SCHEMA_V2 and case_id in V2_ADMITTED_CASE_IDS:
            require(
                entry.get("source_suitability_verdict_owner") == "human_review",
                f"{prefix}.source_suitability_verdict_owner must be human_review",
            )
            validate_review_provenance(entry, prefix, family_owner="technical_review")
        else:
            validate_review_provenance(entry, prefix, family_owner="human_review")
    elif partition == "development":
        require(
            classification == "confirmed_stress",
            f"{prefix}: development stress sources must be confirmed_stress",
        )
        require(
            verdict == "usable_as_stress",
            f"{prefix}: stress source suitability must be usable_as_stress",
        )
        validate_review_provenance(entry, prefix, family_owner="technical_review")
    elif partition == "retired":
        require(
            classification == "consumed_holdout",
            f"{prefix}: retired source must be a consumed_holdout",
        )
        require(
            verdict in {"usable", "weak", "reject"},
            f"{prefix}: invalid retired source suitability",
        )
        validate_review_provenance(entry, prefix, family_owner=None)

    non_empty_string(entry.get("author"), f"{prefix}.author")
    non_empty_string(entry.get("title"), f"{prefix}.title")
    pack_id = non_empty_string(entry.get("source_pack_id"), f"{prefix}.source_pack_id")
    require(SAFE_ID.fullmatch(pack_id) is not None, f"{prefix}.source_pack_id is not safe")
    require(entry.get("provider") == "OpenGameArt", f"{prefix}.provider must be OpenGameArt")
    validate_url(entry.get("page_url"), f"{prefix}.page_url")
    validate_url(entry.get("download_url"), f"{prefix}.download_url")
    require(entry.get("license") == "CC0-1.0", f"{prefix}.license must be CC0-1.0")
    require(
        entry.get("commercial_reference") is False,
        f"{prefix}: commercial/reference-only material is forbidden",
    )
    source_start = entry.get("source_start_seconds")
    require(
        isinstance(source_start, int | float)
        and not isinstance(source_start, bool)
        and math.isfinite(source_start)
        and source_start >= 0,
        f"{prefix}.source_start_seconds must be a non-negative number",
    )

    source_path = validate_source_path(entry.get("source_path"), prefix)
    source_hash = non_empty_string(entry.get("sha256"), f"{prefix}.sha256")
    require(SHA256.fullmatch(source_hash) is not None, f"{prefix}.sha256 must be lowercase SHA-256")
    source_format = resolve_source_format(entry, prefix, schema=schema)
    if schema == SCHEMA_V2 and case_id in V2_ADMITTED_CASE_IDS:
        validate_source_derivation(entry, prefix, source_hash)
    archive_member = entry.get("archive_member")
    if archive_member is not None:
        member = Path(non_empty_string(archive_member, f"{prefix}.archive_member"))
        require(
            not member.is_absolute() and ".." not in member.parts,
            f"{prefix}.archive_member must be safe and relative",
        )

    previous_partition = entry.get("previous_partition")
    if previous_partition is not None:
        require(
            previous_partition in {"holdout_a", "holdout_b"},
            f"{prefix}.previous_partition must name a holdout set",
        )
        require(
            partition in {"development", "retired"},
            f"{prefix}: only consumed sources may carry previous_partition",
        )
    replacement_for = entry.get("replacement_for_case_id")
    acquired_on = entry.get("acquired_on")
    require(
        (replacement_for is None) is (acquired_on is None),
        f"{prefix}: replacement_for_case_id and acquired_on must appear together",
    )
    if replacement_for is not None:
        require(
            SAFE_ID.fullmatch(
                non_empty_string(replacement_for, f"{prefix}.replacement_for_case_id")
            )
            is not None,
            f"{prefix}.replacement_for_case_id is not safe",
        )
        parse_iso_date(acquired_on, f"{prefix}.acquired_on")

    if require_existing_source_files:
        require(repo is not None, f"{prefix}: repository root unavailable")
        validate_wav_file(
            repo / source_path,
            source_hash,
            source_format,
            prefix,
        )
    return case_id


def resolve_source_format(
    entry: dict[str, Any], prefix: str, *, schema: str
) -> dict[str, Any]:
    case_id = str(entry["case_id"])
    raw_override = entry.get("source_format")
    if schema == SCHEMA_V1:
        require(
            raw_override is None,
            f"{prefix}: schema v1 entries must use the implicit 48 kHz PCM16 format",
        )
        return dict(LEGACY_SOURCE_FORMAT)

    if case_id in V2_ADMITTED_CASE_IDS:
        require(
            isinstance(raw_override, dict),
            f"{prefix}.source_format must be an object for a v2 admission",
        )
        source_format = raw_override
        require(
            source_format == V2_SOURCE_FORMATS[case_id],
            f"{prefix}.source_format does not match the registered original-rate format",
        )
    else:
        require(
            raw_override is None,
            f"{prefix}: inherited v1 entries must retain the implicit default format",
        )
        source_format = LEGACY_SOURCE_FORMAT
    validate_source_format(source_format, f"{prefix}.source_format")
    return dict(source_format)


def validate_source_derivation(
    entry: dict[str, Any], prefix: str, derived_sha256: str
) -> None:
    case_id = str(entry["case_id"])
    derivation = object_field(entry, "source_derivation", prefix)
    validate_url(derivation.get("original_url"), f"{prefix}.source_derivation.original_url")
    original_hash = non_empty_string(
        derivation.get("original_sha256"),
        f"{prefix}.source_derivation.original_sha256",
    )
    require(
        SHA256.fullmatch(original_hash) is not None,
        f"{prefix}.source_derivation.original_sha256 must be lowercase SHA-256",
    )
    require(
        derivation.get("derived_sha256") == derived_sha256,
        f"{prefix}.source_derivation.derived_sha256 must match the registered source",
    )
    non_empty_string(
        derivation.get("decoded_format"),
        f"{prefix}.source_derivation.decoded_format",
    )
    decoder = non_empty_string(
        derivation.get("decoder"),
        f"{prefix}.source_derivation.decoder",
    )
    require(
        decoder in {"none_identity_wav", "ffmpeg"},
        f"{prefix}.source_derivation.decoder is unsupported",
    )
    require(
        derivation.get("sample_rate_policy") == "preserve_original",
        f"{prefix}.source_derivation.sample_rate_policy must preserve_original",
    )
    if decoder == "none_identity_wav":
        require(
            original_hash == derived_sha256,
            f"{prefix}: identity WAV derivation must preserve the original SHA-256",
        )
    require(
        derivation == V2_SOURCE_DERIVATIONS[case_id],
        f"{prefix}.source_derivation does not match the frozen v2 provenance",
    )


def validate_candidate_matrix(
    manifest: dict[str, Any],
    entries: dict[str, dict[str, Any]],
    minimums: dict[str, Any],
    prefix: str,
) -> set[str]:
    matrix = object_field(manifest, "candidate_matrix", prefix)
    non_empty_string(matrix.get("purpose"), f"{prefix}: candidate_matrix.purpose")
    ids = unique_string_set(matrix, "source_case_ids", f"{prefix}: candidate_matrix")
    require(ids <= entries.keys(), f"{prefix}: candidate matrix references unknown cases")
    eligible = [entries[case_id] for case_id in ids if entries[case_id]["corpus_eligible"]]
    families = {str(entry["source_family"]) for entry in eligible}
    require(
        len(eligible) >= minimums["candidate_matrix_source_count"],
        f"{prefix}: candidate matrix needs at least "
        f"{minimums['candidate_matrix_source_count']} eligible sources",
    )
    require(
        len(families) >= minimums["candidate_matrix_family_count"],
        f"{prefix}: candidate matrix family collapse: needs at least "
        f"{minimums['candidate_matrix_family_count']} core families",
    )
    require(
        "dense_break" in families,
        f"{prefix}: candidate matrix needs a confirmed development dense_break",
    )
    return ids


def validate_holdout_sets(
    manifest: dict[str, Any],
    entries: dict[str, dict[str, Any]],
    minimums: dict[str, Any],
    prefix: str,
) -> dict[str, set[str]]:
    holdouts = list_field(manifest, "holdout_sets", prefix)
    require(len(holdouts) >= 2, f"{prefix}: at least two rotating holdout sets are required")
    membership: dict[str, set[str]] = {}
    seen_cases: set[str] = set()
    states: set[str] = set()
    for index, raw_holdout in enumerate(holdouts):
        require(isinstance(raw_holdout, dict), f"{prefix}: holdout_sets[{index}] must be object")
        holdout = raw_holdout
        holdout_id = non_empty_string(
            holdout.get("holdout_id"),
            f"{prefix}: holdout_sets[{index}].holdout_id",
        )
        require(SAFE_ID.fullmatch(holdout_id) is not None, f"{prefix}: unsafe holdout_id")
        require(holdout_id not in membership, f"{prefix}: duplicate holdout_id {holdout_id}")
        state = non_empty_string(
            holdout.get("state"),
            f"{prefix}: holdout_sets[{index}].state",
        )
        require(state in {"unseen", "reserve"}, f"{prefix}: invalid holdout state {state}")
        states.add(state)
        non_empty_string(holdout.get("purpose"), f"{prefix}: {holdout_id}.purpose")
        ids = unique_string_set(holdout, "source_case_ids", f"{prefix}: {holdout_id}")
        require(ids <= entries.keys(), f"{prefix}: {holdout_id} references unknown cases")
        require(not (ids & seen_cases), f"{prefix}: source reused across holdout sets")
        require(
            len(ids) >= minimums["holdout_source_count"],
            f"{prefix}: {holdout_id} has insufficient holdout sources",
        )
        families = {str(entries[case_id]["source_family"]) for case_id in ids}
        require(
            len(families) >= minimums["holdout_family_count"],
            f"{prefix}: {holdout_id} has insufficient different-family holdouts",
        )
        require(
            all(entries[case_id]["corpus_eligible"] for case_id in ids),
            f"{prefix}: stress-only material cannot count as a holdout",
        )
        membership[holdout_id] = ids
        seen_cases.update(ids)
    require(
        {"unseen", "reserve"} <= states,
        f"{prefix}: holdout rotation needs both unseen and reserve states",
    )
    return membership


def validate_partitions(
    entries: dict[str, dict[str, Any]],
    candidate_ids: set[str],
    holdout_membership: dict[str, set[str]],
    prefix: str,
) -> None:
    all_holdout_ids = set().union(*holdout_membership.values())
    require(
        not (candidate_ids & all_holdout_ids),
        f"{prefix}: development and holdout sources must be disjoint",
    )
    for case_id, entry in entries.items():
        partition = str(entry["partition"])
        if partition == "development":
            require(
                case_id in candidate_ids,
                f"{prefix}: development case missing from candidate matrix: {case_id}",
            )
        elif partition.startswith("holdout_"):
            require(
                partition in holdout_membership
                and case_id in holdout_membership[partition],
                f"{prefix}: holdout partition mismatch for {case_id}",
            )
        else:
            require(
                case_id not in candidate_ids and case_id not in all_holdout_ids,
                f"{prefix}: retired case cannot remain active: {case_id}",
            )


def load_canonical_predecessor_snapshot() -> tuple[dict[str, Any], Path, str]:
    path = repo_root() / V2_PREDECESSOR_PATH
    payload = path.read_bytes()
    actual_hash = hashlib.sha256(payload).hexdigest()
    value = json.loads(payload, object_pairs_hook=reject_duplicate_object_keys)
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value, V2_PREDECESSOR_PATH, actual_hash


def validate_v2_transition(
    manifest: dict[str, Any],
    prefix: str,
) -> None:
    predecessor = object_field(manifest, "predecessor", prefix)
    require(
        set(predecessor) == {"path", "schema", "sha256"},
        f"{prefix}: predecessor must contain only path, schema, and sha256",
    )
    require(
        predecessor.get("path") == V2_PREDECESSOR_PATH.as_posix(),
        f"{prefix}: wrong predecessor path",
    )
    require(
        predecessor.get("schema") == SCHEMA_V1,
        f"{prefix}: predecessor schema must be {SCHEMA_V1}",
    )
    require(
        predecessor.get("sha256") == V2_PREDECESSOR_SHA256,
        f"{prefix}: wrong predecessor SHA-256",
    )
    (
        predecessor_manifest,
        predecessor_manifest_path,
        predecessor_manifest_sha256,
    ) = load_canonical_predecessor_snapshot()
    validate_manifest(
        predecessor_manifest,
        predecessor_manifest_path,
        require_existing_source_files=False,
    )
    require(
        predecessor_manifest_path == V2_PREDECESSOR_PATH
        and predecessor_manifest.get("schema") == SCHEMA_V1,
        f"{prefix}: loaded predecessor is not schema v1",
    )
    require(
        predecessor_manifest_sha256 == V2_PREDECESSOR_SHA256
        and predecessor.get("sha256") == predecessor_manifest_sha256,
        f"{prefix}: canonical predecessor bytes do not match the frozen SHA-256",
    )

    previous_entries = {
        str(entry["case_id"]): entry for entry in predecessor_manifest["entries"]
    }
    current_entries = {str(entry["case_id"]): entry for entry in manifest["entries"]}
    validate_active_holdout_tuple_freeze(
        predecessor_manifest,
        manifest,
        previous_entries,
        current_entries,
        prefix,
    )

    previous_ids = set(previous_entries)
    current_ids = set(current_entries)
    allowed_additions = set(V2_ADMITTED_CASE_IDS)
    require(
        current_ids == previous_ids | allowed_additions,
        f"{prefix}: v2 entries must contain exactly the two allowed admissions",
    )
    for case_id, previous_entry in previous_entries.items():
        require(
            current_entries[case_id] == previous_entry,
            f"{prefix}: inherited entry changed: {case_id}",
        )

    for case_id in V2_ADMITTED_CASE_IDS:
        entry = current_entries[case_id]
        expected_identity = V2_ADMISSION_IDENTITIES[case_id]
        for field, expected in expected_identity.items():
            require(
                entry.get(field) == expected,
                f"{prefix}: {case_id}.{field} changed from the preregistered identity",
            )
        require(
            entry.get("partition") == "development"
            and entry.get("classification_status") == "human_confirmed"
            and entry.get("source_suitability_verdict") == "usable",
            f"{prefix}: {case_id} must be an admitted human-confirmed development source",
        )
        require(
            entry.get("reviewed_on") == "2026-08-10"
            and entry.get("reviewer_role") == "project_musician"
            and entry.get("source_suitability_verdict_owner") == "human_review"
            and entry.get("family_verdict_owner") == "technical_review",
            f"{prefix}: {case_id} must carry the bounded pre-admission review provenance",
        )

    previous_candidate_ids = predecessor_manifest["candidate_matrix"]["source_case_ids"]
    require(
        manifest["candidate_matrix"]["source_case_ids"]
        == previous_candidate_ids + list(V2_ADMITTED_CASE_IDS),
        f"{prefix}: candidate matrix must append exactly the two admitted cases",
    )
    require(
        manifest["holdout_sets"] == predecessor_manifest["holdout_sets"],
        f"{prefix}: active holdout sets changed during development admission",
    )
    require(
        manifest["rotation_history"] == predecessor_manifest["rotation_history"],
        f"{prefix}: rotation history changed during development admission",
    )
    for field in (
        "evidence_role",
        "quality_proof",
        "source_audio_presence",
        "source_root",
        "commercial_reference_material_allowed",
        "license_policy",
        "stress_families",
        "minimums",
    ):
        require(
            manifest[field] == predecessor_manifest[field],
            f"{prefix}: inherited top-level contract changed: {field}",
        )
    for family, previous_contract in predecessor_manifest["family_contracts"].items():
        require(
            manifest["family_contracts"].get(family) == previous_contract,
            f"{prefix}: inherited family contract changed: {family}",
        )
    require(
        set(manifest["required_core_families"])
        == set(predecessor_manifest["required_core_families"]) | {"electronic_drums"},
        f"{prefix}: v2 must add only electronic_drums to the core families",
    )

    admission = object_field(manifest, "admission_transition", prefix)
    require(
        admission.get("ticket") == V2_OWNER_TICKET,
        f"{prefix}: admission_transition.ticket must be {V2_OWNER_TICKET}",
    )
    require(
        admission.get("review_record") == V2_REVIEW_RECORD.as_posix(),
        f"{prefix}: admission_transition.review_record is not canonical",
    )
    require(
        admission.get("admitted_case_ids") == list(V2_ADMITTED_CASE_IDS),
        f"{prefix}: admission_transition.admitted_case_ids changed",
    )
    require(
        admission.get("inherited_entry_policy")
        == "json_value_unchanged",
        f"{prefix}: inherited-entry policy changed",
    )
    require(
        admission.get("active_holdout_tuple_policy")
        == "holdout_id_case_id_source_path_sha256_strings_unchanged",
        f"{prefix}: active-holdout tuple policy changed",
    )
    require(
        admission.get("holdout_audio_access")
        == "metadata_only_no_audio_read_hash_render_classify_or_play",
        f"{prefix}: holdout-audio access policy changed",
    )
    require(
        admission.get("quality_proof") is False,
        f"{prefix}: source admission must not claim quality proof",
    )


def validate_active_holdout_tuple_freeze(
    predecessor: dict[str, Any],
    current: dict[str, Any],
    predecessor_entries: dict[str, dict[str, Any]],
    current_entries: dict[str, dict[str, Any]],
    prefix: str,
) -> None:
    previous_tuples = active_holdout_tuples(predecessor, predecessor_entries)
    current_tuples = active_holdout_tuples(current, current_entries)
    require(
        current_tuples == previous_tuples,
        f"{prefix}: active holdout ID/path/SHA-256 tuple changed",
    )


def active_holdout_tuples(
    manifest: dict[str, Any], entries: dict[str, dict[str, Any]]
) -> set[tuple[str, str, str, str]]:
    tuples: set[tuple[str, str, str, str]] = set()
    for holdout in manifest["holdout_sets"]:
        holdout_id = str(holdout["holdout_id"])
        for case_id in holdout["source_case_ids"]:
            entry = entries[str(case_id)]
            tuples.add(
                (
                    holdout_id,
                    str(case_id),
                    str(entry["source_path"]),
                    str(entry["sha256"]),
                )
            )
    return tuples


def validate_rotation_history(
    manifest: dict[str, Any],
    entries: dict[str, dict[str, Any]],
    holdout_membership: dict[str, set[str]],
    prefix: str,
) -> None:
    history = manifest.get("rotation_history")
    require(isinstance(history, list), f"{prefix}: rotation_history must be an array")
    active_holdout_ids = set().union(*holdout_membership.values())
    seen_consumed: set[str] = set()
    seen_replacements: set[str] = set()
    for index, raw_event in enumerate(history):
        require(isinstance(raw_event, dict), f"{prefix}: rotation_history[{index}] must be object")
        event = raw_event
        case_id = non_empty_string(event.get("case_id"), f"{prefix}: consumed case_id")
        require(case_id in entries, f"{prefix}: consumed holdout case is unknown: {case_id}")
        require(case_id not in seen_consumed, f"{prefix}: holdout consumed more than once: {case_id}")
        require(
            case_id not in active_holdout_ids,
            f"{prefix}: consumed holdout cannot remain unseen/reserve: {case_id}",
        )
        former = non_empty_string(
            event.get("former_holdout_id"),
            f"{prefix}: former_holdout_id",
        )
        require(former in holdout_membership, f"{prefix}: unknown former holdout {former}")
        require(
            entries[case_id].get("previous_partition") == former,
            f"{prefix}: consumed source must record previous_partition {former}",
        )
        ticket = non_empty_string(event.get("consumed_by_ticket"), f"{prefix}: consumed ticket")
        require(TICKET.fullmatch(ticket) is not None, f"{prefix}: invalid consumed ticket")
        consumed_on = parse_iso_date(event.get("consumed_on"), f"{prefix}: consumed_on")
        replacement = non_empty_string(
            event.get("replacement_case_id"),
            f"{prefix}: replacement_case_id",
        )
        require(
            replacement in holdout_membership[former],
            f"{prefix}: consumed holdout needs a replacement in {former}",
        )
        require(replacement != case_id, f"{prefix}: holdout cannot replace itself")
        require(
            replacement not in seen_replacements,
            f"{prefix}: replacement source reused across rotation events: {replacement}",
        )
        replacement_entry = entries[replacement]
        require(
            replacement_entry.get("replacement_for_case_id") == case_id,
            f"{prefix}: replacement must identify consumed case {case_id}",
        )
        acquired_on = parse_iso_date(
            replacement_entry.get("acquired_on"),
            f"{prefix}: replacement acquired_on",
        )
        require(
            acquired_on >= consumed_on,
            f"{prefix}: replacement must be acquired on or after consumption",
        )
        seen_consumed.add(case_id)
        seen_replacements.add(replacement)

    entries_with_history = {
        case_id
        for case_id, entry in entries.items()
        if entry.get("previous_partition") is not None
    }
    require(
        entries_with_history == seen_consumed,
        f"{prefix}: previous_partition and rotation_history must describe the same cases",
    )
    entries_marked_replacement = {
        case_id
        for case_id, entry in entries.items()
        if entry.get("replacement_for_case_id") is not None
    }
    require(
        entries_marked_replacement == seen_replacements,
        f"{prefix}: replacement metadata and rotation_history must describe the same cases",
    )


def validate_pack_independence(
    entries: list[Any],
    candidate_ids: set[str],
    holdout_membership: dict[str, set[str]],
    prefix: str,
) -> None:
    by_id = {str(entry["case_id"]): entry for entry in entries}
    for set_name, case_ids in [
        ("candidate_matrix", candidate_ids),
        *holdout_membership.items(),
    ]:
        pack_ids = [str(by_id[case_id]["source_pack_id"]) for case_id in case_ids]
        require(
            len(pack_ids) == len(set(pack_ids)),
            f"{prefix}: narrow source pack repeats inside {set_name}",
        )


def verify_development_source_files(
    manifest: dict[str, Any],
    manifest_path: Path,
    requested_case_ids: list[str],
    access_log_path: Path,
    *,
    qualification_owner_id: str,
    qualification_owner: Callable[[SourceIdentity, bytes, dict[str, Any]], None],
    repo: Path | None = None,
) -> dict[str, Any]:
    prefix = str(manifest_path)
    require(
        manifest_path == STAGE_A_REGISTRY_PATH,
        f"{prefix}: Stage A requires the exact frozen registry path "
        f"{STAGE_A_REGISTRY_PATH}",
    )
    registry, pinned_manifest = load_pinned_stage_a_registry(
        manifest_path,
        manifest,
    )
    require(
        registry.raw_sha256 == STAGE_A_REGISTRY_RAW_SHA256,
        f"{prefix}: Stage-A registry raw SHA-256 pin changed",
    )
    validate_manifest(
        pinned_manifest,
        manifest_path,
        require_existing_source_files=False,
    )
    require(
        pinned_manifest.get("schema") == SCHEMA_V2,
        f"{prefix}: development-only access logging requires schema v2",
    )
    validate_stage_a_development_request(requested_case_ids, prefix)
    identities = []
    for entry in pinned_manifest["entries"]:
        case_id = str(entry["case_id"])
        source_path = validate_source_path(entry["source_path"], case_id)
        identities.append(
            SourceIdentity(
                case_id=case_id,
                source_path=source_path.as_posix(),
                expected_sha256=str(entry["sha256"]),
                partition=str(entry["partition"]),
                source_format=resolve_source_format(entry, case_id, schema=SCHEMA_V2),
            )
        )
    return run_development_access_session(
        identities,
        requested_case_ids,
        repo=repo if repo is not None else repo_root(),
        registry=registry,
        access_log_path=access_log_path,
        qualification_owner_id=qualification_owner_id,
        qualification_owner=qualification_owner,
    )


def validate_stage_a_development_request(
    requested_case_ids: list[str],
    prefix: str,
) -> None:
    require(
        tuple(requested_case_ids) == STAGE_A_DEVELOPMENT_CASE_IDS,
        f"{prefix}: Stage-A qualification owner requires exactly the four frozen "
        "development cases in canonical order",
    )


def validate_url(value: Any, message: str) -> None:
    url = non_empty_string(value, message)
    parsed = urlparse(url)
    host = (parsed.hostname or "").lower()
    require(parsed.scheme == "https", f"{message} must use https")
    require(
        host == "opengameart.org" or host.endswith(".opengameart.org"),
        f"{message} must point to OpenGameArt",
    )


def validate_source_path(value: Any, prefix: str) -> Path:
    source_path = Path(non_empty_string(value, f"{prefix}.source_path"))
    lowered = source_path.as_posix().casefold()
    require(
        not source_path.is_absolute() and ".." not in source_path.parts,
        f"{prefix}.source_path must be safe and repo-relative",
    )
    require(
        SOURCE_ROOT == source_path.parent,
        f"{prefix}.source_path must be a direct child of {SOURCE_ROOT}",
    )
    require(source_path.suffix.lower() == ".wav", f"{prefix}.source_path must be WAV")
    require(
        not any(marker in lowered for marker in REFERENCE_PATH_MARKERS),
        f"{prefix}.source_path leaks commercial/reference-only material",
    )
    return source_path


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(
        path.read_text(),
        object_pairs_hook=reject_duplicate_object_keys,
    )
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def reject_duplicate_object_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, child in pairs:
        require(key not in value, f"duplicate JSON object key: {key!r}")
        value[key] = child
    return value


def object_field(data: dict[str, Any], field: str, prefix: str) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{prefix}: {field} must be an object")
    return value


def list_field(data: dict[str, Any], field: str, prefix: str) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{prefix}: {field} must be an array")
    return value


def string_list(data: dict[str, Any], field: str, prefix: str) -> list[str]:
    values = list_field(data, field, prefix)
    require(
        all(isinstance(value, str) and value.strip() for value in values),
        f"{prefix}: {field} must contain non-empty strings",
    )
    return values


def unique_string_set(data: dict[str, Any], field: str, prefix: str) -> set[str]:
    values = string_list(data, field, prefix)
    require(len(values) == len(set(values)), f"{prefix}: {field} contains duplicates")
    return set(values)


def non_empty_string(value: Any, message: str) -> str:
    require(isinstance(value, str) and value.strip(), f"{message} must be non-empty string")
    return str(value)


def validate_review_provenance(
    entry: dict[str, Any],
    prefix: str,
    *,
    family_owner: str | None,
) -> None:
    parse_iso_date(entry.get("reviewed_on"), f"{prefix}.reviewed_on")
    require(
        entry.get("reviewer_role") == "project_musician",
        f"{prefix}.reviewer_role must be project_musician",
    )
    owner = non_empty_string(
        entry.get("family_verdict_owner"),
        f"{prefix}.family_verdict_owner",
    )
    if family_owner is not None:
        require(owner == family_owner, f"{prefix}.family_verdict_owner must be {family_owner}")
    else:
        require(
            owner in {"human_review", "technical_review"},
            f"{prefix}.family_verdict_owner is unsupported",
        )


def parse_iso_date(value: Any, message: str) -> date:
    raw = non_empty_string(value, message)
    try:
        return date.fromisoformat(raw)
    except ValueError as error:
        raise ValueError(f"{message} must be an ISO date") from error


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return Path(result.stdout.strip())


if __name__ == "__main__":
    raise SystemExit(main())
