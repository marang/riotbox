#!/usr/bin/env python3
"""Validate the RIOTBOX-1428 advanced-access freeze without opening audio."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path

import validate_percussive_force_stage_a_protocol_v3 as v3


ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v7.json")
MATRIX_RESULT = Path("artifacts/audio_qa/riotbox-1428/stage-a-v7-f4-matrix/matrix-result.json")
PROTOCOL_SHA = "e92bbcb44ff0c7b14d43dc16173ee5fda5cd421f850d45a18d771fdff3ce9407"
MATRIX_RESULT_SHA = "70edb5d8604f12f634c4f4b0828cd3809af603144e94cd06d6294c391883a3c5"
F4_SHA = "85b6e4f3b19c292ee712a100bf563af90a4a780cac76734ffd36cb9ae782ef0a"
WRAPPER = Path("scripts/validate_percussive_force_stage_a_matrix_v7.py")
WRAPPER_SHA = "5c0778763a5b7a32e7612405ba294eb3666c74720d179af26a8bfc971f491b81"
SHARED = Path("scripts/validate_percussive_force_stage_a_matrix_v6.py")
SHARED_SHA = "21bd63ea95d709231cd714c58d93a99ee20e98cc3d597375373885447d60b39c"
SURVIVORS = [
    "f4_freesound_dabromusic_266735_event1",
    "f4_freesound_dabromusic_266735_event2",
    "f4_freesound_dr_skitz_353853_event1",
    "f4_freesound_dr_skitz_353853_event2",
    "f4_freesound_aikighost_19059_event1",
    "f4_freesound_aikighost_19059_event2",
]
SOURCE_IDS = [
    "freesound_dabromusic_266735",
    "freesound_dr_skitz_353853",
    "freesound_aikighost_19059",
]


def sha(path: Path) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def validate(protocol: dict, result: dict) -> None:
    v3.require(
        protocol.get("schema") == "riotbox.percussive_force_stage_a_protocol.v7"
        and protocol.get("version_decision") == "RBX-276"
        and protocol.get("predecessor", {}).get("raw_sha256")
        == "e201d1a95936c17206ee1a1e151bcde32d593209b0d07cde7acb5b3aff32420a",
        "Protocol-v7 identity or predecessor changed",
    )
    immutable = protocol.get("immutable_f4", {})
    v3.require(
        immutable.get("source_raw_sha256") == F4_SHA
        and immutable.get("algorithm_equations_constants_topology_changed") is False
        and immutable.get("detector_anatomy_thresholds_changed") is False
        and immutable.get("mechanical_thresholds_changed") is False
        and immutable.get("source_or_event_binding_changed") is False,
        "F4 or inherited analysis contract changed",
    )
    matrix = protocol.get("matrix_result", {})
    v3.require(
        matrix.get("raw_sha256") == MATRIX_RESULT_SHA
        and matrix.get("condition_count") == 8
        and matrix.get("basic_survivor_count") == 6
        and matrix.get("basic_survivor_ids") == SURVIVORS
        and matrix.get("terminal_basic_rejections")
        == ["f4_freesound_garzul_213512_event1", "f4_freesound_garzul_213512_event2"],
        "Matrix-v7 basic outcome binding changed",
    )
    access = protocol.get("advanced_access", {})
    v3.require(
        access.get("exact_source_case_ids_in_order") == SOURCE_IDS
        and access.get("maximum_source_reads") == 3
        and access.get("one_read_per_source") is True
        and access.get("candidate_ids") == SURVIVORS
        and access.get("maximum_candidate_reads") == 6
        and access.get("directory_discovery") is False
        and access.get("substitution") is False
        and access.get("automatic_retry") is False
        and access.get("holdout_audio_access") is False
        and access.get("commercial_reference_access") is False,
        "advanced access boundary changed",
    )
    validator = protocol.get("advanced_validator", {})
    v3.require(
        validator.get("wrapper_raw_sha256") == WRAPPER_SHA
        and validator.get("shared_screen_raw_sha256") == SHARED_SHA
        and validator.get("candidate_specific_exception") is False
        and validator.get("automation_may_award_hardness") is False,
        "advanced validator binding changed",
    )
    actual_survivors = [
        item["condition_id"]
        for item in result["conditions"]
        if item["render_state"] == "rendered_basic_screens_passed"
    ]
    v3.require(
        actual_survivors == SURVIVORS
        and result.get("holdout_audio_accessed") is False
        and result.get("commercial_reference_accessed") is False,
        "Matrix result no longer matches Protocol-v7",
    )
    v3.require(
        protocol.get("candidate_audio_accessed_by_advanced_validator") is False
        and protocol.get("advanced_source_audio_reaccessed") is False
        and protocol.get("human_verdict") == "unverified",
        "Protocol-v7 claims unearned advanced or human evidence",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        protocol, raw = v3.load_json(ROOT / PROTOCOL)
        result, result_raw = v3.load_json(ROOT / MATRIX_RESULT)
        v3.require(hashlib.sha256(raw).hexdigest() == PROTOCOL_SHA, "Protocol-v7 raw SHA changed")
        v3.require(hashlib.sha256(result_raw).hexdigest() == MATRIX_RESULT_SHA, "Matrix result raw SHA changed")
        v3.require(sha(WRAPPER) == WRAPPER_SHA, "advanced wrapper raw SHA changed")
        v3.require(sha(SHARED) == SHARED_SHA, "shared advanced screen raw SHA changed")
        validate(protocol, result)
        if args.fixtures:
            for name, mutation in (
                ("algorithm", lambda value: value["immutable_f4"].__setitem__("algorithm_equations_constants_topology_changed", True)),
                ("source", lambda value: value["advanced_access"]["exact_source_case_ids_in_order"].reverse()),
                ("candidate", lambda value: value["advanced_access"]["candidate_ids"].pop()),
                ("holdout", lambda value: value["advanced_access"].__setitem__("holdout_audio_access", True)),
            ):
                changed = copy.deepcopy(protocol)
                mutation(changed)
                try:
                    validate(changed, result)
                except v3.ContractError:
                    print(f"PASS mutation {name}")
                else:
                    raise v3.ContractError(f"mutation unexpectedly passed: {name}")
    except (v3.ContractError, OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}")
        return 1
    print("PASS: Stage-A Protocol-v7 advanced-access freeze")
    print(f"protocol_raw_sha256={PROTOCOL_SHA}")
    print(f"matrix_result_raw_sha256={MATRIX_RESULT_SHA}")
    print("source_or_candidate_audio_accessed_by_validator=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
