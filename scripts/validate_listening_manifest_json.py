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
SOURCE_TIMING_BPM_MATCH_TOLERANCE = 1.0
EPSILON = 0.000001
SOURCE_TIMING_POLICY_PROFILES = {"broad_research", "dance_loop_auto_readiness"}
SOURCE_TIMING_READINESS = {"unavailable", "weak", "needs_review", "ready"}
SOURCE_TIMING_GRID_USE = {
    "locked_grid",
    "short_loop_manual_confirm",
    "manual_confirm_only",
    "fallback_grid",
    "unavailable",
}
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
GROOVE_SUBDIVISIONS = {
    "eighth",
    "triplet",
    "sixteenth",
    "thirty_second",
}
GRID_BPM_DECISION_REASONS = {
    "user_override",
    "source_timing_ready",
    "source_timing_needs_review_manual_confirm",
    "source_timing_requires_manual_confirm",
    "source_timing_not_ready",
    "source_timing_missing_bpm",
    "source_timing_invalid_bpm",
}
GRID_BPM_SOURCES = {
    "user_override",
    "source_timing",
    "static_default",
}
TR909_GROOVE_TIMING_REASONS = {
    "not_source_timing_grid",
    "no_groove_residuals",
    "invalid_groove_offset",
    "groove_offset_too_small",
    "source_timing_groove_residual",
    "source_timing_not_locked",
}
STATIC_DEFAULT_GRID_BPM_REASONS = {
    "source_timing_requires_manual_confirm",
    "source_timing_not_ready",
    "source_timing_missing_bpm",
    "source_timing_invalid_bpm",
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

    if manifest.get("pack_id") == "feral-grid-demo" and "grid_bpm_source" in manifest:
        validate_grid_bpm_decision(manifest, source_timing)
        validate_source_timing_bpm_delta_consistency(manifest, source_timing)

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
    if "grid_use" in source_timing:
        require_one_of(source_timing, "grid_use", SOURCE_TIMING_GRID_USE)
        require_source_timing_grid_use_match(source_timing, source_timing["grid_use"])
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
    validate_source_timing_anchor_evidence(source_timing.get("anchor_evidence"))
    validate_source_timing_groove_evidence(source_timing.get("groove_evidence"))
    require_non_negative_int(
        source_timing,
        "alternate_evidence_count",
        "source_timing",
    )
    require_string_list(source_timing, "warning_codes", "source_timing warning_codes")


def require_source_timing_grid_use_match(source_timing: dict[str, Any], grid_use: str) -> None:
    expected = source_timing_grid_use(source_timing)
    if grid_use != expected:
        raise ValueError(f"source_timing grid_use must be {expected!r}, got {grid_use!r}")


def source_timing_grid_use(source_timing: dict[str, Any]) -> str:
    if source_timing.get("primary_bpm") is None or source_timing["readiness"] == "unavailable":
        return "unavailable"
    if source_timing["readiness"] == "ready" and not source_timing["requires_manual_confirm"]:
        return "locked_grid"
    if is_stable_short_loop_manual_confirm(source_timing):
        return "short_loop_manual_confirm"
    if source_timing["requires_manual_confirm"]:
        return "manual_confirm_only"
    return "fallback_grid"


def is_stable_short_loop_manual_confirm(source_timing: dict[str, Any]) -> bool:
    return (
        source_timing["readiness"] == "needs_review"
        and source_timing["requires_manual_confirm"] is True
        and source_timing.get("primary_bpm") is not None
        and source_timing["beat_status"] == "stable"
        and source_timing["downbeat_status"] == "stable"
        and source_timing["phrase_status"] == "not_enough_material"
        and source_timing["confidence_result"] == "candidate_cautious"
        and source_timing["alternate_evidence_count"] == 0
    )


def validate_source_timing_anchor_evidence(anchor_evidence: Any) -> None:
    evidence = require_object(anchor_evidence, "source_timing anchor_evidence")
    require_non_negative_int(
        evidence,
        "primary_anchor_count",
        "source_timing anchor_evidence",
    )
    require_non_negative_int(
        evidence,
        "primary_kick_anchor_count",
        "source_timing anchor_evidence",
    )
    require_non_negative_int(
        evidence,
        "primary_backbeat_anchor_count",
        "source_timing anchor_evidence",
    )
    require_non_negative_int(
        evidence,
        "primary_transient_anchor_count",
        "source_timing anchor_evidence",
    )
    typed_count = (
        evidence["primary_kick_anchor_count"]
        + evidence["primary_backbeat_anchor_count"]
        + evidence["primary_transient_anchor_count"]
    )
    if typed_count > evidence["primary_anchor_count"]:
        raise ValueError(
            "source_timing anchor_evidence typed anchor counts must not exceed primary_anchor_count"
        )


def validate_source_timing_groove_evidence(groove_evidence: Any) -> None:
    evidence = require_object(groove_evidence, "source_timing groove_evidence")
    require_non_negative_int(
        evidence,
        "primary_groove_residual_count",
        "source_timing groove_evidence",
    )
    total = evidence["primary_groove_residual_count"]
    require_non_negative_number(
        evidence,
        "primary_max_abs_offset_ms",
        "source_timing groove_evidence",
    )
    preview = require_list(
        evidence,
        "primary_groove_preview",
        "source_timing groove_evidence primary_groove_preview",
    )
    if len(preview) > min(total, 4):
        raise ValueError(
            "source_timing groove_evidence preview must contain at most the first four residuals"
        )
    for index, item in enumerate(preview):
        validate_source_timing_groove_preview(item, index)


def validate_source_timing_groove_preview(item: Any, index: int) -> None:
    residual = require_object(item, f"source_timing groove residual {index}")
    require_one_of(residual, "subdivision", GROOVE_SUBDIVISIONS)
    require_number(residual, "offset_ms", f"source_timing groove residual {index} offset_ms")
    require_non_negative_number(
        residual,
        "confidence",
        f"source_timing groove residual {index} confidence",
    )
    confidence = residual["confidence"]
    if confidence > 1.0:
        raise ValueError("source_timing groove residual confidence must be <= 1")


def validate_grid_bpm_decision(
    manifest: dict[str, Any], source_timing: Any | None
) -> None:
    require_one_of(manifest, "grid_bpm_source", GRID_BPM_SOURCES)
    require_one_of(manifest, "grid_bpm_decision_reason", GRID_BPM_DECISION_REASONS)
    source = manifest.get("grid_bpm_source")
    reason = manifest.get("grid_bpm_decision_reason")

    if source == "user_override" and reason != "user_override":
        raise ValueError("user_override grid BPM source requires user_override decision reason")
    if source == "source_timing" and reason not in {
        "source_timing_ready",
        "source_timing_needs_review_manual_confirm",
    }:
        raise ValueError("source_timing grid BPM source requires a source-timing decision reason")
    if source == "static_default" and reason not in STATIC_DEFAULT_GRID_BPM_REASONS:
        raise ValueError("static_default grid BPM source requires a source-timing fallback reason")

    if not isinstance(source_timing, dict):
        return
    if reason == "source_timing_ready":
        if source_timing.get("readiness") != "ready":
            raise ValueError("source_timing_ready requires source_timing.readiness == ready")
        if source_timing.get("requires_manual_confirm") is not False:
            raise ValueError(
                "source_timing_ready requires source_timing.requires_manual_confirm == false"
            )
    if reason == "source_timing_requires_manual_confirm":
        if source_timing.get("requires_manual_confirm") is not True:
            raise ValueError(
                "source_timing_requires_manual_confirm requires manual confirmation evidence"
            )
    if reason == "source_timing_needs_review_manual_confirm":
        if source_timing.get("readiness") != "needs_review":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires source_timing.readiness == needs_review"
            )
        if source_timing.get("requires_manual_confirm") is not True:
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires manual confirmation evidence"
            )
        if source_timing.get("beat_status") != "stable":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires stable beat evidence"
            )
        if source_timing.get("downbeat_status") != "stable":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires stable downbeat evidence"
            )
        if source_timing.get("confidence_result") != "candidate_cautious":
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires candidate_cautious confidence"
            )
        if source_timing.get("alternate_evidence_count") != 0:
            raise ValueError(
                "source_timing_needs_review_manual_confirm requires no alternate evidence"
            )
    if reason == "source_timing_not_ready" and source_timing.get("readiness") == "ready":
        raise ValueError("source_timing_not_ready cannot be used with ready source timing")


