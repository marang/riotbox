#!/usr/bin/env python3
"""Validate Riotbox observer/audio summary JSON v1.

This is intentionally narrower than a general JSON Schema validator: it checks
the repo's current stable summary contract without adding a runtime dependency.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.observer_audio_summary.v1"
SCHEMA_VERSION = 1


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_observer_audio_summary_json.py <summary.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        summary = json.loads(path.read_text())
        validate_summary(summary)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid observer/audio summary JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid {SCHEMA} summary: {path}")
    return 0


def validate_summary(summary: Any) -> None:
    require_object(summary, "summary")
    require_equal(summary, "schema", SCHEMA)
    require_equal(summary, "schema_version", SCHEMA_VERSION)
    require_bool(summary, "needs_human_listening")

    control_path = require_object_field(summary, "control_path")
    require_bool(control_path, "present")
    require_string(control_path, "observer_schema")
    require_string(control_path, "launch_mode")
    require_string(control_path, "audio_runtime_status")
    require_string(control_path, "first_commit")
    require_string_list(control_path, "key_outcomes")

    output_path = require_object_field(summary, "output_path")
    require_bool(output_path, "present")
    require_string_list(output_path, "issues")
    require_string(output_path, "pack_id")
    require_string(output_path, "manifest_result")
    require_int(output_path, "artifact_count")

    metrics = require_object_field(output_path, "metrics")
    require_optional_number(metrics, "full_mix_rms")
    require_optional_number(metrics, "full_mix_low_band_rms")
    require_optional_number(metrics, "mc202_question_answer_delta_rms")


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_object_field(parent: dict[str, Any], field: str) -> dict[str, Any]:
    return require_object(parent.get(field), field)


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    actual = parent.get(field)
    if actual != expected:
        raise ValueError(f"{field} must be {expected!r}, got {actual!r}")


def require_bool(parent: dict[str, Any], field: str) -> None:
    if not isinstance(parent.get(field), bool):
        raise TypeError(f"{field} must be a boolean")


def require_string(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")


def require_string_list(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise TypeError(f"{field} must be an array of strings")


def require_int(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(f"{field} must be an integer")


def require_optional_number(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if value is not None and (not isinstance(value, (int, float)) or isinstance(value, bool)):
        raise TypeError(f"{field} must be a number or null")


if __name__ == "__main__":
    raise SystemExit(main())
