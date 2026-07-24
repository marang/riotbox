#!/usr/bin/env python3
"""Mutation fixtures for the P023 source holdout-rotation contract."""

from __future__ import annotations

import argparse
import copy
import hashlib
import sys
import tempfile
import wave
from array import array
from pathlib import Path
from typing import Any, Callable

from validate_source_holdout_rotation import (
    read_json_object,
    require,
    validate_manifest,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    try:
        manifest = read_json_object(args.manifest)
        validate_manifest(
            manifest,
            args.manifest,
            require_existing_source_files=False,
        )
        run_mutation_fixtures(manifest, args.manifest)
    except (OSError, TypeError, ValueError, wave.Error) as error:
        print(f"invalid source holdout-rotation fixtures: {error}", file=sys.stderr)
        return 1
    print(f"valid source holdout-rotation mutation fixtures: {args.manifest}")
    return 0


def run_mutation_fixtures(manifest: dict[str, Any], manifest_path: Path) -> None:
    holdout_a = list(manifest["holdout_sets"][0]["source_case_ids"])

    def collapse_families(mutated: dict[str, Any]) -> None:
        mutated["candidate_matrix"]["source_case_ids"] = [
            "oga_cinameng_can_be_so_beautiful",
            "oga_ruok_160bpm",
            "oga_illin_robotic",
            "oga_marwan_cinematic_percussion",
            "oga_bart_getequipped",
            "oga_bretbernhoft_beatloops",
        ]

    expect_failure(
        "candidate family collapse",
        manifest,
        manifest_path,
        collapse_families,
        "candidate matrix family collapse",
    )

    def insufficient_holdout(mutated: dict[str, Any]) -> None:
        mutated["holdout_sets"][0]["source_case_ids"] = holdout_a[:1]

    expect_failure(
        "insufficient holdouts",
        manifest,
        manifest_path,
        insufficient_holdout,
        "insufficient holdout sources",
    )

    def reuse_holdout(mutated: dict[str, Any]) -> None:
        mutated["candidate_matrix"]["source_case_ids"].append(holdout_a[0])

    expect_failure(
        "reused holdout",
        manifest,
        manifest_path,
        reuse_holdout,
        "development and holdout sources must be disjoint",
    )

    def consumed_still_unseen(mutated: dict[str, Any]) -> None:
        mutated["rotation_history"].append(
            {
                "case_id": holdout_a[0],
                "former_holdout_id": "holdout_a",
                "consumed_by_ticket": "RIOTBOX-1422",
                "consumed_on": "2026-07-24",
                "replacement_case_id": holdout_a[1],
            }
        )

    expect_failure(
        "consumed holdout remains unseen",
        manifest,
        manifest_path,
        consumed_still_unseen,
        "consumed holdout cannot remain unseen/reserve",
    )

    def unsafe_path(mutated: dict[str, Any]) -> None:
        mutated["entries"][0]["source_path"] = "../.download-examples/The Prodigy.wav"

    expect_failure(
        "unsafe source path",
        manifest,
        manifest_path,
        unsafe_path,
        "source_path must be safe and repo-relative",
    )

    def reference_leak(mutated: dict[str, Any]) -> None:
        mutated["entries"][0]["commercial_reference"] = True

    expect_failure(
        "commercial reference leakage",
        manifest,
        manifest_path,
        reference_leak,
        "commercial/reference-only material is forbidden",
    )

    def missing_review_provenance(mutated: dict[str, Any]) -> None:
        del mutated["entries"][0]["reviewed_on"]

    expect_failure(
        "missing development review provenance",
        manifest,
        manifest_path,
        missing_review_provenance,
        "reviewed_on must be non-empty string",
    )

    def forged_rotation_history(mutated: dict[str, Any]) -> None:
        consumed_id = holdout_a[0]
        mutated["holdout_sets"][0]["source_case_ids"].remove(consumed_id)
        for entry in mutated["entries"]:
            if entry["case_id"] == consumed_id:
                entry.update(
                    {
                        "partition": "retired",
                        "classification_status": "consumed_holdout",
                        "source_suitability_verdict": "weak",
                        "reviewed_on": "2026-07-24",
                        "reviewer_role": "project_musician",
                        "family_verdict_owner": "human_review",
                    }
                )
        mutated["rotation_history"].append(
            {
                "case_id": consumed_id,
                "former_holdout_id": "holdout_a",
                "consumed_by_ticket": "RIOTBOX-1422",
                "consumed_on": "2026-07-24",
                "replacement_case_id": holdout_a[1],
            }
        )

    expect_failure(
        "forged rotation history",
        manifest,
        manifest_path,
        forged_rotation_history,
        "consumed source must record previous_partition",
    )

    def source_pack_collapse(mutated: dict[str, Any]) -> None:
        for entry in mutated["entries"]:
            if entry["corpus_eligible"]:
                entry["source_pack_id"] = "one_narrow_pack"

    expect_failure(
        "narrow source-pack collapse",
        manifest,
        manifest_path,
        source_pack_collapse,
        "eligible source-pack count collapsed",
    )
    run_missing_file_fixture(manifest, manifest_path)


def expect_failure(
    name: str,
    manifest: dict[str, Any],
    manifest_path: Path,
    mutate: Callable[[dict[str, Any]], None],
    expected_fragment: str,
) -> None:
    mutated = copy.deepcopy(manifest)
    mutate(mutated)
    try:
        validate_manifest(
            mutated,
            Path(f"{manifest_path}:{name}"),
            require_existing_source_files=False,
        )
    except ValueError as error:
        require(
            expected_fragment in str(error),
            f"mutation fixture {name!r} failed for the wrong reason: {error}",
        )
        return
    raise ValueError(f"mutation fixture unexpectedly passed: {name}")


def run_missing_file_fixture(manifest: dict[str, Any], manifest_path: Path) -> None:
    mutated = copy.deepcopy(manifest)
    with tempfile.TemporaryDirectory(prefix="riotbox-source-holdout-") as temp:
        temp_repo = Path(temp)
        for index, entry in enumerate(mutated["entries"]):
            path = temp_repo / entry["source_path"]
            path.parent.mkdir(parents=True, exist_ok=True)
            write_fixture_wav(path, index)
            entry["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        validate_manifest(
            mutated,
            Path(f"{manifest_path}:complete temporary source corpus"),
            require_existing_source_files=True,
            repo=temp_repo,
        )
        missing_path = temp_repo / mutated["entries"][-1]["source_path"]
        missing_path.unlink()
        try:
            validate_manifest(
                mutated,
                Path(f"{manifest_path}:missing source file"),
                require_existing_source_files=True,
                repo=temp_repo,
            )
        except ValueError as error:
            require(
                "missing source file" in str(error),
                f"missing-file fixture failed for the wrong reason: {error}",
            )
            return
    raise ValueError("mutation fixture unexpectedly passed: missing source file")


def write_fixture_wav(path: Path, seed: int) -> None:
    samples = array("h")
    for frame in range(480):
        value = ((frame * (seed + 3)) % 1000) - 500
        samples.extend((value, -value))
    with wave.open(str(path), "wb") as target:
        target.setnchannels(2)
        target.setsampwidth(2)
        target.setframerate(48_000)
        target.writeframes(samples.tobytes())


if __name__ == "__main__":
    raise SystemExit(main())
