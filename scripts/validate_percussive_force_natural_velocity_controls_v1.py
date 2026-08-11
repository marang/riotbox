#!/usr/bin/env python3
"""Read-only pre-access validator for the RIOTBOX-1434 control gate.

Only the named JSON contract, its Matrix-v2 predecessor, and the named runner
are read. No audio path is resolved, opened, hashed, decoded, or enumerated.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
from typing import Any, Callable

import run_percussive_force_natural_velocity_controls_v1 as runner


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = Path("docs/benchmarks/percussive_force_natural_velocity_controls_v1.json")
CONTRACT_SHA256 = "f618144e02a6654b6a7c08b0773d91454b26e9b4cf8ac7cbcca723961783a78c"
RUNNER = Path("scripts/run_percussive_force_natural_velocity_controls_v1.py")
RUNNER_SHA256 = "b9df59203c4e78b064a30f28aa9ece8b0aada0870c8c2860043db28a18260552"


def digest(path: Path) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def validate_repository() -> dict[str, str]:
    runner.require(digest(CONTRACT) == CONTRACT_SHA256, "control contract bytes changed")
    runner.require(digest(RUNNER) == RUNNER_SHA256, "control runner bytes changed")
    contract, payload = runner.load_json(CONTRACT)
    runner.validate_contract(contract, payload)
    return {
        "contract_sha256": CONTRACT_SHA256,
        "runner_sha256": RUNNER_SHA256,
        "matrix_v2_sha256": runner.MATRIX_SHA256,
    }


def expect_rejected(name: str, contract: dict[str, Any], mutate: Callable[[dict[str, Any]], None]) -> None:
    changed = copy.deepcopy(contract)
    mutate(changed)
    try:
        runner.validate_contract(changed, b"fixture-not-a-source-file")
    except (runner.ControlError, KeyError, TypeError, ValueError):
        print(f"PASS mutation {name}")
        return
    raise AssertionError(f"mutation unexpectedly passed: {name}")


def run_fixtures() -> None:
    contract, _ = runner.load_json(CONTRACT)
    cases: list[tuple[str, Callable[[dict[str, Any]], None]]] = [
        ("control_path", lambda value: value["controls"][0]["members"][0].update(repo_path="data/elsewhere.wav")),
        ("control_hash", lambda value: value["controls"][1]["members"][2].update(sha256="0" * 64)),
        ("control_order", lambda value: value["controls"][0]["members"].reverse()),
        ("analysis_passport", lambda value: value["analysis"].update(attack_search_ms=51.0)),
        ("matrix_pin", lambda value: value["matrix_v2"].update(raw_sha256="0" * 64)),
        ("runner_pin", lambda value: value["runner"].update(raw_sha256="0" * 64)),
        ("read_count", lambda value: value["access"].update(maximum_file_reads=7)),
        ("directory_discovery", lambda value: value["access"].update(directory_discovery=True)),
        ("development_access", lambda value: value["access"].update(development_audio_access=True)),
        ("holdout_access", lambda value: value["access"].update(holdout_audio_access=True)),
        ("commercial_access", lambda value: value["access"].update(commercial_reference_audio_access=True)),
        ("rerun", lambda value: value["execution"].update(rerun_allowed=True)),
    ]
    for name, mutation in cases:
        expect_rejected(name, contract, mutation)
    print(f"PASS: {len(cases)} fail-closed natural-control mutations")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        hashes = validate_repository()
        if args.fixtures:
            run_fixtures()
    except (runner.ControlError, OSError, ValueError, KeyError, TypeError, json.JSONDecodeError, AssertionError) as error:
        print(f"FAIL: natural-control gate rejected: {error}")
        return 1
    print("PASS: RIOTBOX-1434 pre-access gate; source_audio_accessed=false")
    for name, value in hashes.items():
        print(f"{name}={value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
