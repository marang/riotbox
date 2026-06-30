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
from mc202_source_composed_review_gate import (
    MC202_GATE_FIELD,
    MC202_ROLE_FIELD,
    validate_role_evidence_for_promotion,
    validate_promotion_gate,
)
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
    "answer_bite",
    "hook_restraint",
    "mix_bus",
    "destructive_gesture",
    "destructive_articulation",
    "fixture_threshold",
    "ui_cue",
}
NON_PROMOTION_FIX_CATEGORIES = {"human_listening"}
SLUG_RE = re.compile(r"[^a-z0-9]+")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--review", type=Path, required=True)
    parser.add_argument("--demo-bank", type=Path, required=True)
    parser.add_argument("--json-output", type=Path, required=True)
    parser.add_argument("--entry-id")
    parser.add_argument("--demo-worthiness-note", required=True)
    parser.add_argument("--fix-category", action="append", default=[])
    parser.add_argument("--mc202-producer-closeout", type=Path)
    parser.add_argument("--mc202-review-case-id")
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
            mc202_producer_closeout=args.mc202_producer_closeout,
            mc202_review_case_id=args.mc202_review_case_id,
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
    mc202_producer_closeout: Path | None,
    mc202_review_case_id: str | None,
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
    metadata = object_field(review, LABEL_METADATA_FIELD, review_path)
    mc202_gate = object_field(metadata, MC202_GATE_FIELD, review_path)
    validate_promotion_gate(mc202_gate, review_path)
    mc202_role = object_field(metadata, MC202_ROLE_FIELD, review_path)
    validate_role_evidence_for_promotion(mc202_role, mc202_gate, review_path)
    identity = object_field(metadata, "artifact_identity", review_path)
    paths = object_field(metadata, "artifact_paths", review_path)
    audio_paths = object_field(paths, "audio", review_path)
    audio_identity = object_field(identity, "audio_sha256", review_path)

    rebuild_only_performance = resolve_review_path(
        review_path,
        string_field(audio_paths, "rebuild_only_performance", review_path),
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
            rebuild_only_performance,
            audio_identity.get("rebuild_only_performance"),
            review_path,
            "audio_sha256.rebuild_only_performance",
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
    producer_fix_routing = None
    if mc202_producer_closeout is not None:
        closeout = read_json_object(mc202_producer_closeout)
        producer_fix_routing = producer_fix_routing_for_review(
            closeout=closeout,
            closeout_path=mc202_producer_closeout,
            review=review,
            review_path=review_path,
            explicit_case_id=mc202_review_case_id,
            rendered_wav=rebuild_only_performance,
            rendered_sha256=string_field(audio_identity, "rebuild_only_performance", review_path),
            human_verdict=human_verdict,
        )
    categories = validate_fix_categories(
        fix_categories,
        human_verdict,
        review_path,
        producer_fix_routing,
    )

    readiness = "demo_ready" if human_verdict == "pass" else "not_demo_ready"
    source_id = string_field(metadata, "source_id", review_path)
    entry = {
        "entry_id": entry_id or f"{slug(source_id)}-human-{human_verdict}",
        "source_family": string_field(metadata, "source_family", review_path),
        "source_path": source_path_for(review, review_path),
        "rendered_wav": {
            "path": str(rebuild_only_performance),
            "sha256": string_field(audio_identity, "rebuild_only_performance", review_path),
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
        "demo_readiness_consequence": demo_readiness_consequence(human_verdict),
        "demo_worthiness_note": demo_worthiness_note,
        "fix_categories": categories,
        MC202_GATE_FIELD: mc202_gate,
        MC202_ROLE_FIELD: mc202_role,
        "musical_summary": musical_summary(review, metadata, human_verdict),
    }
    if producer_fix_routing is not None:
        entry["mc202_producer_fix_routing"] = producer_fix_routing
    return entry


def validate_fix_categories(
    categories: list[str],
    human_verdict: str,
    path: Path,
    producer_fix_routing: dict[str, Any] | None,
) -> list[str]:
    if producer_fix_routing is not None and human_verdict != "pass":
        routed_categories = string_list_from_value(
            producer_fix_routing.get("demo_bank_fix_categories")
        )
        if categories:
            require(
                sorted(categories) == sorted(routed_categories),
                f"{path}: manual fix categories must match MC-202 producer closeout routing",
            )
        else:
            categories = routed_categories
    unknown = sorted(set(categories) - FIX_CATEGORIES)
    require(not unknown, f"{path}: unknown fix categories: {', '.join(unknown)}")
    if human_verdict == "pass":
        require(not categories, f"{path}: pass promotion must not carry fix categories")
    else:
        require(categories, f"{path}: weak/fail demo-bank entries need fix categories")
    return categories


def producer_fix_routing_for_review(
    closeout: dict[str, Any],
    closeout_path: Path,
    review: dict[str, Any],
    review_path: Path,
    explicit_case_id: str | None,
    rendered_wav: Path,
    rendered_sha256: str,
    human_verdict: str,
) -> dict[str, Any]:
    require(
        closeout.get("schema") == "riotbox.mc202_producer_grade_closeout.v1",
        f"{closeout_path}: unexpected MC-202 closeout schema",
    )
    require(
        closeout.get("quality_proof") is not True,
        f"{closeout_path}: closeout must not claim quality proof",
    )
    require(
        closeout.get("automated_musical_approval") is not True,
        f"{closeout_path}: closeout must not claim automated musical approval",
    )
    metadata = object_field(review, LABEL_METADATA_FIELD, review_path)
    case_id = explicit_case_id or string_field(metadata, "source_id", review_path)
    review_candidate = find_case(closeout, "review_candidates", "case_id", case_id, closeout_path)
    require(
        Path(string_field(review_candidate, "candidate", closeout_path)).resolve()
        == rendered_wav.resolve(),
        f"{review_path}: MC-202 closeout candidate path does not match reviewed WAV",
    )
    require(
        string_field(review_candidate, "candidate_sha256", closeout_path) == rendered_sha256,
        f"{review_path}: MC-202 closeout candidate hash does not match reviewed WAV",
    )
    route = object_field(review_candidate, "mc202_producer_fix_route", review_path)
    require(
        route.get("quality_proof") is False,
        f"{closeout_path}: MC-202 fix route claims quality",
    )
    require(
        route.get("automated_musical_approval") is False,
        f"{closeout_path}: MC-202 fix route claims automated approval",
    )
    routed_categories = string_list_from_value(route.get("proposed_fix_categories"))
    aggregate_categories = [
        str(candidate["category"])
        for candidate in list_field(closeout, "mc202_producer_fix_candidates", closeout_path)
        if isinstance(candidate, dict)
        and case_id in string_list_from_value(candidate.get("case_ids"))
    ]
    require(
        set(routed_categories).issubset(set(aggregate_categories)),
        f"{closeout_path}: MC-202 aggregate fix candidates are stale for {case_id}",
    )
    demo_categories = [
        category
        for category in routed_categories
        if category not in NON_PROMOTION_FIX_CATEGORIES
    ]
    if human_verdict == "pass":
        demo_categories = []
    require(
        human_verdict == "pass" or bool(demo_categories),
        f"{review_path}: MC-202 weak/fail verdict needs non-human producer fix categories",
    )
    return {
        "schema": "riotbox.mc202_producer_fix_routing_for_human_verdict.v1",
        "case_id": case_id,
        "human_verdict": human_verdict,
        "source_family": string_field(review_candidate, "source_family", closeout_path),
        "candidate": str(rendered_wav),
        "candidate_sha256": rendered_sha256,
        "review_prompt": str(review_artifact_path(review, review_path, "prompt_markdown")),
        "review_candidate_route": route,
        "closeout_fix_categories": routed_categories,
        "demo_bank_fix_categories": demo_categories,
        "resolved_by_human_pass": human_verdict == "pass",
        "quality_proof": False,
        "automated_musical_approval": False,
    }


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


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{path}: missing {field}")
    return value


def string_field(data: dict[str, Any], field: str, path: Path) -> str:
    value = data.get(field)
    require(isinstance(value, str) and value.strip(), f"{path}: missing {field}")
    return value


def string_list_from_value(value: Any) -> list[str]:
    require(isinstance(value, list), "expected string array")
    result = []
    for item in value:
        require(isinstance(item, str) and bool(item), "expected non-empty string array values")
        result.append(item)
    return result


def find_case(
    data: dict[str, Any],
    list_name: str,
    key: str,
    expected: str,
    path: Path,
) -> dict[str, Any]:
    for item in list_field(data, list_name, path):
        if isinstance(item, dict) and item.get(key) == expected:
            return item
    raise ValueError(f"{path}: missing {list_name} entry for {key}={expected}")


def slug(value: str) -> str:
    return SLUG_RE.sub("-", value.lower()).strip("-") or "review"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def demo_readiness_consequence(human_verdict: str) -> str:
    return {
        "pass": "human_pass_allows_demo_ready_candidate",
        "weak": "human_weak_blocks_demo_ready_and_routes_fix",
        "fail": "human_fail_blocks_demo_ready_and_routes_fix",
    }[human_verdict]


if __name__ == "__main__":
    sys.exit(main())
