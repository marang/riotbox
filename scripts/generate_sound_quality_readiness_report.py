#!/usr/bin/env python3
"""Generate the P023 sound-quality readiness report."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any

from sound_quality_readiness_human_review import (
    DEFAULT_HUMAN_REVIEW_QUEUE,
    human_review_queue_summary,
    validate_human_review_queue_section,
)


SCHEMA = "riotbox.sound_quality_readiness_report.v1"
RUBRIC_SCHEMA = "riotbox.sound_product_readiness_rubric.v1"
SOURCE_CORPUS_SCHEMA = "riotbox.sound_excellence_source_corpus.v1"
DEMO_BANK_SCHEMA = "riotbox.release_grade_demo_bank.v1"
WEAK_ROUTING_SCHEMA = "riotbox.weak_output_fix_routing.v1"
PROFESSIONAL_SUITE_SCHEMA = "riotbox.professional_output_suite.v1"

DEFAULT_RUBRIC = Path("scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json")
DEFAULT_SOURCE_CORPUS = Path("docs/benchmarks/sound_excellence_source_corpus_v1.json")
DEFAULT_DEMO_BANK = Path("scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json")
DEFAULT_WEAK_ROUTING = Path("artifacts/audio_qa/local-weak-output-fix-routing/weak-output-fix-routing.json")
DEFAULT_PROFESSIONAL_SUITE = Path("artifacts/audio_qa/local-professional-output-suite/professional-output-suite.json")
DEFAULT_PERFORM_RISK_CUE_CONTRACT = Path(
    "artifacts/audio_qa/local-jam-perform-risk-cue-contract/jam-perform-risk-cue-contract.json"
)
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-sound-quality-readiness-report")
MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO = 0.22
MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO = 0.35
MAX_HOOK_CHOP_RESPONSE_CORRELATION = 0.92
MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO = 0.58
MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ = 1.75
MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ = 17.00
MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO = 2.70
MIN_SPARSE_PRESSURE_LOW_BAND_SHARE = 0.36
MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO = 2.45
MIN_SPARSE_BASS_DOMINANCE_MARGIN = 0.20
MIN_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.145
MAX_MIX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO = 0.08
MIN_MIX_SOURCE_FIRST_MASKING_HEADROOM = 0.04
MAX_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.46
MAX_DESTRUCTIVE_DROPOUT_TO_STUTTER_RMS_RATIO = 0.0065
MAX_DESTRUCTIVE_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO = 0.0065
MIN_DESTRUCTIVE_STUTTER_TO_HOOK_TRANSIENT_RATIO = 1.55
MIN_DESTRUCTIVE_RESTORE_TO_HOOK_TRANSIENT_RATIO = 1.60
MIN_DESTRUCTIVE_STUTTER_TO_SOURCE_TRANSIENT_RATIO = 5.50
MIN_DESTRUCTIVE_RESTORE_TO_SOURCE_TRANSIENT_RATIO = 6.00
MIN_DESTRUCTIVE_RESTORE_TO_PRESSURE_RMS_RATIO = 1.36
MIN_DESTRUCTIVE_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO = 6.00
EXPECTED_SOURCE_SELECTION_DEMOTION_REASONS = [
    "diagnostic_only_not_quality_proof",
    "texture_review_required",
    "timing_review_required",
]
EXPECTED_SOURCE_SELECTION_REVIEW_ACTIONS = [
    "audition_pad_noise_texture_before_demo_promotion",
    "confirm_timing_before_bar_locked_moves",
    "keep_as_diagnostic_until_human_verdict",
]
MIN_SOURCE_SELECTION_POLICY_CANDIDATES = 3
MIN_SOURCE_SELECTION_RMS_RETENTION_RATIO = 0.60
MIN_SOURCE_SELECTION_SCORE_LIFT = 0.0
MIN_DENSE_DRUM_PRESSURE_SCORE = 1.58
MIN_DENSE_SNARE_PRESSURE_MARGIN = 0.22
MIN_DENSE_PRESSURE_TRANSIENT_TO_HOOK_RATIO = 0.70
MIN_TR909_RENDERED_DRUM_PRESSURE_CASES = 8
MAX_TR909_RENDERED_SOURCE_FIRST_RATIO = 0.08
MAX_TR909_RENDERED_SUPPORT_RATIO = 0.46

CORPUS_TO_DEMO_FAMILIES = {
    "dense_break": {"dense_break"},
    "sparse_drums": {"sparse_bass_pressure"},
    "tonal_riff": {"tonal_hook"},
    "pad_noise": {"tonal_pad"},
    "weak_source": {"other"},
    "bad_timing": {"bad_timing"},
}

PERFORM_RISK_CUE_SCHEMA = "riotbox.jam_perform_risk_cue_contract.v1"

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rubric", type=Path, default=DEFAULT_RUBRIC)
    parser.add_argument("--source-corpus", type=Path, default=DEFAULT_SOURCE_CORPUS)
    parser.add_argument("--demo-bank", type=Path, default=DEFAULT_DEMO_BANK)
    parser.add_argument("--weak-routing", type=Path, default=DEFAULT_WEAK_ROUTING)
    parser.add_argument("--professional-output-suite", type=Path, default=DEFAULT_PROFESSIONAL_SUITE)
    parser.add_argument("--perform-risk-cue-contract", type=Path, default=DEFAULT_PERFORM_RISK_CUE_CONTRACT)
    parser.add_argument("--human-review-queue", type=Path, default=DEFAULT_HUMAN_REVIEW_QUEUE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-sound-quality-readiness-report")
    parser.add_argument("--validate-report", type=Path)
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json_object(args.validate_report)
            failures = validate_report(report)
            if failures:
                raise ValueError(", ".join(failures))
            print(f"valid sound-quality readiness report: {args.validate_report}")
            return 0

        report = build_report(args)
        failures = validate_report(report)
        if failures:
            raise ValueError(", ".join(failures))
        args.output.mkdir(parents=True, exist_ok=True)
        write_report(args.output, report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid sound-quality readiness report: {error}", file=sys.stderr)
        return 1

    print(f"sound-quality readiness report written to {args.output}")
    return 0


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    rubric = read_json_object(args.rubric)
    source_corpus = read_json_object(args.source_corpus)
    demo_bank = read_json_object(args.demo_bank)
    weak_routing = read_optional_json_object(args.weak_routing)
    professional_suite = read_optional_json_object(args.professional_output_suite)
    perform_risk_cue_contract = read_optional_json_object(args.perform_risk_cue_contract)
    human_review_queue = read_optional_json_object(args.human_review_queue)

    require(rubric.get("schema") == RUBRIC_SCHEMA, f"{args.rubric}: schema must be {RUBRIC_SCHEMA}")
    require(
        source_corpus.get("schema") == SOURCE_CORPUS_SCHEMA,
        f"{args.source_corpus}: schema must be {SOURCE_CORPUS_SCHEMA}",
    )
    require(
        demo_bank.get("schema") == DEMO_BANK_SCHEMA,
        f"{args.demo_bank}: schema must be {DEMO_BANK_SCHEMA}",
    )

    source_families = source_family_coverage(source_corpus, demo_bank, args.source_corpus)
    demo_summary = demo_bank_summary(demo_bank, args.demo_bank)
    weak_summary = weak_routing_summary(weak_routing, args.weak_routing)
    suite_summary = professional_suite_summary(professional_suite, args.professional_output_suite)
    perform_risk_cue_summary = perform_risk_cue_contract_summary(
        perform_risk_cue_contract,
        args.perform_risk_cue_contract,
    )
    current_evidence = current_evidence_reconciliation(
        weak_summary,
        suite_summary,
        perform_risk_cue_summary,
    )
    source_selection_priority = source_selection_priority_detail(
        weak_summary,
        suite_summary,
        current_evidence,
    )
    ui_cue_priority = ui_cue_priority_detail(
        weak_summary,
        suite_summary,
        current_evidence,
    )
    review_summary = human_review_queue_summary(human_review_queue, args.human_review_queue)
    blockers = readiness_blockers(
        source_families,
        demo_summary,
        weak_summary,
        suite_summary,
        review_summary,
    )

    release_readiness = "release_ready" if not blockers else "blocked"
    quality_claim_allowed = release_readiness == "release_ready"
    stale_categories = set(
        string_list(current_evidence.get("stale_fixture_only_categories"))
    )
    raw_next_fix_categories = set(weak_summary["fix_categories"]) | set(
        demo_summary["weak_fix_categories"]
    )
    next_fix_categories = sorted(
        category for category in raw_next_fix_categories if category not in stale_categories
    )
    if (
        source_families["missing_human_verdict_families"]
        or source_families["missing_demo_ready_families"]
    ):
        next_fix_categories = sorted(set(next_fix_categories) | {"source_selection"})
    if not next_fix_categories and blockers:
        next_fix_categories = ["source_selection", "fixture_threshold"]

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "created_at": args.date,
        "result": "pass",
        "phase": "P023",
        "release_readiness": release_readiness,
        "quality_claim_allowed": quality_claim_allowed,
        "human_verdict_boundary": "human verdicts are required for product-quality claims",
        "evidence_boundary": (
            "This report aggregates existing QA, corpus, demo-bank, and weak-output "
            "routing evidence. It is not a hidden taste oracle and it does not turn "
            "scripted or unverified diagnostics into product-quality proof."
        ),
        "rubric": {
            "path": str(args.rubric),
            "schema": rubric["schema"],
            "state_count": len(object_field(rubric, "states", args.rubric)),
            "quality_states": sorted(
                key for key, state in object_field(rubric, "states", args.rubric).items()
                if state.get("may_claim_product_quality") is True
            ),
            "musical_dimension_count": len(object_field(rubric, "musical_dimensions", args.rubric)),
        },
        "source_family_coverage": source_families,
        "demo_bank": demo_summary,
        "weak_output_routing": weak_summary,
        "professional_output_suite": suite_summary,
        "jam_perform_risk_cue_contract": perform_risk_cue_summary,
        "current_evidence_reconciliation": current_evidence,
        "source_selection_priority": source_selection_priority,
        "ui_cue_priority": ui_cue_priority,
        "human_review_queue": review_summary,
        "blockers": blockers,
        "next_actions": next_actions(
            source_families,
            demo_summary,
            weak_summary,
            suite_summary,
            current_evidence,
            review_summary,
        ),
        "next_fix_categories": next_fix_categories,
        "musician_summary": musician_summary(blockers, next_fix_categories),
    }


def source_family_coverage(source_corpus: dict[str, Any], demo_bank: dict[str, Any], path: Path) -> dict[str, Any]:
    required = string_list_field(source_corpus, "required_source_families", path)
    corpus_entries = list_field(source_corpus, "entries", path)
    demo_entries = list_field(demo_bank, "entries", Path("demo_bank"))
    demo_families = {
        str(entry.get("source_family"))
        for entry in demo_entries
        if entry.get("human_verdict") == "pass" and entry.get("demo_readiness") == "demo_ready"
    }
    human_families = {
        str(entry.get("source_family"))
        for entry in demo_entries
        if entry.get("human_verdict") in {"pass", "weak", "fail"}
    }
    all_demo_families = {str(entry.get("source_family")) for entry in demo_entries}

    families = []
    for family in required:
        mapped = CORPUS_TO_DEMO_FAMILIES.get(family, {family})
        has_any_candidate = bool(mapped & all_demo_families)
        has_human_verdict = bool(mapped & human_families)
        has_demo_ready = bool(mapped & demo_families)
        corpus_case_ids = [
            str(entry.get("case_id"))
            for entry in corpus_entries
            if entry.get("source_family") == family
        ]
        families.append(
            {
                "source_family": family,
                "demo_bank_family_aliases": sorted(mapped),
                "corpus_case_ids": corpus_case_ids,
                "has_demo_candidate": has_any_candidate,
                "has_human_verdict": has_human_verdict,
                "has_demo_ready_human_pass": has_demo_ready,
                "status": source_family_coverage_status(
                    has_any_candidate,
                    has_human_verdict,
                    has_demo_ready,
                ),
            }
        )

    missing_candidates = [item["source_family"] for item in families if not item["has_demo_candidate"]]
    missing_human_verdict = [item["source_family"] for item in families if not item["has_human_verdict"]]
    missing_demo_ready = [item["source_family"] for item in families if not item["has_demo_ready_human_pass"]]
    return {
        "path": str(path),
        "required_source_families": required,
        "covered_demo_candidate_families": [
            item["source_family"] for item in families if item["has_demo_candidate"]
        ],
        "covered_human_verdict_families": [
            item["source_family"] for item in families if item["has_human_verdict"]
        ],
        "covered_demo_ready_families": [
            item["source_family"] for item in families if item["has_demo_ready_human_pass"]
        ],
        "missing_demo_candidate_families": missing_candidates,
        "missing_human_verdict_families": missing_human_verdict,
        "missing_demo_ready_families": missing_demo_ready,
        "families": families,
    }


def source_family_coverage_status(
    has_candidate: bool,
    has_human_verdict: bool,
    has_demo_ready: bool,
) -> str:
    if has_demo_ready:
        return "demo_ready_covered"
    if has_human_verdict:
        return "human_verdict_non_demo"
    if has_candidate:
        return "candidate_only"
    return "missing_candidate"


def demo_bank_summary(demo_bank: dict[str, Any], path: Path) -> dict[str, Any]:
    entries = list_field(demo_bank, "entries", path)
    verdict_counts = Counter(str(entry.get("human_verdict")) for entry in entries)
    readiness_counts = Counter(str(entry.get("demo_readiness")) for entry in entries)
    weak_fix_categories = sorted(
        {
            category
            for entry in entries
            if entry.get("human_verdict") in {"weak", "fail"}
            for category in list(entry.get("fix_categories", []))
            if isinstance(category, str) and category
        }
    )
    unverified = [
        str(entry.get("entry_id"))
        for entry in entries
        if entry.get("human_verdict") == "unverified" or entry.get("demo_readiness") == "unverified"
    ]
    weak_or_fail = [
        {
            "entry_id": str(entry.get("entry_id")),
            "human_verdict": str(entry.get("human_verdict")),
            "demo_readiness": str(entry.get("demo_readiness")),
            "fix_categories": list(entry.get("fix_categories", [])),
            "reason": str(entry.get("demo_worthiness_note", "")),
        }
        for entry in entries
        if entry.get("human_verdict") in {"weak", "fail"}
    ]
    return {
        "path": str(path),
        "entry_count": len(entries),
        "demo_ready_count": readiness_counts.get("demo_ready", 0),
        "human_verdict_counts": dict(sorted(verdict_counts.items())),
        "demo_readiness_counts": dict(sorted(readiness_counts.items())),
        "unverified_candidate_ids": unverified,
        "weak_or_fail_entries": weak_or_fail,
        "weak_fix_categories": weak_fix_categories,
    }


def weak_routing_summary(report: dict[str, Any] | None, path: Path) -> dict[str, Any]:
    if report is None:
        return {
            "path": str(path),
            "available": False,
            "result": "missing",
            "case_count": 0,
            "fix_categories": [],
            "production_fix_candidate_count": 0,
            "production_fix_summary": {},
            "production_fix_candidates": [],
            "cases": [],
        }
    require(report.get("schema") == WEAK_ROUTING_SCHEMA, f"{path}: schema must be {WEAK_ROUTING_SCHEMA}")
    cases = list(report.get("cases", []))
    candidates = weak_routing_candidates(report, path)
    summary = object_or_empty(report.get("production_fix_summary"))
    require(summary, f"{path}: production_fix_summary must be object")
    require(
        summary.get("candidate_count") == len(candidates),
        f"{path}: production_fix_summary candidate_count mismatch",
    )
    require(
        summary.get("categories") == [candidate["category"] for candidate in candidates],
        f"{path}: production_fix_summary categories mismatch",
    )
    return {
        "path": str(path),
        "available": True,
        "result": str(report.get("result")),
        "case_count": int(report.get("case_count", len(cases))),
        "routed_case_count": int(report.get("routed_case_count", 0)),
        "quality_proof": report.get("quality_proof"),
        "automated_musical_approval": report.get("automated_musical_approval"),
        "fix_categories": list(report.get("fix_categories", [])),
        "production_fix_candidate_count": len(candidates),
        "production_fix_summary": summary,
        "production_fix_candidates": candidates,
        "cases": [
            {
                "case_id": str(case.get("case_id")),
                "artifact_to_hear": str(case.get("artifact_to_hear")),
                "proposed_next_fix_category": str(case.get("proposed_next_fix_category")),
                "proposed_fix_categories": string_list(case.get("proposed_fix_categories")),
                "failure_codes": string_list(case.get("failure_codes")),
                "matched_known_routing_signal": case.get("matched_known_routing_signal"),
                "musician_fix_reason": str(case.get("musician_fix_reason")),
            }
            for case in cases
        ],
    }


def weak_routing_candidates(report: dict[str, Any], path: Path) -> list[dict[str, Any]]:
    raw_count = report.get("production_fix_candidate_count", 0)
    raw_candidates = report.get("production_fix_candidates", [])
    require(isinstance(raw_count, int), f"{path}: production_fix_candidate_count must be integer")
    require(isinstance(raw_candidates, list), f"{path}: production_fix_candidates must be array")
    require(raw_count == len(raw_candidates), f"{path}: production_fix_candidate_count mismatch")
    candidates = []
    for index, candidate in enumerate(raw_candidates):
        require(isinstance(candidate, dict), f"{path}: production_fix_candidates[{index}] must be object")
        candidates.append(
            {
                "candidate_id": required_string(candidate, "candidate_id", path, index),
                "category": required_string(candidate, "category", path, index),
                "case_ids": string_list(candidate.get("case_ids")),
                "primary_case_ids": string_list(candidate.get("primary_case_ids")),
                "source_families": string_list(candidate.get("source_families")),
                "artifact_refs": string_list(candidate.get("artifact_refs")),
                "software_next_step": required_string(candidate, "software_next_step", path, index),
                "musician_payoff": required_string(candidate, "musician_payoff", path, index),
            }
        )
    return candidates


def professional_suite_summary(report: dict[str, Any] | None, path: Path) -> dict[str, Any]:
    if report is None:
        return {
            "path": str(path),
            "available": False,
            "result": "missing",
            "human_verdict": "unverified",
            "scripted_generation": None,
            "quality_proof": None,
        }
    require(
        report.get("schema") == PROFESSIONAL_SUITE_SCHEMA,
        f"{path}: schema must be {PROFESSIONAL_SUITE_SCHEMA}",
    )
    children = list(report.get("children", []))
    metrics = {
        str(child.get("id")): object_or_empty(child.get("key_metrics"))
        for child in children
        if isinstance(child, dict)
    }
    dense = metrics.get("dense_break", {})
    matrix = metrics.get("pro_pressure_source_matrix", {})
    source_wav = metrics.get("professional_source_wav_pack", {})
    destructive = metrics.get("destructive_variation", {})
    edge = metrics.get("edge_source_professional_diagnostics", {})
    feral_mix_balance = object_or_empty(report.get("feral_mix_balance"))
    source_character_window_selection = object_or_empty(
        report.get("source_character_window_selection")
    )
    source_selection_policy = object_or_empty(report.get("source_selection_policy"))
    source_selection_policy_cases = list_or_empty(source_selection_policy.get("cases"))
    source_selection_policy_families = string_list(
        source_selection_policy.get("source_families")
    ) or sorted(
        {
            str(case.get("source_family"))
            for case in source_selection_policy_cases
            if isinstance(case, dict) and str(case.get("source_family") or "")
        }
    )
    source_selection_policy_promotion_families = string_list(
        source_selection_policy.get("promotion_allowed_source_families")
    ) or sorted(
        {
            str(case.get("source_family"))
            for case in source_selection_policy_cases
            if isinstance(case, dict)
            and str(case.get("source_family") or "")
            and case.get("promotion_allowed") is True
        }
    )
    source_selection_policy_required_candidate_counts = [
        int(number(case.get("candidate_count")))
        for case in source_selection_policy_cases
        if isinstance(case, dict) and case.get("candidate_floor_required") is True
    ]
    tr909_rendered_drum_pressure = object_or_empty(
        report.get("tr909_rendered_drum_pressure")
    )
    strongest_elements = sorted(
        {
            str(value)
            for value in (
                [dense.get("strongest_audible_element")]
                + list(matrix.get("strongest_audible_elements") or [])
                + list(source_wav.get("strongest_audible_elements") or [])
                + list(edge.get("strongest_audible_elements") or [])
            )
            if isinstance(value, str) and value
        }
    )
    return {
        "path": str(path),
        "available": True,
        "result": str(report.get("result")),
        "human_verdict": str(report.get("human_verdict")),
        "scripted_generation": report.get("scripted_generation"),
        "quality_proof": report.get("quality_proof"),
        "child_report_count": report.get("child_report_count"),
        "passed_child_report_count": report.get("passed_child_report_count"),
        "failed_child_report_count": report.get("failed_child_report_count"),
        "strongest_audible_elements": strongest_elements,
        "source_character_selection": {
            "dense_w30_to_source_rms_ratio": number(
                dense.get("w30_to_source_rms_ratio")
            ),
            "dense_hook_chop_w30_to_source_margin": number(
                dense.get("hook_chop_w30_to_source_margin")
            ),
            "dense_hook_chop_score_floor": number(
                dense.get("hook_chop_source_character_score_floor")
            ),
            "dense_hook_chop_score_span": number(
                dense.get("hook_chop_source_character_score_span")
            ),
            "dense_hook_chop_response_delta_ratio": number(
                dense.get("hook_chop_response_delta_ratio")
            ),
            "dense_hook_chop_response_correlation": number(
                dense.get("hook_chop_response_correlation")
            ),
            "dense_hook_chop_response_transient_ratio": number(
                dense.get("hook_chop_response_transient_ratio")
            ),
            "matrix_dense_hook_chop_score_floor": number(
                matrix.get("min_dense_hook_chop_source_character_score_floor")
            ),
            "matrix_dense_hook_chop_score_span": number(
                matrix.get("min_dense_hook_chop_source_character_score_span")
            ),
            "matrix_dense_w30_to_source_rms_ratio": number(
                matrix.get("min_dense_w30_to_source_rms_ratio")
            ),
            "matrix_dense_hook_chop_w30_to_source_margin": number(
                matrix.get("min_dense_hook_chop_w30_to_source_margin")
            ),
            "matrix_dense_hook_chop_response_delta_ratio": number(
                matrix.get("min_dense_hook_chop_response_delta_ratio")
            ),
            "matrix_dense_hook_chop_response_correlation": number(
                matrix.get("max_dense_hook_chop_response_correlation")
            ),
            "matrix_dense_hook_chop_response_transient_ratio": number(
                matrix.get("min_dense_hook_chop_response_transient_ratio")
            ),
            "tonal_w30_to_source_rms_ratio": number(
                source_wav.get("tonal_w30_to_source_rms_ratio")
            ),
            "tonal_hook_chop_w30_to_source_margin": number(
                source_wav.get("tonal_hook_chop_w30_to_source_margin")
            ),
            "tonal_hook_chop_score_floor": number(
                source_wav.get("tonal_hook_chop_source_character_score_floor")
            ),
            "tonal_hook_chop_score_span": number(
                source_wav.get("tonal_hook_chop_source_character_score_span")
            ),
            "tonal_hook_chop_response_delta_ratio": number(
                source_wav.get("tonal_hook_chop_response_delta_ratio")
            ),
            "tonal_hook_chop_response_correlation": number(
                source_wav.get("tonal_hook_chop_response_correlation")
            ),
            "tonal_hook_chop_response_transient_ratio": number(
                source_wav.get("tonal_hook_chop_response_transient_ratio")
            ),
            "min_rebuild_only_source_character_survival_score": min(
                number(dense.get("rebuild_only_source_character_survival_score")),
                number(matrix.get("min_rebuild_only_source_character_survival_score")),
                number(source_wav.get("min_rebuild_only_source_character_survival_score")),
                number(edge.get("min_rebuild_only_source_character_survival_score")),
            ),
            "min_rebuild_only_source_character_survival_margin": min(
                number(dense.get("rebuild_only_source_character_survival_margin")),
                number(matrix.get("min_rebuild_only_source_character_survival_margin")),
                number(source_wav.get("min_rebuild_only_source_character_survival_margin")),
                number(edge.get("min_rebuild_only_source_character_survival_margin")),
            ),
        },
        "source_character_window_selection": {
            "result": str(source_character_window_selection.get("result") or ""),
            "case_count": int(number(source_character_window_selection.get("case_count"))),
            "searched_case_count": int(
                number(source_character_window_selection.get("searched_case_count"))
            ),
            "promoted_case_count": int(
                number(source_character_window_selection.get("promoted_case_count"))
            ),
            "min_observed_rms_retention_ratio": number(
                source_character_window_selection.get("min_observed_rms_retention_ratio")
            ),
            "max_selected_start_seconds": number(
                source_character_window_selection.get("max_selected_start_seconds")
            ),
            "max_score_lift": number(source_character_window_selection.get("max_score_lift")),
        },
        "source_selection_policy": {
            "result": str(source_selection_policy.get("result") or ""),
            "case_count": int(number(source_selection_policy.get("case_count"))),
            "source_families": source_selection_policy_families,
            "promotion_allowed_source_families": source_selection_policy_promotion_families,
            "promotion_allowed_case_count": int(
                number(source_selection_policy.get("promotion_allowed_case_count"))
            ),
            "min_candidate_count": int(
                number(source_selection_policy.get("min_candidate_count"))
            ),
            "min_required_candidate_count": min(
                source_selection_policy_required_candidate_counts,
                default=0,
            ),
            "min_observed_rms_retention_ratio": number(
                source_selection_policy.get("min_observed_rms_retention_ratio")
            ),
            "max_score_lift": number(source_selection_policy.get("max_score_lift")),
            "failure_codes": list(source_selection_policy.get("failure_codes") or []),
        },
        "source_selection_risk": {
            "edge_blocked_case_count": int(
                number(edge.get("source_selection_promotion_blocked_case_count"))
            ),
            "edge_promotion_allowed": edge.get("source_selection_promotion_allowed"),
            "blocked_source_families": list(
                edge.get("source_selection_blocked_source_families") or []
            ),
            "promotion_blockers": list(
                edge.get("source_selection_promotion_blockers") or []
            ),
            "demotion_reasons": list(
                edge.get("source_selection_demotion_reasons") or []
            ),
            "demotion_reason_counts": dict(
                edge.get("source_selection_demotion_reason_counts") or {}
            ),
            "required_review_actions": list(
                edge.get("source_selection_required_review_actions") or []
            ),
            "required_review_action_count": int(
                number(edge.get("source_selection_required_review_action_count"))
            ),
            "actionable_demotions": edge.get("source_selection_actionable_demotions"),
            "musician_payoff": (
                "Risky edge sources stay review/routing material instead of "
                "being promoted as strong demos before source selection is trusted."
            ),
        },
        "drum_pressure": {
            "dense_strongest_audible_element": str(
                dense.get("strongest_audible_element") or ""
            ),
            "dense_break_physical_drum_pressure_score": number(
                dense.get("dense_break_physical_drum_pressure_score")
            ),
            "dense_break_snare_pressure_margin": number(
                dense.get("dense_break_snare_pressure_margin")
            ),
            "dense_break_pressure_transient_to_hook_ratio": number(
                dense.get("dense_break_pressure_transient_to_hook_ratio")
            ),
            "matrix_strongest_audible_elements": list(
                matrix.get("strongest_audible_elements") or []
            ),
            "tr909_rendered_result": str(
                tr909_rendered_drum_pressure.get("result") or ""
            ),
            "tr909_rendered_case_count": int(
                number(tr909_rendered_drum_pressure.get("case_count"))
            ),
            "tr909_rendered_min_support_mix_contribution_ratio": number(
                tr909_rendered_drum_pressure.get(
                    "min_support_mix_tr909_contribution_ratio"
                )
            ),
            "tr909_rendered_min_low_band_rms": number(
                tr909_rendered_drum_pressure.get("min_tr909_low_band_rms")
            ),
            "tr909_rendered_min_required_support_mix_contribution_ratio": number(
                tr909_rendered_drum_pressure.get(
                    "min_required_support_mix_tr909_contribution_ratio"
                )
            ),
            "tr909_rendered_min_required_low_band_rms": number(
                tr909_rendered_drum_pressure.get("min_required_tr909_low_band_rms")
            ),
            "tr909_rendered_max_source_first_ratio": number(
                tr909_rendered_drum_pressure.get(
                    "max_source_first_generated_to_source_rms_ratio"
                )
            ),
            "tr909_rendered_max_support_ratio": number(
                tr909_rendered_drum_pressure.get(
                    "max_support_generated_to_source_rms_ratio"
                )
            ),
        },
        "bass_pressure": {
            "matrix_sparse_bass_movement_static_distance_hz": number(
                matrix.get("min_sparse_bass_movement_static_distance_hz")
            ),
            "matrix_sparse_bass_movement_frequency_span_hz": number(
                matrix.get("min_sparse_bass_movement_frequency_span_hz")
            ),
            "matrix_sparse_pressure_low_band_lift_ratio": number(
                matrix.get("min_sparse_pressure_low_band_lift_ratio")
            ),
            "matrix_sparse_pressure_low_band_share": number(
                matrix.get("min_sparse_pressure_low_band_share")
            ),
            "matrix_sparse_pressure_low_to_mid_ratio": number(
                matrix.get("min_sparse_pressure_low_to_mid_ratio")
            ),
            "matrix_sparse_bass_dominance_margin": number(
                matrix.get("min_sparse_bass_dominance_margin")
            ),
            "source_wav_sparse_bass_movement_static_distance_hz": number(
                source_wav.get("sparse_bass_movement_static_distance_hz")
            ),
            "source_wav_sparse_bass_movement_frequency_span_hz": number(
                source_wav.get("sparse_bass_movement_frequency_span_hz")
            ),
            "source_wav_sparse_pressure_low_band_lift_ratio": number(
                source_wav.get("sparse_pressure_low_band_lift_ratio")
            ),
            "source_wav_sparse_pressure_low_band_share": number(
                source_wav.get("sparse_pressure_low_band_share")
            ),
            "source_wav_sparse_pressure_low_to_mid_ratio": number(
                source_wav.get("sparse_pressure_low_to_mid_ratio")
            ),
            "source_wav_sparse_bass_dominance_margin": number(
                source_wav.get("sparse_bass_dominance_margin")
            ),
        },
        "destructive_gesture": {
            "dropout_to_stutter_rms_ratio": number(
                destructive.get("dropout_to_stutter_rms_ratio")
            ),
            "dropout_silence_to_stutter_rms_ratio": number(
                destructive.get("dropout_silence_to_stutter_rms_ratio")
            ),
            "stutter_to_hook_transient_ratio": number(
                destructive.get("stutter_to_hook_transient_ratio")
            ),
            "stutter_to_source_transient_ratio": number(
                destructive.get("stutter_to_source_transient_ratio")
            ),
            "restore_to_hook_transient_ratio": number(
                destructive.get("restore_to_hook_transient_ratio")
            ),
            "restore_to_source_transient_ratio": number(
                destructive.get("restore_to_source_transient_ratio")
            ),
            "restore_to_pressure_rms_ratio": number(
                destructive.get("restore_to_pressure_rms_ratio")
            ),
            "restore_to_dropout_silence_rms_ratio": number(
                destructive.get("restore_to_dropout_silence_rms_ratio")
            ),
        },
        "mix_balance": {
            "result": str(feral_mix_balance.get("result") or ""),
            "min_support_generated_to_source_rms_ratio": number(
                feral_mix_balance.get("min_support_generated_to_source_rms_ratio")
            ),
            "max_source_first_generated_to_source_rms_ratio": number(
                feral_mix_balance.get("max_source_first_generated_to_source_rms_ratio")
            ),
            "min_source_first_masking_headroom": number(
                feral_mix_balance.get("min_source_first_masking_headroom")
            ),
            "max_support_generated_to_source_rms_ratio": number(
                feral_mix_balance.get("max_support_generated_to_source_rms_ratio")
            ),
        },
    }


def perform_risk_cue_contract_summary(
    report: dict[str, Any] | None,
    path: Path,
) -> dict[str, Any]:
    if report is None:
        return {
            "path": str(path),
            "available": False,
            "result": "missing",
            "schema": "",
            "cue_surface": "",
            "evidence_role": "",
            "quality_proof": None,
            "automated_musical_approval": None,
            "degraded_state_label": "",
            "degraded_action": "",
            "unavailable_state_label": "",
            "unavailable_action": "",
            "required_player_cues": [],
        }
    require(
        report.get("schema") == PERFORM_RISK_CUE_SCHEMA,
        f"{path}: schema must be {PERFORM_RISK_CUE_SCHEMA}",
    )
    return {
        "path": str(path),
        "available": True,
        "result": str(report.get("result") or ""),
        "schema": str(report.get("schema") or ""),
        "cue_surface": str(report.get("cue_surface") or ""),
        "evidence_role": str(report.get("evidence_role") or ""),
        "quality_proof": report.get("quality_proof"),
        "automated_musical_approval": report.get("automated_musical_approval"),
        "degraded_state_label": str(report.get("degraded_state_label") or ""),
        "degraded_action": str(report.get("degraded_action") or ""),
        "unavailable_state_label": str(report.get("unavailable_state_label") or ""),
        "unavailable_action": str(report.get("unavailable_action") or ""),
        "required_player_cues": string_list(report.get("required_player_cues")),
    }


def current_evidence_reconciliation(
    weak: dict[str, Any],
    suite: dict[str, Any],
    perform_risk_cue: dict[str, Any],
) -> dict[str, Any]:
    weak_summary = object_or_empty(weak.get("production_fix_summary"))
    weak_top = str(weak_summary.get("top_candidate_category") or "none")
    candidate_categories = [
        str(candidate.get("category"))
        for candidate in list(weak.get("production_fix_candidates") or [])
        if isinstance(candidate, dict) and candidate.get("category")
    ]
    stale_fixture_only_categories = []
    category_reconciliations = []

    chop_passed = chop_policy_current_evidence_passed(suite)
    if "chop_policy" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "chop_policy",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_w30_response_gates_passed"
                    if chop_passed
                    else "current_w30_response_still_risky"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if chop_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale hookless fixtures as regression controls; do not "
                    "treat them as the current top product gap while dense, matrix, "
                    "and tonal W-30 response gates pass."
                    if chop_passed
                    else "Keep W-30 hook/chop work as current product priority."
                ),
                "musician_payoff": (
                    "Old hookless examples stop hiding the next audible gap once "
                    "current W-30 response already moves."
                    if chop_passed
                    else "The first two bars still need a stronger current hook or riff."
                ),
            }
        )
        if chop_passed:
            stale_fixture_only_categories.append("chop_policy")

    bass_passed = bass_movement_current_evidence_passed(suite)
    if "bass_movement" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "bass_movement",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_sparse_pressure_gates_passed"
                    if bass_passed
                    else "current_sparse_pressure_still_risky"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if bass_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale sparse-bass fixtures as regression controls; do not "
                    "treat them as the current top product gap while matrix and "
                    "source-WAV sparse pressure gates pass."
                    if bass_passed
                    else "Keep bass movement and pressure work as current product priority."
                ),
                "musician_payoff": (
                    "Old weak-bass examples stop hiding the next audible gap once "
                    "current low-end pressure already carries."
                    if bass_passed
                    else "Low-end pressure still needs to hit harder in current output."
                ),
            }
        )
        if bass_passed:
            stale_fixture_only_categories.append("bass_movement")

    drum_passed = drum_pressure_current_evidence_passed(suite)
    if "drum_pressure" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "drum_pressure",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_drum_pressure_gates_passed"
                    if drum_passed
                    else "current_drum_pressure_still_risky"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if drum_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale weak drum-pressure fixtures as regression controls; "
                    "do not treat them as the current top product gap while dense "
                    "snare pressure and rendered TR-909 support gates pass."
                    if drum_passed
                    else "Keep TR-909 pressure and dense snare/break impact as current product priority."
                ),
                "musician_payoff": (
                    "Old weak drum examples stop hiding the next audible gap once "
                    "current kick/snare/TR-909 pressure already lands."
                    if drum_passed
                    else "Kick, snare, or break pressure still needs to feel physical in current output."
                ),
            }
        )
        if drum_passed:
            stale_fixture_only_categories.append("drum_pressure")

    destructive_passed = destructive_gesture_current_evidence_passed(suite)
    if "destructive_gesture" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "destructive_gesture",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_destructive_gesture_gates_passed"
                    if destructive_passed
                    else "current_destructive_gesture_still_risky"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if destructive_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale flat-stutter fixtures as regression controls; do not "
                    "treat them as the current top product gap while dropout, "
                    "stutter, and restore gesture gates pass."
                    if destructive_passed
                    else "Keep destructive gesture work as current product priority."
                ),
                "musician_payoff": (
                    "Old flat-stutter examples stop hiding the next audible gap once "
                    "current cut/stutter/restore gestures already hit."
                    if destructive_passed
                    else "Cuts, stutters, and restores still need stronger stage impact."
                ),
            }
        )
        if destructive_passed:
            stale_fixture_only_categories.append("destructive_gesture")

    mix_passed = mix_bus_current_evidence_passed(suite)
    if "mix_bus" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "mix_bus",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_mix_balance_gates_passed"
                    if mix_passed
                    else "current_mix_balance_still_risky"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if mix_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale source-masked mix fixtures as regression controls; "
                    "do not treat them as the current top product gap while generated "
                    "support is loud enough and source-first masking stays bounded."
                    if mix_passed
                    else "Keep mix-bus balance and generated-support masking as current product priority."
                ),
                "musician_payoff": (
                    "Old masked-source examples stop hiding the next audible gap once "
                    "current support is present without burying the source."
                    if mix_passed
                    else "The strongest element still needs clearer balance without losing source character."
                ),
            }
        )
        if mix_passed:
            stale_fixture_only_categories.append("mix_bus")

    source_selection_passed = source_selection_current_evidence_passed(
        candidate_categories,
        weak,
        suite,
    )
    if "source_selection" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "source_selection",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_source_selection_candidate_families_covered"
                    if source_selection_passed
                    else "current_source_selection_family_gap"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if source_selection_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale source-selection fixtures as regression controls; "
                    "do not treat them as the current top product gap while all "
                    "candidate families have promotion-allowed policy coverage."
                    if source_selection_passed
                    else "Keep source-selection work current until candidate-family policy coverage is complete."
                ),
                "musician_payoff": (
                    "Old source-lost examples stop hiding the next audible gap once "
                    "current source-window policy covers the relevant source families."
                    if source_selection_passed
                    else "Source choice still needs family-specific coverage before the musician can trust promotion."
                ),
            }
        )
        if source_selection_passed:
            stale_fixture_only_categories.append("source_selection")

    ui_cue_passed = ui_cue_current_evidence_passed(perform_risk_cue)
    if "ui_cue" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "ui_cue",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_tui_perform_risk_cue_passed"
                    if ui_cue_passed
                    else "current_tui_perform_risk_cue_missing"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if ui_cue_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale UI-cue fixtures as regression controls; do not "
                    "treat them as the current top product gap while the Jam Trust "
                    "perform-risk contract exposes degraded/unavailable bar/live risk."
                    if ui_cue_passed
                    else "Keep perform-risk cue work current until the Jam Trust surface proves degraded/unavailable bar/live risk."
                ),
                "musician_payoff": (
                    "Old UI-cue weak-output examples stop asking for the same "
                    "warning once the instrument already shows bar/live risk."
                    if ui_cue_passed
                    else "The musician still needs a visible warning before trusting bar-locked or live-trigger moves."
                ),
            }
        )
        if ui_cue_passed:
            stale_fixture_only_categories.append("ui_cue")

    fixture_threshold_passed = fixture_threshold_current_evidence_passed(
        candidate_categories,
        weak,
        suite,
    )
    if "fixture_threshold" in candidate_categories:
        category_reconciliations.append(
            {
                "category": "fixture_threshold",
                "weak_evidence_role": "negative_control_fixture",
                "current_professional_suite_status": (
                    "current_fixture_threshold_negative_control_covered"
                    if fixture_threshold_passed
                    else "current_fixture_threshold_still_risky"
                ),
                "priority_state": (
                    "stale_fixture_only_top_risk"
                    if fixture_threshold_passed
                    else "current_product_risk"
                ),
                "software_next_step": (
                    "Use stale fixture-threshold routes as regression controls; "
                    "do not treat them as the current product gap while they are "
                    "secondary negative-control evidence and current destructive "
                    "gesture proof passes."
                    if fixture_threshold_passed
                    else "Keep fixture-threshold work current until the route is proven to be secondary negative-control evidence covered by current output proof."
                ),
                "musician_payoff": (
                    "Old expected-fail fixture taxonomy stops hiding the next "
                    "musician-facing blocker once current destructive output already lands."
                    if fixture_threshold_passed
                    else "Weak audio or ambiguous fixtures still need threshold work before the musician can trust the evidence."
                ),
            }
        )
        if fixture_threshold_passed:
            stale_fixture_only_categories.append("fixture_threshold")

    current_product_categories = [
        category
        for category in candidate_categories
        if category not in stale_fixture_only_categories
    ]
    return {
        "weak_top_candidate_category": weak_top,
        "current_product_top_candidate_category": (
            current_product_categories[0] if current_product_categories else "none"
        ),
        "stale_fixture_only_categories": stale_fixture_only_categories,
        "category_reconciliations": category_reconciliations,
        "quality_proof": False,
        "automated_musical_approval": False,
        "musician_payoff": (
            "Readiness can keep negative controls while steering engineers toward "
            "the next audible product gap shown by current evidence."
        ),
    }


def chop_policy_current_evidence_passed(suite: dict[str, Any]) -> bool:
    if suite.get("available") is not True or suite.get("result") != "pass":
        return False
    source_character = object_or_empty(suite.get("source_character_selection"))
    checks = [
        number(source_character.get("dense_hook_chop_score_floor")) >= 0.64,
        number(source_character.get("matrix_dense_hook_chop_score_floor")) >= 0.64,
        number(source_character.get("tonal_hook_chop_score_floor")) >= 0.64,
        number(source_character.get("dense_hook_chop_response_delta_ratio"))
        >= MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO,
        number(source_character.get("matrix_dense_hook_chop_response_delta_ratio"))
        >= MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO,
        number(source_character.get("tonal_hook_chop_response_delta_ratio"))
        >= MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO,
        number(source_character.get("dense_hook_chop_response_correlation"))
        <= MAX_HOOK_CHOP_RESPONSE_CORRELATION,
        number(source_character.get("matrix_dense_hook_chop_response_correlation"))
        <= MAX_HOOK_CHOP_RESPONSE_CORRELATION,
        number(source_character.get("tonal_hook_chop_response_correlation"))
        <= MAX_HOOK_CHOP_RESPONSE_CORRELATION,
        number(source_character.get("dense_hook_chop_response_transient_ratio"))
        >= MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO,
        number(source_character.get("matrix_dense_hook_chop_response_transient_ratio"))
        >= MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO,
        number(source_character.get("tonal_hook_chop_response_transient_ratio"))
        >= MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO,
    ]
    return all(checks)


def bass_movement_current_evidence_passed(suite: dict[str, Any]) -> bool:
    if suite.get("available") is not True or suite.get("result") != "pass":
        return False
    bass_pressure = object_or_empty(suite.get("bass_pressure"))
    checks = [
        number(bass_pressure.get("matrix_sparse_bass_movement_static_distance_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ,
        number(bass_pressure.get("matrix_sparse_bass_movement_frequency_span_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
        number(bass_pressure.get("matrix_sparse_pressure_low_band_lift_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO,
        number(bass_pressure.get("matrix_sparse_pressure_low_band_share"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE,
        number(bass_pressure.get("matrix_sparse_pressure_low_to_mid_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO,
        number(bass_pressure.get("matrix_sparse_bass_dominance_margin"))
        >= MIN_SPARSE_BASS_DOMINANCE_MARGIN,
        number(bass_pressure.get("source_wav_sparse_bass_movement_static_distance_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ,
        number(bass_pressure.get("source_wav_sparse_bass_movement_frequency_span_hz"))
        >= MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
        number(bass_pressure.get("source_wav_sparse_pressure_low_band_lift_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO,
        number(bass_pressure.get("source_wav_sparse_pressure_low_band_share"))
        >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE,
        number(bass_pressure.get("source_wav_sparse_pressure_low_to_mid_ratio"))
        >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO,
        number(bass_pressure.get("source_wav_sparse_bass_dominance_margin"))
        >= MIN_SPARSE_BASS_DOMINANCE_MARGIN,
    ]
    return all(checks)


def drum_pressure_current_evidence_passed(suite: dict[str, Any]) -> bool:
    if suite.get("available") is not True or suite.get("result") != "pass":
        return False
    drum_pressure = object_or_empty(suite.get("drum_pressure"))
    checks = [
        drum_pressure.get("dense_strongest_audible_element") == "snare",
        number(drum_pressure.get("dense_break_physical_drum_pressure_score"))
        >= MIN_DENSE_DRUM_PRESSURE_SCORE,
        number(drum_pressure.get("dense_break_snare_pressure_margin"))
        >= MIN_DENSE_SNARE_PRESSURE_MARGIN,
        number(drum_pressure.get("dense_break_pressure_transient_to_hook_ratio"))
        >= MIN_DENSE_PRESSURE_TRANSIENT_TO_HOOK_RATIO,
        drum_pressure.get("tr909_rendered_result") == "pass",
        number(drum_pressure.get("tr909_rendered_case_count"))
        >= MIN_TR909_RENDERED_DRUM_PRESSURE_CASES,
        number(drum_pressure.get("tr909_rendered_min_support_mix_contribution_ratio"))
        >= number(
            drum_pressure.get("tr909_rendered_min_required_support_mix_contribution_ratio")
        ),
        number(drum_pressure.get("tr909_rendered_min_low_band_rms"))
        >= number(drum_pressure.get("tr909_rendered_min_required_low_band_rms")),
        number(drum_pressure.get("tr909_rendered_max_source_first_ratio"))
        <= MAX_TR909_RENDERED_SOURCE_FIRST_RATIO,
        number(drum_pressure.get("tr909_rendered_max_support_ratio"))
        <= MAX_TR909_RENDERED_SUPPORT_RATIO,
    ]
    return all(checks)


def destructive_gesture_current_evidence_passed(suite: dict[str, Any]) -> bool:
    if suite.get("available") is not True or suite.get("result") != "pass":
        return False
    destructive = object_or_empty(suite.get("destructive_gesture"))
    stutter_transient_passed = (
        number(destructive.get("stutter_to_hook_transient_ratio"))
        >= MIN_DESTRUCTIVE_STUTTER_TO_HOOK_TRANSIENT_RATIO
        or number(destructive.get("stutter_to_source_transient_ratio"))
        >= MIN_DESTRUCTIVE_STUTTER_TO_SOURCE_TRANSIENT_RATIO
    )
    restore_transient_passed = (
        number(destructive.get("restore_to_hook_transient_ratio"))
        >= MIN_DESTRUCTIVE_RESTORE_TO_HOOK_TRANSIENT_RATIO
        or number(destructive.get("restore_to_source_transient_ratio"))
        >= MIN_DESTRUCTIVE_RESTORE_TO_SOURCE_TRANSIENT_RATIO
    )
    checks = [
        number(destructive.get("dropout_to_stutter_rms_ratio"))
        <= MAX_DESTRUCTIVE_DROPOUT_TO_STUTTER_RMS_RATIO,
        number(destructive.get("dropout_silence_to_stutter_rms_ratio"))
        <= MAX_DESTRUCTIVE_DROPOUT_SILENCE_TO_STUTTER_RMS_RATIO,
        stutter_transient_passed,
        restore_transient_passed,
        number(destructive.get("restore_to_pressure_rms_ratio"))
        >= MIN_DESTRUCTIVE_RESTORE_TO_PRESSURE_RMS_RATIO,
        number(destructive.get("restore_to_dropout_silence_rms_ratio"))
        >= MIN_DESTRUCTIVE_RESTORE_TO_DROPOUT_SILENCE_RMS_RATIO,
    ]
    return all(checks)


def mix_bus_current_evidence_passed(suite: dict[str, Any]) -> bool:
    if suite.get("available") is not True or suite.get("result") != "pass":
        return False
    mix_balance = object_or_empty(suite.get("mix_balance"))
    checks = [
        mix_balance.get("result") == "pass",
        number(mix_balance.get("min_support_generated_to_source_rms_ratio"))
        >= MIN_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        number(mix_balance.get("max_source_first_generated_to_source_rms_ratio"))
        <= MAX_MIX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
        number(mix_balance.get("min_source_first_masking_headroom"))
        >= MIN_MIX_SOURCE_FIRST_MASKING_HEADROOM,
        number(mix_balance.get("max_support_generated_to_source_rms_ratio"))
        <= MAX_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
    ]
    return all(checks)


def source_selection_current_evidence_passed(
    candidate_categories: list[str],
    weak: dict[str, Any],
    suite: dict[str, Any],
) -> bool:
    if "source_selection" not in candidate_categories:
        return False
    if suite.get("available") is not True or suite.get("result") != "pass":
        return False
    candidate = next(
        (
            item
            for item in list_or_empty(weak.get("production_fix_candidates"))
            if isinstance(item, dict) and item.get("category") == "source_selection"
        ),
        {},
    )
    candidate_families = set(string_list(candidate.get("source_families")))
    source_policy = object_or_empty(suite.get("source_selection_policy"))
    covered_families = set(
        string_list(source_policy.get("promotion_allowed_source_families"))
    )
    return (
        source_policy.get("result") == "pass"
        and bool(candidate_families)
        and candidate_families.issubset(covered_families)
    )


def ui_cue_current_evidence_passed(perform_risk_cue: dict[str, Any]) -> bool:
    required_cues = string_list(perform_risk_cue.get("required_player_cues"))
    checks = [
        perform_risk_cue.get("available") is True,
        perform_risk_cue.get("result") == "pass",
        perform_risk_cue.get("schema") == PERFORM_RISK_CUE_SCHEMA,
        perform_risk_cue.get("cue_surface") == "timing_source_risk_before_confident_moves",
        perform_risk_cue.get("evidence_role") == "current_tui_contract",
        perform_risk_cue.get("quality_proof") is False,
        perform_risk_cue.get("automated_musical_approval") is False,
        perform_risk_cue.get("degraded_state_label") == "degraded",
        perform_risk_cue.get("degraded_action") == "bar/live?",
        perform_risk_cue.get("unavailable_state_label") == "unavailable",
        perform_risk_cue.get("unavailable_action") == "bar/live?",
        len(required_cues) >= 3,
        any("unavailable/degraded" in cue for cue in required_cues),
        any("bar-locked" in cue for cue in required_cues),
        any("live-trigger" in cue for cue in required_cues),
    ]
    return all(checks)


def fixture_threshold_current_evidence_passed(
    candidate_categories: list[str],
    weak: dict[str, Any],
    suite: dict[str, Any],
) -> bool:
    if "fixture_threshold" not in candidate_categories:
        return False
    if not destructive_gesture_current_evidence_passed(suite):
        return False
    candidate = next(
        (
            item
            for item in list_or_empty(weak.get("production_fix_candidates"))
            if isinstance(item, dict) and item.get("category") == "fixture_threshold"
        ),
        {},
    )
    case_ids = string_list(candidate.get("case_ids"))
    primary_case_ids = string_list(candidate.get("primary_case_ids"))
    if not case_ids or primary_case_ids:
        return False
    cases = {
        str(case.get("case_id")): case
        for case in list_or_empty(weak.get("cases"))
        if isinstance(case, dict)
    }
    selected_cases = [cases.get(case_id, {}) for case_id in case_ids]
    checks = [
        bool(selected_cases),
        all(case for case in selected_cases),
        all(
            "fixture_threshold" in string_list(case.get("proposed_fix_categories"))
            for case in selected_cases
        ),
        all(
            str(case.get("proposed_next_fix_category")) != "fixture_threshold"
            for case in selected_cases
        ),
        all(
            "source_report_not_passed" in string_list(case.get("failure_codes"))
            for case in selected_cases
        ),
        all(case.get("matched_known_routing_signal") is True for case in selected_cases),
    ]
    return all(checks)


def validate_perform_risk_cue_contract(
    detail: dict[str, Any],
    failures: list[str],
) -> None:
    check(detail.get("available") is True, "jam_perform_risk_cue_contract_missing", failures)
    check(
        detail.get("result") == "pass",
        "jam_perform_risk_cue_contract_not_pass",
        failures,
    )
    check(
        detail.get("schema") == PERFORM_RISK_CUE_SCHEMA,
        "jam_perform_risk_cue_contract_schema_mismatch",
        failures,
    )
    check(
        detail.get("cue_surface") == "timing_source_risk_before_confident_moves",
        "jam_perform_risk_cue_contract_surface_missing",
        failures,
    )
    check(
        detail.get("evidence_role") == "current_tui_contract",
        "jam_perform_risk_cue_contract_role_missing",
        failures,
    )
    check(
        detail.get("quality_proof") is False,
        "jam_perform_risk_cue_contract_claims_quality",
        failures,
    )
    check(
        detail.get("automated_musical_approval") is False,
        "jam_perform_risk_cue_contract_claims_approval",
        failures,
    )
    check(
        detail.get("degraded_state_label") == "degraded",
        "jam_perform_risk_cue_contract_degraded_label_missing",
        failures,
    )
    check(
        detail.get("degraded_action") == "bar/live?",
        "jam_perform_risk_cue_contract_degraded_action_missing",
        failures,
    )
    check(
        detail.get("unavailable_state_label") == "unavailable",
        "jam_perform_risk_cue_contract_unavailable_label_missing",
        failures,
    )
    check(
        detail.get("unavailable_action") == "bar/live?",
        "jam_perform_risk_cue_contract_unavailable_action_missing",
        failures,
    )
    required_cues = string_list(detail.get("required_player_cues"))
    check(
        len(required_cues) >= 3
        and any("unavailable/degraded" in cue for cue in required_cues)
        and any("bar-locked" in cue for cue in required_cues)
        and any("live-trigger" in cue for cue in required_cues),
        "jam_perform_risk_cue_contract_player_cues_too_weak",
        failures,
    )


def validate_source_selection_priority(
    detail: dict[str, Any],
    failures: list[str],
) -> None:
    check(
        detail.get("category") == "source_selection",
        "source_selection_priority_category_missing",
        failures,
    )
    check(
        detail.get("available") is True,
        "source_selection_priority_missing",
        failures,
    )
    check(
        detail.get("priority_state") == "current_product_risk",
        "source_selection_priority_not_current_product_risk",
        failures,
    )
    check(
        detail.get("quality_proof") is False,
        "source_selection_priority_claims_quality_proof",
        failures,
    )
    check(
        detail.get("automated_musical_approval") is False,
        "source_selection_priority_claims_approval",
        failures,
    )
    check(
        bool(str(detail.get("candidate_id") or "")),
        "source_selection_priority_candidate_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("case_ids"))),
        "source_selection_priority_cases_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("primary_case_ids"))),
        "source_selection_priority_primary_cases_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("source_families"))),
        "source_selection_priority_source_families_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("artifact_refs"))),
        "source_selection_priority_artifacts_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("demotion_reasons"))),
        "source_selection_priority_demotion_reasons_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("required_review_actions"))),
        "source_selection_priority_review_actions_missing",
        failures,
    )
    check(
        detail.get("actionable_for_musician") is True,
        "source_selection_priority_not_actionable",
        failures,
    )
    check(
        detail.get("source_window_policy_state") == "source_selection_policy_family_gap",
        "source_selection_priority_window_policy_not_applied",
        failures,
    )
    check(
        bool(string_list(detail.get("source_window_policy_covered_families"))),
        "source_selection_priority_window_policy_families_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("source_window_policy_uncovered_families"))),
        "source_selection_priority_uncovered_families_missing",
        failures,
    )
    check(
        number(detail.get("source_window_policy_case_count")) >= 1,
        "source_selection_priority_window_policy_case_count_too_low",
        failures,
    )
    check(
        number(detail.get("source_window_policy_promotion_allowed_count")) >= 1,
        "source_selection_priority_window_policy_promotion_count_too_low",
        failures,
    )
    check(
        number(detail.get("source_window_policy_min_candidate_count"))
        >= MIN_SOURCE_SELECTION_POLICY_CANDIDATES,
        "source_selection_priority_window_policy_candidate_count_too_low",
        failures,
    )
    check(
        number(detail.get("source_window_policy_min_rms_retention_ratio")) + 1e-6
        >= MIN_SOURCE_SELECTION_RMS_RETENTION_RATIO,
        "source_selection_priority_window_policy_rms_retention_too_low",
        failures,
    )
    software_next_step = str(detail.get("software_next_step") or "")
    generic_upstream_step = str(detail.get("generic_upstream_next_step") or "")
    check(
        len(software_next_step) >= 80
        and software_next_step != generic_upstream_step
        and "source-window/source-character" in software_next_step
        and "review actions" in software_next_step,
        "source_selection_priority_next_step_too_generic",
        failures,
    )
    musician_action = str(detail.get("musician_action") or "")
    check(
        len(musician_action) >= 80
        and "unavailable/degraded" in musician_action
        and "diagnostics" in musician_action,
        "source_selection_priority_musician_action_too_generic",
        failures,
    )


def validate_ui_cue_priority(
    detail: dict[str, Any],
    failures: list[str],
) -> None:
    check(
        detail.get("category") == "ui_cue",
        "ui_cue_priority_category_missing",
        failures,
    )
    check(
        detail.get("available") is True,
        "ui_cue_priority_missing",
        failures,
    )
    check(
        detail.get("priority_state") == "current_product_risk",
        "ui_cue_priority_not_current_product_risk",
        failures,
    )
    check(
        detail.get("quality_proof") is False,
        "ui_cue_priority_claims_quality_proof",
        failures,
    )
    check(
        detail.get("automated_musical_approval") is False,
        "ui_cue_priority_claims_approval",
        failures,
    )
    check(
        bool(str(detail.get("candidate_id") or "")),
        "ui_cue_priority_candidate_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("case_ids"))),
        "ui_cue_priority_cases_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("source_families"))),
        "ui_cue_priority_source_families_missing",
        failures,
    )
    check(
        bool(string_list(detail.get("artifact_refs"))),
        "ui_cue_priority_artifacts_missing",
        failures,
    )
    check(
        detail.get("cue_surface") == "timing_source_risk_before_confident_moves",
        "ui_cue_priority_surface_missing",
        failures,
    )
    required_cues = string_list(detail.get("required_player_cues"))
    check(
        len(required_cues) >= 3
        and any("unavailable/degraded" in cue for cue in required_cues)
        and any("bar-locked" in cue for cue in required_cues)
        and any("live-trigger" in cue for cue in required_cues),
        "ui_cue_priority_player_cues_too_weak",
        failures,
    )
    check(
        bool(string_list(detail.get("cue_reasons"))),
        "ui_cue_priority_reasons_missing",
        failures,
    )
    check(
        detail.get("actionable_for_musician") is True,
        "ui_cue_priority_not_actionable",
        failures,
    )
    software_next_step = str(detail.get("software_next_step") or "")
    generic_upstream_step = str(detail.get("generic_upstream_next_step") or "")
    check(
        len(software_next_step) >= 100
        and software_next_step != generic_upstream_step
        and "unavailable/degraded" in software_next_step
        and "bar-locked" in software_next_step
        and "live-trigger" in software_next_step,
        "ui_cue_priority_next_step_too_generic",
        failures,
    )
    musician_action = str(detail.get("musician_action") or "")
    check(
        len(musician_action) >= 90
        and "unavailable/degraded" in musician_action
        and "diagnostics" in musician_action,
        "ui_cue_priority_musician_action_too_generic",
        failures,
    )


def source_selection_priority_detail(
    weak: dict[str, Any],
    suite: dict[str, Any],
    current_evidence: dict[str, Any],
) -> dict[str, Any]:
    candidate = next(
        (
            item
            for item in list_or_empty(weak.get("production_fix_candidates"))
            if isinstance(item, dict) and item.get("category") == "source_selection"
        ),
        {},
    )
    risk = object_or_empty(suite.get("source_selection_risk"))
    source_policy = object_or_empty(suite.get("source_selection_policy"))
    is_current_top = (
        current_evidence.get("current_product_top_candidate_category")
        == "source_selection"
    )
    case_ids = string_list(candidate.get("case_ids"))
    primary_case_ids = string_list(candidate.get("primary_case_ids"))
    source_families = string_list(candidate.get("source_families"))
    artifact_refs = string_list(candidate.get("artifact_refs"))
    demotion_reasons = string_list(risk.get("demotion_reasons"))
    review_actions = string_list(risk.get("required_review_actions"))
    blocked_families = string_list(risk.get("blocked_source_families"))
    covered_policy_families = string_list(
        source_policy.get("promotion_allowed_source_families")
    ) or string_list(source_policy.get("source_families"))
    uncovered_policy_families = [
        family for family in source_families if family not in covered_policy_families
    ]
    generic_step = str(candidate.get("software_next_step") or "")
    return {
        "category": "source_selection",
        "available": bool(candidate),
        "priority_state": "current_product_risk" if is_current_top else "not_current_top",
        "evidence_role": "production_priority_detail",
        "quality_proof": False,
        "automated_musical_approval": False,
        "candidate_id": str(candidate.get("candidate_id") or ""),
        "case_ids": case_ids,
        "primary_case_ids": primary_case_ids,
        "source_families": source_families,
        "artifact_refs": artifact_refs,
        "generic_upstream_next_step": generic_step,
        "software_next_step": source_selection_software_next_step(
            primary_case_ids,
            source_families,
            demotion_reasons,
            review_actions,
            uncovered_policy_families,
        ),
        "musician_action": source_selection_musician_action(
            blocked_families,
            review_actions,
        ),
        "source_selection_surface": "source_window_character_and_edge_source_policy",
        "source_window_policy_state": (
            "source_selection_policy_family_gap"
            if source_policy.get("result") == "pass" and uncovered_policy_families
            else (
                "source_selection_policy_candidate_families_covered"
                if source_policy.get("result") == "pass"
                else "source_window_policy_missing_or_failed"
            )
        ),
        "source_window_policy_covered_families": covered_policy_families,
        "source_window_policy_uncovered_families": uncovered_policy_families,
        "source_window_policy_case_count": int(number(source_policy.get("case_count"))),
        "source_window_policy_promotion_allowed_count": int(
            number(source_policy.get("promotion_allowed_case_count"))
        ),
        "source_window_policy_min_candidate_count": int(
            number(source_policy.get("min_candidate_count"))
        ),
        "source_window_policy_min_rms_retention_ratio": number(
            source_policy.get("min_observed_rms_retention_ratio")
        ),
        "blocked_source_families": blocked_families,
        "promotion_blockers": string_list(risk.get("promotion_blockers")),
        "demotion_reasons": demotion_reasons,
        "demotion_reason_counts": object_or_empty(risk.get("demotion_reason_counts")),
        "required_review_actions": review_actions,
        "required_review_action_count": len(review_actions),
        "actionable_for_musician": bool(review_actions and blocked_families),
    }


def source_selection_software_next_step(
    primary_case_ids: list[str],
    source_families: list[str],
    demotion_reasons: list[str],
    review_actions: list[str],
    uncovered_policy_families: list[str],
) -> str:
    cases = ", ".join(primary_case_ids) if primary_case_ids else "source-selection weak cases"
    families = ", ".join(source_families) if source_families else "unknown families"
    reasons = ", ".join(demotion_reasons) if demotion_reasons else "missing demotion reasons"
    actions = ", ".join(review_actions) if review_actions else "missing review actions"
    uncovered = (
        ", ".join(uncovered_policy_families)
        if uncovered_policy_families
        else "no uncovered policy families"
    )
    return (
        f"Route {cases} through source-window/source-character review for {families}; "
        f"add explicit source-selection policy coverage for {uncovered}; "
        f"preserve demotion reasons ({reasons}) and require review actions ({actions}) "
        "before any edge source can become demo or quality material."
    )


def source_selection_musician_action(
    blocked_families: list[str],
    review_actions: list[str],
) -> str:
    families = ", ".join(blocked_families) if blocked_families else "risky sources"
    actions = ", ".join(review_actions) if review_actions else "explicit review"
    return (
        f"Treat {families} as unavailable/degraded for promotion: {actions}. "
        "Use them for diagnostics until timing, texture, and human verdict are trusted."
    )


def ui_cue_priority_detail(
    weak: dict[str, Any],
    suite: dict[str, Any],
    current_evidence: dict[str, Any],
) -> dict[str, Any]:
    candidate = next(
        (
            item
            for item in list_or_empty(weak.get("production_fix_candidates"))
            if isinstance(item, dict) and item.get("category") == "ui_cue"
        ),
        {},
    )
    risk = object_or_empty(suite.get("source_selection_risk"))
    is_current_top = (
        current_evidence.get("current_product_top_candidate_category") == "ui_cue"
    )
    case_ids = string_list(candidate.get("case_ids"))
    source_families = string_list(candidate.get("source_families"))
    artifact_refs = string_list(candidate.get("artifact_refs"))
    review_actions = string_list(risk.get("required_review_actions"))
    blocked_families = string_list(risk.get("blocked_source_families"))
    generic_step = str(candidate.get("software_next_step") or "")
    cue_reasons = sorted(
        set(review_actions)
        | set(string_list(risk.get("demotion_reasons")))
        | set(string_list(risk.get("promotion_blockers")))
    )
    return {
        "category": "ui_cue",
        "available": bool(candidate),
        "priority_state": "current_product_risk" if is_current_top else "not_current_top",
        "evidence_role": "production_priority_detail",
        "quality_proof": False,
        "automated_musical_approval": False,
        "candidate_id": str(candidate.get("candidate_id") or ""),
        "case_ids": case_ids,
        "primary_case_ids": string_list(candidate.get("primary_case_ids")),
        "source_families": source_families,
        "artifact_refs": artifact_refs,
        "generic_upstream_next_step": generic_step,
        "software_next_step": ui_cue_software_next_step(
            case_ids,
            source_families,
            cue_reasons,
        ),
        "musician_action": ui_cue_musician_action(source_families, review_actions),
        "cue_surface": "timing_source_risk_before_confident_moves",
        "required_player_cues": [
            "show unavailable/degraded state before confident bar-locked moves",
            "show unavailable/degraded state before live-trigger promotion",
            "show timing/source-risk reason instead of generic failure text",
        ],
        "blocked_source_families": blocked_families,
        "cue_reasons": cue_reasons,
        "required_review_actions": review_actions,
        "actionable_for_musician": bool(case_ids and source_families),
    }


def ui_cue_software_next_step(
    case_ids: list[str],
    source_families: list[str],
    cue_reasons: list[str],
) -> str:
    cases = ", ".join(case_ids) if case_ids else "UI-cue weak cases"
    families = ", ".join(source_families) if source_families else "unknown families"
    reasons = ", ".join(cue_reasons) if cue_reasons else "missing risk reasons"
    return (
        f"Route {cases} for {families} through a visible unavailable/degraded "
        "cue before confident bar-locked or live-trigger moves; show the timing "
        f"or source-risk reason ({reasons}) instead of a generic weak-output bucket."
    )


def ui_cue_musician_action(
    source_families: list[str],
    review_actions: list[str],
) -> str:
    families = ", ".join(source_families) if source_families else "risky sources"
    actions = ", ".join(review_actions) if review_actions else "diagnostics first"
    return (
        f"When {families} are risky, show unavailable/degraded before promotion "
        f"or live-trigger confidence; keep the move in diagnostics and ask for {actions}."
    )


def readiness_blockers(
    coverage: dict[str, Any],
    demo: dict[str, Any],
    weak: dict[str, Any],
    suite: dict[str, Any],
    review_queue: dict[str, Any],
) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    missing_candidates = coverage["missing_demo_candidate_families"]
    if missing_candidates:
        blockers.append(
            {
                "code": "source_family_demo_candidate_missing",
                "severity": "release_blocking",
                "families": missing_candidates,
                "reason": "Every P023 source family needs at least one demo-bank candidate before release-ready claims.",
            }
        )
    missing_human_verdict = coverage["missing_human_verdict_families"]
    if missing_human_verdict:
        blockers.append(
            {
                "code": "source_family_human_verdict_missing",
                "severity": "release_blocking",
                "families": missing_human_verdict,
                "reason": "Every P023 source family needs pass, weak, or fail human verdict evidence before release-ready claims.",
            }
        )
    missing_families = coverage["missing_demo_ready_families"]
    if missing_families:
        blockers.append(
            {
                "code": "source_family_demo_ready_coverage_missing",
                "severity": "release_blocking",
                "families": missing_families,
                "reason": "Every P023 source family needs demo-ready human-pass coverage before release-ready claims.",
            }
        )
    if demo["unverified_candidate_ids"]:
        blockers.append(
            {
                "code": "unverified_demo_candidates_present",
                "severity": "claim_blocking",
                "entries": demo["unverified_candidate_ids"],
                "reason": "Unverified candidates can be reviewed but must not be promoted as demo-ready.",
            }
        )
    if demo["weak_or_fail_entries"]:
        blockers.append(
            {
                "code": "weak_or_fail_demo_bank_entries_present",
                "severity": "production_blocking",
                "entries": [entry["entry_id"] for entry in demo["weak_or_fail_entries"]],
                "reason": "Weak/fail entries must route to production fixes before they can improve the demo bank.",
            }
        )
    if not weak["available"] or weak["result"] != "pass":
        blockers.append(
            {
                "code": "weak_output_routing_not_available",
                "severity": "actionability_blocking",
                "reason": "Weak outputs need concrete fix categories before P023 can move quickly.",
            }
        )
    if not suite["available"] or suite["result"] != "pass":
        blockers.append(
            {
                "code": "professional_output_suite_not_available",
                "severity": "evidence_blocking",
                "reason": "The P022 professional-output suite should be available as current diagnostic context.",
            }
        )
    else:
        source_selection_risk = object_or_empty(suite.get("source_selection_risk"))
        if (
            number(source_selection_risk.get("edge_blocked_case_count")) >= 1.0
            and source_selection_risk.get("edge_promotion_allowed") is False
        ):
            blockers.append(
                {
                    "code": "edge_source_selection_promotion_blocked",
                    "severity": "production_blocking",
                    "families": list(source_selection_risk.get("blocked_source_families") or []),
                    "reason": (
                        "Bad-timing and pad/noise edge sources are correctly blocked "
                        "from demo promotion until source-selection review or fixes land."
                    ),
                }
            )
    if suite.get("scripted_generation") is True or suite.get("quality_proof") is False:
        blockers.append(
            {
                "code": "professional_suite_diagnostic_only",
                "severity": "claim_blocking",
                "reason": "The current professional-output suite remains scripted diagnostic evidence, not quality proof.",
            }
        )
    if not review_queue["available"] or review_queue["result"] != "pass":
        blockers.append(
            {
                "code": "human_review_queue_not_available",
                "severity": "review_blocking",
                "reason": "Release-demo candidates need a current human-review queue before release readiness can be interpreted.",
            }
        )
    elif review_queue["review_queue_count"]:
        blockers.append(
            {
                "code": "human_review_queue_unverified_candidates_present",
                "severity": "claim_blocking",
                "entries": [candidate["entry_id"] for candidate in review_queue["candidates"]],
                "source_families": review_queue["source_families"],
                "reason": "Queued release-demo candidates still require structured human listening before quality claims.",
            }
        )
    return blockers

def next_actions(
    coverage: dict[str, Any],
    demo: dict[str, Any],
    weak: dict[str, Any],
    suite: dict[str, Any],
    current_evidence: dict[str, Any],
    review_queue: dict[str, Any],
) -> list[dict[str, Any]]:
    actions: list[dict[str, Any]] = []
    review_candidate_by_family = {
        str(candidate.get("source_family")): candidate
        for candidate in list_or_empty(review_queue.get("candidates"))
        if isinstance(candidate, dict) and candidate.get("source_family")
    }
    for family in coverage["missing_demo_ready_families"]:
        candidate = review_candidate_by_family.get(str(family))
        action = {
            "category": "source_selection",
            "target": family,
            "action": "Create or promote a real-source candidate with structured human listening evidence.",
        }
        if candidate:
            candidate_id = str(candidate.get("entry_id") or "")
            action.update(
                {
                    "candidate_id": candidate_id,
                    "review_priority": candidate.get("review_priority"),
                    "demo_worthy_reason": candidate.get("demo_worthy_reason"),
                    "not_demo_ready_reason": candidate.get("not_demo_ready_reason"),
                    "required_verdict_current_state": candidate.get("required_verdict_current_state"),
                    "rendered_wav": candidate.get("rendered_wav"),
                    "metrics": candidate.get("metrics"),
                    "review_prompt": candidate.get("review_prompt"),
                    "action": (
                        f"Review {candidate_id} before demo promotion; "
                        f"{candidate.get('demo_worthy_reason')} "
                        f"Not demo-ready yet: {candidate.get('not_demo_ready_reason')}"
                    ),
                }
            )
        actions.append(action)
    stale_categories = set(
        string_list(current_evidence.get("stale_fixture_only_categories"))
    )
    current_top = str(current_evidence.get("current_product_top_candidate_category") or "")
    for category in sorted(set(weak["fix_categories"]) | set(demo["weak_fix_categories"])):
        if category in stale_categories and current_top in {"", "none"}:
            continue
        actions.append(
            {
                "category": category,
                "target": "weak output",
                "action": (
                    f"Keep stale {category} fixtures as regression controls; "
                    "do not choose them as the next implementation slice unless "
                    "current evidence regresses."
                    if category in stale_categories
                    else f"Implement the next bounded production fix for {category}."
                ),
            }
        )
    if current_top and current_top != "none":
        actions.append(
            {
                "category": current_top,
                "target": "current evidence priority",
                "action": (
                    "Use current professional-suite reconciliation before choosing "
                    "the next weak-output production slice."
                ),
            }
        )
    if not suite["available"]:
        actions.append(
            {
                "category": "fixture_threshold",
                "target": "professional output suite",
                "action": "Generate the professional-output suite before using the readiness report for branch review.",
            }
        )
    for candidate in review_queue.get("candidates", []):
        if candidate.get("review_priority") in {"high", "medium"}:
            actions.append(
                {
                    "category": "human_review",
                    "target": candidate["entry_id"],
                    "action": (
                        "Listen for "
                        f"{candidate['strongest_audible_element']} and source character, "
                        "then record pass, weak, or fail."
                    ),
                }
            )
    return actions

def musician_summary(blockers: list[dict[str, Any]], fix_categories: list[str]) -> str:
    if not blockers:
        return "P023 release sound is ready for the covered scope."
    category_text = ", ".join(fix_categories) if fix_categories else "source selection and fixture thresholds"
    return (
        "Not release-ready yet: the current evidence still needs human/demo coverage "
        f"and concrete production fixes around {category_text}."
    )


def validate_report(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    release_readiness = report.get("release_readiness")
    check(release_readiness in {"blocked", "release_ready"}, "release_readiness_invalid", failures)
    blockers = list(report.get("blockers", []))
    missing_candidates = nested_list(report, "source_family_coverage", "missing_demo_candidate_families")
    missing_human_verdict = nested_list(report, "source_family_coverage", "missing_human_verdict_families")
    missing_families = nested_list(report, "source_family_coverage", "missing_demo_ready_families")
    unverified = nested_list(report, "demo_bank", "unverified_candidate_ids")
    weak_entries = nested_list(report, "demo_bank", "weak_or_fail_entries")
    weak_routing = object_or_empty(report.get("weak_output_routing"))
    weak_available = nested_value(report, "weak_output_routing", "available")
    weak_fix_summary = nested_value(report, "weak_output_routing", "production_fix_summary")
    current_evidence = object_or_empty(report.get("current_evidence_reconciliation"))
    source_selection_priority = object_or_empty(report.get("source_selection_priority"))
    ui_cue_priority = object_or_empty(report.get("ui_cue_priority"))
    perform_risk_cue = object_or_empty(report.get("jam_perform_risk_cue_contract"))
    perform_risk_cue_contract = object_or_empty(
        report.get("jam_perform_risk_cue_contract")
    )
    suite_available = nested_value(report, "professional_output_suite", "available")
    suite_scripted = nested_value(report, "professional_output_suite", "scripted_generation")
    suite_quality = nested_value(report, "professional_output_suite", "quality_proof")
    suite_source_character = nested_value(
        report,
        "professional_output_suite",
        "source_character_selection",
    )
    suite_source_character_window = nested_value(
        report,
        "professional_output_suite",
        "source_character_window_selection",
    )
    suite_source_selection_policy = nested_value(
        report,
        "professional_output_suite",
        "source_selection_policy",
    )
    suite_source_selection_risk = nested_value(
        report,
        "professional_output_suite",
        "source_selection_risk",
    )
    suite_drum_pressure = nested_value(report, "professional_output_suite", "drum_pressure")
    suite_mix_balance = nested_value(report, "professional_output_suite", "mix_balance")
    suite_strongest = nested_list(
        report,
        "professional_output_suite",
        "strongest_audible_elements",
    )
    review_queue = object_or_empty(report.get("human_review_queue"))
    review_candidates = review_queue.get("candidates", [])

    if suite_available is True:
        check(
            isinstance(suite_source_character, dict),
            "professional_suite_source_character_selection_missing",
            failures,
        )
        check(
            isinstance(suite_drum_pressure, dict),
            "professional_suite_drum_pressure_missing",
            failures,
        )
        check(
            isinstance(suite_source_character_window, dict),
            "professional_suite_source_character_window_selection_missing",
            failures,
        )
        check(
            isinstance(suite_source_selection_policy, dict),
            "professional_suite_source_selection_policy_missing",
            failures,
        )
        check(
            isinstance(suite_source_selection_risk, dict),
            "professional_suite_source_selection_risk_missing",
            failures,
        )
        if isinstance(suite_source_selection_risk, dict):
            check(
                number(suite_source_selection_risk.get("edge_blocked_case_count")) >= 2.0,
                "professional_suite_source_selection_blocked_count_too_low",
                failures,
            )
            check(
                suite_source_selection_risk.get("edge_promotion_allowed") is False,
                "professional_suite_source_selection_promotion_allowed",
                failures,
            )
            blockers_list = list(suite_source_selection_risk.get("promotion_blockers") or [])
            for blocker in (
                "human_verdict_unverified",
                "diagnostic_only_quality_proof_false",
                "source_selection_fix_required",
            ):
                check(
                    blocker in blockers_list,
                    f"professional_suite_source_selection_{blocker}_missing",
                    failures,
                )
            demotion_reasons = list(
                suite_source_selection_risk.get("demotion_reasons") or []
            )
            for reason in EXPECTED_SOURCE_SELECTION_DEMOTION_REASONS:
                check(
                    reason in demotion_reasons,
                    f"professional_suite_source_selection_demotion_{reason}_missing",
                    failures,
                )
            demotion_counts = object_or_empty(
                suite_source_selection_risk.get("demotion_reason_counts")
            )
            for reason in EXPECTED_SOURCE_SELECTION_DEMOTION_REASONS:
                check(
                    number(demotion_counts.get(reason)) >= 1.0,
                    f"professional_suite_source_selection_demotion_{reason}_count_missing",
                    failures,
                )
            review_actions = list(
                suite_source_selection_risk.get("required_review_actions") or []
            )
            for action in EXPECTED_SOURCE_SELECTION_REVIEW_ACTIONS:
                check(
                    action in review_actions,
                    f"professional_suite_source_selection_review_action_{action}_missing",
                    failures,
                )
            check(
                number(suite_source_selection_risk.get("required_review_action_count"))
                >= 3.0,
                "professional_suite_source_selection_review_action_count_too_low",
                failures,
            )
            check(
                suite_source_selection_risk.get("actionable_demotions") is True,
                "professional_suite_source_selection_demotions_not_actionable",
                failures,
            )
        check(
            isinstance(suite_mix_balance, dict),
            "professional_suite_mix_balance_missing",
            failures,
        )
        check(
            {"bass", "snare", "stab"}.issubset(set(str(item) for item in suite_strongest)),
            "professional_suite_strongest_elements_incomplete",
            failures,
        )
        source_character = suite_source_character if isinstance(suite_source_character, dict) else {}
        source_character_window = (
            suite_source_character_window
            if isinstance(suite_source_character_window, dict)
            else {}
        )
        source_selection_policy = (
            suite_source_selection_policy
            if isinstance(suite_source_selection_policy, dict)
            else {}
        )
        check(
            source_character_window.get("result") == "pass",
            "professional_suite_source_character_window_selection_not_pass",
            failures,
        )
        check(
            number(source_character_window.get("case_count")) >= 8,
            "professional_suite_source_character_window_selection_case_count_too_low",
            failures,
        )
        check(
            number(source_character_window.get("searched_case_count")) >= 3,
            "professional_suite_source_character_window_selection_search_coverage_too_low",
            failures,
        )
        check(
            number(source_character_window.get("promoted_case_count")) >= 1,
            "professional_suite_source_character_window_selection_promoted_count_too_low",
            failures,
        )
        check(
            number(source_character_window.get("min_observed_rms_retention_ratio")) + 1e-6
            >= 0.98,
            "professional_suite_source_character_window_selection_rms_retention_too_low",
            failures,
        )
        check(
            source_selection_policy.get("result") == "pass",
            "professional_suite_source_selection_policy_not_pass",
            failures,
        )
        check(
            number(source_selection_policy.get("case_count")) >= 1,
            "professional_suite_source_selection_policy_case_count_too_low",
            failures,
        )
        check(
            number(source_selection_policy.get("promotion_allowed_case_count")) >= 1,
            "professional_suite_source_selection_policy_promotion_count_too_low",
            failures,
        )
        check(
            number(source_selection_policy.get("min_required_candidate_count"))
            >= MIN_SOURCE_SELECTION_POLICY_CANDIDATES,
            "professional_suite_source_selection_policy_candidate_count_too_low",
            failures,
        )
        check(
            number(source_selection_policy.get("min_observed_rms_retention_ratio")) + 1e-6
            >= MIN_SOURCE_SELECTION_RMS_RETENTION_RATIO,
            "professional_suite_source_selection_policy_rms_retention_too_low",
            failures,
        )
        check(
            number(source_selection_policy.get("max_score_lift"))
            >= MIN_SOURCE_SELECTION_SCORE_LIFT,
            "professional_suite_source_selection_policy_score_lift_negative",
            failures,
        )
        check(
            number(source_character.get("dense_hook_chop_score_floor")) >= 0.60,
            "professional_suite_dense_source_character_too_weak",
            failures,
        )
        check(
            number(source_character.get("dense_hook_chop_score_span")) >= 0.10,
            "professional_suite_dense_source_character_too_narrow",
            failures,
        )
        check(
            number(source_character.get("dense_w30_to_source_rms_ratio"))
            >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
            "professional_suite_dense_hook_chop_w30_too_weak",
            failures,
        )
        check(
            number(source_character.get("dense_hook_chop_w30_to_source_margin"))
            >= 0.025,
            "professional_suite_dense_hook_chop_w30_margin_too_low",
            failures,
        )
        check(
            number(source_character.get("dense_hook_chop_response_delta_ratio"))
            >= MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO,
            "professional_suite_dense_hook_chop_response_delta_too_small",
            failures,
        )
        check(
            number(source_character.get("dense_hook_chop_response_correlation"))
            <= MAX_HOOK_CHOP_RESPONSE_CORRELATION,
            "professional_suite_dense_hook_chop_response_too_source_copied",
            failures,
        )
        check(
            number(source_character.get("dense_hook_chop_response_transient_ratio"))
            >= MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO,
            "professional_suite_dense_hook_chop_response_transient_too_weak",
            failures,
        )
        check(
            number(source_character.get("matrix_dense_w30_to_source_rms_ratio"))
            >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
            "professional_suite_matrix_dense_hook_chop_w30_too_weak",
            failures,
        )
        check(
            number(source_character.get("matrix_dense_hook_chop_w30_to_source_margin"))
            >= 0.025,
            "professional_suite_matrix_dense_hook_chop_w30_margin_too_low",
            failures,
        )
        check(
            number(source_character.get("matrix_dense_hook_chop_response_delta_ratio"))
            >= MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO,
            "professional_suite_matrix_dense_hook_chop_response_delta_too_small",
            failures,
        )
        check(
            number(source_character.get("matrix_dense_hook_chop_response_correlation"))
            <= MAX_HOOK_CHOP_RESPONSE_CORRELATION,
            "professional_suite_matrix_dense_hook_chop_response_too_source_copied",
            failures,
        )
        check(
            number(source_character.get("matrix_dense_hook_chop_response_transient_ratio"))
            >= MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO,
            "professional_suite_matrix_dense_hook_chop_response_transient_too_weak",
            failures,
        )
        check(
            number(source_character.get("tonal_w30_to_source_rms_ratio"))
            >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
            "professional_suite_tonal_hook_chop_w30_too_weak",
            failures,
        )
        check(
            number(source_character.get("tonal_hook_chop_w30_to_source_margin"))
            >= 0.025,
            "professional_suite_tonal_hook_chop_w30_margin_too_low",
            failures,
        )
        check(
            number(source_character.get("tonal_hook_chop_score_floor")) >= 0.60,
            "professional_suite_tonal_source_character_too_weak",
            failures,
        )
        check(
            number(source_character.get("tonal_hook_chop_score_span")) >= 0.10,
            "professional_suite_tonal_source_character_too_narrow",
            failures,
        )
        check(
            number(source_character.get("tonal_hook_chop_response_delta_ratio"))
            >= MIN_HOOK_CHOP_RESPONSE_DELTA_RATIO,
            "professional_suite_tonal_hook_chop_response_delta_too_small",
            failures,
        )
        check(
            number(source_character.get("tonal_hook_chop_response_correlation"))
            <= MAX_HOOK_CHOP_RESPONSE_CORRELATION,
            "professional_suite_tonal_hook_chop_response_too_source_copied",
            failures,
        )
        check(
            number(source_character.get("tonal_hook_chop_response_transient_ratio"))
            >= MIN_HOOK_CHOP_RESPONSE_TRANSIENT_RATIO,
            "professional_suite_tonal_hook_chop_response_transient_too_weak",
            failures,
        )
        check(
            number(source_character.get("min_rebuild_only_source_character_survival_score"))
            >= 0.70,
            "professional_suite_source_character_survival_too_low",
            failures,
        )
        check(
            number(source_character.get("min_rebuild_only_source_character_survival_margin"))
            >= 0.10,
            "professional_suite_source_character_survival_margin_too_low",
            failures,
        )
        drum_pressure = suite_drum_pressure if isinstance(suite_drum_pressure, dict) else {}
        check(
            drum_pressure.get("dense_strongest_audible_element") == "snare",
            "professional_suite_dense_snare_not_strongest",
            failures,
        )
        check(
            number(drum_pressure.get("dense_break_physical_drum_pressure_score"))
            >= MIN_DENSE_DRUM_PRESSURE_SCORE,
            "professional_suite_dense_drum_pressure_too_weak",
            failures,
        )
        check(
            number(drum_pressure.get("dense_break_snare_pressure_margin"))
            >= MIN_DENSE_SNARE_PRESSURE_MARGIN,
            "professional_suite_dense_snare_pressure_ambiguous",
            failures,
        )
        check(
            number(drum_pressure.get("dense_break_pressure_transient_to_hook_ratio"))
            >= MIN_DENSE_PRESSURE_TRANSIENT_TO_HOOK_RATIO,
            "professional_suite_dense_pressure_transient_too_soft",
            failures,
        )
        check(
            drum_pressure.get("tr909_rendered_result") == "pass",
            "professional_suite_tr909_rendered_drum_pressure_not_pass",
            failures,
        )
        check(
            number(drum_pressure.get("tr909_rendered_case_count"))
            >= MIN_TR909_RENDERED_DRUM_PRESSURE_CASES,
            "professional_suite_tr909_rendered_drum_pressure_case_count_too_low",
            failures,
        )
        check(
            number(
                drum_pressure.get("tr909_rendered_min_support_mix_contribution_ratio")
            )
            >= number(
                drum_pressure.get(
                    "tr909_rendered_min_required_support_mix_contribution_ratio"
                )
            ),
            "professional_suite_tr909_rendered_drum_pressure_too_buried",
            failures,
        )
        check(
            number(drum_pressure.get("tr909_rendered_min_low_band_rms"))
            >= number(drum_pressure.get("tr909_rendered_min_required_low_band_rms")),
            "professional_suite_tr909_rendered_low_band_too_weak",
            failures,
        )
        check(
            number(drum_pressure.get("tr909_rendered_max_source_first_ratio"))
            <= MAX_TR909_RENDERED_SOURCE_FIRST_RATIO,
            "professional_suite_tr909_rendered_masks_source_first",
            failures,
        )
        check(
            number(drum_pressure.get("tr909_rendered_max_support_ratio"))
            <= MAX_TR909_RENDERED_SUPPORT_RATIO,
            "professional_suite_tr909_rendered_support_masks_source",
            failures,
        )
        bass_pressure = object_or_empty(
            nested_value(report, "professional_output_suite", "bass_pressure")
        )
        check(
            number(bass_pressure.get("matrix_sparse_bass_movement_static_distance_hz"))
            >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ,
            "professional_suite_matrix_sparse_bass_movement_static_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("matrix_sparse_bass_movement_frequency_span_hz"))
            >= MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
            "professional_suite_matrix_sparse_bass_movement_span_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("matrix_sparse_pressure_low_band_lift_ratio"))
            >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO,
            "professional_suite_matrix_sparse_pressure_low_band_too_weak",
            failures,
        )
        check(
            number(bass_pressure.get("matrix_sparse_pressure_low_band_share"))
            >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE,
            "professional_suite_matrix_sparse_pressure_low_band_share_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("matrix_sparse_pressure_low_to_mid_ratio"))
            >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO,
            "professional_suite_matrix_sparse_pressure_reads_as_midrange_phrase",
            failures,
        )
        check(
            number(bass_pressure.get("matrix_sparse_bass_dominance_margin"))
            >= MIN_SPARSE_BASS_DOMINANCE_MARGIN,
            "professional_suite_matrix_sparse_bass_dominance_margin_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("source_wav_sparse_bass_movement_static_distance_hz"))
            >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ,
            "professional_suite_source_wav_sparse_bass_movement_static_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("source_wav_sparse_bass_movement_frequency_span_hz"))
            >= MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ,
            "professional_suite_source_wav_sparse_bass_movement_span_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("source_wav_sparse_pressure_low_band_lift_ratio"))
            >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO,
            "professional_suite_source_wav_sparse_pressure_low_band_too_weak",
            failures,
        )
        check(
            number(bass_pressure.get("source_wav_sparse_pressure_low_band_share"))
            >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE,
            "professional_suite_source_wav_sparse_pressure_low_band_share_too_low",
            failures,
        )
        check(
            number(bass_pressure.get("source_wav_sparse_pressure_low_to_mid_ratio"))
            >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO,
            "professional_suite_source_wav_sparse_pressure_reads_as_midrange_phrase",
            failures,
        )
        check(
            number(bass_pressure.get("source_wav_sparse_bass_dominance_margin"))
            >= MIN_SPARSE_BASS_DOMINANCE_MARGIN,
            "professional_suite_source_wav_sparse_bass_dominance_margin_too_low",
            failures,
        )
        mix_balance = suite_mix_balance if isinstance(suite_mix_balance, dict) else {}
        check(
            mix_balance.get("result") == "pass",
            "professional_suite_mix_balance_not_pass",
            failures,
        )
        check(
            number(mix_balance.get("min_support_generated_to_source_rms_ratio"))
            >= MIN_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
            "professional_suite_mix_support_too_weak",
            failures,
        )
        check(
            number(mix_balance.get("max_source_first_generated_to_source_rms_ratio"))
            <= MAX_MIX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            "professional_suite_source_first_too_generated",
            failures,
        )
        check(
            number(mix_balance.get("min_source_first_masking_headroom"))
            >= MIN_MIX_SOURCE_FIRST_MASKING_HEADROOM,
            "professional_suite_source_first_masking_headroom_too_low",
            failures,
        )
        check(
            number(mix_balance.get("max_support_generated_to_source_rms_ratio"))
            <= MAX_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
            "professional_suite_mix_support_masks_source",
            failures,
        )

    validate_human_review_queue_section(review_queue, blockers, failures)

    if weak_available is True:
        check(
            isinstance(weak_fix_summary, dict),
            "weak_output_routing_fix_summary_missing",
            failures,
        )
        weak_summary = weak_fix_summary if isinstance(weak_fix_summary, dict) else {}
        candidate_count = nested_value(report, "weak_output_routing", "production_fix_candidate_count")
        check(
            weak_summary.get("candidate_count") == candidate_count,
            "weak_output_routing_fix_summary_count_stale",
            failures,
        )
        check(
            isinstance(weak_summary.get("categories"), list) and bool(weak_summary.get("categories")),
            "weak_output_routing_fix_summary_categories_missing",
            failures,
        )
        check(
            isinstance(weak_summary.get("recurring_fix_categories"), list)
            and bool(weak_summary.get("recurring_fix_categories")),
            "weak_output_routing_fix_summary_recurring_missing",
            failures,
        )

    check(
        current_evidence.get("quality_proof") is False,
        "current_evidence_reconciliation_claims_quality_proof",
        failures,
    )
    check(
        current_evidence.get("automated_musical_approval") is False,
        "current_evidence_reconciliation_claims_approval",
        failures,
    )
    check(
        isinstance(current_evidence.get("weak_top_candidate_category"), str)
        and bool(current_evidence.get("weak_top_candidate_category")),
        "current_evidence_reconciliation_weak_top_missing",
        failures,
    )
    if current_evidence.get("weak_top_candidate_category") == "chop_policy":
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "chop_policy",
            ),
            "current_evidence_reconciliation_chop_policy_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category") != "chop_policy",
            "current_evidence_reconciliation_current_top_still_chop_policy",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "chop_policy"
                and item.get("current_professional_suite_status")
                == "current_w30_response_gates_passed"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_chop_policy_status_missing",
            failures,
        )
    weak_categories = list_or_empty(weak_summary.get("categories"))
    suite_summary = object_or_empty(report.get("professional_output_suite"))
    if (
        list_contains(weak_categories, "bass_movement")
        and bass_movement_current_evidence_passed(suite_summary)
    ):
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "bass_movement",
            ),
            "current_evidence_reconciliation_bass_movement_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category")
            != "bass_movement",
            "current_evidence_reconciliation_current_top_still_bass_movement",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "bass_movement"
                and item.get("current_professional_suite_status")
                == "current_sparse_pressure_gates_passed"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_bass_movement_status_missing",
            failures,
        )
    if (
        list_contains(weak_categories, "drum_pressure")
        and drum_pressure_current_evidence_passed(suite_summary)
    ):
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "drum_pressure",
            ),
            "current_evidence_reconciliation_drum_pressure_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category")
            != "drum_pressure",
            "current_evidence_reconciliation_current_top_still_drum_pressure",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "drum_pressure"
                and item.get("current_professional_suite_status")
                == "current_drum_pressure_gates_passed"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_drum_pressure_status_missing",
            failures,
        )
    for item in list_or_empty(current_evidence.get("category_reconciliations")):
        if (
            isinstance(item, dict)
            and item.get("category") == "drum_pressure"
            and (
                item.get("current_professional_suite_status")
                == "current_drum_pressure_gates_passed"
                or item.get("priority_state") == "stale_fixture_only_top_risk"
            )
        ):
            check(
                drum_pressure_current_evidence_passed(suite_summary),
                "current_evidence_reconciliation_drum_pressure_stale_without_current_proof",
                failures,
            )
    if (
        list_contains(weak_categories, "destructive_gesture")
        and destructive_gesture_current_evidence_passed(suite_summary)
    ):
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "destructive_gesture",
            ),
            "current_evidence_reconciliation_destructive_gesture_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category")
            != "destructive_gesture",
            "current_evidence_reconciliation_current_top_still_destructive_gesture",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "destructive_gesture"
                and item.get("current_professional_suite_status")
                == "current_destructive_gesture_gates_passed"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_destructive_gesture_status_missing",
            failures,
        )
    for item in list_or_empty(current_evidence.get("category_reconciliations")):
        if (
            isinstance(item, dict)
            and item.get("category") == "destructive_gesture"
            and (
                item.get("current_professional_suite_status")
                == "current_destructive_gesture_gates_passed"
                or item.get("priority_state") == "stale_fixture_only_top_risk"
            )
        ):
            check(
                destructive_gesture_current_evidence_passed(suite_summary),
                "current_evidence_reconciliation_destructive_gesture_stale_without_current_proof",
                failures,
            )
    if (
        list_contains(weak_categories, "mix_bus")
        and mix_bus_current_evidence_passed(suite_summary)
    ):
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "mix_bus",
            ),
            "current_evidence_reconciliation_mix_bus_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category") != "mix_bus",
            "current_evidence_reconciliation_current_top_still_mix_bus",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "mix_bus"
                and item.get("current_professional_suite_status")
                == "current_mix_balance_gates_passed"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_mix_bus_status_missing",
            failures,
            )
    if source_selection_current_evidence_passed(
        weak_categories,
        weak_routing,
        suite_summary,
    ):
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "source_selection",
            ),
            "current_evidence_reconciliation_source_selection_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category")
            != "source_selection",
            "current_evidence_reconciliation_current_top_still_source_selection",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "source_selection"
                and item.get("current_professional_suite_status")
                == "current_source_selection_candidate_families_covered"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_source_selection_status_missing",
            failures,
        )
    for item in list_or_empty(current_evidence.get("category_reconciliations")):
        if (
            isinstance(item, dict)
            and item.get("category") == "source_selection"
            and (
                item.get("current_professional_suite_status")
                == "current_source_selection_candidate_families_covered"
                or item.get("priority_state") == "stale_fixture_only_top_risk"
            )
        ):
            check(
                source_selection_current_evidence_passed(
                    weak_categories,
                    weak_routing,
                    suite_summary,
                ),
                "current_evidence_reconciliation_source_selection_stale_without_candidate_family_coverage",
                failures,
            )
    if list_contains(weak_categories, "ui_cue"):
        validate_perform_risk_cue_contract(perform_risk_cue_contract, failures)
    if (
        list_contains(weak_categories, "ui_cue")
        and ui_cue_current_evidence_passed(perform_risk_cue_contract)
    ):
        check(
            list_contains(current_evidence.get("stale_fixture_only_categories"), "ui_cue"),
            "current_evidence_reconciliation_ui_cue_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category") != "ui_cue",
            "current_evidence_reconciliation_current_top_still_ui_cue",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "ui_cue"
                and item.get("current_professional_suite_status")
                == "current_tui_perform_risk_cue_passed"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_ui_cue_status_missing",
            failures,
        )
    for item in list_or_empty(current_evidence.get("category_reconciliations")):
        if (
            isinstance(item, dict)
            and item.get("category") == "ui_cue"
            and (
                item.get("current_professional_suite_status")
                == "current_tui_perform_risk_cue_passed"
                or item.get("priority_state") == "stale_fixture_only_top_risk"
            )
        ):
            check(
                ui_cue_current_evidence_passed(perform_risk_cue_contract),
                "current_evidence_reconciliation_ui_cue_stale_without_tui_contract",
                failures,
            )
    if fixture_threshold_current_evidence_passed(
        weak_categories,
        weak_routing,
        suite_summary,
    ):
        check(
            list_contains(
                current_evidence.get("stale_fixture_only_categories"),
                "fixture_threshold",
            ),
            "current_evidence_reconciliation_fixture_threshold_not_reconciled",
            failures,
        )
        check(
            current_evidence.get("current_product_top_candidate_category")
            != "fixture_threshold",
            "current_evidence_reconciliation_current_top_still_fixture_threshold",
            failures,
        )
        reconciliations = list_or_empty(current_evidence.get("category_reconciliations"))
        check(
            any(
                isinstance(item, dict)
                and item.get("category") == "fixture_threshold"
                and item.get("current_professional_suite_status")
                == "current_fixture_threshold_negative_control_covered"
                and item.get("priority_state") == "stale_fixture_only_top_risk"
                for item in reconciliations
            ),
            "current_evidence_reconciliation_fixture_threshold_status_missing",
            failures,
        )
    for item in list_or_empty(current_evidence.get("category_reconciliations")):
        if (
            isinstance(item, dict)
            and item.get("category") == "fixture_threshold"
            and (
                item.get("current_professional_suite_status")
                == "current_fixture_threshold_negative_control_covered"
                or item.get("priority_state") == "stale_fixture_only_top_risk"
            )
        ):
            check(
                fixture_threshold_current_evidence_passed(
                    weak_categories,
                    weak_routing,
                    suite_summary,
                ),
                "current_evidence_reconciliation_fixture_threshold_stale_without_negative_control_proof",
                failures,
            )
    if current_evidence.get("current_product_top_candidate_category") == "source_selection":
        validate_source_selection_priority(source_selection_priority, failures)
    if current_evidence.get("current_product_top_candidate_category") == "ui_cue":
        validate_ui_cue_priority(ui_cue_priority, failures)

    next_action_items = list_or_empty(report.get("next_actions"))
    validate_current_p023_contract(report, blockers, failures)
    validate_current_next_actions(report, next_action_items, failures)

    if release_readiness == "release_ready":
        check(not missing_candidates, "release_ready_without_demo_candidates", failures)
        check(not missing_human_verdict, "release_ready_without_human_verdicts", failures)
        check(not missing_families, "release_ready_without_required_coverage", failures)
        check(not unverified, "release_ready_with_unverified_candidates", failures)
        check(not weak_entries, "release_ready_with_weak_entries", failures)
        check(weak_available is True, "release_ready_without_weak_routing", failures)
        check(suite_available is True, "release_ready_without_professional_suite", failures)
        check(suite_scripted is not True, "release_ready_from_scripted_suite", failures)
        check(suite_quality is True, "release_ready_without_quality_proof", failures)
        check(not review_candidates, "release_ready_with_human_review_queue_candidates", failures)

    if blockers:
        check(release_readiness == "blocked", "blockers_require_blocked_readiness", failures)
        check(report.get("quality_claim_allowed") is False, "blocked_report_claims_quality", failures)
    else:
        check(report.get("quality_claim_allowed") is True, "release_ready_must_allow_quality_claim", failures)

    fix_categories = report.get("next_fix_categories")
    check(isinstance(fix_categories, list) and bool(fix_categories), "next_fix_categories_missing", failures)
    check(isinstance(report.get("musician_summary"), str) and report["musician_summary"], "musician_summary_missing", failures)
    return failures


def validate_current_p023_contract(
    report: dict[str, Any],
    blockers: list[Any],
    failures: list[str],
) -> None:
    weak = object_or_empty(report.get("weak_output_routing"))
    weak_summary = object_or_empty(weak.get("production_fix_summary"))
    weak_candidates = list_or_empty(weak.get("production_fix_candidates"))
    coverage = object_or_empty(report.get("source_family_coverage"))
    demo = object_or_empty(report.get("demo_bank"))
    blocker_codes = {str(blocker.get("code")) for blocker in blockers if isinstance(blocker, dict)}

    check(
        weak.get("available") is True,
        "p023_contract_weak_output_routing_unavailable",
        failures,
    )
    check(
        number(weak.get("production_fix_candidate_count")) >= 5,
        "p023_contract_weak_fix_candidate_count_too_low",
        failures,
    )
    check(
        list_contains(weak_summary.get("recurring_fix_categories"), "chop_policy")
        and list_contains(weak_summary.get("recurring_fix_categories"), "destructive_gesture"),
        "p023_contract_weak_fix_recurring_categories_incomplete",
        failures,
    )
    check(
        any(
            isinstance(candidate, dict)
            and candidate.get("category") == "chop_policy"
            and isinstance(candidate.get("software_next_step"), str)
            and bool(candidate["software_next_step"].strip())
            and isinstance(candidate.get("musician_payoff"), str)
            and bool(candidate["musician_payoff"].strip())
            for candidate in weak_candidates
        ),
        "p023_contract_chop_policy_fix_candidate_missing",
        failures,
    )
    check(
        list_contains(weak.get("fix_categories"), "chop_policy")
        and list_contains(weak.get("fix_categories"), "bass_movement"),
        "p023_contract_weak_fix_categories_incomplete",
        failures,
    )
    check(
        nested_list(report, "source_family_coverage", "missing_demo_candidate_families") == [],
        "p023_contract_demo_candidates_missing",
        failures,
    )
    check(
        list_contains_all(
            coverage.get("missing_human_verdict_families"),
            ["pad_noise", "weak_source", "bad_timing"],
        ),
        "p023_contract_missing_human_verdict_families_incomplete",
        failures,
    )
    check(
        list_contains_all(
            coverage.get("missing_demo_ready_families"),
            ["sparse_drums", "pad_noise", "weak_source", "bad_timing"],
        ),
        "p023_contract_missing_demo_ready_families_incomplete",
        failures,
    )
    check(
        source_family_status(coverage, "sparse_drums") == "human_verdict_non_demo",
        "p023_contract_sparse_drums_status_changed",
        failures,
    )
    for family in ["pad_noise", "weak_source", "bad_timing"]:
        check(
            source_family_status(coverage, family) == "candidate_only",
            f"p023_contract_{family}_status_changed",
            failures,
        )
    check(
        list_contains_all(
            demo.get("unverified_candidate_ids"),
            [
                "tonal-hook-rusharp-unverified-candidate",
                "pad-noise-fadapad-unverified-candidate",
                "sparse-bass-pressure-updated-unverified-candidate",
                "bad-timing-beat20-unverified-candidate",
                "weak-source-beat20-rejection-unverified-candidate",
            ],
        ),
        "p023_contract_unverified_demo_candidate_ids_incomplete",
        failures,
    )
    check(
        not list_contains(report.get("next_fix_categories"), "fixture_threshold"),
        "p023_contract_stale_fixture_threshold_next_fix_present",
        failures,
    )
    check(
        not list_contains(report.get("next_fix_categories"), "ui_cue"),
        "p023_contract_stale_ui_cue_next_fix_present",
        failures,
    )
    check(
        "source_family_demo_candidate_missing" not in blocker_codes,
        "p023_contract_unexpected_demo_candidate_missing_blocker",
        failures,
    )
    check(
        {
            "source_family_human_verdict_missing",
            "source_family_demo_ready_coverage_missing",
            "human_review_queue_unverified_candidates_present",
        }.issubset(blocker_codes),
        "p023_contract_required_blockers_missing",
        failures,
    )


def validate_current_next_actions(
    report: dict[str, Any],
    actions: list[Any],
    failures: list[str],
) -> None:
    current_top = str(
        nested_value(
            report,
            "current_evidence_reconciliation",
            "current_product_top_candidate_category",
        )
        or ""
    )
    if current_top != "none":
        return
    stale_categories = set(
        nested_list(
            report,
            "current_evidence_reconciliation",
            "stale_fixture_only_categories",
        )
    )
    weak_stale_actions = [
        action
        for action in actions
        if isinstance(action, dict)
        and action.get("target") == "weak output"
        and action.get("category") in stale_categories
    ]
    check(
        not weak_stale_actions,
        "next_actions_stale_weak_output_controls_obscure_review_work",
        failures,
    )
    source_family_actions = [
        action
        for action in actions
        if isinstance(action, dict)
        and action.get("category") == "source_selection"
        and str(action.get("target") or "")
        in {"bad_timing", "pad_noise", "sparse_drums", "weak_source"}
    ]
    check(
        len(source_family_actions) >= 4,
        "next_actions_source_family_review_coverage_missing",
        failures,
    )
    review_queue = object_or_empty(report.get("human_review_queue"))
    candidates_by_family = {
        str(candidate.get("source_family")): candidate
        for candidate in list_or_empty(review_queue.get("candidates"))
        if isinstance(candidate, dict) and candidate.get("source_family")
    }
    actions_by_family = {
        str(action.get("target")): action
        for action in source_family_actions
        if isinstance(action, dict) and action.get("target")
    }
    for family in {"bad_timing", "pad_noise", "sparse_drums", "weak_source"}:
        candidate = candidates_by_family.get(family)
        action = actions_by_family.get(family)
        if not candidate:
            continue
        check(
            isinstance(action, dict)
            and action.get("candidate_id") == candidate.get("entry_id")
            and action.get("review_priority") == candidate.get("review_priority")
            and action.get("demo_worthy_reason") == candidate.get("demo_worthy_reason")
            and action.get("not_demo_ready_reason") == candidate.get("not_demo_ready_reason")
            and action.get("required_verdict_current_state")
            == candidate.get("required_verdict_current_state")
            and artifact_ref_matches(action.get("rendered_wav"), candidate.get("rendered_wav"))
            and artifact_ref_matches(action.get("metrics"), candidate.get("metrics"))
            and artifact_ref_matches(action.get("review_prompt"), candidate.get("review_prompt"))
            and str(candidate.get("entry_id") or "") in str(action.get("action") or ""),
            f"next_actions_{family}_review_candidate_context_missing",
            failures,
        )


def artifact_ref_matches(left: Any, right: Any) -> bool:
    return (
        isinstance(left, dict)
        and isinstance(right, dict)
        and isinstance(left.get("path"), str)
        and bool(left["path"])
        and isinstance(left.get("sha256"), str)
        and len(left["sha256"]) == 64
        and left.get("path") == right.get("path")
        and left.get("sha256") == right.get("sha256")
    )


def write_report(output: Path, report: dict[str, Any]) -> None:
    (output / "sound-quality-readiness-report.json").write_text(json.dumps(report, indent=2) + "\n")
    (output / "sound-quality-readiness-report.md").write_text(markdown_report(report))


def markdown_report(report: dict[str, Any]) -> str:
    lines = [
        "# Sound Quality Readiness Report",
        "",
        f"- Phase: {report['phase']}",
        f"- Release readiness: {report['release_readiness']}",
        f"- Quality claim allowed: {str(report['quality_claim_allowed']).lower()}",
        f"- Musician summary: {report['musician_summary']}",
        "",
        "## Blockers",
        "",
    ]
    for blocker in report["blockers"]:
        lines.append(f"- `{blocker['code']}`: {blocker['reason']}")
    if not report["blockers"]:
        lines.append("- none")
    lines.extend(["", "## Next Actions", ""])
    for action in report["next_actions"]:
        lines.append(f"- `{action['category']}` / {action['target']}: {action['action']}")
    if not report["next_actions"]:
        lines.append("- none")
    review_queue = object_or_empty(report.get("human_review_queue"))
    lines.extend(["", "## Human Review Queue", ""])
    if review_queue.get("available"):
        lines.extend(
            [
                f"- Queue entries: `{review_queue.get('review_queue_count')}`",
                f"- High priority: `{review_queue.get('high_priority_count')}`",
                f"- Medium priority: `{review_queue.get('medium_priority_count')}`",
                f"- Source families: `{', '.join(review_queue.get('source_families', []))}`",
            ]
        )
        for candidate in review_queue.get("candidates", []):
            lines.extend(
                [
                    "",
                    f"### `{candidate['entry_id']}`",
                    "",
                    f"- Priority: `{candidate['review_priority']}`",
                    f"- Source family: `{candidate['source_family']}`",
                    f"- Strongest audible element: {candidate['strongest_audible_element']}",
                    f"- Source character: {candidate['source_character']}",
                    f"- Demo-worthy reason: {candidate['demo_worthy_reason']}",
                    f"- Not demo-ready: {candidate['not_demo_ready_reason']}",
                    f"- Required verdict state: `{candidate['required_verdict_current_state']}`",
                ]
            )
    else:
        lines.append("- missing")
    suite = object_or_empty(report.get("professional_output_suite"))
    current_evidence = object_or_empty(report.get("current_evidence_reconciliation"))
    source_selection_priority = object_or_empty(report.get("source_selection_priority"))
    ui_cue_priority = object_or_empty(report.get("ui_cue_priority"))
    perform_risk_cue = object_or_empty(report.get("jam_perform_risk_cue_contract"))
    source_character = object_or_empty(suite.get("source_character_selection"))
    source_character_window = object_or_empty(
        suite.get("source_character_window_selection")
    )
    source_selection_risk = object_or_empty(suite.get("source_selection_risk"))
    drum_pressure = object_or_empty(suite.get("drum_pressure"))
    destructive = object_or_empty(suite.get("destructive_gesture"))
    mix_balance = object_or_empty(suite.get("mix_balance"))
    lines.extend(
        [
            "",
            "## Professional Output Context",
            "",
            f"- Strongest elements: `{', '.join(suite.get('strongest_audible_elements', []))}`",
            (
                "- Source-character floors: "
                f"dense `{source_character.get('dense_hook_chop_score_floor')}`, "
                f"tonal `{source_character.get('tonal_hook_chop_score_floor')}`"
            ),
            (
                "- Source-window selection: "
                f"`{source_character_window.get('result')}`, cases "
                f"`{source_character_window.get('case_count')}`, promoted "
                f"`{source_character_window.get('promoted_case_count')}`, searched "
                f"`{source_character_window.get('searched_case_count')}`, min RMS retention "
                f"`{source_character_window.get('min_observed_rms_retention_ratio')}`, "
                f"max lift `{source_character_window.get('max_score_lift')}`"
            ),
            (
                "- Source-selection risk: "
                f"blocked `{source_selection_risk.get('edge_blocked_case_count')}`, "
                f"families `{', '.join(source_selection_risk.get('blocked_source_families', []))}`, "
                "demotions "
                f"`{', '.join(source_selection_risk.get('demotion_reasons', []))}`, "
                "review actions "
                f"`{', '.join(source_selection_risk.get('required_review_actions', []))}`"
            ),
            (
                "- Current evidence reconciliation: "
                f"weak top `{current_evidence.get('weak_top_candidate_category')}`, "
                "current top "
                f"`{current_evidence.get('current_product_top_candidate_category')}`, "
                "stale controls "
                f"`{', '.join(current_evidence.get('stale_fixture_only_categories', []))}`"
            ),
            (
                "- Source-selection priority: "
                f"`{source_selection_priority.get('priority_state')}`, cases "
                f"`{', '.join(source_selection_priority.get('primary_case_ids', []))}`, "
                "families "
                f"`{', '.join(source_selection_priority.get('source_families', []))}`, "
                "covered "
                f"`{', '.join(source_selection_priority.get('source_window_policy_covered_families', []))}`, "
                "uncovered "
                f"`{', '.join(source_selection_priority.get('source_window_policy_uncovered_families', []))}`, "
                "actions "
                f"`{', '.join(source_selection_priority.get('required_review_actions', []))}`"
            ),
            (
                "- UI-cue priority: "
                f"`{ui_cue_priority.get('priority_state')}`, cases "
                f"`{', '.join(ui_cue_priority.get('case_ids', []))}`, "
                "families "
                f"`{', '.join(ui_cue_priority.get('source_families', []))}`, "
                "surface "
                f"`{ui_cue_priority.get('cue_surface')}`, cues "
                f"`{', '.join(ui_cue_priority.get('required_player_cues', []))}`"
            ),
            (
                "- Jam perform-risk cue contract: "
                f"`{perform_risk_cue.get('result')}`, surface "
                f"`{perform_risk_cue.get('cue_surface')}`, degraded "
                f"`{perform_risk_cue.get('degraded_state_label')} | "
                f"{perform_risk_cue.get('degraded_action')}`, unavailable "
                f"`{perform_risk_cue.get('unavailable_state_label')} | "
                f"{perform_risk_cue.get('unavailable_action')}`"
            ),
            (
                "- Destructive gesture: "
                "dropout/stutter "
                f"`{destructive.get('dropout_to_stutter_rms_ratio')}`, "
                "dropout-silence/stutter "
                f"`{destructive.get('dropout_silence_to_stutter_rms_ratio')}`, "
                "stutter/hook transient "
                f"`{destructive.get('stutter_to_hook_transient_ratio')}`, "
                "stutter/source transient "
                f"`{destructive.get('stutter_to_source_transient_ratio')}`, "
                "restore/hook transient "
                f"`{destructive.get('restore_to_hook_transient_ratio')}`, "
                "restore/source transient "
                f"`{destructive.get('restore_to_source_transient_ratio')}`, "
                "restore/pressure "
                f"`{destructive.get('restore_to_pressure_rms_ratio')}`"
            ),
            (
                "- Drum pressure: "
                f"dense strongest `{drum_pressure.get('dense_strongest_audible_element')}`, "
                f"score `{drum_pressure.get('dense_break_physical_drum_pressure_score')}`, "
                "pressure transient/hook "
                f"`{drum_pressure.get('dense_break_pressure_transient_to_hook_ratio')}`"
            ),
            (
                "- TR-909 rendered pressure: "
                f"`{drum_pressure.get('tr909_rendered_result')}`, cases "
                f"`{drum_pressure.get('tr909_rendered_case_count')}`, support contribution min "
                f"`{drum_pressure.get('tr909_rendered_min_support_mix_contribution_ratio')}`, "
                f"low-band min `{drum_pressure.get('tr909_rendered_min_low_band_rms')}`"
            ),
            (
                "- Mix balance: "
                f"`{mix_balance.get('result')}`, support min "
                f"`{mix_balance.get('min_support_generated_to_source_rms_ratio')}`, "
                "source-first max "
                f"`{mix_balance.get('max_source_first_generated_to_source_rms_ratio')}`, "
                "headroom min "
                f"`{mix_balance.get('min_source_first_masking_headroom')}`, "
                "support max "
                f"`{mix_balance.get('max_support_generated_to_source_rms_ratio')}`"
            ),
        ]
    )
    lines.extend(["", "## Evidence Boundary", "", report["evidence_boundary"], ""])
    return "\n".join(lines)


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def read_optional_json_object(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    return read_json_object(path)


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict) and value, f"{path}: {field} must be non-empty object")
    return value


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def list_or_empty(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def string_list(value: Any) -> list[str]:
    return [str(item) for item in list_or_empty(value) if str(item)]


def string_list_field(data: dict[str, Any], field: str, path: Path) -> list[str]:
    value = list_field(data, field, path)
    require(all(isinstance(item, str) and item for item in value), f"{path}: {field} values must be strings")
    return [str(item) for item in value]


def list_contains(value: Any, expected: str) -> bool:
    return expected in {str(item) for item in list_or_empty(value)}


def list_contains_all(value: Any, expected: list[str]) -> bool:
    values = {str(item) for item in list_or_empty(value)}
    return set(expected).issubset(values)


def source_family_status(coverage: dict[str, Any], family: str) -> str:
    for item in list_or_empty(coverage.get("families")):
        if isinstance(item, dict) and item.get("source_family") == family:
            return str(item.get("status", ""))
    return ""


def required_string(data: dict[str, Any], field: str, path: Path, index: int) -> str:
    value = data.get(field)
    require(
        isinstance(value, str) and bool(value),
        f"{path}: production_fix_candidates[{index}].{field} must be non-empty string",
    )
    return value


def nested_value(report: dict[str, Any], object_name: str, field: str) -> Any:
    value = report.get(object_name)
    if not isinstance(value, dict):
        return None
    return value.get(field)


def nested_list(report: dict[str, Any], object_name: str, field: str) -> list[Any]:
    value = nested_value(report, object_name, field)
    return value if isinstance(value, list) else []


def number(value: Any) -> float:
    if isinstance(value, bool):
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
