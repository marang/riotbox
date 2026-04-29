#!/usr/bin/env python3
"""Validate Riotbox listening manifest JSON v1.

This intentionally checks only the stable manifest envelope documented in
docs/benchmarks/listening_manifest_v1_json_contract_2026-04-29.md. Pack-specific
metrics, thresholds, cases, and source metadata remain flexible.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_listening_manifest_json.py <manifest.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        manifest = json.loads(path.read_text())
        validate_manifest(manifest)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid listening manifest JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid riotbox listening manifest v{SCHEMA_VERSION}: {path}")
    return 0


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


def validate_artifact(artifact: Any, index: int) -> None:
    require_object(artifact, f"artifact {index}")
    require_string(artifact, "role", f"artifact {index} role")
    require_string(artifact, "kind", f"artifact {index} kind")
    require_string(artifact, "path", f"artifact {index} path")
    require_optional_string_or_null(artifact, "metrics_path", f"artifact {index} metrics_path")
    require_optional_string_or_null(artifact, "case_id", f"artifact {index} case_id")


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


def require_list(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_optional_string_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and not isinstance(value, str):
        raise TypeError(f"{name} must be a string or null")


if __name__ == "__main__":
    raise SystemExit(main())
