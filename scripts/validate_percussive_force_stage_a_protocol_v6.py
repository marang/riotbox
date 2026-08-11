#!/usr/bin/env python3
"""Validate the source-blind RIOTBOX-1428 F4 freeze without opening audio."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path

import validate_percussive_force_stage_a_protocol_v3 as v3


ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v6.json")
MATRIX = Path("docs/benchmarks/percussive_force_development_matrix_v7.json")
MATRIX_V6 = Path("docs/benchmarks/percussive_force_development_matrix_v6.json")
F4_SOURCE = Path("crates/riotbox-audio/src/percussive_force/f4.rs")
RUNNER = Path("crates/riotbox-audio/src/bin/percussive_force_stage_a_matrix.rs")

PROTOCOL_SHA = "e201d1a95936c17206ee1a1e151bcde32d593209b0d07cde7acb5b3aff32420a"
MATRIX_SHA = "4018ca070b7cb4193191a8a88c4279cd3bc878b25241c8f57f1f0eaa3227480d"
MATRIX_V6_SHA = "cd29b23fd3d39ac5184f73585b825aabf987b865e6f37253260ce2287ac95c00"
F4_SOURCE_SHA = "85b6e4f3b19c292ee712a100bf563af90a4a780cac76734ffd36cb9ae782ef0a"
RUNNER_SHA = "1eccec22f454f9a0309cb0e65aeaebb8b194d2356926fd0b55dd97aae21e932d"
QUALIFICATION_SHA = "f35f9412f8e07e6ced0922e6433d12cb9133e49003b257ef5850f2d72337f679"


def raw_sha(path: Path) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def validate(protocol: dict, matrix: dict, matrix_v6: dict) -> None:
    v3.require(
        protocol.get("schema") == "riotbox.percussive_force_stage_a_protocol.v6"
        and protocol.get("version_decision") == "RBX-275"
        and protocol.get("status")
        == "preregistered_source_blind_f4_before_development_candidate_access",
        "Protocol-v6 identity changed",
    )
    predecessor = protocol.get("predecessor", {})
    v3.require(
        predecessor.get("raw_sha256")
        == "455440aabc1a433bbc7fbcc2093b85f6d1c66e1bba081526e082c50ed8248519"
        and predecessor.get("terminal_human_decision") == "RBX-273",
        "Protocol-v5 or human rejection predecessor changed",
    )
    qualification = protocol.get("reused_mechanism_blind_qualification", {})
    v3.require(
        qualification.get("raw_sha256") == QUALIFICATION_SHA
        and qualification.get("requalification_or_event_substitution") is False,
        "mechanism-blind qualification identity changed",
    )
    historical = protocol.get("historical_family_state", {})
    v3.require(
        historical
        == {
            "f1_ab_energy_redistribution_v1": "immutable_mechanically_rejected",
            "f2_exact_complementary_three_band_v1": "immutable_human_rejected_near_identity",
            "f3_causal_envelope_contrast_dynamic_residual_v2": "immutable_mechanically_rejected",
        },
        "historical F1-F3 terminal states changed",
    )

    f4 = protocol.get("f4", {})
    signal = f4.get("signal_path", {})
    v3.require(
        f4.get("version_id") == "f4_source_native_body_sustain_v1"
        and f4.get("source_raw_sha256") == F4_SOURCE_SHA
        and signal.get("playback_rate") == [1, 1]
        and signal.get("sample_order") == "unchanged"
        and signal.get("physical_attack") == "bit_identical_through_attack_end"
        and signal.get("outside_event") == "bit_identical"
        and signal.get("body_bands_hz") == [[55, 180], [180, 560], [560, 1120]]
        and signal.get("additional_gain")
        == "0.5*entry*exit*(0.35+0.65*sqrt(1-clamp(envelope/body_peak,0,1)))"
        and signal.get("limiter") is False
        and signal.get("generated_oscillator") is False
        and signal.get("delay_or_duplicate") is False
        and signal.get("resampling_or_transposition") is False,
        "F4 topology or equation changed",
    )
    passports = f4.get("numeric_passports", {})
    expected_passports = {
        "body_band_edges_hz": [55, 180, 560, 1120],
        "lookbehind_noise_multiplier": 4,
        "quantization_lsb_multiplier": 16,
        "body_envelope_ms": 8,
        "body_entry_ms": 2,
        "body_exit_ms": 10,
        "maximum_additional_band_gain": 0.5,
        "minimum_additional_gain_fraction": 0.35,
    }
    v3.require(
        set(passports) == set(expected_passports)
        and all(
            passports[name].get("value") == value
            and passports[name].get("perceptual_threshold") is False
            for name, value in expected_passports.items()
        ),
        "F4 numeric passport changed",
    )
    preflight = f4.get("synthetic_preflight", {})
    v3.require(
        preflight.get("status_before_freeze") == "passed_5_of_5"
        and len(preflight.get("required_tests", [])) == 5,
        "F4 synthetic preflight changed",
    )

    v3.require(
        matrix.get("schema") == "riotbox.percussive_force_development_matrix.v7"
        and matrix.get("status") == "bound_before_f4_candidate_access"
        and matrix.get("condition_count") == 8
        and matrix.get("protocol", {}).get("raw_sha256") == PROTOCOL_SHA,
        "Matrix-v7 identity changed",
    )
    expected_ids = [
        f"f4_{source['case_id']}_event{event['ordinal']}"
        for source in matrix_v6.get("selected_sources", [])
        for event in source.get("events", [])
    ]
    v3.require(
        matrix.get("condition_ids") == expected_ids
        and len(expected_ids) == 8
        and matrix.get("selected_sources") == matrix_v6.get("selected_sources"),
        "Matrix-v7 source, event, condition, or order binding changed",
    )
    renderer = matrix.get("renderer", {})
    v3.require(
        renderer.get("raw_sha256") == RUNNER_SHA
        and renderer.get("f4_raw_sha256") == F4_SOURCE_SHA
        and renderer.get("f4") == "f4_source_native_body_sustain_v1",
        "Matrix-v7 renderer binding changed",
    )
    access = matrix.get("source_access", {})
    v3.require(
        access.get("exact_registered_development_paths_only") is True
        and access.get("one_read_per_source") is True
        and access.get("access_log_embedded_in_result") is True
        and access.get("directory_discovery") is False
        and access.get("holdout_audio_access") is False
        and access.get("commercial_reference_access") is False,
        "Matrix-v7 source-access boundary changed",
    )
    v3.require(
        protocol.get("human_verdict") == "unverified"
        and protocol.get("hardness_proof") is False
        and matrix.get("candidate_render_started") is False
        and matrix.get("human_verdict") == "unverified",
        "Protocol-v6 or Matrix-v7 claims unearned evidence",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        protocol, protocol_raw = v3.load_json(ROOT / PROTOCOL)
        matrix, matrix_raw = v3.load_json(ROOT / MATRIX)
        matrix_v6, matrix_v6_raw = v3.load_json(ROOT / MATRIX_V6)
        v3.require(hashlib.sha256(protocol_raw).hexdigest() == PROTOCOL_SHA, "Protocol-v6 raw SHA changed")
        v3.require(hashlib.sha256(matrix_raw).hexdigest() == MATRIX_SHA, "Matrix-v7 raw SHA changed")
        v3.require(hashlib.sha256(matrix_v6_raw).hexdigest() == MATRIX_V6_SHA, "Matrix-v6 historical SHA changed")
        v3.require(raw_sha(F4_SOURCE) == F4_SOURCE_SHA, "F4 source raw SHA changed")
        v3.require(raw_sha(RUNNER) == RUNNER_SHA, "matrix runner raw SHA changed")
        validate(protocol, matrix, matrix_v6)
        if args.fixtures:
            for name, target, mutation in (
                ("gain", "protocol", lambda value: value["f4"]["numeric_passports"]["maximum_additional_band_gain"].__setitem__("value", 0.6)),
                ("topology", "protocol", lambda value: value["f4"]["signal_path"].__setitem__("limiter", True)),
                ("source", "matrix", lambda value: value["selected_sources"].reverse()),
                ("holdout", "matrix", lambda value: value["source_access"].__setitem__("holdout_audio_access", True)),
            ):
                changed_protocol = copy.deepcopy(protocol)
                changed_matrix = copy.deepcopy(matrix)
                mutation(changed_protocol if target == "protocol" else changed_matrix)
                try:
                    validate(changed_protocol, changed_matrix, matrix_v6)
                except v3.ContractError:
                    print(f"PASS mutation {name}")
                else:
                    raise v3.ContractError(f"mutation unexpectedly passed: {name}")
    except (v3.ContractError, OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}")
        return 1
    print("PASS: Stage-A Protocol-v6 and Matrix-v7 F4 freeze")
    print(f"protocol_raw_sha256={PROTOCOL_SHA}")
    print(f"matrix_raw_sha256={MATRIX_SHA}")
    print(f"f4_source_raw_sha256={F4_SOURCE_SHA}")
    print(f"runner_raw_sha256={RUNNER_SHA}")
    print("source_audio_accessed_by_validator=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