def validate_source_timing_bpm_delta_consistency(
    manifest: dict[str, Any], source_timing: Any | None
) -> None:
    if "source_timing_bpm_delta" not in manifest:
        raise TypeError("source_timing_bpm_delta must be present as a number or null")
    require_optional_float_or_null(
        manifest,
        "source_timing_bpm_delta",
        "source_timing_bpm_delta",
    )
    source = manifest.get("grid_bpm_source")
    reason = manifest.get("grid_bpm_decision_reason")
    delta = manifest.get("source_timing_bpm_delta")

    if source == "source_timing":
        if not isinstance(delta, (int, float)) or isinstance(delta, bool):
            raise TypeError("source_timing grid BPM source requires numeric source_timing_bpm_delta")
        if abs(float(delta)) > EPSILON:
            raise ValueError("source_timing grid BPM source requires source_timing_bpm_delta == 0")
        require_bpm_agreement(source_timing, True, "source_timing grid BPM source")
        return

    if reason in {"source_timing_missing_bpm", "source_timing_invalid_bpm"}:
        if delta is not None:
            raise ValueError(f"{reason} requires null source_timing_bpm_delta")
        require_bpm_agreement(source_timing, None, reason)
        return

    if source == "user_override" and delta is None:
        require_bpm_agreement(source_timing, None, "user_override without usable source BPM")
        return

    if source in {"static_default", "user_override"}:
        if not isinstance(delta, (int, float)) or isinstance(delta, bool):
            raise TypeError(
                f"{source}/{reason} requires numeric source_timing_bpm_delta when source BPM is usable"
            )
        expected_agrees = float(delta) <= SOURCE_TIMING_BPM_MATCH_TOLERANCE
        require_bpm_agreement(source_timing, expected_agrees, f"{source}/{reason}")


