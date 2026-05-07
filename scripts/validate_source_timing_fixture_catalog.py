#!/usr/bin/env python3
"""Validate the P012 source timing fixture catalog."""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path


SCHEMA = "riotbox.source_timing_fixture_catalog.v1"
QUALITY = {"low", "medium", "high", "unknown"}
POLICY = {"locked", "cautious", "manual_confirm", "fallback_grid", "disabled", "unknown"}
CATEGORIES = {
    "clean_rhythm",
    "dense_break",
    "hook_forward",
    "half_time_ambiguity",
    "double_time_ambiguity",
    "weak_timing",
}
WARNINGS = {
    "weak_kick_anchor",
    "weak_backbeat_anchor",
    "ambiguous_downbeat",
    "half_time_possible",
    "double_time_possible",
    "drift_high",
    "phrase_uncertain",
    "low_timing_confidence",
}
ALTERNATIVE_KINDS = {
    "half_time",
    "double_time",
    "alternate_downbeat",
    "ambiguous",
}


def fail(message: str) -> None:
    raise SystemExit(f"invalid source timing fixture catalog: {message}")


def finite_number(value: object, field: str, *, minimum: float | None = None) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
        fail(f"{field} must be a finite number")
    number = float(value)
    if minimum is not None and number < minimum:
        fail(f"{field} must be >= {minimum}")
    return number


def finite_int(value: object, field: str, *, minimum: int | None = None) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        fail(f"{field} must be an integer")
    if minimum is not None and value < minimum:
        fail(f"{field} must be >= {minimum}")
    return value


def validate_case(case: object, seen_ids: set[str]) -> str:
    if not isinstance(case, dict):
        fail("each case must be an object")

    fixture_id = case.get("fixture_id")
    if not isinstance(fixture_id, str) or not fixture_id:
        fail("fixture_id must be a non-empty string")
    if fixture_id in seen_ids:
        fail(f"duplicate fixture_id {fixture_id!r}")
    seen_ids.add(fixture_id)

    category = case.get("category")
    if category not in CATEGORIES:
        fail(f"{fixture_id}: category must be one of {sorted(CATEGORIES)}")

    title = case.get("title")
    if not isinstance(title, str) or not title:
        fail(f"{fixture_id}: title must be a non-empty string")

    finite_number(case.get("duration_seconds"), f"{fixture_id}: duration_seconds", minimum=1.0)

    expected = case.get("expected")
    if not isinstance(expected, dict):
        fail(f"{fixture_id}: expected must be an object")

    bpm = finite_number(expected.get("primary_bpm"), f"{fixture_id}: primary_bpm", minimum=1.0)
    if bpm > 320.0:
        fail(f"{fixture_id}: primary_bpm is outside the supported seed range")
    finite_number(expected.get("bpm_tolerance"), f"{fixture_id}: bpm_tolerance", minimum=0.0)

    meter = expected.get("meter")
    if not isinstance(meter, dict):
        fail(f"{fixture_id}: meter must be an object")
    beats_per_bar = finite_int(meter.get("beats_per_bar"), f"{fixture_id}: beats_per_bar", minimum=1)
    beat_unit = finite_int(meter.get("beat_unit"), f"{fixture_id}: beat_unit", minimum=1)
    if beats_per_bar > 16 or beat_unit not in {2, 4, 8, 16}:
        fail(f"{fixture_id}: meter is outside the bounded fixture seed range")

    quality = expected.get("timing_quality")
    if quality not in QUALITY:
        fail(f"{fixture_id}: timing_quality must be one of {sorted(QUALITY)}")

    policy = expected.get("degraded_policy")
    if policy not in POLICY:
        fail(f"{fixture_id}: degraded_policy must be one of {sorted(POLICY)}")

    finite_number(
        expected.get("beat_hit_tolerance_ms"),
        f"{fixture_id}: beat_hit_tolerance_ms",
        minimum=0.0,
    )
    finite_number(
        expected.get("downbeat_tolerance_ms"),
        f"{fixture_id}: downbeat_tolerance_ms",
        minimum=0.0,
    )
    finite_int(
        expected.get("phrase_tolerance_bars"),
        f"{fixture_id}: phrase_tolerance_bars",
        minimum=0,
    )
    finite_int(
        expected.get("expected_beat_count_min"),
        f"{fixture_id}: expected_beat_count_min",
        minimum=0,
    )
    finite_int(
        expected.get("expected_bar_count_min"),
        f"{fixture_id}: expected_bar_count_min",
        minimum=0,
    )
    finite_int(
        expected.get("expected_phrase_count_min"),
        f"{fixture_id}: expected_phrase_count_min",
        minimum=0,
    )
    confidence_floor = finite_number(
        expected.get("confidence_floor"),
        f"{fixture_id}: confidence_floor",
        minimum=0.0,
    )
    if confidence_floor > 1.0:
        fail(f"{fixture_id}: confidence_floor must be <= 1.0")

    warnings = expected.get("warnings")
    if not isinstance(warnings, list) or any(warning not in WARNINGS for warning in warnings):
        fail(f"{fixture_id}: warnings must be known warning strings")

    alternatives = expected.get("alternatives")
    if not isinstance(alternatives, list):
        fail(f"{fixture_id}: alternatives must be an array")
    for index, alternative in enumerate(alternatives):
        if not isinstance(alternative, dict):
            fail(f"{fixture_id}: alternative {index} must be an object")
        if alternative.get("kind") not in ALTERNATIVE_KINDS:
            fail(f"{fixture_id}: alternative {index} has unknown kind")
        finite_number(alternative.get("bpm"), f"{fixture_id}: alternative {index} bpm", minimum=1.0)
        alt_confidence = finite_number(
            alternative.get("confidence_floor"),
            f"{fixture_id}: alternative {index} confidence_floor",
            minimum=0.0,
        )
        if alt_confidence > 1.0:
            fail(f"{fixture_id}: alternative {index} confidence_floor must be <= 1.0")

    if category == "clean_rhythm" and (quality != "high" or policy != "locked"):
        fail(f"{fixture_id}: clean rhythm seeds must be high/locked")
    if category == "weak_timing" and quality == "high":
        fail(f"{fixture_id}: weak timing seeds must not be high quality")
    if "ambiguity" in category and not alternatives:
        fail(f"{fixture_id}: ambiguity seeds must preserve at least one alternative")
    if policy == "locked" and warnings:
        fail(f"{fixture_id}: locked seeds must not carry timing warnings")

    return fixture_id


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        fail("usage: validate_source_timing_fixture_catalog.py <catalog.json>")

    path = Path(argv[1])
    try:
        data = json.loads(path.read_text())
    except OSError as exc:
        fail(f"could not read {path}: {exc}")
    except json.JSONDecodeError as exc:
        fail(f"could not parse {path}: {exc}")

    if data.get("schema") != SCHEMA:
        fail(f"schema must be {SCHEMA!r}")
    if data.get("schema_version") != 1:
        fail("schema_version must be integer 1")

    cases = data.get("cases")
    if not isinstance(cases, list) or len(cases) < 5:
        fail("cases must contain at least five fixture seeds")

    seen_ids: set[str] = set()
    categories = {data_case.get("category") for data_case in cases if isinstance(data_case, dict)}
    required_categories = {"clean_rhythm", "dense_break", "hook_forward", "half_time_ambiguity", "weak_timing"}
    missing_categories = sorted(required_categories - categories)
    if missing_categories:
        fail(f"missing required categories: {', '.join(missing_categories)}")

    for case in cases:
        validate_case(case, seen_ids)

    print(f"valid {SCHEMA}: {path} cases={len(cases)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
