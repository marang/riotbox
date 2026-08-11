#!/usr/bin/env python3
"""Run the RIOTBOX-1434 controls from the documented local-audio root."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import run_percussive_force_natural_velocity_controls_v1 as v1
from source_holdout_development_access import read_contained_regular_file


CONTRACT = Path("docs/benchmarks/percussive_force_natural_velocity_controls_v2.json")
PREDECESSOR_CONTRACT_SHA256 = "f618144e02a6654b6a7c08b0773d91454b26e9b4cf8ac7cbcca723961783a78c"
MATRIX_V1 = Path("docs/benchmarks/percussive_force_development_matrix_v1.json")
MATRIX_V1_SHA256 = "3290011471bb1ae0fc66e54c8bb4e1382f82ceee6266245a44755d8f62f1f970"
LOCAL_AUDIO_ROOT = Path(".download-examples")
OUTPUT_REL = Path("artifacts/audio_qa/riotbox-1434/natural-velocity-controls-v2")


def validate_contract(contract: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    v1.require(contract.get("schema") == "riotbox.percussive_force_natural_velocity_controls.v2", "v2 schema changed")
    predecessor, predecessor_payload = v1.load_json(v1.CONTRACT)
    v1.require(v1.sha256(predecessor_payload) == PREDECESSOR_CONTRACT_SHA256, "v1 contract bytes changed")
    v1.validate_contract(predecessor, predecessor_payload)
    matrix_v1, matrix_v1_payload = v1.load_json(MATRIX_V1)
    v1.require(v1.sha256(matrix_v1_payload) == MATRIX_V1_SHA256, "Matrix-v1 bytes changed")
    v1.require(
        matrix_v1.get("path_resolution", {}).get("directional_reference_sources")
        == "Resolve local_path from RIOTBOX_LOCAL_AUDIO_ROOT; the recommended ignored root is .download-examples.",
        "documented local-audio root changed",
    )
    v1.require(
        contract.get("correction")
        == {
            "only_change": "resolve_registered_local_path_from_documented_local_audio_root",
            "local_audio_root": LOCAL_AUDIO_ROOT.as_posix(),
            "matrix_v1_path_resolution_sha256": MATRIX_V1_SHA256,
            "v1_control_audio_opened": False,
            "v1_rerun_allowed": False,
        },
        "v2 correction boundary changed",
    )
    v1.require(
        contract.get("inheritance")
        == {
            "analysis_passport_unchanged": True,
            "control_catalog_unchanged": True,
            "control_order_unchanged": True,
            "control_hashes_unchanged": True,
            "human_protocol_unchanged": True,
            "claim_boundary_unchanged": True,
        },
        "v2 inheritance changed",
    )
    v1.require(
        contract.get("execution")
        == {
            "exact_output_path": OUTPUT_REL.as_posix(),
            "maximum_file_reads": 6,
            "one_read_per_file": True,
            "directory_discovery": False,
            "path_substitution": False,
            "rerun_allowed": False,
        },
        "v2 execution boundary changed",
    )
    runner_path = Path(str(contract.get("runner", {}).get("path")))
    v1.require(
        runner_path == Path("scripts/run_percussive_force_natural_velocity_controls_v2.py")
        and v1.sha256((v1.ROOT / runner_path).read_bytes()) == contract["runner"]["raw_sha256"],
        "v2 runner pin changed",
    )
    matrix_v2, _ = v1.load_json(v1.MATRIX)
    return predecessor, matrix_v2


def exact_controls(predecessor: dict[str, Any], matrix_v2: dict[str, Any]) -> list[dict[str, Any]]:
    catalog_sets = matrix_v2["natural_directional_reference_controls"]["sets"]
    controls = []
    for predecessor_set, catalog_set in zip(predecessor["controls"], catalog_sets, strict=True):
        v1.require(predecessor_set["control_set_id"] == catalog_set["control_set_id"], "control-set order changed")
        members = []
        for predecessor_member, catalog_member in zip(predecessor_set["members"], catalog_set["members"], strict=True):
            v1.require(
                predecessor_member["provisional_dynamic"] == catalog_member["provisional_dynamic"]
                and predecessor_member["sha256"] == catalog_member["sha256"],
                "control member changed",
            )
            members.append(
                {
                    "case_id": predecessor_member["case_id"],
                    "provisional_dynamic": catalog_member["provisional_dynamic"],
                    "repo_path": (LOCAL_AUDIO_ROOT / catalog_member["local_path"]).as_posix(),
                    "sha256": catalog_member["sha256"],
                }
            )
        controls.append(
            {
                "control_set_id": catalog_set["control_set_id"],
                "instrument": catalog_set["instrument"],
                "articulation": catalog_set["articulation"],
                "members": members,
            }
        )
    v1.require(len(controls) == 2 and sum(len(item["members"]) for item in controls) == 6, "control count changed")
    return controls


def run(output_dir: Path) -> dict[str, Any]:
    v1.require(output_dir == v1.ROOT / OUTPUT_REL, f"output must be exactly {v1.ROOT / OUTPUT_REL}")
    v1.require(not output_dir.exists(), f"output already exists: {output_dir}")
    contract, contract_payload = v1.load_json(CONTRACT)
    predecessor, matrix_v2 = validate_contract(contract)
    controls = exact_controls(predecessor, matrix_v2)
    output_dir.mkdir(parents=True)
    log_path = output_dir / "access-log.json"
    log: dict[str, Any] = {
        "schema": "riotbox.percussive_force_natural_velocity_access.v2",
        "owner_ticket": "RIOTBOX-1434",
        "status": "started",
        "contract_sha256": v1.sha256(contract_payload),
        "records": [],
        "directory_discovery_performed": False,
        "control_audio_accessed": False,
        "development_audio_accessed": False,
        "holdout_audio_accessed": False,
        "commercial_reference_audio_accessed": False,
    }
    v1.write_json(log_path, log)
    analyzed_sets = []
    try:
        for control in controls:
            analyzed_members = []
            for member in control["members"]:
                record = {
                    "access_ordinal": len(log["records"]) + 1,
                    "case_id": member["case_id"],
                    "repo_path": member["repo_path"],
                    "expected_sha256": member["sha256"],
                    "status": "opening_exact_registered_control",
                }
                log["records"].append(record)
                v1.write_json(log_path, log)
                payload = read_contained_regular_file(
                    v1.ROOT,
                    Path(member["repo_path"]),
                    f"RIOTBOX-1434-v2:{member['case_id']}",
                    maximum_bytes=v1.MAXIMUM_FILE_BYTES,
                )
                actual_sha = v1.sha256(payload)
                v1.require(actual_sha == member["sha256"], f"{member['case_id']}: SHA-256 changed")
                samples, sample_rate, source_format = v1.decode_pcm_wave(payload, member["case_id"])
                analysis = v1.analyze(samples, sample_rate, member["case_id"], predecessor["analysis"])
                record.update(status="verified_and_analyzed", byte_count=len(payload), actual_sha256=actual_sha, source_format=source_format)
                log["control_audio_accessed"] = True
                v1.write_json(log_path, log)
                analyzed_members.append({**member, "source_format": source_format, "analysis": analysis})
            analyzed_sets.append(
                {
                    "control_set_id": control["control_set_id"],
                    "instrument": control["instrument"],
                    "articulation": control["articulation"],
                    "members": analyzed_members,
                    "technical_directions": v1.directions(analyzed_members),
                    "human_directional_sanity": "pending",
                }
            )
        log["status"] = "completed"
        v1.write_json(log_path, log)
    except Exception:
        log["status"] = "rejected_fail_closed"
        v1.write_json(log_path, log)
        raise
    result = {
        "schema": "riotbox.percussive_force_natural_velocity_analysis.v2",
        "owner_ticket": "RIOTBOX-1434",
        "status": "technical_analysis_complete_human_sanity_pending",
        "contract_sha256": v1.sha256(contract_payload),
        "access_log_path": log_path.as_posix(),
        "access_log_sha256": v1.sha256(log_path.read_bytes()),
        "control_sets": analyzed_sets,
        "algorithm_selection_allowed": False,
        "perceptual_threshold_fitting_allowed": False,
        "hardness_proof": False,
        "quality_proof": False,
        "human_verdict": "unverified",
        "development_audio_accessed": False,
        "holdout_audio_accessed": False,
        "commercial_reference_audio_accessed": False,
    }
    v1.write_json(output_dir / "technical-analysis.json", result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    try:
        contract, _ = v1.load_json(CONTRACT)
        validate_contract(contract)
        if args.validate_only:
            print("PASS: natural velocity control v2 path correction; source_audio_accessed=false")
            return 0
        v1.require(args.output is not None, "--output is required")
        output = args.output if args.output.is_absolute() else v1.ROOT / args.output
        run(output)
    except (v1.ControlError, OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"FAIL: natural velocity controls v2 stopped fail-closed: {error}")
        return 1
    print("PASS: six natural velocity controls analyzed; human_verdict=unverified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
