#!/usr/bin/env python3
"""Validate Riotbox source timing fixture evaluation report JSON v1."""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.source_timing_fixture_evaluation_report.v1"
SCHEMA_VERSION = 1
KNOWN_ISSUES = {
    "missing_bpm_estimate",
    "bpm_outside_tolerance",
    "beat_count_below_minimum",
    "bar_count_below_minimum",
    "phrase_count_below_minimum",
    "quality_mismatch",
    "degraded_policy_mismatch",
    "primary_confidence_below_floor",
    "missing_timing_drift",
    "beat_drift_outside_tolerance",
    "downbeat_drift_outside_tolerance",
    "missing_primary_hypothesis",
}
ISSUES_WITH_VALUES = {
    "missing_warning",
    "missing_alternative",
}
TIMING_WARNING_CODES = {
    "WeakKickAnchor",
    "WeakBackbeatAnchor",
    "AmbiguousDownbeat",
    "HalfTimePossible",
    "DoubleTimePossible",
    "DriftHigh",
    "PhraseUncertain",
    "LowTimingConfidence",
}
TIMING_HYPOTHESIS_KINDS = {
    "Primary",
    "HalfTime",
    "DoubleTime",
    "AlternateDownbeat",
    "Ambiguous",
}


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate_source_timing_fixture_report_json.py <report.json>", file=sys.stderr)
        return 2

    path = Path(sys.argv[1])
    try:
        report = json.loads(path.read_text())
        validate_report(report)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid source timing fixture report JSON: {error}", file=sys.stderr)
        return 1

    print(f"valid {SCHEMA} report: {path}")
    return 0


def validate_report(report: Any) -> None:
    report = require_object(report, "report")
    require_equal(report, "schema", SCHEMA)
    require_equal(report, "schema_version", SCHEMA_VERSION)
    require_string(report, "catalog_path")
    case_count = require_non_negative_int(report, "case_count")
    passed = require_bool(report, "passed")
    evaluations = require_array(report, "evaluations")
    if case_count != len(evaluations):
        raise ValueError(f"case_count must match evaluations length, got {case_count} and {len(evaluations)}")
    if case_count == 0:
        raise ValueError("evaluations must contain at least one fixture evaluation")
    if "category_coverage" in report:
        validate_category_coverage(report["category_coverage"], case_count, passed)

    seen_ids: set[str] = set()
    evaluation_results = []
    for index, value in enumerate(evaluations):
        evaluation = require_object(value, f"evaluations[{index}]")
        evaluation_results.append(validate_evaluation(evaluation, seen_ids))

    expected_passed = all(evaluation_results)
    if passed != expected_passed:
        raise ValueError(f"passed must equal all evaluation results, expected {expected_passed!r}")


def validate_category_coverage(value: Any, case_count: int, report_passed: bool) -> None:
    coverage_items = require_array({"category_coverage": value}, "category_coverage")
    if not coverage_items:
        raise ValueError("category_coverage must contain at least one category")

    seen_categories: set[str] = set()
    covered_case_count = 0
    category_results = []
    for index, value in enumerate(coverage_items):
        coverage = require_object(value, f"category_coverage[{index}]")
        category = require_string(coverage, "category")
        if category in seen_categories:
            raise ValueError(f"duplicate category coverage {category!r}")
        seen_categories.add(category)
        covered_case_count += require_positive_int(coverage, "case_count")
        category_results.append(require_bool(coverage, "passed"))

    if covered_case_count != case_count:
        raise ValueError(
            f"category_coverage case counts must sum to case_count, got {covered_case_count} and {case_count}"
        )
    if report_passed != all(category_results):
        raise ValueError("category_coverage passed values must agree with report passed")


