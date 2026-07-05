#!/usr/bin/env python3
"""Validate listening-review label import fixtures."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


DEFAULT_FIXTURE_DIR = Path("scripts/fixtures/listening_review_label_import")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", type=Path, default=DEFAULT_FIXTURE_DIR)
    args = parser.parse_args()

    try:
        validate_positive_import(args.fixtures)
        validate_missing_metadata_rejection(args.fixtures)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid listening-review label import fixtures: {error}", file=sys.stderr)
        return 1

    print(f"valid listening-review label import fixtures: {args.fixtures}")
    return 0


def validate_positive_import(fixtures: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        output = Path(tmp) / "imported-label-corpus.json"
        run(
            [
                sys.executable,
                "scripts/import_listening_review_label.py",
                "--json-output",
                str(output),
                str(fixtures / "valid_review.json"),
            ]
        )
        corpus = read_json_object(output)
        require(corpus.get("schema") == "riotbox.human_listening_label_corpus.v1", "schema mismatch")
        labels = list_field(corpus, "labels")
        require(len(labels) == 1, "expected one imported label")
        label = object_field(labels[0], "labels[0]")
        require(label.get("human_verdict") == "weak", "human verdict mismatch")
        require(label.get("reviewer") == "fixture-listener", "reviewer mismatch")
        require(label.get("created_at") == "2026-06-04", "created_at mismatch")
        identity = object_field(label.get("artifact_identity"), "artifact_identity")
        require(
            identity.get("performance_report_sha256")
            == "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "performance report hash mismatch",
        )
        reason_tags = object_field(label.get("reason_tags"), "reason_tags")
        require(reason_tags.get("hook_clarity") == "weak", "hook clarity reason missing")
        run([sys.executable, "scripts/validate_human_listening_label_corpus.py", str(output)])


def validate_missing_metadata_rejection(fixtures: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        output = Path(tmp) / "invalid.json"
        failed = run(
            [
                sys.executable,
                "scripts/import_listening_review_label.py",
                "--json-output",
                str(output),
                str(fixtures / "invalid_missing_metadata.json"),
            ],
            check=False,
        )
        require(failed.returncode != 0, "missing metadata import unexpectedly passed")
        require("missing audio_judge_label" in failed.combined, "missing metadata rejection missing")


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


def list_field(data: dict[str, Any], field: str) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{field} must be list")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
