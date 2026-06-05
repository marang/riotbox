#!/usr/bin/env python3
"""Import a structured listening review as an audio-judge label corpus entry."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

from listening_review_workflow import validate_review
from validate_human_listening_label_corpus import SCHEMA as LABEL_SCHEMA
from validate_human_listening_label_corpus import validate_manifest


LISTENING_REVIEW_SCHEMA = "riotbox.listening_review.v1"
LABEL_METADATA_FIELD = "audio_judge_label"
VERDICT_MAP = {
    "keep": "pass",
    "technically_ok_but_musically_weak": "weak",
    "reject": "fail",
    "inconclusive": "inconclusive",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("review", type=Path)
    parser.add_argument("--json-output", type=Path, required=True)
    parser.add_argument("--require-artifact-hashes", action="store_true")
    args = parser.parse_args()

    try:
        review = read_json_object(args.review)
        corpus = build_label_corpus(review, args.review, args.require_artifact_hashes)
        validate_manifest(corpus, args.review)
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(corpus, indent=2) + "\n")
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid listening-review label import: {error}", file=sys.stderr)
        return 1

    print(f"listening-review label corpus written: {args.json_output}")
    return 0


def build_label_corpus(
    review: dict[str, Any],
    path: Path,
    require_artifact_hashes: bool = False,
) -> dict[str, Any]:
    require(review.get("schema") == LISTENING_REVIEW_SCHEMA, f"{path}: unexpected schema")
    require(review.get("schema_version") == 1, f"{path}: schema_version must be 1")
    human_verdict = string_field(review, "human_verdict", path)
    require(human_verdict in VERDICT_MAP, f"{path}: cannot import human_verdict {human_verdict}")
    validate_structured_review(review, path)
    reviewer = string_field(review, "reviewer", path)
    metadata = review.get(LABEL_METADATA_FIELD)
    require(isinstance(metadata, dict), f"{path}: missing {LABEL_METADATA_FIELD}")
    if require_artifact_hashes:
        validate_artifact_hashes(metadata, path)

    label = {
        "label_id": string_or_default(
            metadata.get("label_id"),
            stable_label_id(review, metadata, path),
        ),
        "created_at": string_field(metadata, "created_at", path),
        "reviewer": reviewer,
        "human_verdict": VERDICT_MAP[human_verdict],
        "source_family": string_field(metadata, "source_family", path),
        "source_id": string_field(metadata, "source_id", path),
        "review_pack_schema": string_field(metadata, "review_pack_schema", path),
        "review_pack_id": string_field(metadata, "review_pack_id", path),
        "artifact_identity": object_field(metadata, "artifact_identity", path),
        "reason_tags": object_field(metadata, "reason_tags", path),
        "summary": string_or_default(
            metadata.get("summary"),
            string_or_default(review.get("concrete_follow_up"), "Imported listening review."),
        ),
        "failure_reason": string_or_default(
            metadata.get("failure_reason"),
            string_or_default(review.get("failure_reason"), ""),
        ),
        "preferred_direction": string_or_default(
            metadata.get("preferred_direction"),
            string_or_default(review.get("preferred_direction"), ""),
        ),
        "avoid": list_field(review, "avoid", path),
    }
    if label["human_verdict"] in {"weak", "fail"}:
        require(label["failure_reason"], f"{path}: weak/fail import needs failure_reason")
        require(label["preferred_direction"], f"{path}: weak/fail import needs preferred_direction")
    return {
        "schema": LABEL_SCHEMA,
        "schema_version": 1,
        "description": "Imported from structured Riotbox listening review.",
        "labels": [label],
    }


def validate_artifact_hashes(metadata: dict[str, Any], review_path: Path) -> None:
    identity = object_field(metadata, "artifact_identity", review_path)
    paths = object_field(metadata, "artifact_paths", review_path)
    assert_hash_matches(
        resolve_review_path(review_path, string_field(paths, "performance_report", review_path)),
        identity.get("performance_report_sha256"),
        review_path,
        "performance_report_sha256",
    )
    assert_hash_matches(
        resolve_review_path(review_path, string_field(paths, "agent_review", review_path)),
        identity.get("agent_review_sha256"),
        review_path,
        "agent_review_sha256",
    )
    audio_identity = object_field(identity, "audio_sha256", review_path)
    audio_paths = object_field(paths, "audio", review_path)
    for role, expected_hash in audio_identity.items():
        require(isinstance(role, str) and role, f"{review_path}: invalid audio role")
        path_value = audio_paths.get(role)
        require(isinstance(path_value, str) and path_value, f"{review_path}: missing artifact path for {role}")
        assert_hash_matches(
            resolve_review_path(review_path, path_value),
            expected_hash,
            review_path,
            f"audio_sha256.{role}",
        )


def assert_hash_matches(path: Path, expected: Any, review_path: Path, field: str) -> None:
    require(path.is_file(), f"{review_path}: missing artifact for {field}: {path}")
    actual = sha256_file(path)
    require(
        actual == expected,
        f"{review_path}: stale artifact hash for {field}: expected {expected}, got {actual}",
    )


def resolve_review_path(review_path: Path, value: str) -> Path:
    path = Path(value).expanduser()
    return path if path.is_absolute() else review_path.parent / path


def stable_label_id(review: dict[str, Any], metadata: dict[str, Any], path: Path) -> str:
    material = "|".join(
        [
            str(review.get("ticket", "")),
            str(metadata.get("review_pack_id", "")),
            str(metadata.get("created_at", "")),
            str(review.get("human_verdict", "")),
            str(path),
        ]
    )
    return "listening-review-" + hashlib.sha256(material.encode("utf-8")).hexdigest()[:16]


def validate_structured_review(review: dict[str, Any], path: Path) -> None:
    try:
        validate_review(review)
    except SystemExit as error:
        raise ValueError(f"{path}: invalid listening review: {error}") from error


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def string_field(data: dict[str, Any], field: str, path: Path) -> str:
    value = data.get(field)
    require(isinstance(value, str) and value.strip(), f"{path}: missing {field}")
    return value


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{path}: missing {field}")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list), f"{path}: {field} must be array")
    for item in value:
        require(isinstance(item, str) and item, f"{path}: {field} values must be strings")
    return value


def string_or_default(value: Any, default: str) -> str:
    if isinstance(value, str) and value.strip():
        return value
    return default


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
