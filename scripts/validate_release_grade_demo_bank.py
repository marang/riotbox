#!/usr/bin/env python3
"""Validate Riotbox release-grade musician demo bank manifests."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

from mc202_source_composed_review_gate import (
    MC202_GATE_FIELD,
    MC202_ROLE_FIELD,
    validate_promotion_gate,
    validate_role_evidence_for_promotion,
)


SCHEMA = "riotbox.release_grade_demo_bank.v1"
RUBRIC_SCHEMA = "riotbox.sound_product_readiness_rubric.v1"
HUMAN_VERDICTS = {"pass", "weak", "fail", "unverified"}
DEMO_READINESS = {"demo_ready", "not_demo_ready", "unverified"}
SOURCE_FAMILIES = {
    "bad_timing",
    "dense_break",
    "tonal_hook",
    "tonal_pad",
    "kick_snare_loop",
    "sparse_bass_pressure",
    "other",
}
REQUIRED_VERDICTS = {"pass", "weak", "fail", "unverified"}
REQUIRED_DIMENSIONS = {
    "hook_within_two_bars",
    "hardest_audible_element",
    "source_character",
    "destructive_contrast",
    "bass_drum_pressure",
    "live_triggerability",
    "eight_bar_replay_value",
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
HEX_64_RE = re.compile(r"^[0-9a-f]{64}$")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()

    try:
        manifest = read_json_object(args.manifest)
        summary = validate_manifest(manifest, args.manifest)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(summary, indent=2) + "\n")
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid release-grade demo bank: {error}", file=sys.stderr)
        return 1

    print(f"valid release-grade demo bank: {args.manifest}")
    return 0


def validate_manifest(manifest: dict[str, Any], path: Path) -> dict[str, Any]:
    require(manifest.get("schema") == SCHEMA, f"{path}: schema must be {SCHEMA}")
    require(manifest.get("schema_version") == 1, f"{path}: schema_version must be 1")
    require(
        manifest.get("readiness_rubric_schema") == RUBRIC_SCHEMA,
        f"{path}: readiness_rubric_schema must be {RUBRIC_SCHEMA}",
    )
    require(
        manifest.get("hidden_taste_oracle_allowed") is False,
        f"{path}: hidden_taste_oracle_allowed must be false",
    )

    entries = list_field(manifest, "entries", path)
    seen_ids: set[str] = set()
    verdict_counts = {verdict: 0 for verdict in sorted(HUMAN_VERDICTS)}
    source_families: set[str] = set()
    demo_ready_count = 0

    for index, entry in enumerate(entries):
        require(isinstance(entry, dict), f"{path}: entries[{index}] must be object")
        entry_id = non_empty_string(entry.get("entry_id"), f"{path}: entries[{index}].entry_id")
        require(entry_id not in seen_ids, f"{path}: duplicate entry_id {entry_id}")
        seen_ids.add(entry_id)
        verdict, readiness = validate_entry(entry, path, index)
        verdict_counts[verdict] += 1
        source_families.add(entry["source_family"])
        if readiness == "demo_ready":
            demo_ready_count += 1

    missing_verdicts = sorted(REQUIRED_VERDICTS - {key for key, count in verdict_counts.items() if count > 0})
    require(not missing_verdicts, f"{path}: missing verdict examples: {', '.join(missing_verdicts)}")
    require("dense_break" in source_families, f"{path}: demo bank needs at least one dense_break entry")
    require(
        any(family != "dense_break" for family in source_families),
        f"{path}: demo bank needs at least one non-dense-break entry",
    )
    require(demo_ready_count >= 1, f"{path}: demo bank needs at least one demo_ready human pass entry")

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "entry_count": len(entries),
        "demo_ready_count": demo_ready_count,
        "verdict_counts": verdict_counts,
        "source_families": sorted(source_families),
    }


def validate_entry(entry: dict[str, Any], path: Path, index: int) -> tuple[str, str]:
    prefix = f"{path}: entries[{index}]"
    require_enum(entry, "source_family", SOURCE_FAMILIES, prefix)
    source_path = non_empty_string(entry.get("source_path"), f"{prefix}.source_path")
    require(source_path.endswith(".wav"), f"{prefix}.source_path must point to a WAV source")
    verdict = require_enum(entry, "human_verdict", HUMAN_VERDICTS, prefix)
    readiness = require_enum(entry, "demo_readiness", DEMO_READINESS, prefix)
    non_empty_string(entry.get("demo_worthiness_note"), f"{prefix}.demo_worthiness_note")

    validate_artifact_ref(object_field(entry, "rendered_wav", prefix), "rendered_wav", prefix, ".wav")
    validate_artifact_ref(object_field(entry, "metrics", prefix), "metrics", prefix, ".json")
    validate_artifact_ref(object_field(entry, "review_prompt", prefix), "review_prompt", prefix, ".md")
    validate_musical_summary(object_field(entry, "musical_summary", prefix), prefix)
    validate_optional_mc202_role_evidence(entry, path, index)

    fix_categories = string_list(entry, "fix_categories", prefix, allow_empty=True)
    unknown_categories = sorted(set(fix_categories) - FIX_CATEGORIES)
    require(not unknown_categories, f"{prefix}.fix_categories unknown: {', '.join(unknown_categories)}")

    if verdict == "pass":
        require(readiness == "demo_ready", f"{prefix}: pass entries must be demo_ready")
        require(not fix_categories, f"{prefix}: pass entries must not carry fix categories")
    elif verdict in {"weak", "fail"}:
        require(readiness == "not_demo_ready", f"{prefix}: weak/fail entries must be not_demo_ready")
        require(fix_categories, f"{prefix}: weak/fail entries need fix_categories")
    else:
        require(readiness == "unverified", f"{prefix}: unverified entries must stay unverified")
        require(not entry.get("quality_claim", False), f"{prefix}: unverified entries must not claim quality")

    return verdict, readiness


def validate_optional_mc202_role_evidence(entry: dict[str, Any], path: Path, index: int) -> None:
    gate = entry.get(MC202_GATE_FIELD)
    role = entry.get(MC202_ROLE_FIELD)
    if gate is None and role is None:
        return
    prefix = f"{path}: entries[{index}]"
    require(isinstance(gate, dict), f"{prefix}.{MC202_GATE_FIELD} must be object")
    require(isinstance(role, dict), f"{prefix}.{MC202_ROLE_FIELD} must be object")
    validate_promotion_gate(gate, path)
    validate_role_evidence_for_promotion(role, gate, path)


def validate_artifact_ref(ref: dict[str, Any], field: str, prefix: str, suffix: str) -> None:
    artifact_path = non_empty_string(ref.get("path"), f"{prefix}.{field}.path")
    require(artifact_path.endswith(suffix), f"{prefix}.{field}.path must end with {suffix}")
    require_hash(ref.get("sha256"), f"{prefix}.{field}.sha256")


def validate_musical_summary(summary: dict[str, Any], prefix: str) -> None:
    missing = sorted(REQUIRED_DIMENSIONS - set(summary))
    extra = sorted(set(summary) - REQUIRED_DIMENSIONS)
    require(not missing, f"{prefix}.musical_summary missing: {', '.join(missing)}")
    require(not extra, f"{prefix}.musical_summary unknown: {', '.join(extra)}")
    for name in sorted(REQUIRED_DIMENSIONS):
        non_empty_string(summary[name], f"{prefix}.musical_summary.{name}")


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def object_field(data: dict[str, Any], field: str, prefix: str) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{prefix}.{field} must be object")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def string_list(data: dict[str, Any], field: str, prefix: str, allow_empty: bool = False) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list), f"{prefix}.{field} must be array")
    require(allow_empty or bool(value), f"{prefix}.{field} must be non-empty array")
    for item in value:
        require(isinstance(item, str) and item, f"{prefix}.{field} values must be strings")
    return value


def require_enum(data: dict[str, Any], field: str, allowed: set[str], prefix: str) -> str:
    value = data.get(field)
    require(value in allowed, f"{prefix}.{field} must be one of {sorted(allowed)}")
    return str(value)


def require_hash(value: Any, message: str) -> None:
    require(isinstance(value, str) and bool(HEX_64_RE.fullmatch(value)), f"{message} must be lowercase sha256")


def non_empty_string(value: Any, message: str) -> str:
    require(isinstance(value, str) and value.strip(), message)
    return str(value)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
