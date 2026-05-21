#!/usr/bin/env python3
"""Exercise observer/audio W-30 loop-closure validator edge cases."""

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
        validate_case(base, tmpdir / "valid_w30_loop_closure.json")

        missing_loop_closure = copy.deepcopy(base)
        missing_loop_closure["output_path"]["metrics"].pop("w30_source_loop_closure")
        reject_case(
            missing_loop_closure,
            "w30_source_loop_closure must be present as an object or null",
            tmpdir / "w30_loop_closure_missing_key.json",
        )
        reject_case(
            with_loop_closure_value(base, "passed", "true"),
            "passed must be a boolean",
            tmpdir / "w30_loop_closure_passed_string.json",
        )
        reject_case(
            with_loop_closure_value(base, "preview_rms", -0.1),
            "w30_source_loop_closure.preview_rms must be non-negative",
            tmpdir / "w30_loop_closure_preview_rms_negative.json",
        )
        reject_case(
            with_loop_closure_value(base, "edge_delta_abs", -0.1),
            "w30_source_loop_closure.edge_delta_abs must be non-negative",
            tmpdir / "w30_loop_closure_edge_delta_negative.json",
        )
        reject_case(
            without_loop_closure_field(base, "max_allowed_edge_delta_abs"),
            "max_allowed_edge_delta_abs must be a number",
            tmpdir / "w30_loop_closure_missing_delta_budget.json",
        )
        reject_case(
            with_loop_closure_value(base, "edge_abs_max", -0.1),
            "w30_source_loop_closure.edge_abs_max must be non-negative",
            tmpdir / "w30_loop_closure_edge_abs_negative.json",
        )
        reject_case(
            with_loop_closure_value(base, "max_allowed_edge_abs", -0.1),
            "w30_source_loop_closure.max_allowed_edge_abs must be non-negative",
            tmpdir / "w30_loop_closure_edge_abs_budget_negative.json",
        )
        reject_case(
            with_loop_closure_value(base, "source_contains_selection", "yes"),
            "source_contains_selection must be a boolean",
            tmpdir / "w30_loop_closure_source_contains_selection_string.json",
        )

    print("observer/audio W-30 loop-closure validator fixtures ok")
    return 0


def with_loop_closure_value(
    base: dict[str, Any], field: str, value: Any
) -> dict[str, Any]:
    data = copy.deepcopy(base)
    data["output_path"]["metrics"]["w30_source_loop_closure"][field] = value
    return data


def without_loop_closure_field(base: dict[str, Any], field: str) -> dict[str, Any]:
    data = copy.deepcopy(base)
    del data["output_path"]["metrics"]["w30_source_loop_closure"][field]
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
        raise SystemExit(f"expected invalid W-30 loop-closure fixture to fail: {path}")
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
