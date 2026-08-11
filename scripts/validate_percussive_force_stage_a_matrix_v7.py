#!/usr/bin/env python3
"""Run the frozen RIOTBOX-1428 F4 Matrix-v7 advanced screens."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
from typing import Any

import validate_percussive_force_stage_a_matrix_v6 as v1
import validate_percussive_force_stage_a_matrix_v6_v2 as v2


MATRIX = Path("docs/benchmarks/percussive_force_development_matrix_v7.json")
MATRIX_SHA = "4018ca070b7cb4193191a8a88c4279cd3bc878b25241c8f57f1f0eaa3227480d"
MATRIX_RESULT = Path("artifacts/audio_qa/riotbox-1428/stage-a-v7-f4-matrix/matrix-result.json")
MATRIX_RESULT_SHA = "70edb5d8604f12f634c4f4b0828cd3809af603144e94cd06d6294c391883a3c5"
OUTPUT_NAME = "advanced-mechanical-result-v3.json"
EXPECTED_BASIC_SURVIVORS = [
    "f4_freesound_dabromusic_266735_event1",
    "f4_freesound_dabromusic_266735_event2",
    "f4_freesound_dr_skitz_353853_event1",
    "f4_freesound_dr_skitz_353853_event2",
    "f4_freesound_aikighost_19059_event1",
    "f4_freesound_aikighost_19059_event2",
]
EXPECTED_ADVANCED_SOURCE_IDS = [
    "freesound_dabromusic_266735",
    "freesound_dr_skitz_353853",
    "freesound_aikighost_19059",
]


def validate_contracts() -> None:
    matrix, _ = v1.shared.read_pinned_json(MATRIX, MATRIX_SHA)
    result, _ = v1.shared.read_pinned_json(MATRIX_RESULT, MATRIX_RESULT_SHA)
    v1.shared.read_pinned_json(v1.SOURCE_SET, v1.SOURCE_SET_SHA)
    protocol = v1.stage_a.load_frozen_protocol(v1.PROTOCOL)
    v1.require(protocol.sha256 == v1.PROTOCOL_SHA, "Protocol-v2 pin changed")
    v1.require(matrix.get("condition_count") == 8, "Matrix-v7 cardinality changed")
    v1.require(
        result.get("schema") == "riotbox.percussive_force_development_matrix_result.v2"
        and result.get("matrix_sha256") == MATRIX_SHA
        and result.get("condition_count") == 8
        and result.get("rendered_basic_screen_pass_count") == 6,
        "Matrix-v7 result identity changed",
    )
    basic_survivors = [
        condition["condition_id"]
        for condition in result["conditions"]
        if condition["render_state"] == "rendered_basic_screens_passed"
    ]
    v1.require(basic_survivors == EXPECTED_BASIC_SURVIVORS, "basic survivor order changed")
    for condition in result["conditions"]:
        if condition["condition_id"] in EXPECTED_BASIC_SURVIVORS:
            v1.require(
                condition.get("policy", {}).get("version_id")
                == "f4_source_native_body_sustain_v1"
                and condition.get("output_path")
                and condition.get("output_wav_sha256"),
                f"F4 candidate binding incomplete: {condition['condition_id']}",
            )
    v1.require(
        result.get("holdout_audio_accessed") is False
        and result.get("commercial_reference_accessed") is False
        and result.get("human_verdict") == "unverified",
        "Matrix-v7 result claims forbidden evidence",
    )


def run(matrix_result_path: Path) -> dict[str, Any]:
    v1.require(matrix_result_path == MATRIX_RESULT, "only the frozen Matrix-v7 result is allowed")
    original_matrix = v1.MATRIX
    original_matrix_sha = v1.MATRIX_SHA
    original_screen = v1.screen_view
    try:
        v1.MATRIX = MATRIX
        v1.MATRIX_SHA = MATRIX_SHA
        v1.screen_view = v2.screen_view_v2
        result = v1.run(matrix_result_path)
    finally:
        v1.MATRIX = original_matrix
        v1.MATRIX_SHA = original_matrix_sha
        v1.screen_view = original_screen
    result["schema"] = "riotbox.percussive_force_development_matrix_advanced_result.v3"
    result["algorithm_version"] = "f4_source_frozen_event_identity_and_confound_screens_v1"
    result["matrix_result_sha256"] = MATRIX_RESULT_SHA
    accessed = [entry["case_id"] for entry in result["advanced_source_access_log"]]
    v1.require(accessed == EXPECTED_ADVANCED_SOURCE_IDS, "advanced source access order changed")
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--matrix-result", type=Path, default=MATRIX_RESULT)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        validate_contracts()
        if args.validate_only:
            print("PASS: Matrix-v7 advanced-screen contracts; candidate_audio_accessed=false")
            return 0
        output = run(args.matrix_result)
        output_path = args.matrix_result.parent / OUTPUT_NAME
        v1.shared.create_exclusive_json(output_path, output)
    except Exception as error:
        print(f"FAIL: Matrix-v7 advanced screens stopped fail-closed: {error}")
        return 1
    print("PASS: Matrix-v7 advanced screens complete")
    print(f"advanced_survivor_count={output['advanced_survivor_count']}")
    print(f"result={output_path}")
    print("human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
