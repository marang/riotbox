#!/usr/bin/env python3
"""Validate Riotbox human listening label corpus manifests."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.human_listening_label_corpus.v1"
VERDICTS = {"pass", "weak", "fail", "inconclusive"}
SOURCE_FAMILIES = {
    "dense_break",
    "tonal_hook",
    "tonal_pad",
    "kick_snare_loop",
    "sparse_bass_pressure",
    "other",
}
HOOK_CLARITY = {"clear", "weak", "missing", "annoying", "inconclusive"}
HARDEST_HIT = {"kick", "snare", "break_transient", "bass", "chop", "silence", "none"}
BASS_PRESSURE = {"strong", "present", "weak", "missing", "overpowering"}
DESTRUCTIVE_CONTRAST = {"strong", "present", "weak", "missing", "overdone"}
SOURCE_CHARACTER = {
    "source_clear",
    "source_transformed_but_present",
    "source_lost",
    "source_too_close_to_original",
}
REPLAY_VALUE = {"high", "medium", "low", "none", "inconclusive"}
HEX_64_RE = re.compile(r"^[0-9a-f]{64}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()

    try:
        manifest = json.loads(args.manifest.read_text())
        summary = validate_manifest(manifest, args.manifest)
    except (OSError, json.JSONDecodeError, ValueError, TypeError) as error:
        print(f"invalid human listening label corpus: {error}", file=sys.stderr)
        return 1

    if args.json_output:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(summary, indent=2) + "\n")
    print(f"valid human listening label corpus: {args.manifest}")
    return 0


def validate_manifest(manifest: dict[str, Any], path: Path) -> dict[str, Any]:
    require(manifest.get("schema") == SCHEMA, f"{path}: schema must be {SCHEMA}")
    require(manifest.get("schema_version") == 1, f"{path}: schema_version must be 1")
    labels = manifest.get("labels")
    require(isinstance(labels, list) and labels, f"{path}: labels must be a non-empty array")

    seen_ids = set()
    verdict_counts = {verdict: 0 for verdict in sorted(VERDICTS)}
    source_families = set()
    for index, label in enumerate(labels):
        require(isinstance(label, dict), f"{path}: label {index} must be an object")
        label_id = require_string(label, "label_id", path, index)
        require(label_id not in seen_ids, f"{path}: duplicate label_id {label_id}")
        seen_ids.add(label_id)
        validate_label(label, path, index)
        verdict_counts[label["human_verdict"]] += 1
        source_families.add(label["source_family"])

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "label_count": len(labels),
        "verdict_counts": verdict_counts,
        "source_families": sorted(source_families),
    }


def validate_label(label: dict[str, Any], path: Path, index: int) -> None:
    prefix = f"{path}: label {index}"
    require_enum(label, "human_verdict", VERDICTS, prefix)
    require_enum(label, "source_family", SOURCE_FAMILIES, prefix)
    require_string(label, "source_id", path, index)
    require_string(label, "review_pack_id", path, index)
    require_string(label, "review_pack_schema", path, index)
    created_at = require_string(label, "created_at", path, index)
    require(bool(DATE_RE.fullmatch(created_at)), f"{prefix}: created_at must use YYYY-MM-DD")
    require_string(label, "reviewer", path, index)
    require_string(label, "summary", path, index)

    artifact_identity = label.get("artifact_identity")
    require(isinstance(artifact_identity, dict), f"{prefix}: artifact_identity must be an object")
    require_hash_field(artifact_identity, "performance_report_sha256", prefix)
    require_hash_field(artifact_identity, "agent_review_sha256", prefix)
    audio_hashes = artifact_identity.get("audio_sha256")
    require(isinstance(audio_hashes, dict) and audio_hashes, f"{prefix}: audio_sha256 must be object")
    for role, digest in audio_hashes.items():
        require(isinstance(role, str) and role, f"{prefix}: audio role must be a non-empty string")
        require_hash(digest, f"{prefix}: audio_sha256.{role} must be a lowercase sha256")

    reasons = label.get("reason_tags")
    require(isinstance(reasons, dict), f"{prefix}: reason_tags must be an object")
    require_enum(reasons, "hook_clarity", HOOK_CLARITY, prefix)
    require_enum(reasons, "hardest_hit", HARDEST_HIT, prefix)
    require_enum(reasons, "bass_pressure", BASS_PRESSURE, prefix)
    require_enum(reasons, "destructive_contrast", DESTRUCTIVE_CONTRAST, prefix)
    require_enum(reasons, "source_character", SOURCE_CHARACTER, prefix)
    require_enum(reasons, "replay_value_after_eight_bars", REPLAY_VALUE, prefix)

    if label["human_verdict"] in {"weak", "fail"}:
        require_string(label, "failure_reason", path, index)
        require_string(label, "preferred_direction", path, index)
    avoid = label.get("avoid")
    require(isinstance(avoid, list), f"{prefix}: avoid must be an array")
    for item in avoid:
        require(isinstance(item, str) and item, f"{prefix}: avoid values must be strings")


def require_string(label: dict[str, Any], field: str, path: Path, index: int) -> str:
    value = label.get(field)
    require(isinstance(value, str) and value.strip(), f"{path}: label {index} missing {field}")
    return value


def require_hash_field(data: dict[str, Any], field: str, prefix: str) -> None:
    require_hash(data.get(field), f"{prefix}: {field} must be a lowercase sha256")


def require_hash(value: Any, message: str) -> None:
    require(isinstance(value, str) and bool(HEX_64_RE.fullmatch(value)), message)


def require_enum(data: dict[str, Any], field: str, allowed: set[str], prefix: str) -> None:
    value = data.get(field)
    require(value in allowed, f"{prefix}: {field} must be one of {sorted(allowed)}")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
