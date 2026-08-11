#!/usr/bin/env python3
"""Validate the minimal RIOTBOX-1430 Stage-A-v5 continuation contract."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path

import validate_percussive_force_stage_a_protocol_v3 as v3
import validate_percussive_force_stage_a_protocol_v4 as v4


ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = Path("docs/benchmarks/percussive_force_stage_a_protocol_v5.json")
ACCESS = Path("artifacts/audio_qa/riotbox-1430/stage-a-v4-source-pool-access.json")
PROTOCOL_SHA = "455440aabc1a433bbc7fbcc2093b85f6d1c66e1bba081526e082c50ed8248519"
ACCESS_SHA = "fa15bb32c1b9285f878ffbec676ffae627e8aac1d2ea1d4d810241919abcce9e"


def validate(protocol: dict, access: dict) -> None:
    v4_documents = [v4.load(path)[0] for path in (v4.PROTOCOL_PATH, v4.POOL_PATH, v4.MATRIX_PATH, v4.ACCESS_PATH)]
    v4.validate(*v4_documents)
    v3.require(
        protocol.get("schema") == "riotbox.percussive_force_stage_a_protocol.v5"
        and protocol.get("version_decision") == "RBX-267"
        and protocol.get("status")
        == "preregistered_final_continuation_before_further_audio_access",
        "Protocol-v5 identity changed",
    )
    predecessor = protocol.get("predecessor", {})
    v3.require(
        predecessor.get("raw_sha256") == v4.EXPECTED["protocol"]
        and predecessor.get("terminal_access_log_raw_sha256") == ACCESS_SHA
        and predecessor.get("terminal_state")
        == "rejected_fail_closed_before_qualification",
        "Protocol-v4 terminal boundary changed",
    )
    unchanged = protocol.get("unchanged_contracts", {})
    v3.require(
        unchanged.get("algorithm_source_raw_sha256") == v3.EXPECTED_PROTOCOL_V2_SHA256
        and unchanged.get("detector_anatomy_source_contrast_ordinals_f1_f2_f3")
        == "unchanged"
        and unchanged.get("pcm_wave_admission")
        == "riotbox.percussive_force_stage_a_pcm_wave_admission.v2"
        and unchanged.get("metadata_pool_raw_sha256") == v3.EXPECTED_POOL_SHA256,
        "inherited algorithm, format, or metadata contract changed",
    )
    pool = protocol.get("source_pool_v3", {})
    v3.require(
        pool.get("inherited_header_admitted_ordinals") == [1, 2, 4, 5, 6]
        and pool.get("terminal_rejected_ordinals") == [3, 7]
        and pool.get("terminal_reasons")
        == {"3": "unsupported_ieee_float_32", "7": "incoherent_extended_fmt_chunk"}
        and pool.get("exact_request_ordinals") == list(range(8, 16))
        and pool.get("maximum_original_file_gets") == 8,
        "v5 continuation topology changed",
    )
    bounded = protocol.get("access", {})
    v3.require(
        bounded.get("redirects") is False
        and bounded.get("automatic_retries") is False
        and bounded.get("preview_or_search") is False
        and bounded.get("directory_discovery") is False
        and bounded.get("substitution") is False
        and bounded.get("holdout_audio_access") is False
        and bounded.get("commercial_reference_access") is False,
        "v5 access boundary changed",
    )
    qualification = protocol.get("qualification_and_matrix", {})
    v3.require(
        qualification.get("early_stop") is False
        and qualification.get("minimum_events_per_source") == 2
        and qualification.get("maximum_frozen_events_per_source") == 3
        and qualification.get("bound_matrix")
        == "riotbox.percussive_force_development_matrix.v6"
        and qualification.get("cross_product")
        == "3_families_x_4_sources_x_2_events_equals_24",
        "qualification or Matrix-v6 contract changed",
    )
    v3.require(
        access.get("state") == "rejected_fail_closed"
        and access.get("requests_attempted") == 1
        and access.get("requests_completed") == 0
        and access.get("failure")
        == "request_7_strict_header_incoherent_fmt_extension_size"
        and access.get("entries", [{}])[0].get("ordinal") == 7
        and access.get("entries", [{}])[0].get("state")
        == "rejected_strict_header_incoherent_fmt_extension_size"
        and access.get("pcm_sample_iteration") is False
        and access.get("source_audio_playback") is False
        and access.get("holdout_audio_access") is False,
        "terminal v4 access evidence changed",
    )
    v3.require(
        protocol.get("quality_proof") is False
        and protocol.get("hardness_proof") is False
        and protocol.get("human_verdict") == "unverified",
        "v5 claims unearned evidence",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        protocol, protocol_raw = v3.load_json(ROOT / PROTOCOL)
        access, access_raw = v3.load_json(ROOT / ACCESS)
        v3.require(hashlib.sha256(protocol_raw).hexdigest() == PROTOCOL_SHA, "Protocol-v5 raw SHA changed")
        v3.require(hashlib.sha256(access_raw).hexdigest() == ACCESS_SHA, "v4 access raw SHA changed")
        validate(protocol, access)
        if args.fixtures:
            for name, mutation in (
                ("request", lambda value: value["source_pool_v3"]["exact_request_ordinals"].append(16)),
                ("retry", lambda value: value["access"].__setitem__("automatic_retries", True)),
                ("algorithm", lambda value: value["unchanged_contracts"].__setitem__("algorithm_source_raw_sha256", "0" * 64)),
                ("matrix", lambda value: value["qualification_and_matrix"].__setitem__("cross_product", "wrong")),
            ):
                changed = copy.deepcopy(protocol)
                mutation(changed)
                try:
                    validate(changed, access)
                except v3.ContractError:
                    print(f"PASS mutation {name}")
                else:
                    raise v3.ContractError(f"mutation unexpectedly passed: {name}")
    except (v3.ContractError, OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        print(f"FAIL: {error}")
        return 1
    print("PASS: minimal Stage-A-v5 continuation contract")
    print(f"protocol_raw_sha256={PROTOCOL_SHA}")
    print(f"v4_access_raw_sha256={ACCESS_SHA}")
    print("source_audio_accessed_by_validator=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
