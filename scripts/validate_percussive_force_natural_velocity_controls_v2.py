#!/usr/bin/env python3
"""Read-only validator for the RIOTBOX-1434 local-root correction."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
from typing import Any, Callable

import run_percussive_force_natural_velocity_controls_v1 as v1
import run_percussive_force_natural_velocity_controls_v2 as v2


ROOT = Path(__file__).resolve().parents[1]
CONTRACT_SHA256 = "a0485af6cb30e401a6bc3bd6e900e3a0f8afdb64ef3c44f6f8445a5386c4c14f"
RUNNER_SHA256 = "4a1f4f7ff2b12374eb768f2287c581938b387aefbfeee925835935a709f38370"


def digest(path: Path) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def validate_repository() -> None:
    v1.require(digest(v2.CONTRACT) == CONTRACT_SHA256, "v2 contract bytes changed")
    runner_path = Path("scripts/run_percussive_force_natural_velocity_controls_v2.py")
    v1.require(digest(runner_path) == RUNNER_SHA256, "v2 runner bytes changed")
    contract, _ = v1.load_json(v2.CONTRACT)
    v2.validate_contract(contract)


def reject(name: str, contract: dict[str, Any], mutate: Callable[[dict[str, Any]], None]) -> None:
    changed = copy.deepcopy(contract)
    mutate(changed)
    try:
        v2.validate_contract(changed)
    except (v1.ControlError, OSError, KeyError, TypeError, ValueError):
        print(f"PASS mutation {name}")
        return
    raise AssertionError(f"mutation unexpectedly passed: {name}")


def fixtures() -> None:
    contract, _ = v1.load_json(v2.CONTRACT)
    cases: list[tuple[str, Callable[[dict[str, Any]], None]]] = [
        ("local_root", lambda value: value["correction"].update(local_audio_root="data/test_audio/external")),
        ("v1_audio_claim", lambda value: value["correction"].update(v1_control_audio_opened=True)),
        ("analysis_inheritance", lambda value: value["inheritance"].update(analysis_passport_unchanged=False)),
        ("control_inheritance", lambda value: value["inheritance"].update(control_catalog_unchanged=False)),
        ("output_path", lambda value: value["execution"].update(exact_output_path="artifacts/other")),
        ("read_count", lambda value: value["execution"].update(maximum_file_reads=7)),
        ("discovery", lambda value: value["execution"].update(directory_discovery=True)),
        ("rerun", lambda value: value["execution"].update(rerun_allowed=True)),
        ("runner_pin", lambda value: value["runner"].update(raw_sha256="0" * 64)),
    ]
    for name, mutation in cases:
        reject(name, contract, mutation)
    print(f"PASS: {len(cases)} fail-closed v2 path-correction mutations")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    try:
        validate_repository()
        if args.fixtures:
            fixtures()
    except (v1.ControlError, OSError, KeyError, TypeError, ValueError, json.JSONDecodeError, AssertionError) as error:
        print(f"FAIL: natural-control v2 gate rejected: {error}")
        return 1
    print("PASS: RIOTBOX-1434 v2 pre-access gate; source_audio_accessed=false")
    print(f"contract_sha256={CONTRACT_SHA256}")
    print(f"runner_sha256={RUNNER_SHA256}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
