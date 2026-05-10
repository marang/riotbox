#!/usr/bin/env python3
"""Exercise source timing catalog validator negative label fixtures."""

from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


CATALOG = Path("crates/riotbox-core/tests/fixtures/source_timing/timing_fixture_catalog.json")
VALIDATOR = Path("scripts/validate_source_timing_fixture_catalog.py")


def main() -> int:
    catalog = json.loads(CATALOG.read_text())
    cases = require_cases(catalog)

    mutations = [
        (
            "unknown timing_quality",
            "timing_quality must be one of",
            lambda data: set_expected(data, "fx_timing_clean_128_4x4", "timing_quality", "pretty_good"),
        ),
        (
            "unknown degraded_policy",
            "degraded_policy must be one of",
            lambda data: set_expected(data, "fx_timing_clean_128_4x4", "degraded_policy", "mostly_locked"),
        ),
        (
            "unknown warning",
            "warnings must be known warning strings",
            lambda data: set_warning(data, "fx_timing_weak_noisy_123", 2, "phrase_uncertain_typo"),
        ),
        (
            "unknown alternative kind",
            "alternative 0 has unknown kind",
            lambda data: set_alternative_kind(
                data,
                "fx_timing_halftime_140_ambiguous",
                0,
                "halff_time_typo",
            ),
        ),
        (
            "high drift missing warning",
            "high drift seeds must carry drift_high",
            lambda data: set_warning(data, "fx_timing_drifting_live_118", 0, "phrase_uncertain"),
        ),
        (
            "high drift missing metrics",
            "high drift seeds must define explicit drift metrics",
            lambda data: remove_expected(data, "fx_timing_drifting_live_118", "drift"),
        ),
        (
            "high drift below threshold",
            "high drift seeds must exceed the high-drift threshold",
            lambda data: set_drift_thresholds(data, "fx_timing_drifting_live_118", 60.0, 60.0),
        ),
    ]

    if not cases:
        raise ValueError("fixture catalog must contain cases")

    with tempfile.TemporaryDirectory() as tmp_dir:
        for name, expected_error, mutate in mutations:
            candidate = copy.deepcopy(catalog)
            mutate(candidate)
            path = Path(tmp_dir) / f"{slug(name)}.json"
            path.write_text(json.dumps(candidate, indent=2) + "\n")
            result = subprocess.run(
                [sys.executable, str(VALIDATOR), str(path)],
                check=False,
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                raise AssertionError(f"expected {name} fixture to fail")
            if expected_error not in result.stderr:
                raise AssertionError(
                    f"expected {name} fixture to fail with {expected_error!r}, got {result.stderr!r}"
                )
            print(f"invalid source timing catalog label fixture rejected: {name}")

    return 0


def require_cases(catalog: dict[str, Any]) -> list[dict[str, Any]]:
    cases = catalog.get("cases")
    if not isinstance(cases, list):
        raise TypeError("catalog cases must be an array")
    return cases


def case_by_id(catalog: dict[str, Any], fixture_id: str) -> dict[str, Any]:
    for case in require_cases(catalog):
        if case.get("fixture_id") == fixture_id:
            return case
    raise KeyError(f"missing fixture {fixture_id}")


def set_expected(catalog: dict[str, Any], fixture_id: str, field: str, value: Any) -> None:
    case_by_id(catalog, fixture_id)["expected"][field] = value


def set_warning(catalog: dict[str, Any], fixture_id: str, index: int, value: str) -> None:
    case_by_id(catalog, fixture_id)["expected"]["warnings"][index] = value


def set_alternative_kind(catalog: dict[str, Any], fixture_id: str, index: int, value: str) -> None:
    case_by_id(catalog, fixture_id)["expected"]["alternatives"][index]["kind"] = value


def remove_expected(catalog: dict[str, Any], fixture_id: str, field: str) -> None:
    del case_by_id(catalog, fixture_id)["expected"][field]


def set_drift_thresholds(
    catalog: dict[str, Any],
    fixture_id: str,
    max_drift_ms: float,
    end_drift_ms: float,
) -> None:
    drift = case_by_id(catalog, fixture_id)["expected"]["drift"]
    drift["max_drift_ms"] = max_drift_ms
    drift["end_drift_ms"] = end_drift_ms


def slug(value: str) -> str:
    return value.replace(" ", "-").replace("_", "-")


if __name__ == "__main__":
    raise SystemExit(main())
