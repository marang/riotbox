#!/usr/bin/env python3
"""Validate the compact RIOTBOX-1430 Stage-A-v4 resume contract.

Metadata only: this validator never opens source, holdout, reference, preview,
or generated audio.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
from typing import Any, Callable

import validate_percussive_force_stage_a_protocol_v3 as v3


ROOT = Path(__file__).resolve().parents[1]
PROTOCOL_PATH = Path("docs/benchmarks/percussive_force_stage_a_protocol_v4.json")
POOL_PATH = Path("docs/benchmarks/percussive_force_stage_a_source_pool_v2.json")
MATRIX_PATH = Path("docs/benchmarks/percussive_force_development_matrix_template_v5.json")
ACCESS_PATH = Path("artifacts/audio_qa/riotbox-1430/stage-a-v3-source-pool-access.json")
EXPECTED = {
    "protocol": "0f7bff0744a0b229136f192b93ea1e537849c30f2906ab9247e279baf567e724",
    "pool": "e82f21c965678ba8fe1937ee0652ad8456ac73ddb66c2956a871fc26d0404505",
    "matrix": "2fcdcab61b72858419ae53aa9f74fb7a4fb6fd61c3889c6511d0c385b887201e",
    "access": "417e6b41f8ddb9c03650dc42f7c900d4ff56aebf929caaf38fa3dfe175cb15eb",
}
ADMITTED = {
    1: (183441, "b342ee4a9412de14f460c2c295634c53801f2549c71bfc486644a1b02030abc9"),
    2: (268110, "6122a35f22ff910fc9291e0eeafc98c910e03a9c563c4ef58d50efa3ec21251c"),
    4: (211478, "daf38fe7403f8afb9da0d3bf334b50acbbac3c46dd6cf231e4827489b056a57a"),
    5: (266735, "b3ee8908b0433e9d286f6174369cfebe78ee928656e52935d1992fdb2dba7c73"),
    6: (385676, "868f0798a9542e9d04ee3c9ab940d50c24d87364417d04a687a4678d7396f033"),
}
RESUME = list(range(7, 16))
QUALIFICATION = [1, 2, *range(4, 16)]


def load(path: Path) -> tuple[dict[str, Any], bytes]:
    return v3.load_json(ROOT / path)


def validate(
    protocol: dict[str, Any],
    pool: dict[str, Any],
    matrix: dict[str, Any],
    access: dict[str, Any],
) -> None:
    v3_docs = v3.load_repository()
    v3.validate_documents(*v3_docs[:5])
    v3.require(
        protocol.get("schema") == "riotbox.percussive_force_stage_a_protocol.v4"
        and protocol.get("version_decision") == "RBX-266"
        and protocol.get("status") == "preregistered_before_v4_resume_audio_access",
        "Protocol-v4 identity changed",
    )
    v3.require(
        protocol.get("predecessor", {}).get("raw_sha256") == v3.EXPECTED_PROTOCOL_SHA256
        and protocol.get("predecessor", {}).get("terminal_state")
        == "acquisition_rejected_fail_closed_before_qualification",
        "Protocol-v3 terminal boundary changed",
    )
    algorithm = protocol.get("algorithm_contract", {})
    v3.require(
        algorithm.get("raw_sha256") == v3.EXPECTED_PROTOCOL_V2_SHA256
        and algorithm.get("canonical_section_sha256")
        == v3.EXPECTED_INHERITED_SECTION_HASHES
        and algorithm.get("detector_anatomy_source_contrast_ordinals_f1_f2_f3_unchanged")
        is True
        and algorithm.get("source_specific_algorithm_or_threshold_change") is False,
        "frozen Stage-A algorithm contract changed",
    )
    fmt = protocol.get("format_admission_v2", {})
    v3.require(
        fmt.get("component") == "riotbox.percussive_force_stage_a_pcm_wave_admission.v2"
        and fmt.get("format_tag") == 1
        and fmt.get("sample_width_bits") == [16, 24]
        and fmt.get("sample_rate_hz") == [44100, 48000]
        and fmt.get("channel_count") == [1, 2]
        and fmt.get("maximum_duration_seconds") == 16
        and fmt.get("fmt_chunk", {}).get("allowed_extended_size_range") == [18, 64]
        and fmt.get("fmt_chunk", {}).get("extended_rule")
        == "uint16_cbSize_at_offset_16_must_equal_fmt_chunk_size_minus_18",
        "versioned PCM/WAVE admission changed",
    )
    resume = protocol.get("resume_access", {})
    v3.require(
        resume.get("request_ordinals") == RESUME
        and resume.get("maximum_original_file_gets") == 9
        and resume.get("redirects") is False
        and resume.get("automatic_retries_within_v4") is False
        and resume.get("preview_or_search") is False
        and resume.get("directory_discovery") is False
        and resume.get("substitution") is False
        and resume.get("holdout_audio_access") is False
        and resume.get("commercial_reference_access") is False,
        "bounded v4 resume access changed",
    )
    qualification = protocol.get("qualification_and_selection", {})
    v3.require(
        qualification.get("early_stop") is False
        and qualification.get("minimum_events_per_source") == 2
        and qualification.get("maximum_frozen_events_per_source") == 3
        and qualification.get("selection")
        == "first_lexicographic_four_source_combination_with_four_authors_all_three_families_and_unchanged_source_contrast",
        "qualification or deterministic selection changed",
    )
    v3.require(
        pool.get("schema") == "riotbox.percussive_force_stage_a_source_pool.v2"
        and pool.get("status") == "preregistered_resume_before_further_original_audio_access"
        and pool.get("predecessor", {}).get("raw_sha256") == v3.EXPECTED_POOL_SHA256,
        "source-pool-v2 identity changed",
    )
    actual_admitted = {
        int(item["ordinal"]): (int(item["id"]), str(item["raw_sha256"]))
        for item in pool.get("inherited_header_only_admissions", [])
    }
    v3.require(actual_admitted == ADMITTED, "inherited header-only admissions changed")
    v3.require(
        pool.get("terminal_candidate_rejections")
        == [
            {
                "ordinal": 3,
                "id": 217345,
                "reason": "unsupported_ieee_float_32",
                "format_tag": 3,
                "sample_width_bits": 32,
                "retry": False,
            }
        ]
        and pool.get("exact_resume_request_ordinals") == RESUME
        and pool.get("maximum_original_file_gets") == 9
        and pool.get("qualification_candidate_ordinals_if_all_resume_headers_admit")
        == QUALIFICATION,
        "resume pool topology changed",
    )
    v3.require(
        access.get("state") == "rejected_fail_closed"
        and access.get("requests_attempted") == 7
        and access.get("requests_completed") == 6
        and access.get("failure") == "pool[7]: PCM fmt chunk must be exactly 16 bytes"
        and access.get("pcm_sample_iteration") is not True
        and access.get("source_audio_playback") is not True
        and access.get("holdout_audio_access") is False
        and access.get("commercial_reference_access") is False,
        "terminal v3 access evidence changed",
    )
    v3.require(
        matrix.get("schema") == "riotbox.percussive_force_development_matrix_template.v5"
        and matrix.get("selected_set_state") == "not_started"
        and matrix.get("required_cross_product", {}).get("candidate_event_condition_count")
        == 24
        and matrix.get("required_cross_product", {}).get("execution") == "not_started",
        "Matrix-v5 template changed",
    )
    for document in (protocol, pool, matrix):
        v3.require(
            document.get("quality_proof") is False
            and document.get("hardness_proof") is False
            and document.get("human_verdict") == "unverified",
            "v4 document claims unearned evidence",
        )


def fixtures(base: tuple[dict[str, Any], ...]) -> None:
    cases: list[tuple[str, Callable[[dict[str, Any], dict[str, Any], dict[str, Any]], None]]] = [
        ("algorithm", lambda p, _s, _m: p["algorithm_contract"].__setitem__("raw_sha256", "0" * 64)),
        ("fmt", lambda p, _s, _m: p["format_admission_v2"]["fmt_chunk"].__setitem__("allowed_extended_size_range", [18, 128])),
        ("resume", lambda p, _s, _m: p["resume_access"]["request_ordinals"].append(16)),
        ("admitted", lambda _p, s, _m: s["inherited_header_only_admissions"].pop()),
        ("retry_float", lambda _p, s, _m: s["terminal_candidate_rejections"][0].__setitem__("retry", True)),
        ("matrix", lambda _p, _s, m: m["required_cross_product"].__setitem__("candidate_event_condition_count", 23)),
    ]
    protocol, pool, matrix, access = base
    for name, mutate in cases:
        changed = [copy.deepcopy(value) for value in (protocol, pool, matrix)]
        mutate(*changed)
        try:
            validate(*changed, access)
        except v3.ContractError:
            print(f"PASS mutation {name}")
        else:
            raise v3.ContractError(f"mutation unexpectedly passed: {name}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        loaded = [load(path) for path in (PROTOCOL_PATH, POOL_PATH, MATRIX_PATH, ACCESS_PATH)]
        documents = tuple(item[0] for item in loaded)
        hashes = {
            name: hashlib.sha256(item[1]).hexdigest()
            for name, item in zip(("protocol", "pool", "matrix", "access"), loaded)
        }
        v3.require(hashes == EXPECTED, "v4 raw document pin changed")
        validate(*documents)
        if args.fixtures:
            fixtures(documents)
    except (v3.ContractError, OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}")
        return 1
    print("PASS: compact Stage-A-v4 resume contract")
    for name, value in hashes.items():
        print(f"{name}_raw_sha256={value}")
    print("source_audio_accessed_by_validator=false")
    print("holdout_audio_accessed=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
