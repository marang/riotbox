#!/usr/bin/env python3
"""Promote structured listening reviews into release-grade demo-bank entries."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

from listening_review_workflow import validate_review
from validate_release_grade_demo_bank import validate_manifest as validate_demo_bank


LISTENING_REVIEW_SCHEMA = "riotbox.listening_review.v1"
LABEL_METADATA_FIELD = "audio_judge_label"
VERDICT_MAP = {
    "keep": "pass",
    "technically_ok_but_musically_weak": "weak",
    "reject": "fail",
}
FIX_CATEGORIES = {
    "source_selection",
    "chop_policy",
    "drum_pressure",
    "bass_movement",
    "mix_bus",
    "destructive_gesture",
    "fixture_threshold",
    "ui_cue",
}
SLUG_RE = re.compile(r"[^a-z0-9]+")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--review", type=Path, required=True)
    parser.add_argument("--demo-bank", type=Path, required=True)
    parser.add_argument("--json-output", type=Path, required=True)
    parser.add_argument("--entry-id")
    parser.add_argument("--demo-worthiness-note", required=True)
    parser.add_argument("--fix-category", action="append", default=[])
    parser.add_argument("--require-artifact-hashes", action="store_true")
    args = parser.parse_args()

    try:
        review = read_json_object(args.review)
        entry = build_demo_bank_entry(
            review=review,
            review_path=args.review,
            entry_id=args.entry_id,
            demo_worthiness_note=args.demo_worthiness_note,
            fix_categories=args.fix_category,
            require_artifact_hashes=args.require_artifact_hashes,
        )
        manifest = read_json_object(args.demo_bank)
        upsert_entry(manifest, entry)
        summary = validate_demo_bank(manifest, args.json_output)
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(manifest, indent=2) + "\n")
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid demo-bank promotion: {error}", file=sys.stderr)
        return 1

    print(
        "demo-bank promotion written: "
        f"{args.json_output} ({summary['entry_count']} entries)"
    )
    return 0


def build_demo_bank_entry(
    review: dict[str, Any],
    review_path: Path,
    entry_id: str | None,
    demo_worthiness_note: str,
    fix_categories: list[str],
    require_artifact_hashes: bool,
) -> dict[str, Any]:
    require(review.get("schema") == LISTENING_REVIEW_SCHEMA, f"{review_path}: unexpected schema")
    require(review.get("schema_version") == 1, f"{review_path}: schema_version must be 1")
    human_verdict_raw = string_field(review, "human_verdict", review_path)
    require(
        human_verdict_raw in VERDICT_MAP,
        f"{review_path}: cannot promote human_verdict {human_verdict_raw}",
    )
    validate_structured_review(review, review_path)
    human_verdict = VERDICT_MAP[human_verdict_raw]
    categories = validate_fix_categories(fix_categories, human_verdict, review_path)
    metadata = object_field(review, LABEL_METADATA_FIELD, review_path)
    identity = object_field(metadata, "artifact_identity", review_path)
    paths = object_field(metadata, "artifact_paths", review_path)
    audio_paths = object_field(paths, "audio", review_path)
    audio_identity = object_field(identity, "audio_sha256", review_path)

    full_performance = resolve_review_path(
        review_path,
        string_field(audio_paths, "full_performance", review_path),
    )
    performance_report = resolve_review_path(
        review_path,
        string_field(paths, "performance_report", review_path),
    )
    prompt_path = review_artifact_path(review, review_path, "prompt_markdown")

    if require_artifact_hashes:
        assert_hash_matches(
            performance_report,
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
        assert_hash_matches(
            full_performance,
            audio_identity.get("full_performance"),
            review_path,
            "audio_sha256.full_performance",
        )
        source_window_path = audio_paths.get("source_window")
        if source_window_path is not None:
            assert_hash_matches(
                resolve_review_path(review_path, string_field(audio_paths, "source_window", review_path)),
                audio_identity.get("source_window"),
                review_path,
                "audio_sha256.source_window",
            )
    require(prompt_path.is_file(), f"{review_path}: missing review prompt artifact: {prompt_path}")

    readiness = "demo_ready" if human_verdict == "pass" else "not_demo_ready"
    source_id = string_field(metadata, "source_id", review_path)
    return {
        "entry_id": entry_id or f"{slug(source_id)}-human-{human_verdict}",
        "source_family": string_field(metadata, "source_family", review_path),
        "source_path": source_path_for(review, review_path),
        "rendered_wav": {
            "path": str(full_performance),
            "sha256": string_field(audio_identity, "full_performance", review_path),
        },
        "metrics": {
            "path": str(performance_report),
            "sha256": string_field(identity, "performance_report_sha256", review_path),
        },
        "review_prompt": {
            "path": str(prompt_path),
            "sha256": sha256_file(prompt_path),
        },
        "human_verdict": human_verdict,
        "demo_readiness": readiness,
        "demo_worthiness_note": demo_worthiness_note,
        "fix_categories": categories,
        "musical_summary": musical_summary(review, metadata, human_verdict),
    }


def validate_fix_categories(categories: list[str], human_verdict: str, path: Path) -> list[str]:
    unknown = sorted(set(categories) - FIX_CATEGORIES)
    require(not unknown, f"{path}: unknown fix categories: {', '.join(unknown)}")
    if human_verdict == "pass":
        require(not categories, f"{path}: pass promotion must not carry fix categories")
    else:
        require(categories, f"{path}: weak/fail demo-bank entries need fix categories")
    return categories


def musical_summary(review: dict[str, Any], metadata: dict[str, Any], human_verdict: str) -> dict[str, str]:
    tags = object_field(metadata, "reason_tags", Path("<review>"))
    verdict_text = human_verdict.replace("_", " ")
    return {
        "hook_within_two_bars": (
            f"Human {verdict_text} review marked hook_after_two_bars as "
            f"{review.get('hook_after_two_bars')} and hook clarity as {tags.get('hook_clarity')}."
        ),
        "hardest_audible_element": f"Human review tagged the hardest element as {tags.get('hardest_hit')}.",
        "source_character": f"Human review tagged source character as {tags.get('source_character')}.",
        "destructive_contrast": (
            f"Human review tagged destructive contrast as {tags.get('destructive_contrast')}."
        ),
        "bass_drum_pressure": f"Human review tagged bass pressure as {tags.get('bass_pressure')}.",
        "live_triggerability": (
            "Structured review strongest element "
            f"{review.get('strongest_element')} is the current live-gesture cue."
        ),
        "eight_bar_replay_value": (
            "Human review tagged replay value after eight bars as "
            f"{tags.get('replay_value_after_eight_bars')}."
        ),
    }


def upsert_entry(manifest: dict[str, Any], entry: dict[str, Any]) -> None:
    entries = manifest.get("entries")
    require(isinstance(entries, list), "demo bank manifest entries must be an array")
    for index, existing in enumerate(entries):
        if isinstance(existing, dict) and existing.get("entry_id") == entry["entry_id"]:
            entries[index] = entry
            return
    entries.append(entry)


def review_artifact_path(review: dict[str, Any], review_path: Path, key: str) -> Path:
    artifacts = object_field(review, "artifacts", review_path)
    return resolve_review_path(review_path, string_field(artifacts, key, review_path))


def source_path_for(review: dict[str, Any], review_path: Path) -> str:
    artifacts = object_field(review, "artifacts", review_path)
    source = string_field(artifacts, "source_audio", review_path)
    require(source.endswith(".wav"), f"{review_path}: source_audio must be a WAV path")
    return source


def validate_structured_review(review: dict[str, Any], path: Path) -> None:
    try:
        validate_review(review)
    except SystemExit as error:
        raise ValueError(f"{path}: invalid listening review: {error}") from error


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


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{path}: missing {field}")
    return value


def string_field(data: dict[str, Any], field: str, path: Path) -> str:
    value = data.get(field)
    require(isinstance(value, str) and value.strip(), f"{path}: missing {field}")
    return value


def slug(value: str) -> str:
    return SLUG_RE.sub("-", value.lower()).strip("-") or "review"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
