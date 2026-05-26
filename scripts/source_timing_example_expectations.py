"""Expectation loading and comparison for Source Timing example reports."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


EXPECTATION_KEYS = {
    "cue",
    "actionability",
    "readiness",
    "requires_manual_confirm",
    "grid_use",
    "confidence_result",
    "drift_status",
    "beat_status",
    "primary_beat_count",
    "primary_bar_count",
    "downbeat_status",
    "phrase_status",
    "primary_phrase_count",
    "primary_phrase_bar_count",
    "alternate_evidence_count",
    "alternate_beat_candidate_count",
    "alternate_downbeat_phase_count",
    "primary_downbeat_offset_beats",
    "primary_bpm",
    "primary_beat_score",
    "primary_beat_matched_onset_ratio",
    "primary_beat_median_distance_ratio",
    "primary_downbeat_score",
    "primary_downbeat_margin",
    "anchor_evidence",
    "warning_codes_include",
    "warning_codes_exact",
}

ANCHOR_EVIDENCE_KEYS = {
    "primary_anchor_count",
    "primary_kick_anchor_count",
    "primary_backbeat_anchor_count",
    "primary_transient_anchor_count",
}


def load_expectations(path: Path) -> dict[str, dict[str, Any]]:
    payload = require_object(json.loads(path.read_text()), str(path))
    sources = require_object(payload.get("sources"), "expectations.sources")
    expectations = {}
    for source, expectation in sources.items():
        if not isinstance(source, str) or not source:
            raise TypeError("expectation source keys must be non-empty strings")
        expectation_object = require_object(expectation, f"expectations[{source}]")
        require_known_expectation_keys(expectation_object, f"expectations[{source}]")
        require_warning_expectation_mode(expectation_object, f"expectations[{source}]")
        expectations[source] = expectation_object
    return expectations


def format_expectation(
    source_name: str,
    payload: dict[str, Any],
    expectations: dict[str, dict[str, Any]],
) -> str:
    expectation = expectations.get(source_name)
    if expectation is None:
        return "-"

    issues = expectation_issues(payload, expectation)
    if not issues:
        return "ok"
    return "mismatch: " + "; ".join(issues)


def expectation_issues(payload: dict[str, Any], expectation: dict[str, Any]) -> list[str]:
    require_known_expectation_keys(expectation, "expectation")
    require_warning_expectation_mode(expectation, "expectation")
    issues = []
    compare_string(payload, expectation, issues, "cue")
    compare_string(payload, expectation, issues, "actionability")
    compare_string(payload, expectation, issues, "readiness")
    compare_bool(payload, expectation, issues, "requires_manual_confirm")
    compare_string(payload, expectation, issues, "grid_use")
    compare_string(payload, expectation, issues, "confidence_result")
    compare_string(payload, expectation, issues, "drift_status")
    compare_string(payload, expectation, issues, "beat_status")
    compare_int(payload, expectation, issues, "primary_beat_count")
    compare_int(payload, expectation, issues, "primary_bar_count")
    compare_string(payload, expectation, issues, "downbeat_status")
    compare_string(payload, expectation, issues, "phrase_status")
    compare_int(payload, expectation, issues, "primary_phrase_count")
    compare_int(payload, expectation, issues, "primary_phrase_bar_count")
    compare_int(payload, expectation, issues, "alternate_evidence_count")
    compare_int(payload, expectation, issues, "alternate_beat_candidate_count")
    compare_int(payload, expectation, issues, "alternate_downbeat_phase_count")
    compare_optional_int(payload, expectation, issues, "primary_downbeat_offset_beats")
    compare_bpm(payload, expectation, issues)
    compare_number_range(payload, expectation, issues, "primary_beat_score")
    compare_number_range(payload, expectation, issues, "primary_beat_matched_onset_ratio")
    compare_number_range(payload, expectation, issues, "primary_beat_median_distance_ratio")
    compare_number_range(payload, expectation, issues, "primary_downbeat_score")
    compare_number_range(payload, expectation, issues, "primary_downbeat_margin")
    compare_anchor_evidence(payload, expectation, issues)
    compare_warning_includes(payload, expectation, issues)
    compare_warning_exact(payload, expectation, issues)
    return issues


def require_known_expectation_keys(expectation: dict[str, Any], label: str) -> None:
    unknown_keys = sorted(set(expectation) - EXPECTATION_KEYS)
    if unknown_keys:
        raise ValueError(f"{label} has unknown keys: {', '.join(unknown_keys)}")


def require_warning_expectation_mode(expectation: dict[str, Any], label: str) -> None:
    if "warning_codes_include" in expectation and "warning_codes_exact" in expectation:
        raise ValueError(
            f"{label} must not mix warning_codes_include and warning_codes_exact"
        )


def compare_string(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = require_string(expectation, key)
    actual = require_string(payload, key)
    if actual != expected:
        issues.append(f"{key} expected {expected!r} got {actual!r}")


def compare_bool(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = require_bool(expectation, key)
    actual = require_bool(payload, key)
    if actual != expected:
        issues.append(f"{key} expected {expected!r} got {actual!r}")


def compare_int(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = require_int(expectation, key)
    actual = require_int(payload, key)
    if actual != expected:
        issues.append(f"{key} expected {expected!r} got {actual!r}")


def compare_optional_int(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = expectation.get(key)
    if expected is not None and (type(expected) is not int or expected < 0):
        raise TypeError(f"{key} must be a non-negative integer or null")
    actual = payload.get(key)
    if actual is not None and (type(actual) is not int or actual < 0):
        raise TypeError(f"{key} must be a non-negative integer or null")
    if actual != expected:
        issues.append(f"{key} expected {expected!r} got {actual!r}")


def compare_bpm(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
) -> None:
    if "primary_bpm" not in expectation:
        return
    expected = require_object(expectation["primary_bpm"], "primary_bpm expectation")
    target = require_number(expected, "target")
    tolerance = require_number(expected, "tolerance")
    if tolerance < 0:
        raise ValueError("primary_bpm tolerance must be non-negative")
    actual = payload.get("primary_bpm")
    if type(actual) not in {int, float}:
        issues.append("primary_bpm expected numeric value got none")
        return
    delta = abs(float(actual) - target)
    if delta > tolerance:
        issues.append(
            f"primary_bpm delta {delta:.6f} exceeds tolerance {tolerance:.6f}"
        )


def compare_number_range(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
    key: str,
) -> None:
    if key not in expectation:
        return
    expected = require_number_range_expectation(expectation[key], f"{key} expectation")
    minimum = require_number(expected, "min") if "min" in expected else None
    maximum = require_number(expected, "max") if "max" in expected else None
    actual = payload.get(key)
    if type(actual) not in {int, float}:
        issues.append(f"{key} expected numeric value got none")
        return
    actual_value = float(actual)
    if minimum is not None and actual_value < minimum:
        issues.append(f"{key} {actual_value:.6f} below minimum {minimum:.6f}")
    if maximum is not None and actual_value > maximum:
        issues.append(f"{key} {actual_value:.6f} above maximum {maximum:.6f}")


def require_number_range_expectation(value: Any, label: str) -> dict[str, Any]:
    expected = require_object(value, label)
    allowed_keys = {"min", "max"}
    unknown_keys = sorted(set(expected) - allowed_keys)
    if unknown_keys:
        raise ValueError(f"{label} has unknown keys: {', '.join(unknown_keys)}")
    if "min" not in expected and "max" not in expected:
        raise ValueError(f"{label} must include min or max")
    minimum = require_number(expected, "min") if "min" in expected else None
    maximum = require_number(expected, "max") if "max" in expected else None
    if minimum is not None and maximum is not None and minimum > maximum:
        raise ValueError(f"{label} min must be <= max")
    return expected


def compare_warning_includes(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
) -> None:
    if "warning_codes_include" not in expectation:
        return
    expected_warnings = require_string_list(expectation, "warning_codes_include")
    actual_warnings = set(require_string_list(payload, "warning_codes"))
    for warning in expected_warnings:
        if warning not in actual_warnings:
            issues.append(f"warning_codes missing {warning!r}")


def compare_warning_exact(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
) -> None:
    if "warning_codes_exact" not in expectation:
        return
    expected_warnings = sorted(
        set(require_string_list(expectation, "warning_codes_exact"))
    )
    actual_warnings = sorted(set(require_string_list(payload, "warning_codes")))
    if actual_warnings != expected_warnings:
        message = (
            f"warning_codes expected exactly {expected_warnings!r} "
            f"got {actual_warnings!r}"
        )
        issues.append(message)


def compare_anchor_evidence(
    payload: dict[str, Any],
    expectation: dict[str, Any],
    issues: list[str],
) -> None:
    if "anchor_evidence" not in expectation:
        return
    expected = require_anchor_evidence_expectation(expectation["anchor_evidence"])
    actual = require_object(payload.get("anchor_evidence"), "anchor_evidence")
    for key in sorted(expected):
        expected_value = expected[key]
        actual_value = require_int(actual, key)
        if actual_value != expected_value:
            issues.append(
                f"anchor_evidence.{key} expected {expected_value!r} got {actual_value!r}"
            )


def require_anchor_evidence_expectation(value: Any) -> dict[str, int]:
    expected = require_object(value, "anchor_evidence expectation")
    unknown_keys = sorted(set(expected) - ANCHOR_EVIDENCE_KEYS)
    if unknown_keys:
        raise ValueError(
            f"anchor_evidence expectation has unknown keys: {', '.join(unknown_keys)}"
        )
    if not expected:
        raise ValueError("anchor_evidence expectation must include at least one key")
    return {
        key: require_int(expected, key)
        for key in sorted(expected)
    }


def require_object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise TypeError(f"{label} must be an object")
    return value


def require_string(value: dict[str, Any], key: str) -> str:
    item = value.get(key)
    if not isinstance(item, str) or not item:
        raise TypeError(f"{key} must be a non-empty string")
    return item


def require_bool(value: dict[str, Any], key: str) -> bool:
    item = value.get(key)
    if not isinstance(item, bool):
        raise TypeError(f"{key} must be a boolean")
    return item


def require_number(value: dict[str, Any], key: str) -> float:
    item = value.get(key)
    if type(item) not in {int, float}:
        raise TypeError(f"{key} must be a number")
    return float(item)


def require_int(value: dict[str, Any], key: str) -> int:
    item = value.get(key)
    if type(item) is not int or item < 0:
        raise TypeError(f"{key} must be a non-negative integer")
    return item


def require_string_list(value: dict[str, Any], key: str) -> list[str]:
    item = value.get(key)
    if not isinstance(item, list) or any(
        not isinstance(entry, str) or not entry for entry in item
    ):
        raise TypeError(f"{key} must be a list of non-empty strings")
    return item
