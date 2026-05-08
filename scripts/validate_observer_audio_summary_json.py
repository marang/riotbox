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
    require_int(control_path, "commit_count")
    require_string_list(control_path, "commit_boundaries")
    require_optional_observer_source_timing(control_path)
    require_string_list(control_path, "key_outcomes")

    output_path = require_object_field(summary, "output_path")
    require_bool(output_path, "present")
    require_string_list(output_path, "issues")
    require_string(output_path, "pack_id")
    require_string(output_path, "manifest_result")
    require_int(output_path, "artifact_count")
    require_string(output_path, "grid_bpm_source")
    require_string(output_path, "grid_bpm_decision_reason")
    require_optional_number(output_path, "source_timing_bpm_delta")
    require_optional_source_timing(output_path)
    require_optional_source_timing_alignment(output_path)

    metrics = require_object_field(output_path, "metrics")
    require_optional_number(metrics, "full_mix_rms")
    require_optional_number(metrics, "full_mix_low_band_rms")
    require_optional_number(metrics, "mc202_question_answer_delta_rms")
    require_optional_source_grid_output_drift(metrics)
    require_optional_number(metrics, "w30_candidate_rms")
    require_optional_number(metrics, "w30_candidate_active_sample_ratio")
    require_optional_number(metrics, "w30_rms_delta")


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


def require_optional_bool(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a boolean or null")
    value = parent.get(field)
    if value is not None and not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean or null")


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
    if field not in parent:
        raise TypeError(f"{field} must be present as a number or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, (int, float)) or isinstance(value, bool)):
        raise TypeError(f"{field} must be a number or null")


def require_optional_source_grid_output_drift(parent: dict[str, Any]) -> None:
    field = "source_grid_output_drift"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    drift = require_object(value, field)
    require_number(drift, "hit_ratio")
    require_number(drift, "max_peak_offset_ms")
    require_number(drift, "max_allowed_peak_offset_ms")


def require_optional_source_timing(parent: dict[str, Any]) -> None:
    field = "source_timing"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    timing = require_object(value, field)
    require_string(timing, "source_id")
    require_one_of(
        timing,
        "cue",
        {
            "grid locked",
            "needs confirm",
            "listen first",
            "fallback grid",
            "not available",
            "unknown",
        },
    )
    require_string(timing, "policy_profile")
    require_string(timing, "readiness")
    require_bool(timing, "requires_manual_confirm")
    require_optional_number(timing, "primary_bpm")
    require_optional_bool(timing, "bpm_agrees_with_grid")
    require_string(timing, "beat_status")
    require_string(timing, "downbeat_status")
    require_optional_int(timing, "primary_downbeat_offset_beats")
    require_string(timing, "confidence_result")
    require_string(timing, "drift_status")
    require_string(timing, "phrase_status")
    require_int(timing, "alternate_evidence_count")
    require_string_list(timing, "warning_codes")


def require_optional_source_timing_alignment(parent: dict[str, Any]) -> None:
    field = "source_timing_alignment"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    alignment = require_object(value, field)
    require_one_of(alignment, "status", {"aligned", "partial", "mismatch"})
    require_optional_number(alignment, "bpm_delta")
    require_number(alignment, "bpm_tolerance")
    require_string_list(alignment, "warning_overlap")
    require_string_list(alignment, "issues")


def require_optional_observer_source_timing(parent: dict[str, Any]) -> None:
    field = "observer_source_timing"
    if field not in parent:
        raise TypeError(f"{field} must be present as an object or null")
    value = parent.get(field)
    if value is None:
        return
    timing = require_object(value, field)
    require_string(timing, "source_id")
    require_one_of(
        timing,
        "cue",
        {
            "grid locked",
            "needs confirm",
            "listen first",
            "fallback grid",
            "not available",
            "unknown",
        },
    )
    require_optional_number(timing, "bpm_estimate")
    require_number(timing, "bpm_confidence")
    require_one_of(timing, "quality", {"low", "medium", "high", "unknown"})
    require_one_of(
        timing,
        "degraded_policy",
        {"locked", "cautious", "manual_confirm", "fallback_grid", "disabled", "unknown"},
    )
    require_optional_string(timing, "primary_hypothesis_id")
    require_int(timing, "hypothesis_count")
    require_optional_string(timing, "primary_warning_code")
    require_string_list(timing, "warning_codes")


def require_number(parent: dict[str, Any], field: str) -> None:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{field} must be a number")


def require_optional_int(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as an integer or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, int) or isinstance(value, bool)):
        raise TypeError(f"{field} must be an integer or null")


def require_optional_string(parent: dict[str, Any], field: str) -> None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a string or null")
    value = parent.get(field)
    if value is not None and (not isinstance(value, str) or not value):
        raise TypeError(f"{field} must be a non-empty string or null")


def require_one_of(parent: dict[str, Any], field: str, allowed: set[str]) -> None:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    if value not in allowed:
        raise ValueError(f"{field} must be one of {sorted(allowed)}, got {value!r}")


if __name__ == "__main__":
    raise SystemExit(main())
