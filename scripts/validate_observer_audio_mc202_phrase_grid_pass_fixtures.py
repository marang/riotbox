#!/usr/bin/env python3
"""Exercise observer/audio MC-202 phrase-grid pass validator edge cases."""

from __future__ import annotations

import copy
import json
import pathlib
import subprocess
import tempfile
from typing import Any


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SUMMARY_FIXTURE = (
    REPO_ROOT
    / "crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_lane_recipe_case_phrase_grid.json"
)
VALIDATOR = ["python3", "scripts/validate_observer_audio_summary_json.py"]


def main() -> int:
    base = valid_phrase_grid_summary()

    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = pathlib.Path(tmp)
        validate_case(base, tmpdir / "valid_mc202_phrase_grid_pass.json")

        reject_case(
            with_phrase_grid_field(base, "starts_on_phrase_boundary", False),
            "mc202_phrase_grid.passed requires starts_on_phrase_boundary",
            tmpdir / "passed_without_phrase_boundary.json",
        )
        reject_case(
            with_phrase_grid_fields(
                base,
                {
                    "candidate_onset_count": 0,
                    "grid_aligned_onset_count": 0,
                },
            ),
            "mc202_phrase_grid.passed requires candidate_onset_count > 0",
            tmpdir / "passed_without_candidate_onsets.json",
        )
        reject_case(
            with_phrase_grid_field(base, "hit_ratio", 0.94),
            "mc202_phrase_grid.passed requires hit_ratio >= 0.95",
            tmpdir / "passed_with_low_hit_ratio.json",
        )
        reject_case(
            with_phrase_grid_field(base, "max_onset_offset_ms", 8.1),
            "mc202_phrase_grid.passed requires max_onset_offset_ms <= max_allowed_onset_offset_ms",
            tmpdir / "passed_with_high_onset_offset.json",
        )
        reject_case(
            with_phrase_grid_field(base, "passed", False),
            "mc202_phrase_grid.passed=false contradicts passing phrase-grid evidence",
            tmpdir / "failed_with_passing_evidence.json",
        )

    print("observer/audio MC-202 phrase-grid pass validator fixtures ok")
    return 0


def valid_phrase_grid_summary() -> dict[str, Any]:
    data = read_json(SUMMARY_FIXTURE)
    data["output_path"]["lane_recipe_cases"][0]["mc202_phrase_grid"]["hit_ratio"] = 1.0
    return data


def with_phrase_grid_field(base: dict[str, Any], field: str, value: Any) -> dict[str, Any]:
    return with_phrase_grid_fields(base, {field: value})


def with_phrase_grid_fields(base: dict[str, Any], fields: dict[str, Any]) -> dict[str, Any]:
    data = copy.deepcopy(base)
    data["output_path"]["lane_recipe_cases"][0]["mc202_phrase_grid"].update(fields)
    return data


def validate_case(data: dict[str, Any], path: pathlib.Path) -> None:
    write_json(path, data)
    subprocess.run([*VALIDATOR, str(path)], cwd=REPO_ROOT, check=True)


def reject_case(data: dict[str, Any], expected_error: str, path: pathlib.Path) -> None:
    write_json(path, data)
    result = subprocess.run(
        [*VALIDATOR, str(path)],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode == 0:
        raise SystemExit(f"expected invalid MC-202 phrase-grid pass fixture to fail: {path}")
    if expected_error not in result.stderr:
        raise SystemExit(
            f"expected {expected_error!r} in validator error for {path}, got:\n{result.stderr}"
        )


def read_json(path: pathlib.Path) -> dict[str, Any]:
    with path.open() as handle:
        return json.load(handle)


def write_json(path: pathlib.Path, data: dict[str, Any]) -> None:
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    raise SystemExit(main())
