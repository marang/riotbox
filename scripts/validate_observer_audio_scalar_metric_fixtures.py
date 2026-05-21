#!/usr/bin/env python3
"""Exercise observer/audio scalar metric validator edge cases."""

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
    / "crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_valid_locked_grid_alignment.json"
)
VALIDATOR = ["python3", "scripts/validate_observer_audio_summary_json.py"]


def main() -> int:
    base = read_json(SUMMARY_FIXTURE)

    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = pathlib.Path(tmp)
        validate_case(base, tmpdir / "valid_scalar_metrics.json")

        reject_case(
            with_metric_value(base, "full_mix_rms", -0.1),
            "full_mix_rms must be non-negative",
            tmpdir / "full_mix_rms_negative.json",
        )
        reject_case(
            with_metric_value(base, "full_mix_low_band_rms", -0.1),
            "full_mix_low_band_rms must be non-negative",
            tmpdir / "full_mix_low_band_rms_negative.json",
        )
        reject_case(
            with_metric_value(base, "mc202_question_answer_delta_rms", -0.1),
            "mc202_question_answer_delta_rms must be non-negative",
            tmpdir / "mc202_question_answer_delta_rms_negative.json",
        )
        reject_case(
            with_metric_value(base, "w30_candidate_rms", -0.1),
            "w30_candidate_rms must be non-negative",
            tmpdir / "w30_candidate_rms_negative.json",
        )
        reject_case(
            with_metric_value(base, "w30_candidate_active_sample_ratio", 1.1),
            "w30_candidate_active_sample_ratio must be between 0 and 1",
            tmpdir / "w30_candidate_active_sample_ratio_high.json",
        )
        reject_case(
            with_metric_value(base, "w30_candidate_active_sample_ratio", -0.1),
            "w30_candidate_active_sample_ratio must be between 0 and 1",
            tmpdir / "w30_candidate_active_sample_ratio_negative.json",
        )
        reject_case(
            with_metric_value(base, "w30_rms_delta", -0.1),
            "w30_rms_delta must be non-negative",
            tmpdir / "w30_rms_delta_negative.json",
        )

    print("observer/audio scalar metric validator fixtures ok")
    return 0


def with_metric_value(base: dict[str, Any], field: str, value: float) -> dict[str, Any]:
    data = copy.deepcopy(base)
    data["output_path"]["metrics"][field] = value
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
        raise SystemExit(f"expected invalid scalar metric fixture to fail: {path}")
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
