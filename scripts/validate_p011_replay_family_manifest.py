#!/usr/bin/env python3
"""Validate the P011 replay-family manifest against repo-local proof tests."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

SCHEMA = "riotbox.p011_replay_family_manifest.v1"
SCHEMA_VERSION = 1
DEFAULT_MANIFEST = Path("docs/benchmarks/p011_replay_family_manifest.json")
TEST_PATTERN = re.compile(r"^\s*fn\s+([a-zA-Z0-9_]+)\s*\(", re.MULTILINE)


def main() -> int:
    manifest_path = Path(sys.argv[1]) if len(sys.argv) == 2 else DEFAULT_MANIFEST
    if len(sys.argv) > 2:
        print(
            "usage: validate_p011_replay_family_manifest.py [manifest.json]",
            file=sys.stderr,
        )
        return 2

    try:
        manifest = json.loads(manifest_path.read_text())
        validate_manifest(manifest)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid P011 replay-family manifest: {error}", file=sys.stderr)
        return 1

    families = manifest["families"]
    output_families = sum(1 for family in families if family["output_path"])
    print(
        f"valid {SCHEMA}: {manifest_path} "
        f"families={len(families)} output_path={output_families}"
    )
    return 0


def validate_manifest(manifest: Any) -> None:
    require_object(manifest, "manifest")
    require_equal(manifest, "schema", SCHEMA)
    require_equal(manifest, "schema_version", SCHEMA_VERSION)
    require_string(manifest, "project")
    require_string(manifest, "boundary")
    require_string_list(manifest, "known_open_boundaries")

    families = require_list(manifest, "families")
    if not families:
        raise ValueError("families must not be empty")

    ids: set[str] = set()
    output_path_count = 0
    for index, family in enumerate(families):
        family_id = validate_family(family, index)
        if family_id in ids:
            raise ValueError(f"duplicate family id: {family_id}")
        ids.add(family_id)
        if family["output_path"]:
            output_path_count += 1

    if output_path_count == 0:
        raise ValueError("expected at least one output-path replay family")


def validate_family(family: Any, index: int) -> str:
    require_object(family, f"family {index}")
    family_id = require_string(family, "id")
    require_string(family, "name")
    require_equal(family, "status", "supported")
    require_string(family, "covered_suffix")
    require_bool(family, "control_path")
    require_bool(family, "output_path")
    require_string(family, "open_boundary")

    if not family["control_path"]:
        raise ValueError(f"{family_id}: control_path must be true for P011 support claims")

    proofs = require_list(family, "proofs")
    if not proofs:
        raise ValueError(f"{family_id}: proofs must not be empty")
    for proof_index, proof in enumerate(proofs):
        validate_proof(family_id, proof, proof_index)
    return family_id


def validate_proof(family_id: str, proof: Any, proof_index: int) -> None:
    require_object(proof, f"{family_id}.proofs[{proof_index}]")
    path = Path(require_string(proof, "path"))
    command = require_string(proof, "command")
    test_names = require_string_list(proof, "tests")
    if not test_names:
        raise ValueError(f"{family_id}: proof {path} must list at least one test")
    if not path.exists():
        raise ValueError(f"{family_id}: proof path does not exist: {path}")

    available_tests = set(TEST_PATTERN.findall(path.read_text()))
    for test_name in test_names:
        if test_name not in available_tests:
            raise ValueError(f"{family_id}: {path} does not define test {test_name}")
    if "cargo test" not in command and "just " not in command:
        raise ValueError(f"{family_id}: command must be a test or just gate: {command}")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_list(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    actual = parent.get(field)
    if actual != expected:
        raise ValueError(f"{field} must be {expected!r}, got {actual!r}")


def require_bool(parent: dict[str, Any], field: str) -> None:
    if not isinstance(parent.get(field), bool):
        raise TypeError(f"{field} must be a boolean")


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


def require_string_list(parent: dict[str, Any], field: str) -> list[str]:
    value = parent.get(field)
    if not isinstance(value, list) or any(not isinstance(item, str) or not item for item in value):
        raise TypeError(f"{field} must be an array of non-empty strings")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
