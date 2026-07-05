#!/usr/bin/env python3
"""Validate the generated P023 sound-quality readiness smoke artifacts."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any, Callable

from generate_sound_quality_readiness_report import validate_report


DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-sound-quality-readiness-report")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--report-json", type=Path)
    parser.add_argument("--report-markdown", type=Path)
    args = parser.parse_args()

    report_json = args.report_json or args.output / "sound-quality-readiness-report.json"
    report_markdown = args.report_markdown or args.output / "sound-quality-readiness-report.md"

    try:
        report = read_json_object(report_json)
        markdown = report_markdown.read_text()
        failures = validate_report(report)
        if failures:
            raise ValueError(f"{report_json}: {', '.join(failures)}")
        validate_markdown(markdown, args.output)
        validate_mutation_fixtures(report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid sound-quality readiness smoke: {error}", file=sys.stderr)
        return 1

    print(f"valid sound-quality readiness smoke: {report_json}")
    return 0


def validate_markdown(markdown: str, output: Path) -> None:
    expected = [
        "Release-Demo Review Worklist",
        "Generated review packs: `5`",
        (
            "Review pack: "
            f"`{output}/release-demo-review-packs/bad-timing-beat20-unverified-candidate`"
        ),
        (
            "Pack prompt: "
            f"`{output}/release-demo-review-packs/bad-timing-beat20-unverified-candidate/prompt.md`"
        ),
        "Verdict target: record `pass`, `weak`, or `fail`",
        "Listening questions:",
        "Human Review Queue",
        (
            "Rendered WAV: "
            "`artifacts/audio_qa/local-edge-source-professional-diagnostics/"
            "bad_timing_beat20_128/render/05_rebuild_only_performance.wav`"
        ),
        "Review prompt: `artifacts/audio_qa/release-demo-bank/bad-timing-beat20/prompt.md`",
    ]
    for needle in expected:
        require(needle in markdown, f"markdown missing expected text: {needle}")


def validate_mutation_fixtures(report: dict[str, Any]) -> None:
    expect_failure(
        report,
        "missing_source_referenced_destructive_proof",
        lambda data: update_nested(
            data,
            [
                ("professional_output_suite", "destructive_gesture", "stutter_to_source_transient_ratio"),
                ("professional_output_suite", "destructive_gesture", "restore_to_source_transient_ratio"),
            ],
            0,
        ),
        "current_evidence_reconciliation_destructive_gesture_stale_without_current_proof",
    )
    expect_failure(
        report,
        "missing_tonal_source_selection_policy_coverage",
        lambda data: remove_list_value(
            nested_object(data, "professional_output_suite", "source_selection_policy"),
            "promotion_allowed_source_families",
            "tonal_hook",
        ),
        "current_evidence_reconciliation_source_selection_stale_without_candidate_family_coverage",
    )
    expect_failure(
        report,
        "missing_rendered_tr909_drum_pressure_proof",
        lambda data: set_field(
            nested_object(data, "professional_output_suite", "drum_pressure"),
            "tr909_rendered_min_support_mix_contribution_ratio",
            0,
        ),
        "current_evidence_reconciliation_drum_pressure_stale_without_current_proof",
    )
    expect_failure(
        report,
        "broken_perform_risk_cue_contract",
        lambda data: set_field(
            nested_object(data, "jam_perform_risk_cue_contract"),
            "unavailable_action",
            "load source",
        ),
        "current_evidence_reconciliation_ui_cue_stale_without_tui_contract",
    )
    expect_failure(
        report,
        "stale_fixture_threshold_primary_case",
        make_fixture_threshold_primary_case_stale,
        "current_evidence_reconciliation_fixture_threshold_stale_without_negative_control_proof",
    )
    expect_failure(
        report,
        "stale_weak_output_next_action",
        lambda data: list_field(data, "next_actions").append(
            {
                "category": "fixture_threshold",
                "target": "weak output",
                "action": "Keep stale fixture_threshold fixtures as regression controls.",
            }
        ),
        "next_actions_stale_weak_output_controls_obscure_review_work",
    )
    expect_failure(
        report,
        "missing_source_family_review_candidate_id",
        lambda data: mutate_source_selection_action(data, "bad_timing", "candidate_id"),
        "next_actions_bad_timing_review_candidate_context_missing",
    )
    expect_failure(
        report,
        "missing_source_family_review_artifact_ref",
        lambda data: mutate_source_selection_action(data, "bad_timing", "rendered_wav"),
        "next_actions_bad_timing_review_candidate_context_missing",
    )
    expect_failure(
        report,
        "blocked_report_claims_quality",
        lambda data: set_field(data, "quality_claim_allowed", True),
        "blocked_report_claims_quality",
    )
    expect_failure(
        report,
        "premature_release_ready_report",
        lambda data: set_field(data, "release_readiness", "release_ready"),
        "release_ready_without_required_coverage",
    )
    expect_failure(
        report,
        "missing_professional_suite_source_character_context",
        lambda data: nested_object(data, "professional_output_suite").pop(
            "source_character_selection",
            None,
        ),
        "professional_suite_source_character_selection_missing",
    )
    expect_failure(
        report,
        "missing_weak_output_fix_summary",
        lambda data: nested_object(data, "weak_output_routing").pop(
            "production_fix_summary",
            None,
        ),
        "weak_output_routing_fix_summary_missing",
    )
    expect_failure(
        report,
        "missing_human_review_blockers",
        lambda data: remove_first_list_item_field(
            nested_object(data, "human_review_queue"),
            "candidates",
            "review_blockers",
        ),
        "human_review_queue_candidate_0_review_blockers_missing",
    )
    expect_failure(
        report,
        "stale_human_review_verdict_state",
        lambda data: set_first_list_item_field(
            nested_object(data, "human_review_queue"),
            "candidates",
            "required_verdict_current_state",
            "human_verdict:pass/demo_readiness:demo_ready",
        ),
        "human_review_queue_candidate_0_stale_verdict_state",
    )
    expect_failure(
        report,
        "human_review_quality_claim",
        lambda data: set_first_list_item_field(
            nested_object(data, "human_review_queue"),
            "candidates",
            "quality_claim",
            True,
        ),
        "human_review_queue_candidate_0_claims_quality",
    )
    expect_failure(
        report,
        "verified_release_demo_review_pack",
        lambda data: set_first_list_item_field(
            nested_object(data, "release_demo_review_packs"),
            "packs",
            "human_verdict",
            "pass",
        ),
        "release_demo_review_pack_0_human_verdict_not_unverified",
    )
    expect_failure(
        report,
        "missing_candidate_review_pack",
        remove_bad_timing_review_pack,
        "release_demo_review_pack_bad-timing-beat20-unverified-candidate_candidate_context_missing",
    )


def expect_failure(
    report: dict[str, Any],
    fixture_name: str,
    mutate: Callable[[dict[str, Any]], object],
    expected_code: str,
) -> None:
    mutated = copy.deepcopy(report)
    mutate(mutated)
    failures = validate_report(mutated)
    require(failures, f"{fixture_name}: mutation unexpectedly passed")
    require(
        expected_code in failures,
        f"{fixture_name}: expected {expected_code}, got {', '.join(failures)}",
    )


def make_fixture_threshold_primary_case_stale(data: dict[str, Any]) -> None:
    for candidate in list_field(nested_object(data, "weak_output_routing"), "production_fix_candidates"):
        if isinstance(candidate, dict) and candidate.get("category") == "fixture_threshold":
            candidate["primary_case_ids"] = ["rendered_dense_flat_stutter"]


def mutate_source_selection_action(data: dict[str, Any], target: str, field: str) -> None:
    for action in list_field(data, "next_actions"):
        if (
            isinstance(action, dict)
            and action.get("category") == "source_selection"
            and action.get("target") == target
        ):
            action.pop(field, None)


def remove_bad_timing_review_pack(data: dict[str, Any]) -> None:
    review_packs = nested_object(data, "release_demo_review_packs")
    packs = list_field(review_packs, "packs")
    review_packs["packs"] = [
        pack
        for pack in packs
        if not (
            isinstance(pack, dict)
            and pack.get("entry_id") == "bad-timing-beat20-unverified-candidate"
        )
    ]
    review_packs["review_pack_count"] = len(review_packs["packs"])


def update_nested(
    data: dict[str, Any],
    paths: list[tuple[str, ...]],
    value: Any,
) -> None:
    for path in paths:
        current: dict[str, Any] = data
        for key in path[:-1]:
            current = nested_object(current, key)
        current[path[-1]] = value


def remove_list_value(data: dict[str, Any], field: str, value: str) -> None:
    data[field] = [item for item in list_field(data, field) if item != value]


def set_field(data: dict[str, Any], field: str, value: Any) -> None:
    data[field] = value


def set_first_list_item_field(
    data: dict[str, Any],
    list_name: str,
    field: str,
    value: Any,
) -> None:
    first = first_dict_item(data, list_name)
    first[field] = value


def remove_first_list_item_field(
    data: dict[str, Any],
    list_name: str,
    field: str,
) -> None:
    first = first_dict_item(data, list_name)
    first.pop(field, None)


def first_dict_item(data: dict[str, Any], list_name: str) -> dict[str, Any]:
    values = list_field(data, list_name)
    require(values, f"{list_name} must not be empty")
    first = values[0]
    require(isinstance(first, dict), f"{list_name}[0] must be object")
    return first


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def nested_object(data: dict[str, Any], *path: str) -> dict[str, Any]:
    current: Any = data
    for key in path:
        current = current.get(key) if isinstance(current, dict) else None
        require(isinstance(current, dict), f"{'.'.join(path)} must be object")
    return current


def list_field(data: dict[str, Any], field: str) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{field} must be list")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
