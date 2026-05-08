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
SOURCE_TIMING_POLICY_PROFILES = {"broad_research", "dance_loop_auto_readiness"}
SOURCE_TIMING_READINESS = {"unavailable", "weak", "needs_review", "ready"}
SOURCE_TIMING_BEAT_STATUSES = {"unavailable", "weak", "stable", "ambiguous"}
SOURCE_TIMING_DOWNBEAT_STATUSES = {"unavailable", "weak", "stable", "ambiguous"}
SOURCE_TIMING_CONFIDENCE_RESULTS = {
    "degraded",
    "candidate_cautious",
    "candidate_ambiguous",
}
SOURCE_TIMING_DRIFT_STATUSES = {
    "unavailable",
    "not_enough_material",
    "stable",
    "high",
}
SOURCE_TIMING_PHRASE_STATUSES = {
    "unavailable",
    "not_enough_material",
    "ambiguous_downbeat",
    "high_drift",
    "stable",
}


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

    source_timing = manifest.get("source_timing")
    if source_timing is not None:
        validate_source_timing(source_timing)
    elif manifest.get("pack_id") == "feral-grid-demo" and "grid_bpm_source" in manifest:
        raise ValueError("source_timing must be present for feral-grid-demo grid BPM manifests")

    validate_source_grid_output_drift(manifest)


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


def validate_source_timing(source_timing: Any) -> None:
    require_object(source_timing, "source_timing")
    require_string(source_timing, "schema", "source_timing schema")
    require_schema_version(source_timing)
    require_string(source_timing, "source_id", "source_timing source_id")
    require_one_of(source_timing, "policy_profile", SOURCE_TIMING_POLICY_PROFILES)
    require_one_of(source_timing, "readiness", SOURCE_TIMING_READINESS)
    require_bool(source_timing, "requires_manual_confirm", "source_timing")
    require_optional_float_or_null(source_timing, "primary_bpm", "source_timing primary_bpm")
    require_optional_bool_or_null(
        source_timing,
        "bpm_agrees_with_grid",
        "source_timing bpm_agrees_with_grid",
    )
    require_optional_non_negative_int_or_null(
        source_timing,
        "primary_downbeat_offset_beats",
        "source_timing primary_downbeat_offset_beats",
    )
    require_one_of(source_timing, "beat_status", SOURCE_TIMING_BEAT_STATUSES)
    require_one_of(source_timing, "downbeat_status", SOURCE_TIMING_DOWNBEAT_STATUSES)
    require_one_of(source_timing, "confidence_result", SOURCE_TIMING_CONFIDENCE_RESULTS)
    require_one_of(source_timing, "drift_status", SOURCE_TIMING_DRIFT_STATUSES)
    require_one_of(source_timing, "phrase_status", SOURCE_TIMING_PHRASE_STATUSES)
    require_non_negative_int(
        source_timing,
        "alternate_evidence_count",
        "source_timing",
    )
    require_string_list(source_timing, "warning_codes", "source_timing warning_codes")


def validate_source_grid_output_drift(manifest: dict[str, Any]) -> None:
    metrics = manifest.get("metrics")
    if not isinstance(metrics, dict) or "source_grid_output_drift" not in metrics:
        return

    drift = require_object(
        metrics.get("source_grid_output_drift"),
        "metrics source_grid_output_drift",
    )
    require_non_negative_int(drift, "beat_count", "source_grid_output_drift")
    require_non_negative_int(drift, "hit_count", "source_grid_output_drift")
    if drift["hit_count"] > drift["beat_count"]:
        raise ValueError("source_grid_output_drift hit_count must not exceed beat_count")
    hit_ratio = require_number(drift, "hit_ratio", "source_grid_output_drift hit_ratio")
    if hit_ratio < 0.0 or hit_ratio > 1.0:
        raise ValueError("source_grid_output_drift hit_ratio must be between 0 and 1")
    require_non_negative_number(
        drift,
        "max_peak_offset_ms",
        "source_grid_output_drift max_peak_offset_ms",
    )
    require_non_negative_number(
        drift,
        "max_allowed_peak_offset_ms",
        "source_grid_output_drift max_allowed_peak_offset_ms",
    )


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


def require_bool(parent: dict[str, Any], field: str, prefix: str = "feral_scorecard") -> None:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"{prefix} {field} must be a boolean")


def require_non_negative_int(
    parent: dict[str, Any],
    field: str,
    prefix: str = "feral_scorecard",
) -> None:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise TypeError(f"{prefix} {field} must be a non-negative integer")


def require_number(parent: dict[str, Any], field: str, name: str) -> float:
    value = parent.get(field)
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError(f"{name} must be a number")
    return float(value)


def require_non_negative_number(parent: dict[str, Any], field: str, name: str) -> None:
    value = require_number(parent, field, name)
    if value < 0.0:
        raise ValueError(f"{name} must be non-negative")


def require_non_empty_string_list(parent: dict[str, Any], field: str) -> None:
    name = f"feral_scorecard {field}"
    values = require_list(parent, field, name)
    if not values:
        raise ValueError(f"{name} must not be empty")
    require_string_list_values(values, name)


def require_string_list(parent: dict[str, Any], field: str, name: str) -> None:
    values = require_list(parent, field, name)
    require_string_list_values(values, name)


def require_string_list_values(values: list[Any], name: str) -> None:
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip():
            raise TypeError(f"{name} entry {index} must be a non-empty string")


def require_optional_string_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and not isinstance(value, str):
        raise TypeError(f"{name} must be a string or null")


def require_optional_bool_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and not isinstance(value, bool):
        raise TypeError(f"{name} must be a boolean or null")


def require_optional_float_or_null(parent: dict[str, Any], field: str, name: str) -> None:
    value = parent.get(field)
    if value is not None and (not isinstance(value, (int, float)) or isinstance(value, bool)):
        raise TypeError(f"{name} must be a number or null")


def require_optional_non_negative_int_or_null(
    parent: dict[str, Any],
    field: str,
    name: str,
) -> None:
    value = parent.get(field)
    if value is not None and (
        not isinstance(value, int) or isinstance(value, bool) or value < 0
    ):
        raise TypeError(f"{name} must be a non-negative integer or null")


if __name__ == "__main__":
    raise SystemExit(main())
