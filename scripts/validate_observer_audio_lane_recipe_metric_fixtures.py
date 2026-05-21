#!/usr/bin/env python3
"""Exercise observer/audio lane recipe metric validator edge cases."""

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
    base = valid_lane_recipe_summary()

    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = pathlib.Path(tmp)
        validate_case(base, tmpdir / "valid_lane_recipe_metrics.json")

        reject_case(
            with_case_field(base, "result", "maybe"),
            "result must be one of",
            tmpdir / "lane_recipe_result_unknown.json",
        )
        reject_case(
            with_case_field(base, "candidate_rms", -0.1),
            "candidate_rms must be non-negative",
            tmpdir / "candidate_rms_negative.json",
        )
        reject_case(
            with_case_field(base, "signal_delta_rms", -0.1),
            "signal_delta_rms must be non-negative",
            tmpdir / "signal_delta_rms_negative.json",
        )
        reject_case(
            with_case_field(base, "min_signal_delta_rms", -0.1),
            "min_signal_delta_rms must be non-negative",
            tmpdir / "min_signal_delta_rms_negative.json",
        )
        reject_case(
            with_phrase_grid_field(base, "hit_ratio", 1.1),
            "mc202_phrase_grid.hit_ratio must be between 0 and 1",
            tmpdir / "hit_ratio_high.json",
        )
        reject_case(
            with_phrase_grid_field(base, "hit_ratio", -0.1),
            "mc202_phrase_grid.hit_ratio must be between 0 and 1",
            tmpdir / "hit_ratio_negative.json",
        )
        reject_case(
            with_phrase_grid_field(base, "candidate_onset_count", -1),
            "candidate_onset_count must be non-negative",
            tmpdir / "candidate_onset_count_negative.json",
        )
        reject_case(
            with_phrase_grid_field(base, "grid_aligned_onset_count", -1),
            "grid_aligned_onset_count must be non-negative",
            tmpdir / "grid_aligned_onset_count_negative.json",
        )
        reject_case(
            with_phrase_grid_fields(
                base,
                {
                    "candidate_onset_count": 3,
                    "grid_aligned_onset_count": 4,
                },
            ),
            "mc202_phrase_grid.grid_aligned_onset_count cannot exceed candidate_onset_count",
            tmpdir / "grid_aligned_onset_count_exceeds_candidate.json",
        )
        reject_case(
            with_phrase_grid_field(base, "max_onset_offset_ms", -0.1),
            "mc202_phrase_grid.max_onset_offset_ms must be non-negative",
            tmpdir / "max_onset_offset_ms_negative.json",
        )
        reject_case(
            with_phrase_grid_field(base, "max_allowed_onset_offset_ms", -0.1),
            "mc202_phrase_grid.max_allowed_onset_offset_ms must be non-negative",
            tmpdir / "max_allowed_onset_offset_ms_negative.json",
        )

    print("observer/audio lane recipe metric validator fixtures ok")
    return 0


def valid_lane_recipe_summary() -> dict[str, Any]:
    data = read_json(SUMMARY_FIXTURE)
    data["output_path"]["lane_recipe_cases"][0]["mc202_phrase_grid"]["hit_ratio"] = 1.0
    return data


def with_case_field(base: dict[str, Any], field: str, value: Any) -> dict[str, Any]:
    data = copy.deepcopy(base)
    data["output_path"]["lane_recipe_cases"][0][field] = value
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
        raise SystemExit(f"expected invalid lane recipe metric fixture to fail: {path}")
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
