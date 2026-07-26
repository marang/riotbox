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
from array import array
from datetime import date
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


SCHEMA = "riotbox.source_holdout_rotation.v1"
OWNER_TICKET = "RIOTBOX-1423"
FOLLOWUP_TICKET = "RIOTBOX-1422"
SOURCE_ROOT = Path("data/test_audio/external/RIOTBOX-1423/wav")
CORE_FAMILIES = {
    "bad_timing",
    "dense_break",
    "pad_noise",
    "sparse_drums",
    "tonal_riff",
    "weak_source",
}
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


def validate_manifest(
    manifest: dict[str, Any],
    manifest_path: Path,
    *,
    require_existing_source_files: bool,
    repo: Path | None = None,
) -> dict[str, int]:
    prefix = str(manifest_path)
    require(manifest.get("schema") == SCHEMA, f"{prefix}: schema must be {SCHEMA}")
    require(manifest.get("schema_version") == 1, f"{prefix}: schema_version must be 1")
    require(
        manifest.get("owner_ticket") == OWNER_TICKET,
        f"{prefix}: owner_ticket must be {OWNER_TICKET}",
    )
    followup = object_field(manifest, "directly_enabled_followup", prefix)
    require(
        followup.get("ticket") == FOLLOWUP_TICKET,
        f"{prefix}: directly_enabled_followup.ticket must be {FOLLOWUP_TICKET}",
    )
    non_empty_string(followup.get("outcome"), f"{prefix}: followup outcome")
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
        core_families == CORE_FAMILIES,
        f"{prefix}: required_core_families must be exactly {sorted(CORE_FAMILIES)}",
    )
    require(
        stress_families == STRESS_FAMILIES,
        f"{prefix}: stress_families must be exactly {sorted(STRESS_FAMILIES)}",
    )
    validate_family_contracts(manifest, prefix)

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
            require_existing_source_files=require_existing_source_files,
            repo=resolved_repo,
        )
        require(case_id not in validated_entries, f"{prefix}: duplicate case_id {case_id}")
        source_hash = str(entry["sha256"])
        require(source_hash not in seen_hashes, f"{prefix}: duplicate source sha256 {source_hash}")
        validated_entries[case_id] = entry
        seen_hashes.add(source_hash)

    active_entries = [entry for entry in entries if entry["partition"] != "retired"]
    eligible = [entry for entry in active_entries if entry["corpus_eligible"]]
    eligible_families = {str(entry["source_family"]) for entry in eligible}
    eligible_packs = {str(entry["source_pack_id"]) for entry in eligible}
    authors = {str(entry["author"]).strip().casefold() for entry in active_entries}
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
        eligible_families == CORE_FAMILIES,
        f"{prefix}: eligible core-family coverage must be exactly {sorted(CORE_FAMILIES)}",
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


def validate_family_contracts(manifest: dict[str, Any], prefix: str) -> None:
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


def validate_entry(
    entry: dict[str, Any],
    prefix: str,
    *,
    require_existing_source_files: bool,
    repo: Path | None,
) -> str:
    case_id = non_empty_string(entry.get("case_id"), f"{prefix}.case_id")
    require(SAFE_ID.fullmatch(case_id) is not None, f"{prefix}.case_id is not safe")
    family = non_empty_string(entry.get("source_family"), f"{prefix}.source_family")
    require(
        family in CORE_FAMILIES | STRESS_FAMILIES,
        f"{prefix}.source_family is unsupported: {family}",
    )
    eligible = entry.get("corpus_eligible")
    require(isinstance(eligible, bool), f"{prefix}.corpus_eligible must be boolean")
    require(
        eligible is (family in CORE_FAMILIES),
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
            verdict in {"unverified", "usable", "weak", "reject"},
            f"{prefix}: invalid retired source suitability",
        )
        if verdict == "unverified":
            validate_unverified_retirement_provenance(entry, prefix)
        else:
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
        validate_source_file(repo / source_path, source_hash, prefix)
    return case_id


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


def validate_source_file(path: Path, expected_hash: str, prefix: str) -> None:
    require(path.is_file(), f"{prefix}: missing source file: {path}")
    actual_hash = hashlib.sha256(path.read_bytes()).hexdigest()
    require(actual_hash == expected_hash, f"{prefix}: source SHA-256 mismatch: {path}")
    with wave.open(str(path), "rb") as source:
        require(source.getnchannels() == 2, f"{prefix}: source WAV must be stereo")
        require(source.getframerate() == 48_000, f"{prefix}: source WAV must be 48 kHz")
        require(source.getsampwidth() == 2, f"{prefix}: source WAV must be PCM16")
        require(source.getcomptype() == "NONE", f"{prefix}: source WAV must be PCM")
        frame_count = source.getnframes()
        require(frame_count > 0, f"{prefix}: source WAV must not be empty")
        require(
            frame_count <= 48_000 * 16 + 1,
            f"{prefix}: source WAV must be no longer than 16 seconds",
        )
        samples = array("h")
        samples.frombytes(source.readframes(frame_count))
        if sys.byteorder != "little":
            samples.byteswap()
        require(
            max((abs(sample) for sample in samples), default=0) < 32_767,
            f"{prefix}: source WAV contains clipped integer samples",
        )


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
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


def validate_unverified_retirement_provenance(
    entry: dict[str, Any],
    prefix: str,
) -> None:
    parse_iso_date(entry.get("reviewed_on"), f"{prefix}.reviewed_on")
    reviewer_role = non_empty_string(
        entry.get("reviewer_role"),
        f"{prefix}.reviewer_role",
    )
    require(
        reviewer_role in {"project_musician", "technical_reviewer"},
        f"{prefix}.reviewer_role is unsupported for unverified retirement",
    )
    require(
        entry.get("family_verdict_owner") == "technical_review",
        f"{prefix}.family_verdict_owner must be technical_review",
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
