#!/usr/bin/env python3
"""Validate human listening label corpus fixtures."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


DEFAULT_FIXTURE_DIR = Path("scripts/fixtures/human_listening_label_corpus")
EXPECTED_SOURCE_FAMILIES = ["dense_break", "sparse_bass_pressure", "tonal_hook"]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", type=Path, default=DEFAULT_FIXTURE_DIR)
    args = parser.parse_args()

    try:
        validate_positive_corpus(args.fixtures)
        validate_invalid_fixture(args.fixtures / "invalid_bad_hash.json", "invalid bad-hash")
        validate_invalid_fixture(
            args.fixtures / "invalid_weak_missing_reason.json",
            "invalid weak-missing-reason",
        )
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid human listening label corpus fixtures: {error}", file=sys.stderr)
        return 1

    print(f"valid human listening label corpus fixtures: {args.fixtures}")
    return 0


def validate_positive_corpus(fixtures: Path) -> None:
    with tempfile.NamedTemporaryFile() as tmp:
        run(
            [
                sys.executable,
                "scripts/validate_human_listening_label_corpus.py",
                "--json-output",
                tmp.name,
                str(fixtures / "valid_dense_break.json"),
            ]
        )
        report = read_json_object(Path(tmp.name))
    require(report.get("schema") == "riotbox.human_listening_label_corpus.v1", "schema mismatch")
    require(report.get("result") == "pass", "result must be pass")
    require(report.get("label_count") == 5, "label count mismatch")
    verdict_counts = object_field(report.get("verdict_counts"), "verdict_counts")
    require(verdict_counts.get("pass") == 2, "pass verdict count mismatch")
    require(verdict_counts.get("weak") == 2, "weak verdict count mismatch")
    require(verdict_counts.get("fail") == 1, "fail verdict count mismatch")
    require(report.get("source_families") == EXPECTED_SOURCE_FAMILIES, "source families mismatch")


def validate_invalid_fixture(path: Path, label: str) -> None:
    failed = run(
        [sys.executable, "scripts/validate_human_listening_label_corpus.py", str(path)],
        check=False,
    )
    require(failed.returncode != 0, f"{label} fixture unexpectedly passed")


class RunResult:
    def __init__(self, completed: subprocess.CompletedProcess[str]) -> None:
        self.returncode = completed.returncode
        self.combined = f"{completed.stdout}\n{completed.stderr}"


def run(argv: list[str], *, check: bool = True) -> RunResult:
    completed = subprocess.run(argv, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if check and completed.returncode != 0:
        raise ValueError(f"{' '.join(argv)} failed: {completed.stdout}\n{completed.stderr}")
    return RunResult(completed)


def read_json_object(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    require(isinstance(data, dict), f"{path} must contain a JSON object")
    return data


def object_field(value: Any, name: str) -> dict[str, Any]:
    require(isinstance(value, dict), f"{name} must be object")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
