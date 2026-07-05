#!/usr/bin/env python3
"""Validate destructive-variation professional smoke artifacts."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any, Callable

import validate_destructive_variation_professional as destructive_variation


DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-destructive-variation-professional")
DEFAULT_INVALID_FLAT_STUTTER = Path(
    "scripts/fixtures/destructive_variation_professional/invalid_flat_stutter/performance-report.json"
)
REQUIRED_INVALID_FAILURE_CODES = {
    "dropout_not_contrasting_with_stutter",
    "dropout_silence_not_deep_enough_before_stutter",
    "stutter_lacks_transient_impact",
    "restore_does_not_slam_out_of_cut",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--report-json", type=Path)
    parser.add_argument("--report-markdown", type=Path)
    parser.add_argument("--invalid-flat-stutter", type=Path, default=DEFAULT_INVALID_FLAT_STUTTER)
    args = parser.parse_args()

    report_json = args.report_json or args.output / "destructive-variation.json"
    report_markdown = args.report_markdown or args.output / "destructive-variation.md"
    try:
        report = read_json_object(report_json)
        validate_positive_report(report)
        validate_markdown(report_markdown)
        validate_mutation_fixtures(report)
        validate_invalid_flat_stutter(args.invalid_flat_stutter)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid destructive variation professional smoke: {error}", file=sys.stderr)
        return 1

    print(f"valid destructive variation professional smoke: {report_json}")
    return 0


def validate_positive_report(report: dict[str, Any]) -> None:
    require(report.get("schema") == destructive_variation.SCHEMA, "schema mismatch")
    require(report.get("schema_version") == 1, "schema version mismatch")
    require(report.get("result") == "pass", "result must be pass")
    require(report.get("agent_verdict") == "agent_promising", "agent verdict mismatch")
    require(report.get("human_verdict") == "unverified", "human verdict must be unverified")
    require(report.get("evidence_role") == "diagnostic", "evidence role mismatch")
    require(report.get("source_backed") is True, "report must be source-backed")
    require(report.get("source_timing_backed") is True, "report must be source-timing-backed")
    require(report.get("scripted_generation") is True, "scripted generation flag missing")
    require(report.get("quality_proof") is False, "report must not claim quality")
    metrics = object_field(report, "metrics")
    thresholds = object_field(report, "thresholds")
    require_at_most(metrics, thresholds, "dropout_to_stutter_rms_ratio", "max_dropout_to_stutter_rms_ratio")
    require_at_most(
        metrics,
        thresholds,
        "dropout_silence_to_stutter_rms_ratio",
        "max_dropout_silence_to_stutter_rms_ratio",
    )
    require_any_at_least(
        metrics,
        thresholds,
        [
            ("stutter_to_hook_transient_ratio", "min_stutter_to_hook_transient_ratio"),
            ("stutter_to_source_transient_ratio", "min_stutter_to_source_transient_ratio"),
        ],
        "stutter lacks transient impact",
    )
    require_any_at_least(
        metrics,
        thresholds,
        [
            ("restore_to_hook_transient_ratio", "min_restore_to_hook_transient_ratio"),
            ("restore_to_source_transient_ratio", "min_restore_to_source_transient_ratio"),
        ],
        "restore lacks transient impact",
    )
    require_at_least(
        metrics,
        thresholds,
        "restore_to_dropout_silence_rms_ratio",
        "min_restore_to_dropout_silence_rms_ratio",
    )
    require_at_least(metrics, thresholds, "restore_hit_rms", "min_restore_rms")
    require(number(metrics.get("destructive_gesture_source_derived")) == 1.0, "destructive gesture not source-derived")
    require_at_least(
        metrics,
        thresholds,
        "destructive_static_distance_frames",
        "min_destructive_static_distance_frames",
    )
    require_at_least(
        metrics,
        thresholds,
        "destructive_offset_distance_frames",
        "min_destructive_offset_distance_frames",
    )


def validate_markdown(path: Path) -> None:
    markdown = path.read_text()
    require("Destructive Variation Professional Report" in markdown, "markdown title missing")
    require("diagnostic evidence, not product-quality proof" in markdown, "markdown boundary missing")


def validate_mutation_fixtures(report: dict[str, Any]) -> None:
    expect_report_failure(
        report,
        "dropout_ratio_stale",
        lambda data: set_nested_number(
            data,
            "metrics",
            "dropout_to_stutter_rms_ratio",
            number(object_field(data, "thresholds").get("max_dropout_to_stutter_rms_ratio")) + 1.0,
        ),
        "dropout_to_stutter_rms_ratio above max_dropout_to_stutter_rms_ratio",
    )
    expect_report_failure(
        report,
        "quality_claim",
        lambda data: set_field(data, "quality_proof", True),
        "report must not claim quality",
    )
    expect_report_failure(
        report,
        "not_source_derived",
        lambda data: set_nested_number(data, "metrics", "destructive_gesture_source_derived", 0.0),
        "destructive gesture not source-derived",
    )


def validate_invalid_flat_stutter(path: Path) -> None:
    report = destructive_variation.build_report(path)
    require(report.get("result") == "fail", "invalid flat-stutter fixture unexpectedly passed")
    failure_codes = set(string_list(report.get("failure_codes")))
    missing = sorted(REQUIRED_INVALID_FAILURE_CODES - failure_codes)
    require(not missing, f"invalid flat-stutter failure codes missing: {', '.join(missing)}")


def expect_report_failure(
    report: dict[str, Any],
    fixture_name: str,
    mutate: Callable[[dict[str, Any]], object],
    expected_message: str,
) -> None:
    mutated = copy.deepcopy(report)
    mutate(mutated)
    try:
        validate_positive_report(mutated)
    except ValueError as error:
        require(
            expected_message in str(error),
            f"{fixture_name}: expected {expected_message}, got {error}",
        )
    else:
        raise ValueError(f"{fixture_name}: mutation unexpectedly passed")


def require_at_most(
    metrics: dict[str, Any],
    thresholds: dict[str, Any],
    metric_name: str,
    threshold_name: str,
) -> None:
    require(
        number(metrics.get(metric_name)) <= number(thresholds.get(threshold_name)),
        f"{metric_name} above {threshold_name}",
    )


def require_at_least(
    metrics: dict[str, Any],
    thresholds: dict[str, Any],
    metric_name: str,
    threshold_name: str,
) -> None:
    require(
        number(metrics.get(metric_name)) >= number(thresholds.get(threshold_name)),
        f"{metric_name} below {threshold_name}",
    )


def require_any_at_least(
    metrics: dict[str, Any],
    thresholds: dict[str, Any],
    pairs: list[tuple[str, str]],
    message: str,
) -> None:
    require(
        any(number(metrics.get(metric)) >= number(thresholds.get(threshold)) for metric, threshold in pairs),
        message,
    )


def read_json_object(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    require(isinstance(data, dict), f"{path} must contain a JSON object")
    return data


def object_field(data: dict[str, Any], field: str) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{field} must be object")
    return value


def string_list(value: Any) -> list[str]:
    return [str(item) for item in value] if isinstance(value, list) else []


def set_field(data: dict[str, Any], field: str, value: Any) -> None:
    data[field] = value


def set_nested_number(data: dict[str, Any], object_name: str, field: str, value: float) -> None:
    object_field(data, object_name)[field] = value


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