def require_bpm_agreement(
    source_timing: Any | None, expected: bool | None, context: str
) -> None:
    if not isinstance(source_timing, dict):
        return
    actual = source_timing.get("bpm_agrees_with_grid")
    if actual is not expected:
        raise ValueError(
            f"{context} requires source_timing.bpm_agrees_with_grid == {expected!r}"
        )


def validate_source_grid_output_drift(manifest: dict[str, Any]) -> None:
    metrics = manifest.get("metrics")
    if not isinstance(metrics, dict):
        return

    for metric_key in (
        "source_grid_output_drift",
        "tr909_source_grid_alignment",
        "w30_source_grid_alignment",
    ):
        if metric_key in metrics:
            validate_source_grid_output_drift_metric(metrics, metric_key)

    if "tr909_groove_timing" in metrics:
        validate_tr909_groove_timing(metrics["tr909_groove_timing"])


def validate_tr909_groove_timing(value: Any) -> None:
    timing = require_object(value, "metrics tr909_groove_timing")
    require_bool(timing, "applied", "metrics tr909_groove_timing")
    require_one_of(timing, "reason", TR909_GROOVE_TIMING_REASONS)
    require_number(timing, "offset_ms", "metrics tr909_groove_timing offset_ms")
    require_non_negative_int(
        timing,
        "source_residual_count",
        "metrics tr909_groove_timing",
    )
    require_non_negative_number(
        timing,
        "source_max_abs_offset_ms",
        "metrics tr909_groove_timing source_max_abs_offset_ms",
    )
    subdivision = timing.get("source_subdivision")
    if subdivision is not None and subdivision not in GROOVE_SUBDIVISIONS:
        raise ValueError(
            "metrics tr909_groove_timing source_subdivision must be a known groove subdivision or null"
        )
    if timing["applied"] and timing["reason"] != "source_timing_groove_residual":
        raise ValueError("applied tr909_groove_timing requires source_timing_groove_residual")
    if not timing["applied"] and timing["offset_ms"] != 0:
        raise ValueError("inactive tr909_groove_timing must use offset_ms 0")


def validate_source_grid_output_drift_metric(metrics: dict[str, Any], metric_key: str) -> None:
    drift = require_object(
        metrics.get(metric_key),
        f"metrics {metric_key}",
    )
    require_non_negative_int(drift, "beat_count", metric_key)
    require_non_negative_int(drift, "hit_count", metric_key)
    if drift["hit_count"] > drift["beat_count"]:
        raise ValueError(f"{metric_key} hit_count must not exceed beat_count")
    hit_ratio = require_number(drift, "hit_ratio", f"{metric_key} hit_ratio")
    if hit_ratio < 0.0 or hit_ratio > 1.0:
        raise ValueError(f"{metric_key} hit_ratio must be between 0 and 1")
    require_non_negative_number(
        drift,
        "max_peak_offset_ms",
        f"{metric_key} max_peak_offset_ms",
    )
    require_non_negative_number(
        drift,
        "max_allowed_peak_offset_ms",
        f"{metric_key} max_allowed_peak_offset_ms",
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
