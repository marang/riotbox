#!/usr/bin/env python3
"""Exercise observer/audio Source Timing alignment validator edge cases."""

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
        validate_case(base, tmpdir / "valid_locked_grid_alignment.json")

        reject_case(
            with_alignment_issues(
                base,
                "source_timing_anchor_alignment",
                "aligned",
                ["source_timing_anchor_alignment.manifest_anchor_count=0"],
            ),
            "source_timing_anchor_alignment non-mismatch status must not include issues",
            tmpdir / "anchor_aligned_with_issues.json",
        )
        reject_case(
            with_alignment_issues(
                base,
                "source_timing_anchor_alignment",
                "mismatch",
                [],
            ),
            "source_timing_anchor_alignment mismatch must include an issue",
            tmpdir / "anchor_mismatch_without_issues.json",
        )
        reject_case(
            with_alignment_issues(
                base,
                "source_timing_groove_alignment",
                "mismatch",
                ["source_timing_anchor_alignment.manifest_anchor_count=0"],
            ),
            "source_timing_groove_alignment mismatch issue must start",
            tmpdir / "groove_mismatch_wrong_issue_prefix.json",
        )

    print("observer/audio source timing alignment validator fixtures ok")
    return 0


def with_alignment_issues(
    base: dict[str, Any], field: str, status: str, issues: list[str]
) -> dict[str, Any]:
    data = copy.deepcopy(base)
    alignment = data["output_path"][field]
    alignment["status"] = status
    alignment["issues"] = issues
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
        raise SystemExit(f"expected invalid alignment fixture to fail: {path}")
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
