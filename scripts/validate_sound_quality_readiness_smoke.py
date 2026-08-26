#!/usr/bin/env python3
"""Validate the generated P023 sound-quality readiness smoke artifacts."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any, Callable

from generate_sound_quality_readiness_report import (
    demo_bank_summary,
    next_actions,
    readiness_blockers,
    reconcile_source_selection_risk,
    routed_next_fix_categories,
    source_family_coverage,
    validate_report,
)


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
        validate_outcome_reconciliation_fixture()
        validate_direct_sparse_drums_alias_fixture()
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
        "fixture_reconciliation_claims_quality",
        lambda data: set_field(
            nested_object(data, "release_demo_evidence_reconciliation"),
            "quality_proof",
            True,
        ),
        "release_demo_evidence_reconciliation_stale",
    )
    expect_failure(
        report,
        "premature_release_ready_report",
        lambda data: set_field(data, "release_readiness", "release_ready"),
        "release_ready_without_required_coverage",
    )
    expect_failure(
        report,
        "stale_aggregate_edge_block_state",
        lambda data: set_field(
            nested_object(
                data,
                "professional_output_suite",
                "source_selection_risk",
            ),
            "aggregate_edge_promotion_blocked",
            False,
        ),
        "professional_suite_source_selection_aggregate_block_state_stale",
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


def validate_outcome_reconciliation_fixture() -> None:
    def reviewed_entry(
        entry_id: str,
        source_family: str,
        outcome: str,
    ) -> dict[str, Any]:
        return {
            "entry_id": entry_id,
            "source_family": source_family,
            "human_verdict": "fail",
            "demo_readiness": "not_demo_ready",
            "fix_categories": ["source_selection"],
            "demo_worthiness_note": "Reviewed negative product outcome.",
            "degraded_or_reject_evidence": {
                "schema": "riotbox.demo_bank_degraded_or_reject_review_evidence.v1",
                "outcome": outcome,
                "product_path_reviewed": True,
                "fallback_music_present": False,
                "reason": "Fixture-calibration product handling stayed honest.",
            },
        }

    entries = [
        reviewed_entry("pad-reviewed-unavailable", "pad_noise", "unavailable"),
        reviewed_entry("timing-reviewed-reject", "bad_timing", "reject"),
        reviewed_entry("weak-reviewed-degraded", "weak_source", "degraded"),
        reviewed_entry("sparse-must-stay-weak", "sparse_bass_pressure", "degraded"),
    ]
    demo = demo_bank_summary(entries, None, "fixture_calibration")
    reviewed_ids = {
        str(entry.get("entry_id"))
        for entry in list_field(demo, "reviewed_degraded_or_reject_entries")
        if isinstance(entry, dict)
    }
    weak_ids = {
        str(entry.get("entry_id"))
        for entry in list_field(demo, "weak_or_fail_entries")
        if isinstance(entry, dict)
    }
    require(
        reviewed_ids
        == {
            "pad-reviewed-unavailable",
            "timing-reviewed-reject",
            "weak-reviewed-degraded",
        },
        "eligible reviewed-negative outcome classification changed",
    )
    require(
        weak_ids == {"sparse-must-stay-weak"},
        "positive-family weak output escaped production blocking",
    )

    routed_demo = copy.deepcopy(demo)
    for entry in list_field(routed_demo, "weak_or_fail_entries"):
        if isinstance(entry, dict) and entry.get("entry_id") == "sparse-must-stay-weak":
            entry["fix_categories"] = ["chop_policy"]
    routed_demo["weak_fix_categories"] = ["chop_policy"]
    failed_positive_coverage = {
        "missing_family_success_families": ["sparse_drums"],
        "missing_human_verdict_families": [],
    }
    empty_review_queue = {"candidates": []}
    current_evidence = {
        "current_product_top_candidate_category": "none",
        "stale_fixture_only_categories": ["chop_policy"],
    }
    categories = routed_next_fix_categories(
        failed_positive_coverage,
        routed_demo,
        {"fix_categories": []},
        current_evidence,
        empty_review_queue,
        True,
    )
    require(
        categories == ["chop_policy"],
        "current human failure category was incorrectly suppressed as stale fixture evidence",
    )
    actions = next_actions(
        failed_positive_coverage,
        routed_demo,
        {"fix_categories": []},
        {"available": True},
        current_evidence,
        empty_review_queue,
        {"demo_bank_state": "available"},
    )
    require(
        any(
            action.get("category") == "chop_policy"
            and action.get("target") == "sparse_drums"
            and action.get("entry_ids") == ["sparse-must-stay-weak"]
            for action in actions
        ),
        "current human failure did not route its bounded production correction",
    )

    suite = {
        "available": True,
        "result": "pass",
        "scripted_generation": False,
        "quality_proof": True,
        "source_selection_risk": {
            "blocked_source_families": ["bad_timing", "pad_noise"],
            "required_review_actions": [
                "audition_pad_noise_texture_before_demo_promotion",
                "confirm_timing_before_bar_locked_moves",
                "keep_as_diagnostic_until_human_verdict",
            ],
        },
    }
    coverage = {
        "families": [
            {
                "source_family": "bad_timing",
                "status": "reviewed_degraded_or_reject",
                "has_family_success": True,
            },
            {
                "source_family": "pad_noise",
                "status": "reviewed_degraded_or_reject",
                "has_family_success": True,
            },
        ],
        "missing_demo_candidate_families": [],
        "missing_human_verdict_families": [],
        "missing_family_success_families": [],
    }
    reconcile_source_selection_risk(suite, coverage)
    risk = nested_object(suite, "source_selection_risk")
    require(
        risk.get("reviewed_product_outcome_families") == ["bad_timing", "pad_noise"]
        and risk.get("unresolved_blocked_source_families") == []
        and risk.get("aggregate_edge_promotion_blocked") is False,
        "reviewed edge-source outcomes did not clear aggregate risk",
    )

    blockers = readiness_blockers(
        coverage,
        demo,
        {"available": True, "result": "pass"},
        suite,
        {
            "available": True,
            "result": "pass",
            "review_queue_count": 0,
            "candidates": [],
            "source_families": [],
        },
        {"demo_bank_state": "available"},
    )
    weak_blocker = next(
        (
            blocker
            for blocker in blockers
            if blocker.get("code") == "weak_or_fail_demo_bank_entries_present"
        ),
        None,
    )
    require(
        isinstance(weak_blocker, dict)
        and weak_blocker.get("entries") == ["sparse-must-stay-weak"],
        "generic weak blocker did not retain only the positive-family failure",
    )
    require(
        all(
            blocker.get("code") != "edge_source_selection_promotion_blocked"
            for blocker in blockers
        ),
        "reviewed edge-source outcomes still emitted aggregate blocker",
    )

    partial_coverage = copy.deepcopy(coverage)
    for family in list_field(partial_coverage, "families"):
        if isinstance(family, dict) and family.get("source_family") == "bad_timing":
            family["status"] = "candidate_only"
            family["has_family_success"] = False
    partial_suite = copy.deepcopy(suite)
    reconcile_source_selection_risk(partial_suite, partial_coverage)
    partial_risk = nested_object(partial_suite, "source_selection_risk")
    require(
        partial_risk.get("reviewed_product_outcome_families") == ["pad_noise"]
        and partial_risk.get("unresolved_blocked_source_families") == ["bad_timing"]
        and partial_risk.get("unresolved_required_review_actions")
        == [
            "confirm_timing_before_bar_locked_moves",
            "keep_as_diagnostic_until_human_verdict",
        ]
        and partial_risk.get("aggregate_edge_promotion_blocked") is True,
        "partial edge-source review did not retain only unresolved timing risk",
    )
    partial_blockers = readiness_blockers(
        partial_coverage,
        demo,
        {"available": True, "result": "pass"},
        partial_suite,
        {
            "available": True,
            "result": "pass",
            "review_queue_count": 0,
            "candidates": [],
            "source_families": [],
        },
        {"demo_bank_state": "available"},
    )
    require(
        any(
            blocker.get("code") == "edge_source_selection_promotion_blocked"
            and blocker.get("families") == ["bad_timing"]
            for blocker in partial_blockers
        ),
        "partial edge-source review did not keep bad_timing blocked",
    )


def validate_direct_sparse_drums_alias_fixture() -> None:
    coverage = source_family_coverage(
        {
            "required_source_families": ["sparse_drums"],
            "entries": [
                {
                    "case_id": "sparse_kicksnr_120",
                    "source_family": "sparse_drums",
                }
            ],
        },
        [
            {
                "entry_id": "sparse-drums-exact-product-pass",
                "source_family": "sparse_drums",
                "human_verdict": "pass",
                "demo_readiness": "demo_ready",
            }
        ],
        Path("fixture-source-corpus.json"),
        "fixture_calibration",
    )
    sparse = first_dict_item(coverage, "families")
    require(
        sparse.get("demo_bank_family_aliases")
        == ["sparse_bass_pressure", "sparse_drums"]
        and sparse.get("status") == "demo_ready_covered"
        and coverage.get("missing_family_success_families") == [],
        "direct sparse_drums release-demo pass was not recognized by readiness",
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
