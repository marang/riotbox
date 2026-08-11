#!/usr/bin/env python3
"""Validate the compact RIOTBOX-1430 Stage-A-v3 source-pool contract.

This validator reads contract metadata only. It never discovers or opens source,
holdout, commercial-reference, preview, or generated audio.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
from collections import Counter
from pathlib import Path
from typing import Any, Callable
from urllib.parse import quote


REPO_ROOT = Path(__file__).resolve().parents[1]
PROTOCOL_PATH = Path("docs/benchmarks/percussive_force_stage_a_protocol_v3.json")
POOL_PATH = Path("docs/benchmarks/percussive_force_stage_a_source_pool_v1.json")
MATRIX_PATH = Path(
    "docs/benchmarks/percussive_force_development_matrix_template_v4.json"
)
PROTOCOL_V2_PATH = Path("docs/benchmarks/percussive_force_stage_a_protocol_v2.json")
REGISTRY_V3_PATH = Path("docs/benchmarks/source_holdout_rotation_v3.json")

EXPECTED_PROTOCOL_SHA256 = (
    "21f716e3aeb6c9198671e34be21e225585a41135f875bccedb3c57df625e7eb4"
)
EXPECTED_POOL_SHA256 = (
    "9c729eaac4156cb556d9f2538635e04829b3ffec3ad012d742fb765ad1e2a8ba"
)
EXPECTED_MATRIX_SHA256 = (
    "0bdff640827e92fba9675ab9ede69dea5789bf0a6f3702bcf84254a9eec85df3"
)
EXPECTED_PROTOCOL_V2_SHA256 = (
    "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878"
)
EXPECTED_REGISTRY_V3_SHA256 = (
    "9e5e03ad64319061a4baaa6cee7c40fc5e993171b0d11003ec29767f273bc502"
)
EXPECTED_INHERITED_SECTION_HASHES = {
    "/numeric_passports": (
        "0b486bf697d92c48bf6bd42544132c9e974a0f29d747e3ac787c14e684875c4a"
    ),
    "/prequalification": (
        "9a1e5683e2e60585578781c84983e7bb3803619570f117a4391bc7958dd34555"
    ),
    "/precandidate": (
        "4965700d4bb424f3873ecd0a20dfeb906a2b982eaa9c678b0af0429a5dbddb16"
    ),
}
FAMILIES = ("dense_break", "sparse_drums", "electronic_drums")
KNOWN_V2_FREESOUND_IDS = {724939, 493560, 458897}
CC0_URL = "http://creativecommons.org/publicdomain/zero/1.0/"


class ContractError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def reject_nonfinite(value: str) -> None:
    raise ContractError(f"non-finite JSON number: {value}")


def load_json(path: Path) -> tuple[dict[str, Any], bytes]:
    payload = path.read_bytes()
    value = json.loads(
        payload,
        object_pairs_hook=reject_duplicate_keys,
        parse_constant=reject_nonfinite,
    )
    require(isinstance(value, dict), f"{path}: root must be an object")
    return value, payload


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def canonical_sha256(value: Any) -> str:
    payload = json.dumps(
        value, ensure_ascii=False, separators=(",", ":"), sort_keys=True
    ).encode()
    return sha256(payload)


def validate_documents(
    protocol: dict[str, Any],
    pool: dict[str, Any],
    matrix: dict[str, Any],
    protocol_v2: dict[str, Any],
    registry_v3: dict[str, Any],
) -> None:
    require(
        protocol.get("schema") == "riotbox.percussive_force_stage_a_protocol.v3"
        and protocol.get("schema_version") == 3
        and protocol.get("owner_ticket") == "RIOTBOX-1430"
        and protocol.get("status") == "preregistered_before_new_source_audio_access"
        and protocol.get("version_decision") == "RBX-265",
        "Protocol-v3 identity or preregistration state changed",
    )
    predecessor = protocol.get("predecessor")
    require(
        isinstance(predecessor, dict)
        and predecessor.get("raw_sha256") == EXPECTED_PROTOCOL_V2_SHA256
        and predecessor.get("terminal_decision") == "RBX-263"
        and predecessor.get("terminal_state") == "rejected_fail_closed",
        "Protocol-v2 historical boundary changed",
    )
    inherited = protocol.get("inherited_algorithm_contract")
    require(isinstance(inherited, dict), "inherited algorithm contract missing")
    sections = inherited.get("sections")
    require(isinstance(sections, list), "inherited sections must be an array")
    declared_hashes = {
        item.get("json_pointer"): item.get("canonical_json_sha256")
        for item in sections
        if isinstance(item, dict)
    }
    require(
        declared_hashes == EXPECTED_INHERITED_SECTION_HASHES,
        "inherited algorithm section pins changed",
    )
    for pointer, expected_hash in EXPECTED_INHERITED_SECTION_HASHES.items():
        key = pointer.removeprefix("/")
        require(key in protocol_v2, f"Protocol-v2 inherited section missing: {pointer}")
        require(
            canonical_sha256(protocol_v2[key]) == expected_hash,
            f"Protocol-v2 inherited section drifted: {pointer}",
        )
    selection = protocol.get("qualification_and_selection")
    require(isinstance(selection, dict), "selection contract missing")
    require(
        selection.get("qualification_order")
        == "all_15_candidates_in_ascending_pool_ordinal"
        and selection.get("early_stop") is False
        and selection.get("minimum_events_per_source") == 2
        and selection.get("maximum_frozen_events_per_source") == 3
        and selection.get("selected_set")
        == "first_combination_meeting_every_requirement",
        "qualification or deterministic reserve rule changed",
    )
    requirements = selection.get("combination_requirements")
    require(
        isinstance(requirements, dict)
        and requirements.get("source_count") == 4
        and requirements.get("author_count") == 4
        and requirements.get("family_count") == 3
        and requirements.get("required_families") == list(FAMILIES),
        "selected-set topology changed",
    )
    access = protocol.get("access_contract")
    require(isinstance(access, dict), "access contract missing")
    require(
        access.get("maximum_original_file_gets") == 15
        and access.get("redirects") is False
        and access.get("automatic_retries") is False
        and access.get("previews") is False
        and access.get("directory_discovery") is False
        and access.get("substitution") is False
        and access.get("holdout_audio_access") is False
        and access.get("commercial_reference_access") is False,
        "bounded source-access rule changed",
    )
    accepted_audio = access.get("accepted_audio")
    require(
        isinstance(accepted_audio, dict)
        and accepted_audio.get("format_tag") == 1
        and accepted_audio.get("sample_width_bits") == [16, 24]
        and accepted_audio.get("sample_rate_hz") == [44100, 48000]
        and accepted_audio.get("channel_count") == [1, 2]
        and accepted_audio.get("maximum_duration_seconds") == 16,
        "source format gate changed",
    )

    require(
        pool.get("schema") == "riotbox.percussive_force_stage_a_source_pool.v1"
        and pool.get("schema_version") == 1
        and pool.get("owner_ticket") == "RIOTBOX-1430"
        and pool.get("status") == "metadata_frozen_before_original_audio_access"
        and pool.get("audio_access_state") == "not_started",
        "source-pool identity or audio-access state changed",
    )
    holdout_binding = pool.get("protected_holdout_binding")
    require(
        isinstance(holdout_binding, dict)
        and holdout_binding.get("raw_sha256") == EXPECTED_REGISTRY_V3_SHA256
        and holdout_binding.get("active_holdout_count") == 9
        and holdout_binding.get("holdout_audio_accessed") is False,
        "holdout binding changed",
    )
    entries = pool.get("entries")
    require(isinstance(entries, list) and len(entries) == 15, "pool must contain 15 entries")
    ordinals = [entry.get("ordinal") for entry in entries if isinstance(entry, dict)]
    require(ordinals == list(range(1, 16)), "pool ordinals must be exactly 1 through 15")
    require(all(isinstance(entry, dict) for entry in entries), "pool entries must be objects")
    case_ids = [str(entry["case_id"]) for entry in entries]
    sound_ids = [int(entry["id"]) for entry in entries]
    authors = [str(entry["username"]).casefold() for entry in entries]
    destinations = [str(entry["destination"]) for entry in entries]
    require(len(set(case_ids)) == 15, "pool case IDs must be unique")
    require(len(set(sound_ids)) == 15, "pool sound IDs must be unique")
    require(len(set(authors)) == 15, "pool authors must be unique")
    require(len(set(destinations)) == 15, "pool destinations must be unique")
    require(not (set(sound_ids) & KNOWN_V2_FREESOUND_IDS), "known v2 source reused as fresh")
    require(Counter(entry["source_family"] for entry in entries) == Counter({family: 5 for family in FAMILIES}), "pool must contain five candidates per family")

    holdout_entries = [
        entry
        for entry in registry_v3.get("entries", [])
        if isinstance(entry, dict) and str(entry.get("partition", "")).startswith("holdout_")
    ]
    require(len(holdout_entries) == 9, "protected holdout metadata count changed")
    holdout_urls = {str(entry.get("page_url")) for entry in holdout_entries}
    for entry in entries:
        sound_id = int(entry["id"])
        username = str(entry["username"])
        require(entry.get("license") == CC0_URL, f"non-CC0 source: {sound_id}")
        require(entry.get("type") == "wav" and entry.get("is_remix") is False, f"non-original WAV: {sound_id}")
        require(entry.get("gen_ai_preference") == "no-additional-preferences", f"unexpected author preference: {sound_id}")
        require(2 <= float(entry["duration"]) <= 16, f"duration out of range: {sound_id}")
        require(float(entry["samplerate"]) >= 44100, f"sample rate out of range: {sound_id}")
        require(int(entry["channels"]) in {1, 2}, f"channel count out of range: {sound_id}")
        require(int(entry["filesize"]) > 0, f"invalid file size: {sound_id}")
        md5 = str(entry["md5"])
        require(len(md5) == 32 and all(c in "0123456789abcdef" for c in md5), f"invalid MD5: {sound_id}")
        expected_url = f"https://freesound.org/people/{quote(username, safe='')}/sounds/{sound_id}/"
        require(entry.get("url") == expected_url, f"page URL mismatch: {sound_id}")
        require(entry.get("download") == f"https://freesound.org/apiv2/sounds/{sound_id}/download/", f"download URL mismatch: {sound_id}")
        require(entry.get("url") not in holdout_urls, f"candidate overlaps holdout metadata: {sound_id}")
        destination = Path(str(entry["destination"]))
        require(
            not destination.is_absolute()
            and destination.parts[:5]
            == ("data", "test_audio", "external", "RIOTBOX-1430", "freesound-v3-pool")
            and all(part not in {"", ".", ".."} for part in destination.parts),
            f"unsafe destination: {sound_id}",
        )

    require(
        matrix.get("schema") == "riotbox.percussive_force_development_matrix_template.v4"
        and matrix.get("schema_version") == 4
        and matrix.get("owner_ticket") == "RIOTBOX-1430"
        and matrix.get("status") == "preregistered_unbound_template"
        and matrix.get("selected_set_state") == "not_started",
        "matrix-template identity or state changed",
    )
    cross_product = matrix.get("required_cross_product")
    require(isinstance(cross_product, dict), "matrix cross-product missing")
    require(
        cross_product.get("families")
        == [
            "f1_ab_energy_redistribution_v1",
            "f2_exact_complementary_three_band_v1",
            "f3_causal_envelope_contrast_dynamic_residual_v2",
        ]
        and cross_product.get("source_count") == 4
        and cross_product.get("event_ordinals") == [1, 2]
        and cross_product.get("candidate_event_condition_count") == 24
        and cross_product.get("execution") == "not_started",
        "matrix 3x4x2 cross-product changed",
    )
    require(protocol.get("quality_proof") is False and protocol.get("hardness_proof") is False and protocol.get("human_verdict") == "unverified", "Protocol-v3 claims evidence before execution")
    require(pool.get("quality_proof") is False and pool.get("hardness_proof") is False and pool.get("human_verdict") == "unverified", "source pool claims evidence before execution")
    require(matrix.get("quality_proof") is False and matrix.get("hardness_proof") is False and matrix.get("human_verdict") == "unverified", "matrix template claims evidence before execution")


def load_repository() -> tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any], dict[str, str]]:
    protocol, protocol_raw = load_json(REPO_ROOT / PROTOCOL_PATH)
    pool, pool_raw = load_json(REPO_ROOT / POOL_PATH)
    matrix, matrix_raw = load_json(REPO_ROOT / MATRIX_PATH)
    protocol_v2, protocol_v2_raw = load_json(REPO_ROOT / PROTOCOL_V2_PATH)
    registry_v3, registry_v3_raw = load_json(REPO_ROOT / REGISTRY_V3_PATH)
    pins = {
        "protocol_v3": sha256(protocol_raw),
        "source_pool_v1": sha256(pool_raw),
        "matrix_template_v4": sha256(matrix_raw),
        "protocol_v2": sha256(protocol_v2_raw),
        "registry_v3": sha256(registry_v3_raw),
    }
    require(pins["protocol_v3"] == EXPECTED_PROTOCOL_SHA256, "Protocol-v3 raw SHA-256 changed")
    require(pins["source_pool_v1"] == EXPECTED_POOL_SHA256, "source-pool raw SHA-256 changed")
    require(pins["matrix_template_v4"] == EXPECTED_MATRIX_SHA256, "matrix-template raw SHA-256 changed")
    require(pins["protocol_v2"] == EXPECTED_PROTOCOL_V2_SHA256, "Protocol-v2 historical pin changed")
    require(pins["registry_v3"] == EXPECTED_REGISTRY_V3_SHA256, "Registry-v3 historical pin changed")
    return protocol, pool, matrix, protocol_v2, registry_v3, pins


def run_fixtures(base: tuple[dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any], dict[str, Any]]) -> None:
    fixtures: list[tuple[str, Callable[[dict[str, Any], dict[str, Any], dict[str, Any]], None]]] = [
        ("early_stop", lambda p, _s, _m: p["qualification_and_selection"].__setitem__("early_stop", True)),
        ("algorithm_pin", lambda p, _s, _m: p["inherited_algorithm_contract"]["sections"][0].__setitem__("canonical_json_sha256", "0" * 64)),
        ("pool_size", lambda _p, s, _m: s["entries"].pop()),
        ("duplicate_author", lambda _p, s, _m: s["entries"][1].__setitem__("username", s["entries"][0]["username"])),
        ("non_cc0", lambda _p, s, _m: s["entries"][0].__setitem__("license", "Attribution")),
        ("known_v2_reuse", lambda _p, s, _m: s["entries"][0].__setitem__("id", 724939)),
        ("matrix_count", lambda _p, _s, m: m["required_cross_product"].__setitem__("candidate_event_condition_count", 23)),
    ]
    protocol, pool, matrix, protocol_v2, registry_v3 = base
    for name, mutate in fixtures:
        changed_protocol = copy.deepcopy(protocol)
        changed_pool = copy.deepcopy(pool)
        changed_matrix = copy.deepcopy(matrix)
        mutate(changed_protocol, changed_pool, changed_matrix)
        try:
            validate_documents(changed_protocol, changed_pool, changed_matrix, protocol_v2, registry_v3)
        except ContractError:
            print(f"PASS mutation {name}")
        else:
            raise ContractError(f"mutation unexpectedly passed: {name}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        protocol, pool, matrix, protocol_v2, registry_v3, pins = load_repository()
        validate_documents(protocol, pool, matrix, protocol_v2, registry_v3)
        if args.fixtures:
            run_fixtures((protocol, pool, matrix, protocol_v2, registry_v3))
    except (ContractError, OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}")
        return 1
    print("PASS: compact Stage-A-v3 protocol, source pool, and matrix template")
    for name, value in pins.items():
        print(f"{name}_raw_sha256={value}")
    print("source_audio_accessed=false")
    print("holdout_audio_accessed=false")
    print("commercial_reference_accessed=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
