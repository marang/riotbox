#!/usr/bin/env python3
"""Validate the P023 sound-excellence real-source corpus contract."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.sound_excellence_source_corpus.v1"
REQUIRED_FAMILIES = {
    "bad_timing",
    "dense_break",
    "pad_noise",
    "sparse_drums",
    "tonal_riff",
    "weak_source",
}
VALID_CONFIDENCE = {"high", "medium", "low"}
VALID_GRID_USE = {
    "auto_candidate",
    "manual_confirm",
    "degraded_do_not_trust",
    "unknown",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--require-existing-source-files", action="store_true")
    args = parser.parse_args()

    try:
        manifest = json.loads(args.manifest.read_text())
        validate_manifest(
            manifest,
            args.manifest,
            require_existing_source_files=args.require_existing_source_files,
        )
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid sound-excellence source corpus: {error}", file=sys.stderr)
        return 1

    print(f"valid sound-excellence source corpus: {args.manifest}")
    return 0


def validate_manifest(
    manifest: dict[str, Any],
    manifest_path: Path,
    *,
    require_existing_source_files: bool,
) -> None:
    _ = manifest_path
    repo = repo_root() if require_existing_source_files else None
    require_equal(manifest, "schema", SCHEMA)
    require_equal(manifest, "schema_version", 1)
    require_equal(manifest, "evidence_role", "source_corpus_contract")
    require_equal(manifest, "quality_proof", False)
    require_equal(manifest, "human_verdict", "unverified")
    require_non_empty_string(manifest, "source_presence")
    require_non_empty_string(manifest, "license_note")

    required_families = set(require_string_list(manifest, "required_source_families"))
    if required_families != REQUIRED_FAMILIES:
        raise ValueError(
            "required_source_families must be exactly "
            f"{sorted(REQUIRED_FAMILIES)}, got {sorted(required_families)}"
        )

    entries = manifest.get("entries")
    if not isinstance(entries, list) or not entries:
        raise ValueError("entries must be a non-empty array")

    seen_case_ids: set[str] = set()
    covered_families: set[str] = set()
    for index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            raise ValueError(f"entries[{index}] must be an object")
        family = validate_entry(
            entry,
            index,
            repo,
            require_existing_source_files=require_existing_source_files,
        )
        covered_families.add(family)
        case_id = require_non_empty_string(entry, "case_id")
        if case_id in seen_case_ids:
            raise ValueError(f"duplicate case_id: {case_id}")
        seen_case_ids.add(case_id)

    missing = REQUIRED_FAMILIES - covered_families
    if missing:
        raise ValueError(f"missing source family coverage: {sorted(missing)}")


def validate_entry(
    entry: dict[str, Any],
    index: int,
    repo: Path | None,
    *,
    require_existing_source_files: bool,
) -> str:
    prefix = f"entries[{index}]"
    case_id = require_non_empty_string(entry, "case_id")
    source_family = require_non_empty_string(entry, "source_family")
    if source_family not in REQUIRED_FAMILIES:
        raise ValueError(f"{prefix}.source_family is not supported: {source_family}")

    source_path = Path(require_non_empty_string(entry, "source_path"))
    if source_path.is_absolute() or ".." in source_path.parts:
        raise ValueError(f"{case_id}: source_path must be repo-relative and safe")
    if not source_path.as_posix().startswith("data/test_audio/examples/"):
        raise ValueError(f"{case_id}: source_path must stay under data/test_audio/examples")
    if require_existing_source_files and repo is not None and not (repo / source_path).is_file():
        raise ValueError(f"{case_id}: missing source file: {source_path}")

    bpm_hint = entry.get("bpm_hint")
    if not isinstance(bpm_hint, int | float) or bpm_hint <= 0:
        raise ValueError(f"{case_id}: bpm_hint must be a positive number")

    timing = entry.get("timing_expectation")
    if not isinstance(timing, dict):
        raise ValueError(f"{case_id}: timing_expectation must be an object")
    confidence = require_non_empty_string(timing, "confidence")
    if confidence not in VALID_CONFIDENCE:
        raise ValueError(f"{case_id}: invalid timing confidence: {confidence}")
    grid_use = require_non_empty_string(timing, "grid_use")
    if grid_use not in VALID_GRID_USE:
        raise ValueError(f"{case_id}: invalid grid_use: {grid_use}")
    expected_bpm = timing.get("expected_bpm")
    if expected_bpm is not None and (
        not isinstance(expected_bpm, int | float) or expected_bpm <= 0
    ):
        raise ValueError(f"{case_id}: expected_bpm must be a positive number or null")
    require_non_empty_string(timing, "known_risk")

    require_min_string_list(entry, "expected_musical_payoff", 2)
    require_min_string_list(entry, "likely_failure_modes", 2)
    require_min_string_list(entry, "target_review_questions", 2)
    require_min_string_list(entry, "feeds", 1)
    return source_family


def require_equal(data: dict[str, Any], key: str, expected: Any) -> None:
    actual = data.get(key)
    if actual != expected:
        raise ValueError(f"{key} must be {expected!r}, got {actual!r}")


def require_non_empty_string(data: dict[str, Any], key: str) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{key} must be a non-empty string")
    return value


def require_string_list(data: dict[str, Any], key: str) -> list[str]:
    value = data.get(key)
    if not isinstance(value, list) or not all(
        isinstance(item, str) and item.strip() for item in value
    ):
        raise ValueError(f"{key} must be an array of non-empty strings")
    return value


def require_min_string_list(data: dict[str, Any], key: str, minimum: int) -> list[str]:
    value = require_string_list(data, key)
    if len(value) < minimum:
        raise ValueError(f"{key} must contain at least {minimum} items")
    return value


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return Path(result.stdout.strip())


if __name__ == "__main__":
    raise SystemExit(main())
