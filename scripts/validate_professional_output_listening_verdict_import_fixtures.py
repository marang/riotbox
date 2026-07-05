#!/usr/bin/env python3
"""Validate professional-output listening verdict import fixtures."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


DEFAULT_PACK = Path("artifacts/audio_qa/local-professional-output-listening-pack")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pack", type=Path, default=DEFAULT_PACK)
    args = parser.parse_args()

    try:
        validate_positive_import(args.pack)
        validate_unverified_rejection(args.pack)
        validate_stale_hash_rejection(args.pack)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid professional output listening verdict import fixtures: {error}", file=sys.stderr)
        return 1

    print(f"valid professional output listening verdict import fixtures: {args.pack}")
    return 0


def validate_positive_import(pack: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        review = copy_review(pack, "dense_beat03_130", tmp_path / "review.json")
        record_keep_review(review, concrete_follow_up="keep professional dense-break suite gate green")
        imported = tmp_path / "imported-label-corpus.json"
        run(
            [
                sys.executable,
                "scripts/import_listening_review_label.py",
                "--require-artifact-hashes",
                "--json-output",
                str(imported),
                str(review),
            ]
        )
        corpus = read_json_object(imported)
        require(corpus.get("schema") == "riotbox.human_listening_label_corpus.v1", "schema mismatch")
        labels = list_field(corpus, "labels")
        require(len(labels) == 1, "expected one imported label")
        label = object_field(labels[0], "labels[0]")
        require(label.get("human_verdict") == "pass", "human verdict mismatch")
        require(label.get("source_family") == "dense_break", "source family mismatch")
        require(
            label.get("review_pack_schema") == "riotbox.professional_output_listening_pack.v1",
            "review pack schema mismatch",
        )
        identity = object_field(label.get("artifact_identity"), "artifact_identity")
        performance_hash = identity.get("performance_report_sha256")
        require(isinstance(performance_hash, str) and len(performance_hash) == 64, "performance hash missing")


def validate_unverified_rejection(pack: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        review = copy_review(pack, "tonal_rusharp_120", tmp_path / "unverified.json")
        failed = run(
            [
                sys.executable,
                "scripts/import_listening_review_label.py",
                "--require-artifact-hashes",
                "--json-output",
                str(tmp_path / "imported-unverified.json"),
                str(review),
            ],
            check=False,
        )
        require(failed.returncode != 0, "unverified review import unexpectedly passed")
        require("cannot import human_verdict unverified" in failed.combined, "unverified rejection missing")


def validate_stale_hash_rejection(pack: Path) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        review = copy_review(pack, "dense_beat03_130", tmp_path / "stale.json")
        record_keep_review(review, concrete_follow_up="stale hash fixture")
        data = read_json_object(review)
        audio_hashes = object_field(
            object_field(data.get("audio_judge_label"), "audio_judge_label").get("artifact_identity"),
            "artifact_identity",
        ).get("audio_sha256")
        require(isinstance(audio_hashes, dict), "audio_sha256 must be object")
        audio_hashes["rebuild_only_performance"] = "0" * 64
        review.write_text(json.dumps(data, indent=2) + "\n")
        failed = run(
            [
                sys.executable,
                "scripts/import_listening_review_label.py",
                "--require-artifact-hashes",
                "--json-output",
                str(tmp_path / "imported-stale.json"),
                str(review),
            ],
            check=False,
        )
        require(failed.returncode != 0, "stale review import unexpectedly passed")
        require("stale artifact hash" in failed.combined, "stale hash rejection missing")


def record_keep_review(review: Path, *, concrete_follow_up: str) -> None:
    run(
        [
            sys.executable,
            "scripts/listening_review_workflow.py",
            "record",
            "--review",
            str(review),
            "--human-verdict",
            "keep",
            "--strongest-element",
            "snare",
            "--source-recognition",
            "source_transformed_but_present",
            "--hook-after-two-bars",
            "clear",
            "--preferred-direction",
            "keep the break transient and restore pressure forward",
            "--avoid",
            "flat stutter,source copy",
            "--concrete-follow-up",
            concrete_follow_up,
            "--reviewer",
            "fixture-listener",
        ]
    )


def copy_review(pack: Path, case_id: str, target: Path) -> Path:
    source = pack / "reviews" / case_id / "review.json"
    require(source.is_file(), f"review missing: {source}")
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    return target


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
