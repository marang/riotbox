#!/usr/bin/env python3
"""Generate the P023 sound-quality readiness report."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.sound_quality_readiness_report.v1"
RUBRIC_SCHEMA = "riotbox.sound_product_readiness_rubric.v1"
SOURCE_CORPUS_SCHEMA = "riotbox.sound_excellence_source_corpus.v1"
DEMO_BANK_SCHEMA = "riotbox.release_grade_demo_bank.v1"
WEAK_ROUTING_SCHEMA = "riotbox.weak_output_fix_routing.v1"
PROFESSIONAL_SUITE_SCHEMA = "riotbox.professional_output_suite.v1"
HUMAN_REVIEW_QUEUE_SCHEMA = "riotbox.release_demo_human_review_queue.v1"

DEFAULT_RUBRIC = Path("scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json")
DEFAULT_SOURCE_CORPUS = Path("docs/benchmarks/sound_excellence_source_corpus_v1.json")
DEFAULT_DEMO_BANK = Path("scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json")
DEFAULT_WEAK_ROUTING = Path("artifacts/audio_qa/local-weak-output-fix-routing/weak-output-fix-routing.json")
DEFAULT_PROFESSIONAL_SUITE = Path("artifacts/audio_qa/local-professional-output-suite/professional-output-suite.json")
DEFAULT_HUMAN_REVIEW_QUEUE = Path(
    "artifacts/audio_qa/local-release-demo-human-review-queue/release-demo-human-review-queue.json"
)
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-sound-quality-readiness-report")
MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO = 0.22
MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ = 1.25
MIN_SPARSE_BASS_MOVEMENT_SPAN_HZ = 8.00
MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO = 1.60
MIN_SPARSE_BASS_DOMINANCE_MARGIN = 0.08
MIN_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.16
MAX_MIX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO = 0.16
MAX_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.46

CORPUS_TO_DEMO_FAMILIES = {
    "dense_break": {"dense_break"},
    "sparse_drums": {"sparse_bass_pressure"},
    "tonal_riff": {"tonal_hook"},
    "pad_noise": {"tonal_pad"},
    "weak_source": {"other"},
    "bad_timing": {"bad_timing"},
}

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rubric", type=Path, default=DEFAULT_RUBRIC)
    parser.add_argument("--source-corpus", type=Path, default=DEFAULT_SOURCE_CORPUS)
    parser.add_argument("--demo-bank", type=Path, default=DEFAULT_DEMO_BANK)
    parser.add_argument("--weak-routing", type=Path, default=DEFAULT_WEAK_ROUTING)
    parser.add_argument("--professional-output-suite", type=Path, default=DEFAULT_PROFESSIONAL_SUITE)
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
    next_fix_categories = sorted(set(weak_summary["fix_categories"]) | set(demo_summary["weak_fix_categories"]))
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
        "human_review_queue": review_summary,
        "blockers": blockers,
        "next_actions": next_actions(
            source_families,
            demo_summary,
            weak_summary,
            suite_summary,
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
                "musician_fix_reason": str(case.get("musician_fix_reason")),
            }
            for case in cases
        ],
    }


def weak_routing_candidates(report: dict[str, Any], path: Path) -> list[dict[str, str]]:
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
    edge = metrics.get("edge_source_professional_diagnostics", {})
    feral_mix_balance = object_or_empty(report.get("feral_mix_balance"))
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
            "dense_hook_chop_score_floor": number(
                dense.get("hook_chop_source_character_score_floor")
            ),
            "dense_hook_chop_score_span": number(
                dense.get("hook_chop_source_character_score_span")
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
            "tonal_w30_to_source_rms_ratio": number(
                source_wav.get("tonal_w30_to_source_rms_ratio")
            ),
            "tonal_hook_chop_score_floor": number(
                source_wav.get("tonal_hook_chop_source_character_score_floor")
            ),
            "tonal_hook_chop_score_span": number(
                source_wav.get("tonal_hook_chop_source_character_score_span")
            ),
            "min_rebuild_only_source_character_survival_score": min(
                number(dense.get("rebuild_only_source_character_survival_score")),
                number(matrix.get("min_rebuild_only_source_character_survival_score")),
                number(source_wav.get("min_rebuild_only_source_character_survival_score")),
                number(edge.get("min_rebuild_only_source_character_survival_score")),
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
            "source_wav_sparse_bass_dominance_margin": number(
                source_wav.get("sparse_bass_dominance_margin")
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
            "max_support_generated_to_source_rms_ratio": number(
                feral_mix_balance.get("max_support_generated_to_source_rms_ratio")
            ),
        },
    }


def human_review_queue_summary(report: dict[str, Any] | None, path: Path) -> dict[str, Any]:
    if report is None:
        return {
            "path": str(path),
            "available": False,
            "result": "missing",
            "review_queue_count": 0,
            "priority_counts": {},
            "high_priority_count": 0,
            "medium_priority_count": 0,
            "source_families": [],
            "review_blockers": [],
            "candidates": [],
        }
    require(
        report.get("schema") == HUMAN_REVIEW_QUEUE_SCHEMA,
        f"{path}: schema must be {HUMAN_REVIEW_QUEUE_SCHEMA}",
    )
    queue = list_field(report, "review_queue", path)
    priority_counts = Counter(str(entry.get("review_priority")) for entry in queue if isinstance(entry, dict))
    candidates = []
    review_blockers: set[str] = set()
    source_families: set[str] = set()
    for index, entry in enumerate(queue):
        require(isinstance(entry, dict), f"{path}: review_queue[{index}] must be object")
        source_family = required_queue_string(entry, "source_family", path, index)
        source_families.add(source_family)
        blockers = queue_string_list(entry, "review_blockers", path, index)
        review_blockers.update(blockers)
        questions = queue_string_list(entry, "required_listening_questions", path, index)
        verdict_path = object_or_empty(entry.get("required_verdict_path"))
        candidates.append(
            {
                "entry_id": required_queue_string(entry, "entry_id", path, index),
                "review_priority": required_queue_string(entry, "review_priority", path, index),
                "source_family": source_family,
                "human_verdict": required_queue_string(entry, "human_verdict", path, index),
                "demo_readiness": required_queue_string(entry, "demo_readiness", path, index),
                "quality_claim": entry.get("quality_claim"),
                "strongest_audible_element": required_queue_string(
                    entry,
                    "strongest_audible_element",
                    path,
                    index,
                ),
                "source_character": required_queue_string(entry, "source_character", path, index),
                "demo_worthy_reason": required_queue_string(
                    entry,
                    "demo_worthy_reason",
                    path,
                    index,
                ),
                "not_demo_ready_reason": required_queue_string(
                    entry,
                    "not_demo_ready_reason",
                    path,
                    index,
                ),
                "review_blockers": blockers,
                "required_verdict_current_state": str(verdict_path.get("current_state", "")),
                "required_listening_question_count": len(questions),
                "required_listening_questions": questions,
            }
        )
    return {
        "path": str(path),
        "available": True,
        "result": str(report.get("result")),
        "review_queue_count": len(candidates),
        "priority_counts": dict(sorted(priority_counts.items())),
        "high_priority_count": priority_counts.get("high", 0),
        "medium_priority_count": priority_counts.get("medium", 0),
        "source_families": sorted(source_families),
        "review_blockers": sorted(review_blockers),
        "candidates": candidates,
    }


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
    review_queue: dict[str, Any],
) -> list[dict[str, Any]]:
    actions: list[dict[str, Any]] = []
    for family in coverage["missing_demo_ready_families"]:
        actions.append(
            {
                "category": "source_selection",
                "target": family,
                "action": "Create or promote a real-source candidate with structured human listening evidence.",
            }
        )
    for category in sorted(set(weak["fix_categories"]) | set(demo["weak_fix_categories"])):
        actions.append(
            {
                "category": category,
                "target": "weak output",
                "action": f"Implement the next bounded production fix for {category}.",
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
    weak_available = nested_value(report, "weak_output_routing", "available")
    weak_fix_summary = nested_value(report, "weak_output_routing", "production_fix_summary")
    suite_available = nested_value(report, "professional_output_suite", "available")
    suite_scripted = nested_value(report, "professional_output_suite", "scripted_generation")
    suite_quality = nested_value(report, "professional_output_suite", "quality_proof")
    suite_source_character = nested_value(
        report,
        "professional_output_suite",
        "source_character_selection",
    )
    suite_drum_pressure = nested_value(report, "professional_output_suite", "drum_pressure")
    suite_mix_balance = nested_value(report, "professional_output_suite", "mix_balance")
    suite_strongest = nested_list(
        report,
        "professional_output_suite",
        "strongest_audible_elements",
    )
    review_queue = object_or_empty(report.get("human_review_queue"))
    review_queue_available = review_queue.get("available")
    review_candidates = review_queue.get("candidates", [])
    review_count = review_queue.get("review_queue_count")

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
            number(source_character.get("matrix_dense_w30_to_source_rms_ratio"))
            >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
            "professional_suite_matrix_dense_hook_chop_w30_too_weak",
            failures,
        )
        check(
            number(source_character.get("tonal_w30_to_source_rms_ratio"))
            >= MIN_HOOK_FORWARD_W30_TO_SOURCE_RMS_RATIO,
            "professional_suite_tonal_hook_chop_w30_too_weak",
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
            number(source_character.get("min_rebuild_only_source_character_survival_score"))
            >= 0.70,
            "professional_suite_source_character_survival_too_low",
            failures,
        )
        drum_pressure = suite_drum_pressure if isinstance(suite_drum_pressure, dict) else {}
        check(
            drum_pressure.get("dense_strongest_audible_element") == "snare",
            "professional_suite_dense_snare_not_strongest",
            failures,
        )
        check(
            number(drum_pressure.get("dense_break_physical_drum_pressure_score")) >= 1.58,
            "professional_suite_dense_drum_pressure_too_weak",
            failures,
        )
        check(
            number(drum_pressure.get("dense_break_snare_pressure_margin")) >= 0.22,
            "professional_suite_dense_snare_pressure_ambiguous",
            failures,
        )
        check(
            number(drum_pressure.get("dense_break_pressure_transient_to_hook_ratio"))
            >= 0.70,
            "professional_suite_dense_pressure_transient_too_soft",
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
            number(mix_balance.get("max_support_generated_to_source_rms_ratio"))
            <= MAX_MIX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
            "professional_suite_mix_support_masks_source",
            failures,
        )

    if review_queue_available is True:
        check(
            isinstance(review_candidates, list) and bool(review_candidates),
            "human_review_queue_candidates_missing",
            failures,
        )
        check(
            review_count == len(review_candidates) and review_count > 0,
            "human_review_queue_count_mismatch",
            failures,
        )
        check(
            number(review_queue.get("high_priority_count")) >= 1,
            "human_review_queue_high_priority_missing",
            failures,
        )
        check(
            isinstance(review_queue.get("source_families"), list)
            and {"pad_noise", "weak_source", "bad_timing"}.issubset(
                set(str(item) for item in review_queue.get("source_families", []))
            ),
            "human_review_queue_source_families_incomplete",
            failures,
        )
        review_blockers = set(str(item) for item in review_queue.get("review_blockers", []))
        check(
            {
                "human_verdict_unverified",
                "demo_readiness_unverified",
                "quality_claim_blocked",
            }.issubset(review_blockers),
            "human_review_queue_review_blockers_missing",
            failures,
        )
        if isinstance(review_candidates, list):
            for index, candidate in enumerate(review_candidates):
                validate_review_queue_candidate(candidate, index, failures)
    else:
        check(
            any(blocker.get("code") == "human_review_queue_not_available" for blocker in blockers),
            "missing_human_review_queue_blocker",
            failures,
        )

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


def validate_review_queue_candidate(candidate: Any, index: int, failures: list[str]) -> None:
    if not isinstance(candidate, dict):
        failures.append(f"human_review_queue_candidate_{index}_not_object")
        return
    for field in [
        "entry_id",
        "review_priority",
        "source_family",
        "strongest_audible_element",
        "source_character",
        "demo_worthy_reason",
        "not_demo_ready_reason",
    ]:
        check(
            isinstance(candidate.get(field), str) and bool(candidate[field].strip()),
            f"human_review_queue_candidate_{index}_{field}_missing",
            failures,
        )
    check(
        candidate.get("human_verdict") == "unverified",
        f"human_review_queue_candidate_{index}_not_unverified",
        failures,
    )
    check(
        candidate.get("demo_readiness") == "unverified",
        f"human_review_queue_candidate_{index}_not_unverified_demo",
        failures,
    )
    check(
        candidate.get("quality_claim") is False,
        f"human_review_queue_candidate_{index}_claims_quality",
        failures,
    )
    blockers = candidate.get("review_blockers")
    check(
        isinstance(blockers, list)
        and {
            "human_verdict_unverified",
            "demo_readiness_unverified",
            "quality_claim_blocked",
        }.issubset(set(str(item) for item in blockers)),
        f"human_review_queue_candidate_{index}_review_blockers_missing",
        failures,
    )
    check(
        candidate.get("required_verdict_current_state")
        == "human_verdict:unverified/demo_readiness:unverified",
        f"human_review_queue_candidate_{index}_stale_verdict_state",
        failures,
    )
    check(
        isinstance(candidate.get("required_listening_questions"), list)
        and candidate.get("required_listening_question_count") == len(candidate["required_listening_questions"])
        and candidate.get("required_listening_question_count") >= 6,
        f"human_review_queue_candidate_{index}_listening_questions_incomplete",
        failures,
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
    source_character = object_or_empty(suite.get("source_character_selection"))
    drum_pressure = object_or_empty(suite.get("drum_pressure"))
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
                "- Drum pressure: "
                f"dense strongest `{drum_pressure.get('dense_strongest_audible_element')}`, "
                f"score `{drum_pressure.get('dense_break_physical_drum_pressure_score')}`, "
                "pressure transient/hook "
                f"`{drum_pressure.get('dense_break_pressure_transient_to_hook_ratio')}`"
            ),
            (
                "- Mix balance: "
                f"`{mix_balance.get('result')}`, support min "
                f"`{mix_balance.get('min_support_generated_to_source_rms_ratio')}`"
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


def string_list_field(data: dict[str, Any], field: str, path: Path) -> list[str]:
    value = list_field(data, field, path)
    require(all(isinstance(item, str) and item for item in value), f"{path}: {field} values must be strings")
    return [str(item) for item in value]


def queue_string_list(data: dict[str, Any], field: str, path: Path, index: int) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list), f"{path}: review_queue[{index}].{field} must be array")
    strings = [str(item) for item in value if isinstance(item, str) and item]
    require(len(strings) == len(value), f"{path}: review_queue[{index}].{field} values must be strings")
    return strings


def required_queue_string(data: dict[str, Any], field: str, path: Path, index: int) -> str:
    value = data.get(field)
    require(
        isinstance(value, str) and bool(value),
        f"{path}: review_queue[{index}].{field} must be non-empty string",
    )
    return value


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