def validate_evaluation(evaluation: dict[str, Any], seen_ids: set[str]) -> bool:
    fixture_id = require_string(evaluation, "fixture_id")
    if fixture_id in seen_ids:
        raise ValueError(f"duplicate fixture_id {fixture_id!r}")
    seen_ids.add(fixture_id)

    passed = require_bool(evaluation, "passed")
    require_non_negative_number(evaluation, "bpm_error")
    require_non_negative_int(evaluation, "beat_count")
    require_non_negative_int(evaluation, "bar_count")
    require_non_negative_int(evaluation, "phrase_count")
    confidence = require_optional_number(evaluation, "primary_confidence")
    if confidence is not None and confidence > 1:
        raise ValueError("primary_confidence must be <= 1")
    require_optional_non_negative_number(evaluation, "primary_max_mean_abs_drift_ms")
    require_optional_non_negative_number(evaluation, "primary_max_drift_ms")

    issues = require_array(evaluation, "issues")
    for index, issue in enumerate(issues):
        validate_issue(issue, f"issues[{index}]")
    if passed and issues:
        raise ValueError(f"{fixture_id}: passed evaluations must not contain issues")
    if not passed and not issues:
        raise ValueError(f"{fixture_id}: failed evaluations must contain at least one issue")

    return passed


def validate_issue(issue: Any, name: str) -> None:
    if isinstance(issue, str):
        if issue not in KNOWN_ISSUES:
            raise ValueError(f"{name} must be a known issue code, got {issue!r}")
        return

    issue_object = require_object(issue, name)
    if len(issue_object) != 1:
        raise ValueError(f"{name} object issues must contain exactly one key")
    kind, value = next(iter(issue_object.items()))
    if kind not in ISSUES_WITH_VALUES:
        raise ValueError(f"{name} object issue must be one of {sorted(ISSUES_WITH_VALUES)}, got {kind!r}")
    if not isinstance(value, str) or not value:
        raise TypeError(f"{name}.{kind} must be a non-empty string")
    if kind == "missing_warning" and value not in TIMING_WARNING_CODES:
        raise ValueError(f"{name}.{kind} must be one of {sorted(TIMING_WARNING_CODES)}, got {value!r}")
    if kind == "missing_alternative" and value not in TIMING_HYPOTHESIS_KINDS:
        raise ValueError(
            f"{name}.{kind} must be one of {sorted(TIMING_HYPOTHESIS_KINDS)}, got {value!r}"
        )


def require_object(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{name} must be an object")
    return value


def require_equal(parent: dict[str, Any], field: str, expected: Any) -> None:
    actual = parent.get(field)
    if actual != expected:
        raise ValueError(f"{field} must be {expected!r}, got {actual!r}")


def require_bool(parent: dict[str, Any], field: str) -> bool:
    value = parent.get(field)
    if not isinstance(value, bool):
        raise TypeError(f"{field} must be a boolean")
    return value


def require_string(parent: dict[str, Any], field: str) -> str:
    value = parent.get(field)
    if not isinstance(value, str) or not value:
        raise TypeError(f"{field} must be a non-empty string")
    return value


def require_array(parent: dict[str, Any], field: str) -> list[Any]:
    value = parent.get(field)
    if not isinstance(value, list):
        raise TypeError(f"{field} must be an array")
    return value


def require_optional_number(parent: dict[str, Any], field: str) -> float | int | None:
    if field not in parent:
        raise TypeError(f"{field} must be present as a number or null")
    value = parent.get(field)
    if value is None:
        return None
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
        raise TypeError(f"{field} must be a finite number or null")
    return value


def require_non_negative_number(parent: dict[str, Any], field: str) -> float | int:
    value = require_optional_number(parent, field)
    if value is None or value < 0:
        raise ValueError(f"{field} must be a non-negative number")
    return value


def require_optional_non_negative_number(parent: dict[str, Any], field: str) -> float | int | None:
    value = require_optional_number(parent, field)
    if value is not None and value < 0:
        raise ValueError(f"{field} must be a non-negative number or null")
    return value


def require_non_negative_int(parent: dict[str, Any], field: str) -> int:
    value = parent.get(field)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise TypeError(f"{field} must be a non-negative integer")
    return value


def require_positive_int(parent: dict[str, Any], field: str) -> int:
    value = require_non_negative_int(parent, field)
    if value == 0:
        raise ValueError(f"{field} must be > 0")
    return value


if __name__ == "__main__":
    raise SystemExit(main())
