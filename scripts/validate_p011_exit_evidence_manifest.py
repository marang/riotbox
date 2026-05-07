#!/usr/bin/env python3
"""Validate the P011 exit-evidence manifest against repo-local proof files."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.p011_exit_evidence_manifest.v1"
SCHEMA_VERSION = 1
DEFAULT_MANIFEST = Path("docs/benchmarks/p011_exit_evidence_manifest.json")
REQUIRED_CATEGORIES = {
    "replay",
    "recovery",
    "export_reproducibility",
    "stage_style_stability",
}
ALLOWED_STATUS = {"active_not_exit_ready"}
ALLOWED_CATEGORY_STATUS = {"bounded_supported", "partial", "open"}
ALLOWED_EVIDENCE_TYPES = {"manifest", "review", "script", "validator", "benchmark_doc"}


def main() -> int:
    manifest_path = Path(sys.argv[1]) if len(sys.argv) == 2 else DEFAULT_MANIFEST
    if len(sys.argv) > 2:
        print(
            "usage: validate_p011_exit_evidence_manifest.py [manifest.json]",
            file=sys.stderr,
        )
        return 2

    try:
        manifest = json.loads(manifest_path.read_text())
        just_recipes = load_just_recipes()
        validate_manifest(manifest, just_recipes)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid P011 exit evidence manifest: {error}", file=sys.stderr)
        return 1

    categories = manifest["categories"]
    output_categories = sum(1 for category in categories if category["output_path"])
    print(
        f"valid {SCHEMA}: {manifest_path} "
        f"categories={len(categories)} output_path={output_categories}"
    )
    return 0


def load_just_recipes() -> set[str]:
    try:
        result = subprocess.run(
            ["just", "--summary"],
            check=True,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as error:
        return load_justfile_recipes(Path("Justfile"), error)
    except subprocess.CalledProcessError as error:
        stderr = error.stderr.strip()
        detail = f": {stderr}" if stderr else ""
        raise ValueError(f"failed to list just recipes{detail}") from error

    recipes = set(result.stdout.split())
    if not recipes:
        raise ValueError("just --summary returned no recipes")
    return recipes


def load_justfile_recipes(path: Path, missing_just_error: FileNotFoundError) -> set[str]:
    try:
        text = path.read_text()
    except OSError as error:
        raise ValueError(
            "just is unavailable and Justfile could not be read to validate recipe references"
        ) from missing_just_error

    recipes: set[str] = set()
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or line[0].isspace() or stripped.startswith("#") or ":" not in stripped:
            continue
        recipe_head = stripped.split(":", 1)[0].strip()
        if not recipe_head:
            continue
        recipe = recipe_head.split()[0]
        if "=" not in recipe:
            recipes.add(recipe)

    if not recipes:
        raise ValueError("Justfile contains no recipe definitions") from missing_just_error
    return recipes


def validate_manifest(manifest: Any, just_recipes: set[str]) -> None:
    require_object(manifest, "manifest")
    require_equal(manifest, "schema", SCHEMA)
    require_equal(manifest, "schema_version", SCHEMA_VERSION)
    require_equal(manifest, "project", "P011 | Pro Hardening")
    require_string(manifest, "boundary")
    require_one_of(manifest, "status", ALLOWED_STATUS)
    require_string_list(manifest, "known_open_boundaries")
    if not manifest["known_open_boundaries"]:
        raise ValueError("known_open_boundaries must not be empty")

    categories = require_list(manifest, "categories")
    if not categories:
        raise ValueError("categories must not be empty")

    ids: set[str] = set()
    output_path_count = 0
    for index, category in enumerate(categories):
        category_id = validate_category(category, index, just_recipes)
        if category_id in ids:
            raise ValueError(f"duplicate category id: {category_id}")
        ids.add(category_id)
        if category["output_path"]:
            output_path_count += 1

    missing = REQUIRED_CATEGORIES - ids
    if missing:
        raise ValueError(f"missing required categories: {sorted(missing)}")
    if output_path_count == 0:
        raise ValueError("expected at least one output-path evidence category")


def validate_category(category: Any, index: int, just_recipes: set[str]) -> str:
    require_object(category, f"category {index}")
    category_id = require_string(category, "id")
    require_string(category, "name")
    require_one_of(category, "status", ALLOWED_CATEGORY_STATUS)
    require_bool(category, "control_path")
    require_bool(category, "output_path")
    require_string(category, "open_boundary")
    if not category["control_path"]:
        raise ValueError(f"{category_id}: control_path must be true for P011 exit evidence")

    proofs = require_list(category, "proofs")
    if not proofs:
        raise ValueError(f"{category_id}: proofs must not be empty")
    for proof_index, proof in enumerate(proofs):
        validate_proof(category_id, proof, proof_index, just_recipes)
    return category_id


def validate_proof(category_id: str, proof: Any, proof_index: int, just_recipes: set[str]) -> None:
    require_object(proof, f"{category_id}.proofs[{proof_index}]")
    path = Path(require_string(proof, "path"))
    command = require_string(proof, "command")
    require_one_of(proof, "evidence_type", ALLOWED_EVIDENCE_TYPES)
    require_string(proof, "why")
    if not path.exists():
        raise ValueError(f"{category_id}: proof path does not exist: {path}")
    validate_command(category_id, command, just_recipes)


def validate_command(category_id: str, command: str, just_recipes: set[str]) -> None:
    if command == "cargo test" or command.startswith("cargo test "):
        return

    if command.startswith("just "):
        parts = command.split()
        if len(parts) < 2:
            raise ValueError(f"{category_id}: proof command is missing just recipe: {command}")
        recipe = parts[1]
        if recipe not in just_recipes:
            raise ValueError(
                f"{category_id}: proof command references unknown just recipe "
                f"{recipe!r}: {command}"
            )
        return

    raise ValueError(f"{category_id}: proof command must be just or cargo test: {command}")


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


def require_one_of(parent: dict[str, Any], field: str, expected: set[str]) -> None:
    actual = parent.get(field)
    if actual not in expected:
        choices = ", ".join(sorted(expected))
        raise ValueError(f"{field} must be one of {choices}, got {actual!r}")


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
