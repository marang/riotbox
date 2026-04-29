#!/usr/bin/env python3
"""Validate Riotbox listening manifest JSON v1.

This checks the stable manifest envelope documented in
docs/benchmarks/listening_manifest_v1_json_contract_2026-04-29.md. Pack-specific
metrics, thresholds, cases, and source metadata remain flexible, but known
optional QA contracts such as Feral scorecards are validated when present.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1


def main() -> int:
    try:
        path, require_existing_artifacts = parse_args(sys.argv[1:])
    except ValueError as error:
        print(error, file=sys.stderr)
        print(
            "usage: validate_listening_manifest_json.py [--require-existing-artifacts] <manifest.json>",
            file=sys.stderr,
        )
        return 2

    try:
        manifest = json.loads(path.read_text())
        validate_manifest(manifest)
        if require_existing_artifacts:
            validate_artifact_paths(manifest, path.parent)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid listening manifest JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid riotbox listening manifest v{SCHEMA_VERSION}: {path}")
    return 0


def parse_args(args: list[str]) -> tuple[Path, bool]:
    require_existing_artifacts = False
    paths: list[str] = []

    for arg in args:
        if arg == "--require-existing-artifacts":
            require_existing_artifacts = True
        elif arg.startswith("-"):
            raise ValueError(f"unknown option: {arg}")
        else:
            paths.append(arg)

    if len(paths) != 1:
        raise ValueError("expected exactly one manifest path")

    return Path(paths[0]), require_existing_artifacts


def validate_manifest(manifest: Any) -> None:
    require_object(manifest, "manifest")
    require_schema_version(manifest)
    require_string(manifest, "pack_id")
    require_one_of(manifest, "result", {"pass", "fail"})

    artifacts = require_list(manifest, "artifacts")
    if not artifacts:
        raise ValueError("artifacts must not be empty")

    for index, artifact in enumerate(artifacts):
        validate_artifact(artifact, index)

    scorecard = manifest.get("feral_scorecard")
    if scorecard is not None:
        validate_feral_scorecard(scorecard)


def validate_artifact(artifact: Any, index: int) -> None:
    require_object(artifact, f"artifact {index}")
    require_string(artifact, "role", f"artifact {index} role")
    require_string(artifact, "kind", f"artifact {index} kind")
    require_string(artifact, "path", f"artifact {index} path")
    require_optional_string_or_null(artifact, "metrics_path", f"artifact {index} metrics_path")
    require_optional_string_or_null(artifact, "case_id", f"artifact {index} case_id")


def validate_feral_scorecard(scorecard: Any) -> None:
    require_object(scorecard, "feral_scorecard")
    require_string(scorecard, "readiness", "feral_scorecard readiness")
    require_string(
        scorecard,
        "break_rebuild_potential",
        "feral_scorecard break_rebuild_potential",
    )
    require_non_negative_int(scorecard, "hook_fragment_count")
    require_non_negative_int(scorecard, "break_support_count")
    require_non_negative_int(scorecard, "quote_risk_count")
    require_non_negative_int(scorecard, "capture_candidate_count")
    require_string(scorecard, "top_reason", "feral_scorecard top_reason")
    require_bool(scorecard, "source_backed")
    require_bool(scorecard, "generated")
    require_bool(scorecard, "fallback_like")
    require_non_empty_string_list(scorecard, "lane_gestures")
    require_non_empty_string_list(scorecard, "material_sources")
    require_non_empty_string_list(scorecard, "warnings")


def validate_artifact_paths(manifest: dict[str, Any], manifest_dir: Path) -> None:
    for index, artifact in enumerate(manifest["artifacts"]):
        require_file(manifest_dir / artifact["path"], f"artifact {index} path")
        metrics_path = artifact.get("metrics_path")
        if metrics_path is not None:
            require_file(manifest_dir / metrics_path, f"artifact {index} metrics_path")


def require_file(path: Path, name: str) -> None:
    if not path.is_file():
        raise ValueError(f"{name} does not exist: {path}")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_schema_version(parent: dict[str, Any]) -> None:
    value = parent.get("schema_version")
    if not isinstance(value, int) or isinstance(value, bool) or value != SCHEMA_VERSION:
        raise ValueError(f"schema_version must be integer {SCHEMA_VERSION}, got {value!r}")


def require_string(parent: dict[str, Any], field: str, name: str | None = None) -> None:
    value = parent.get(field)
    if not isinstance(value, str) or not value.strip():
        raise TypeError(f"{name or field} must be a non-empty string")


def require_one_of(parent: dict[str, Any], field: str, expected: set[str]) -> None:
    value = parent.get(field)
    if value not in expected:
        choices = ", ".join(sorted(expected))
        raise ValueError(f"{field} must be one of {choices}, got {value!r}")


def require_list(parent: dict[str, Any], field: str, name: str | None = None) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{name or field} must be an array")
    return value


def require_bool(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"feral_scorecard {field} must be a boolean")


def require_non_negative_int(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise TypeError(f"feral_scorecard {field} must be a non-negative integer")


def require_non_empty_string_list(parent: dict[str, Any], field: str) -> None:
    name = f"feral_scorecard {field}"
    values = require_list(parent, field, name)
    if not values:
        raise ValueError(f"{name} must not be empty")
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip():
            raise TypeError(f"{name} entry {index} must be a non-empty string")


def require_optional_string_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and not isinstance(value, str):
        raise TypeError(f"{name} must be a string or null")


if __name__ == "__main__":
    raise SystemExit(main())
