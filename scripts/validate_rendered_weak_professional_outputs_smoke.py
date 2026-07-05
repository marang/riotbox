#!/usr/bin/env python3
"""Validate rendered weak professional-output smoke artifacts."""

from __future__ import annotations

import argparse
import copy
import json
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable


DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-rendered-weak-professional-outputs")
REPORT_NAME = "rendered-weak-professional-outputs.json"
REQUIRED_FAILURE_CODES = {
    "dropout_not_contrasting_with_stutter",
    "dropout_silence_not_deep_enough_before_stutter",
    "restore_not_bigger_than_pressure",
    "restore_does_not_slam_out_of_cut",
}
REQUIRED_ARTIFACTS = [
    Path("dense_flat_stutter/05_rebuild_only_performance.wav"),
    Path("dense_flat_stutter/destructive-validation.json"),
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--report-json", type=Path)
    args = parser.parse_args()

    report_json = args.report_json or args.output / REPORT_NAME
    try:
        report = read_json_object(report_json)
        validate_report(report)
        validate_required_artifacts(args.output)
        validate_mutation_fixtures(report, args.output)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid rendered weak professional outputs smoke: {error}", file=sys.stderr)
        return 1

    print(f"valid rendered weak professional outputs smoke: {report_json}")
    return 0


def validate_report(report: dict[str, Any]) -> None:
    require(
        report.get("schema") == "riotbox.rendered_weak_professional_outputs.v1",
        "schema mismatch",
    )
    require(report.get("schema_version") == 1, "schema version mismatch")
    require(report.get("result") == "pass", "result must be pass")
    require(report.get("agent_verdict") == "agent_promising", "agent verdict mismatch")
    require(report.get("human_verdict") == "unverified", "human verdict must be unverified")
    require(report.get("evidence_role") == "negative_diagnostic", "evidence role mismatch")
    require(report.get("source_backed") is False, "report must not be source-backed")
    require(
        report.get("source_timing_backed") is False,
        "report must not be source-timing-backed",
    )
    require(report.get("scripted_generation") is True, "scripted generation flag missing")
    require(report.get("quality_proof") is False, "report must not claim quality")
    require(
        report.get("automated_musical_approval") is not True,
        "report must not claim automated approval",
    )
    cases = list_field(report, "cases")
    require(report.get("case_count") == 1, "case count mismatch")
    require(len(cases) == 1, "expected exactly one rendered weak case")
    case = object_field(cases[0], "case")
    require(case.get("case_id") == "dense_flat_stutter", "case id mismatch")
    require(case.get("evidence_role") == "negative_diagnostic", "case evidence role mismatch")
    require(case.get("quality_proof") is False, "case must not claim quality")
    require(
        case.get("automated_musical_approval") is not True,
        "case must not claim automated approval",
    )
    require(case.get("validator_result") == "expected_fail", "validator result mismatch")
    failure_codes = set(string_list(case.get("failure_codes")))
    missing_codes = sorted(REQUIRED_FAILURE_CODES - failure_codes)
    require(not missing_codes, f"failure codes missing: {', '.join(missing_codes)}")


def validate_required_artifacts(output: Path) -> None:
    for relative_path in REQUIRED_ARTIFACTS:
        path = output / relative_path
        require(path.is_file(), f"artifact missing: {relative_path}")
        require(path.stat().st_size > 0, f"artifact empty: {relative_path}")


def validate_mutation_fixtures(report: dict[str, Any], output: Path) -> None:
    expect_report_failure(
        report,
        "quality_claim",
        lambda data: set_field(data, "quality_proof", True),
        "report must not claim quality",
    )
    expect_report_failure(
        report,
        "missing_failure_code",
        lambda data: list_field(data["cases"][0], "failure_codes").remove(
            "restore_does_not_slam_out_of_cut"
        ),
        "failure codes missing",
    )
    expect_report_failure(
        report,
        "case_count_stale",
        lambda data: set_field(data, "case_count", 2),
        "case count mismatch",
    )
    with tempfile.TemporaryDirectory() as tmp:
        tmp_output = Path(tmp)
        (tmp_output / REPORT_NAME).write_text(json.dumps(report, indent=2) + "\n")
        for relative_path in REQUIRED_ARTIFACTS:
            source = output / relative_path
            target = tmp_output / relative_path
            target.parent.mkdir(parents=True, exist_ok=True)
            if source.exists():
                target.write_bytes(source.read_bytes())
        missing = tmp_output / REQUIRED_ARTIFACTS[0]
        missing.unlink()
        try:
            validate_required_artifacts(tmp_output)
        except ValueError as error:
            require("artifact missing" in str(error), f"missing artifact fixture failed unexpectedly: {error}")
        else:
            raise ValueError("missing artifact fixture unexpectedly passed")


def expect_report_failure(
    report: dict[str, Any],
    fixture_name: str,
    mutate: Callable[[dict[str, Any]], object],
    expected_message: str,
) -> None:
    mutated = copy.deepcopy(report)
    mutate(mutated)
    try:
        validate_report(mutated)
    except ValueError as error:
        require(
            expected_message in str(error),
            f"{fixture_name}: expected {expected_message}, got {error}",
        )
    else:
        raise ValueError(f"{fixture_name}: mutation unexpectedly passed")


def read_json_object(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    require(isinstance(data, dict), f"{path} must contain a JSON object")
    return data


def object_field(value: Any, name: str) -> dict[str, Any]:
    require(isinstance(value, dict), f"{name} must be object")
    return value


def list_field(data: dict[str, Any], field: str) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{field} must be list")
    return value


def string_list(value: Any) -> list[str]:
    return [str(item) for item in value] if isinstance(value, list) else []


def set_field(data: dict[str, Any], field: str, value: Any) -> None:
    data[field] = value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
