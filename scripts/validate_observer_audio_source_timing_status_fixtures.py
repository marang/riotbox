#!/usr/bin/env python3
"""Exercise observer/audio Source Timing status validator edge cases."""

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
        validate_case(base, tmpdir / "valid_source_timing_statuses.json")

        missing_grid_use = copy.deepcopy(base)
        missing_grid_use["output_path"]["source_timing"].pop("grid_use")
        reject_case(
            missing_grid_use,
            "grid_use must be present as a string or null",
            tmpdir / "source_timing_grid_use_missing.json",
        )
        reject_source_timing_case(
            base,
            "readiness",
            "readyish",
            "readiness must be one of",
            tmpdir / "source_timing_readiness_unknown.json",
        )
        reject_source_timing_case(
            base,
            "beat_status",
            "grid",
            "beat_status must be one of",
            tmpdir / "source_timing_beat_status_unknown.json",
        )
        reject_source_timing_case(
            base,
            "downbeat_status",
            "bar_locked",
            "downbeat_status must be one of",
            tmpdir / "source_timing_downbeat_status_unknown.json",
        )
        reject_source_timing_case(
            base,
            "confidence_result",
            "candidate_locked",
            "confidence_result must be one of",
            tmpdir / "source_timing_confidence_result_unknown.json",
        )
        reject_source_timing_case(
            base,
            "drift_status",
            "drifting",
            "drift_status must be one of",
            tmpdir / "source_timing_drift_status_unknown.json",
        )
        reject_source_timing_case(
            base,
            "phrase_status",
            "phrase_locked",
            "phrase_status must be one of",
            tmpdir / "source_timing_phrase_status_unknown.json",
        )
        reject_source_timing_case(
            base,
            "alternate_evidence_count",
            -1,
            "alternate_evidence_count must be non-negative",
            tmpdir / "source_timing_alternate_evidence_count_negative.json",
        )

    print("observer/audio Source Timing status validator fixtures ok")
    return 0


def reject_source_timing_case(
    base: dict[str, Any],
    field: str,
    value: Any,
    expected_error: str,
    path: pathlib.Path,
) -> None:
    data = copy.deepcopy(base)
    data["output_path"]["source_timing"][field] = value
    reject_case(data, expected_error, path)


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
        raise SystemExit(f"expected invalid Source Timing status fixture to fail: {path}")
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
