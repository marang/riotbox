#!/usr/bin/env python3
"""Exercise observer/audio MC-202 source phrase-slot validator edge cases."""

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
    / "crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_lane_recipe_case_source_phrase_slot.json"
)
VALIDATOR = ["python3", "scripts/validate_observer_audio_summary_json.py"]


def main() -> int:
    base = valid_source_phrase_slot_summary()

    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = pathlib.Path(tmp)
        validate_case(base, tmpdir / "valid_mc202_source_phrase_slot.json")

        reject_case(
            with_slot_field(base, "phrase_index", -1),
            "mc202_source_phrase_slot.phrase_index must be non-negative",
            tmpdir / "phrase_index_negative.json",
        )
        reject_case(
            with_slot_fields(base, {"passed": True, "phrase_grid_available": False}),
            "mc202_source_phrase_slot.passed requires phrase_grid_available",
            tmpdir / "passed_without_phrase_grid.json",
        )
        reject_case(
            with_slot_fields(base, {"passed": True, "phrase_index": None}),
            "mc202_source_phrase_slot.passed requires phrase_index",
            tmpdir / "passed_without_phrase_index.json",
        )
        reject_case(
            with_slot_fields(base, {"passed": True, "starts_on_source_phrase_boundary": False}),
            "mc202_source_phrase_slot.passed requires starts_on_source_phrase_boundary",
            tmpdir / "passed_without_source_boundary.json",
        )
        reject_case(
            with_slot_fields(
                base,
                {
                    "phrase_grid_available": False,
                    "phrase_index": 3,
                    "starts_on_source_phrase_boundary": False,
                    "passed": False,
                },
            ),
            "mc202_source_phrase_slot.phrase_index requires phrase_grid_available",
            tmpdir / "phrase_index_without_phrase_grid.json",
        )
        reject_case(
            with_slot_fields(
                base,
                {
                    "phrase_grid_available": False,
                    "phrase_index": None,
                    "starts_on_source_phrase_boundary": True,
                    "passed": False,
                },
            ),
            "mc202_source_phrase_slot.starts_on_source_phrase_boundary requires phrase_grid_available",
            tmpdir / "source_boundary_without_phrase_grid.json",
        )
        reject_case(
            with_slot_fields(
                base,
                {
                    "phrase_index": None,
                    "starts_on_source_phrase_boundary": True,
                    "passed": False,
                },
            ),
            "mc202_source_phrase_slot.starts_on_source_phrase_boundary requires phrase_index",
            tmpdir / "source_boundary_without_phrase_index.json",
        )

    print("observer/audio MC-202 source phrase-slot validator fixtures ok")
    return 0


def valid_source_phrase_slot_summary() -> dict[str, Any]:
    data = read_json(SUMMARY_FIXTURE)
    data["output_path"]["lane_recipe_cases"][0]["mc202_source_phrase_slot"][
        "phrase_index"
    ] = 3
    return data


def with_slot_field(base: dict[str, Any], field: str, value: Any) -> dict[str, Any]:
    return with_slot_fields(base, {field: value})


def with_slot_fields(base: dict[str, Any], fields: dict[str, Any]) -> dict[str, Any]:
    data = copy.deepcopy(base)
    data["output_path"]["lane_recipe_cases"][0]["mc202_source_phrase_slot"].update(
        fields
    )
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
        raise SystemExit(
            f"expected invalid MC-202 source phrase-slot fixture to fail: {path}"
        )
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
