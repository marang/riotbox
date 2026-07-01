#!/usr/bin/env python3
"""Generate the RIOTBOX-1279 MC-202 producer-grade closeout gate."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

import mc202_producer_fix_routing
from mc202_source_composed_review_gate import MC202_GATE_FIELD, pack_gate
from validate_human_listening_label_corpus import SCHEMA as LABEL_CORPUS_SCHEMA
from validate_human_listening_label_corpus import validate_manifest as validate_label_corpus


SCHEMA = "riotbox.mc202_producer_grade_closeout.v1"
PACK_SCHEMA = "riotbox.professional_output_listening_pack.v1"
REAL_SOURCE_SCHEMA = "riotbox.mc202_real_source_listening_pack.v1"
DEFAULT_PROFESSIONAL_PACK = Path(
    "artifacts/audio_qa/local-professional-output-listening-pack/"
    "professional-output-listening-pack.json"
)
DEFAULT_REAL_SOURCE_PACK = Path(
    "artifacts/audio_qa/local-mc202-real-source-listening-pack/"
    "mc202-real-source-listening-pack.json"
)
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-mc202-producer-grade-closeout")
TICKET = "RIOTBOX-1279"
PARENT_TICKET = "RIOTBOX-1264"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--professional-pack", type=Path, default=DEFAULT_PROFESSIONAL_PACK)
    parser.add_argument("--real-source-pack", type=Path, default=DEFAULT_REAL_SOURCE_PACK)
    parser.add_argument("--label-corpus", type=Path)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-mc202-producer-grade-closeout")
    parser.add_argument("--validate-report", type=Path)
    parser.add_argument("--require-all-source-composed-candidates", action="store_true")
    parser.add_argument("--mutation-fixtures", action="store_true")
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json_object(args.validate_report)
        else:
            professional_pack = read_json_object(args.professional_pack)
            real_source_pack = read_json_object(args.real_source_pack)
            label_corpus = read_json_object(args.label_corpus) if args.label_corpus else None
            report = build_report(args, professional_pack, real_source_pack, label_corpus)
            write_reports(args.output, report)
        failures = validate_report(report)
        if args.require_all_source_composed_candidates:
            failures.extend(validate_all_source_composed_candidates(report))
        if failures:
            raise ValueError(", ".join(failures))
        if args.mutation_fixtures:
            run_mutation_fixtures(report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid MC-202 producer-grade closeout: {error}", file=sys.stderr)
        return 1

    print("valid MC-202 producer-grade closeout")
    return 0


def build_report(
    args: argparse.Namespace,
    professional_pack: dict[str, Any],
    real_source_pack: dict[str, Any],
    label_corpus: dict[str, Any] | None,
) -> dict[str, Any]:
    require(
        professional_pack.get("schema") == PACK_SCHEMA,
        f"{args.professional_pack}: schema must be {PACK_SCHEMA}",
    )
    require(
        real_source_pack.get("schema") == REAL_SOURCE_SCHEMA,
        f"{args.real_source_pack}: schema must be {REAL_SOURCE_SCHEMA}",
    )
    professional_cases = list_field(professional_pack, "cases", args.professional_pack)
    real_source_cases = list_field(real_source_pack, "cases", args.real_source_pack)
    gate = pack_gate(professional_cases)
    candidates = [candidate_summary(case) for case in professional_cases]
    fix_candidates = mc202_producer_fix_routing.build_fix_candidates(candidates)
    label_summary = structured_label_corpus_summary(args.label_corpus, label_corpus)
    source_scaffold = real_source_scaffold_summary(
        args.real_source_pack,
        real_source_pack,
        real_source_cases,
    )
    review_queue = structured_listening_review_queue(candidates, label_summary["labels"])
    label_summary["matched_label_count"] = sum(
        1
        for entry in review_queue
        if object_or_empty(entry.get("listening_label_resolution")).get("label_id")
    )
    label_summary["resolved_queue_count"] = sum(
        1
        for entry in review_queue
        if object_or_empty(entry.get("listening_label_resolution")).get("status") == "resolved"
    )
    blockers = closeout_blockers(gate, candidates, source_scaffold, review_queue)
    technical_result = "pass" if gate["result"] == "pass" and not source_scaffold["failure_codes"] else "fail"
    promotion_result = "blocked_for_human_promotion" if blockers else "ready_for_human_promotion"
    parent_state = "keep_open" if blockers else "ready_to_close"
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "ticket": TICKET,
        "parent_ticket": PARENT_TICKET,
        "phase": "P023",
        "created_at": args.date,
        "result": "pass" if technical_result == "pass" else "fail",
        "technical_closeout_result": technical_result,
        "producer_grade_promotion_result": promotion_result,
        "quality_claim_allowed": False,
        "demo_bank_promotion_allowed": False,
        "parent_ticket_state": parent_state,
        "human_verdict_boundary": (
            "MC-202 candidates are source-composed and reviewable, but producer-grade "
            "quality and demo-bank promotion require structured human pass/weak/fail "
            "verdicts. Unverified or primitive/template-only evidence must not claim "
            "product quality."
        ),
        "professional_pack": {
            "path": str(args.professional_pack),
            "schema": professional_pack["schema"],
            "human_verdict": str(professional_pack.get("human_verdict")),
            "quality_proof": professional_pack.get("quality_proof"),
            "mc202_source_composed_review_gate": gate,
        },
        "real_source_listening_scaffold": source_scaffold,
        "structured_listening_label_corpus": {
            key: value for key, value in label_summary.items() if key != "labels"
        },
        "review_candidates": candidates,
        "structured_listening_review_queue_count": len(review_queue),
        "structured_listening_review_queue": review_queue,
        "mc202_producer_fix_candidate_count": len(fix_candidates),
        "mc202_producer_fix_summary": mc202_producer_fix_routing.build_fix_summary(
            candidates,
            fix_candidates,
        ),
        "mc202_producer_fix_candidates": fix_candidates,
        "blockers": blockers,
        "next_actions": next_actions(blockers, candidates, fix_candidates),
        "musician_summary": musician_summary(blockers),
    }


def candidate_summary(case: dict[str, Any]) -> dict[str, Any]:
    gate = object_field(case, MC202_GATE_FIELD, Path("<professional-pack>"))
    metrics = object_field(gate, "metrics", Path("<professional-pack>"))
    source_family = required_string(case, "source_family", Path("<professional-pack>"))
    review_artifacts = object_field(case, "review_artifacts", Path("<professional-pack>"))
    summary = {
        "case_id": required_string(case, "case_id", Path("<professional-pack>")),
        "source_family": source_family,
        "source": required_string(case, "source", Path("<professional-pack>")),
        "candidate": required_string(case, "candidate", Path("<professional-pack>")),
        "candidate_sha256": required_string(case, "candidate_sha256", Path("<professional-pack>")),
        "review": required_string(case, "review", Path("<professional-pack>")),
        "review_sha256": required_string(case, "review_sha256", Path("<professional-pack>")),
        "review_artifacts": review_artifacts,
        "human_verdict": str(case.get("human_verdict")),
        "demo_readiness": str(case.get("demo_readiness")),
        "quality_proof": case.get("quality_proof"),
        "demo_worthy_reason": required_string(case, "demo_worthy_reason", Path("<professional-pack>")),
        "not_demo_worthy_reason": required_string(case, "not_demo_worthy_reason", Path("<professional-pack>")),
        "source_composed_evidence": gate.get("source_composed_evidence") is True,
        "primitive_or_template_only": gate.get("primitive_or_template_only") is True,
        "promotion_blocked_until_human_pass": gate.get("promotion_blocked_until_human_pass") is True,
        "template_only_blocks_promotion": gate.get("template_only_blocks_promotion") is True,
        "gate_result": str(gate.get("result")),
        "gate_failure_codes": list(gate.get("failure_codes", [])),
        "metrics": metrics,
        "mc202_role_evidence": mc202_role_evidence(source_family, metrics),
    }
    summary["mc202_producer_fix_route"] = mc202_producer_fix_routing.route_candidate(summary)
    return summary


def structured_label_corpus_summary(
    path: Path | None,
    corpus: dict[str, Any] | None,
) -> dict[str, Any]:
    if corpus is None:
        return {
            "path": None,
            "schema": LABEL_CORPUS_SCHEMA,
            "label_count": 0,
            "matched_label_count": 0,
            "resolved_queue_count": 0,
            "failure_codes": [],
            "labels": [],
        }
    failures: list[str] = []
    try:
        summary = validate_label_corpus(corpus, path or Path("<label-corpus>"))
    except (TypeError, ValueError) as error:
        failures.append(f"label_corpus_invalid:{error}")
        summary = {"label_count": 0}
    labels = list_field(corpus, "labels", path or Path("<label-corpus>")) if not failures else []
    return {
        "path": str(path) if path else None,
        "schema": corpus.get("schema"),
        "label_count": int(number(summary.get("label_count"))),
        "matched_label_count": 0,
        "resolved_queue_count": 0,
        "failure_codes": failures,
        "labels": labels,
    }


def structured_listening_review_queue(
    candidates: list[dict[str, Any]],
    labels: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    entries = []
    for candidate in candidates:
        route = object_or_empty(candidate.get("mc202_producer_fix_route"))
        route_categories = string_list(route.get("proposed_fix_categories"))
        if candidate["human_verdict"] != "unverified" and candidate["demo_readiness"] != "unverified":
            continue
        if "human_listening" not in route_categories:
            continue
        review_artifacts = object_or_empty(candidate.get("review_artifacts"))
        review_dir = Path(candidate["review"]).parent
        prompt = str(review_dir / str(review_artifacts.get("prompt_markdown", "prompt.md")))
        metrics = str(review_dir / str(review_artifacts.get("metrics_json", "metrics.json")))
        role = object_or_empty(candidate.get("mc202_role_evidence"))
        label_resolution = listening_label_resolution(candidate, labels)
        entries.append(
            {
                "queue_id": f"mc202_human_listening_{candidate['case_id']}",
                "case_id": candidate["case_id"],
                "source_family": candidate["source_family"],
                "source": candidate["source"],
                "candidate": candidate["candidate"],
                "candidate_sha256": candidate["candidate_sha256"],
                "review": candidate["review"],
                "review_sha256": candidate["review_sha256"],
                "review_prompt": prompt,
                "review_metrics": metrics,
                "mc202_role": str(role.get("role", "")),
                "mc202_role_reason": str(role.get("musician_reason", "")),
                "producer_fix_categories": route_categories,
                "producer_fix_reason": str(route.get("musician_fix_reason", "")),
                "listening_label_resolution": label_resolution,
                "why_human_review_required": (
                    "Automated source-composed checks can prove reviewability, "
                    "but producer-grade quality needs a structured human listening verdict."
                ),
                "quality_proof": False,
                "automated_musical_approval": False,
                "allowed_human_verdicts": [
                    "keep",
                    "technically_ok_but_musically_weak",
                    "reject",
                ],
            }
        )
    return entries


def listening_label_resolution(
    candidate: dict[str, Any],
    labels: list[dict[str, Any]],
) -> dict[str, Any]:
    matches = [
        label
        for label in labels
        if label_matches_candidate(label, candidate)
    ]
    if not matches:
        return {
            "status": "unresolved",
            "reason": "no matching structured listening label",
            "quality_proof": False,
            "automated_musical_approval": False,
        }
    label = matches[0]
    verdict = str(label.get("human_verdict"))
    accepted = verdict in {"pass", "weak", "fail"}
    return {
        "status": "resolved" if accepted else "unresolved",
        "label_id": str(label.get("label_id", "")),
        "human_verdict": verdict,
        "reviewer": str(label.get("reviewer", "")),
        "summary": str(label.get("summary", "")),
        "failure_reason": str(label.get("failure_reason", "")),
        "preferred_direction": str(label.get("preferred_direction", "")),
        "reason_tags": object_or_empty(label.get("reason_tags")),
        "matched_artifact_hash": True,
        "quality_proof": False,
        "automated_musical_approval": False,
    }


def label_matches_candidate(label: dict[str, Any], candidate: dict[str, Any]) -> bool:
    identity = object_or_empty(label.get("artifact_identity"))
    audio_hashes = object_or_empty(identity.get("audio_sha256"))
    return (
        label.get("source_id") == candidate.get("case_id")
        and label.get("source_family") == candidate.get("source_family")
        and label.get("review_pack_schema") == PACK_SCHEMA
        and audio_hashes.get("rebuild_only_performance") == candidate.get("candidate_sha256")
    )


def mc202_role_evidence(source_family: str, metrics: dict[str, Any]) -> dict[str, Any]:
    if source_family == "sparse_bass_pressure":
        role = "bass_pressure"
        failure_codes = []
        if number(metrics.get("bass_movement_source_derived")) < 1.0:
            failure_codes.append("bass_movement_not_source_derived")
        if number(metrics.get("sparse_bass_movement_static_distance_hz")) < 1.75:
            failure_codes.append("bass_movement_static_distance_too_low")
        if number(metrics.get("sparse_bass_movement_frequency_span_hz")) < 17.0:
            failure_codes.append("bass_movement_span_too_low")
        reason = "MC-202 bass pressure follows source-derived sparse low-end movement."
    elif source_family == "tonal_hook":
        role = "hook_restraint_stab_answer"
        failure_codes = answer_role_failure_codes(metrics, min_scripted_distance=3.0)
        reason = "MC-202 answers the tonal hook with source-derived pressure/stab restraint instead of fake bass movement."
    elif source_family in {"dense_break", "non_dense_break"}:
        role = "pressure_answer"
        failure_codes = answer_role_failure_codes(metrics, min_scripted_distance=2.0)
        if source_family == "dense_break":
            failure_codes.extend(dense_answer_bite_failure_codes(metrics))
        reason = "MC-202 reinforces the break with source-derived pressure/answer movement."
    else:
        role = "unsupported_source_family"
        failure_codes = ["unsupported_source_family"]
        reason = "MC-202 role evidence is unavailable because the source family is not recognized."
    return {
        "role": role,
        "result": "pass" if not failure_codes else "fail",
        "source_family": source_family,
        "source_derived": not failure_codes,
        "quality_proof": False,
        "failure_codes": failure_codes,
        "musician_reason": reason,
    }


def answer_role_failure_codes(metrics: dict[str, Any], *, min_scripted_distance: float) -> list[str]:
    failures = []
    if number(metrics.get("pressure_lift_policy_decision_count")) < 6.0:
        failures.append("answer_role_decision_count_too_low")
    if number(metrics.get("arrangement_role_order_source_derived")) < 1.0:
        failures.append("answer_role_not_source_derived")
    if number(metrics.get("arrangement_scripted_role_distance")) < min_scripted_distance:
        failures.append("answer_role_too_close_to_scripted_template")
    if number(metrics.get("mc202_to_w30_rms_ratio")) < 0.16:
        failures.append("answer_role_mc202_too_weak")
    if number(metrics.get("pressure_low_band_lift_ratio")) < 1.50:
        failures.append("answer_role_pressure_lift_too_weak")
    return failures


def dense_answer_bite_failure_codes(metrics: dict[str, Any]) -> list[str]:
    failures = []
    if number(metrics.get("dense_answer_bite_source_derived")) < 1.0:
        failures.append("dense_answer_bite_not_source_derived")
    if number(metrics.get("dense_answer_bite_scripted_role_distance")) < 3.0:
        failures.append("dense_answer_bite_too_close_to_scripted_template")
    if number(metrics.get("dense_answer_bite_stab_score")) < 1.65:
        failures.append("dense_answer_bite_stab_too_weak")
    if number(metrics.get("dense_answer_bite_stab_margin")) < 0.15:
        failures.append("dense_answer_bite_stab_margin_too_low")
    if number(metrics.get("dense_answer_bite_pressure_snap_ratio")) < 1.06:
        failures.append("dense_answer_bite_snap_too_soft")
    if number(metrics.get("dense_answer_bite_score")) < 1.0:
        failures.append("dense_answer_bite_score_too_low")
    return failures


def real_source_scaffold_summary(
    path: Path,
    report: dict[str, Any],
    cases: list[dict[str, Any]],
) -> dict[str, Any]:
    failure_codes: list[str] = []
    if report.get("human_verdict") != "unverified":
        failure_codes.append("real_source_pack_human_verdict_not_unverified")
    if report.get("quality_proof") is not False:
        failure_codes.append("real_source_pack_claims_quality")
    dense_count = sum(1 for case in cases if case.get("source_family") == "dense_break")
    non_dense_count = len(cases) - dense_count
    if dense_count < 1:
        failure_codes.append("real_source_dense_case_missing")
    if non_dense_count < 1:
        failure_codes.append("real_source_non_dense_case_missing")
    for index, case in enumerate(cases):
        if case.get("quality_proof") is not False:
            failure_codes.append(f"real_source_case_{index}_claims_quality")
        if case.get("human_verdict") != "unverified":
            failure_codes.append(f"real_source_case_{index}_human_verdict_not_unverified")
        control = object_or_empty(case.get("primitive_ab_control"))
        if control.get("product_fallback_allowed") is not False:
            failure_codes.append(f"real_source_case_{index}_allows_product_fallback")
        if control.get("control_kind") != "primitive_renderer_non_product_control":
            failure_codes.append(f"real_source_case_{index}_control_kind_invalid")
    return {
        "path": str(path),
        "schema": report["schema"],
        "case_count": len(cases),
        "dense_case_count": dense_count,
        "non_dense_case_count": non_dense_count,
        "human_verdict": str(report.get("human_verdict")),
        "quality_proof": report.get("quality_proof"),
        "primitive_controls_are_product_output": False,
        "failure_codes": failure_codes,
    }


def closeout_blockers(
    gate: dict[str, Any],
    candidates: list[dict[str, Any]],
    source_scaffold: dict[str, Any],
    review_queue: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    if gate["result"] != "pass":
        blockers.append(
            {
                "code": "mc202_source_composed_pack_gate_failed",
                "severity": "release_blocking",
                "reason": "Dense and non-dense MC-202 source-composed review coverage is required.",
                "failure_codes": gate["failure_codes"],
            }
        )
    if source_scaffold["failure_codes"]:
        blockers.append(
            {
                "code": "mc202_real_source_scaffold_invalid",
                "severity": "release_blocking",
                "reason": "Real-source listening scaffold must stay unverified and keep primitive controls out of product output.",
                "failure_codes": source_scaffold["failure_codes"],
            }
        )
    unresolved = [
        str(entry["case_id"])
        for entry in review_queue
        if object_or_empty(entry.get("listening_label_resolution")).get("status") != "resolved"
    ]
    if unresolved:
        blockers.append(
            {
                "code": "structured_human_verdict_missing",
                "severity": "producer_grade_blocking",
                "case_ids": unresolved,
                "reason": "Producer-grade closeout needs structured listener verdicts before quality or demo-bank promotion.",
            }
        )
    weak_or_fail = [
        str(entry["case_id"])
        for entry in review_queue
        if object_or_empty(entry.get("listening_label_resolution")).get("human_verdict")
        in {"weak", "fail"}
    ]
    if weak_or_fail:
        blockers.append(
            {
                "code": "structured_human_verdict_not_pass",
                "severity": "producer_grade_blocking",
                "case_ids": weak_or_fail,
                "reason": "Weak/fail structured listening verdicts resolve review work but block producer-grade promotion.",
            }
        )
    primitive_or_template = primitive_or_template_case_ids(candidates)
    if primitive_or_template:
        blockers.append(
            {
                "code": "primitive_or_template_candidate_blocks_promotion",
                "severity": "producer_grade_blocking",
                "case_ids": primitive_or_template,
                "reason": "Primitive/template-only MC-202 output may be reviewed as a blocker, not promoted.",
            }
        )
    return blockers


def next_actions(
    blockers: list[dict[str, Any]],
    candidates: list[dict[str, Any]],
    fix_candidates: list[dict[str, Any]],
) -> list[dict[str, str]]:
    actions: list[dict[str, str]] = []
    if any(blocker["code"] == "structured_human_verdict_missing" for blocker in blockers):
        actions.append(
            {
                "category": "human_listening",
                "target": "mc202_source_composed_review_candidates",
                "software_benefit": "Keeps demo-bank promotion deterministic and hash-backed.",
                "musician_payoff": "The musician hears the exact candidate before it can be called producer-grade.",
            }
        )
    if any(blocker["code"] == "primitive_or_template_candidate_blocks_promotion" for blocker in blockers):
        actions.append(
            {
                "category": "production_fix",
                "target": "primitive_or_template_mc202_candidates",
                "software_benefit": "Routes non-source-composed cases to implementation work instead of hiding them as pass evidence.",
                "musician_payoff": "Weak or template-like bass parts become explicit fix work, not fake confidence.",
            }
        )
    if candidates:
        actions.append(
            {
                "category": "review_pack",
                "target": "candidate_wavs_and_prompts",
                "software_benefit": "Preserves exact WAV, metrics, prompt, and source-composed gate identities.",
                "musician_payoff": "Listening decisions can focus on hook, pressure, source character, and replay value.",
            }
        )
    if fix_candidates:
        top = fix_candidates[0]
        actions.append(
            {
                "category": "producer_fix_candidate",
                "target": str(top["candidate_id"]),
                "software_benefit": str(top["software_next_step"]),
                "musician_payoff": str(top["musician_payoff"]),
            }
        )
    return actions


def validate_report(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(report.get("ticket") == TICKET, "ticket_mismatch", failures)
    check(report.get("parent_ticket") == PARENT_TICKET, "parent_ticket_mismatch", failures)
    check(report.get("phase") == "P023", "phase_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    check(report.get("technical_closeout_result") == "pass", "technical_closeout_not_pass", failures)
    check(
        report.get("producer_grade_promotion_result") == "blocked_for_human_promotion",
        "producer_grade_promotion_not_blocked",
        failures,
    )
    check(report.get("quality_claim_allowed") is False, "quality_claim_allowed", failures)
    check(report.get("demo_bank_promotion_allowed") is False, "demo_bank_promotion_allowed", failures)
    check(report.get("parent_ticket_state") == "keep_open", "parent_ticket_not_kept_open", failures)
    boundary = str(report.get("human_verdict_boundary", ""))
    check("must not claim product quality" in boundary, "human_verdict_boundary_missing", failures)

    professional = object_or_empty(report.get("professional_pack"))
    gate = object_or_empty(professional.get("mc202_source_composed_review_gate"))
    check(gate.get("result") == "pass", "pack_gate_not_pass", failures)
    check(gate.get("quality_proof") is False, "pack_gate_claims_quality", failures)
    check(gate.get("dense_break_case_count", 0) >= 1, "dense_break_case_missing", failures)
    check(gate.get("non_dense_break_case_count", 0) >= 1, "non_dense_break_case_missing", failures)

    scaffold = object_or_empty(report.get("real_source_listening_scaffold"))
    check(scaffold.get("case_count", 0) >= 2, "real_source_case_count_too_low", failures)
    check(scaffold.get("dense_case_count", 0) >= 1, "real_source_dense_missing", failures)
    check(scaffold.get("non_dense_case_count", 0) >= 1, "real_source_non_dense_missing", failures)
    check(scaffold.get("quality_proof") is False, "real_source_claims_quality", failures)
    check(scaffold.get("primitive_controls_are_product_output") is False, "primitive_controls_are_product_output", failures)
    check(scaffold.get("failure_codes") == [], "real_source_scaffold_failures", failures)

    candidates = report.get("review_candidates")
    label_corpus = object_or_empty(report.get("structured_listening_label_corpus"))
    check(label_corpus.get("schema") == LABEL_CORPUS_SCHEMA, "label_corpus_schema_mismatch", failures)
    check(label_corpus.get("failure_codes") == [], "label_corpus_failures", failures)
    check(isinstance(candidates, list) and len(candidates) >= 2, "review_candidates_missing", failures)
    if isinstance(candidates, list):
        families = {str(candidate.get("source_family")) for candidate in candidates if isinstance(candidate, dict)}
        check("dense_break" in families, "candidate_dense_break_missing", failures)
        check(
            bool(families & {"sparse_bass_pressure", "tonal_hook", "non_dense_break"}),
            "candidate_non_dense_missing",
            failures,
        )
        check(
            any(
                isinstance(candidate, dict)
                and candidate.get("source_family") == "dense_break"
                and candidate.get("source_composed_evidence") is True
                for candidate in candidates
            ),
            "candidate_dense_break_source_composed_missing",
            failures,
        )
        check(
            any(
                isinstance(candidate, dict)
                and candidate.get("source_family") != "dense_break"
                and candidate.get("source_composed_evidence") is True
                for candidate in candidates
            ),
            "candidate_non_dense_source_composed_missing",
            failures,
        )
        for index, candidate in enumerate(candidates):
            validate_candidate(candidate, index, failures)
    fix_candidates = report.get("mc202_producer_fix_candidates")
    fix_summary = object_or_empty(report.get("mc202_producer_fix_summary"))
    check(
        report.get("mc202_producer_fix_candidate_count") == len(fix_candidates)
        if isinstance(fix_candidates, list)
        else False,
        "mc202_fix_candidate_count_mismatch",
        failures,
    )
    check(
        isinstance(fix_candidates, list) and bool(fix_candidates),
        "mc202_fix_candidates_missing",
        failures,
    )
    if isinstance(fix_candidates, list):
        mc202_producer_fix_routing.validate_fix_candidates(
            fix_candidates,
            candidates if isinstance(candidates, list) else [],
            fix_summary,
            failures,
        )
    blockers = report.get("blockers")
    check(isinstance(blockers, list) and bool(blockers), "blockers_missing", failures)
    if isinstance(blockers, list):
        codes = {str(blocker.get("code")) for blocker in blockers if isinstance(blocker, dict)}
        check("structured_human_verdict_missing" in codes, "human_verdict_blocker_missing", failures)
        primitive_case_ids = primitive_or_template_case_ids(candidates if isinstance(candidates, list) else [])
        if primitive_case_ids:
            check(
                "primitive_or_template_candidate_blocks_promotion" in codes,
                "primitive_template_blocker_missing",
                failures,
            )
        else:
            check(
                "primitive_or_template_candidate_blocks_promotion" not in codes,
                "primitive_template_blocker_stale",
                failures,
            )
    check(isinstance(report.get("next_actions"), list) and report["next_actions"], "next_actions_missing", failures)
    validate_structured_listening_review_queue(
        report,
        candidates if isinstance(candidates, list) else [],
        failures,
    )
    summary = str(report.get("musician_summary", ""))
    check("not demo-ready" in summary and "human pass" in summary, "musician_summary_missing", failures)
    return failures


def validate_candidate(candidate: Any, index: int, failures: list[str]) -> None:
    if not isinstance(candidate, dict):
        failures.append(f"candidate_{index}_not_object")
        return
    prefix = f"candidate_{index}"
    for field in (
        "case_id",
        "source_family",
        "candidate",
        "candidate_sha256",
        "review",
        "review_sha256",
        "human_verdict",
        "demo_readiness",
        "quality_proof",
        "source_composed_evidence",
        "primitive_or_template_only",
        "promotion_blocked_until_human_pass",
        "template_only_blocks_promotion",
        "mc202_role_evidence",
        "mc202_producer_fix_route",
    ):
        check(field in candidate, f"{prefix}_{field}_missing", failures)
    check(candidate.get("human_verdict") == "unverified", f"{prefix}_human_verdict_not_unverified", failures)
    check(candidate.get("demo_readiness") == "unverified", f"{prefix}_demo_readiness_not_unverified", failures)
    check(candidate.get("quality_proof") is False, f"{prefix}_claims_quality", failures)
    check(candidate.get("promotion_blocked_until_human_pass") is True, f"{prefix}_promotion_not_blocked", failures)
    check(candidate.get("template_only_blocks_promotion") is True, f"{prefix}_template_blocker_missing", failures)
    check(
        candidate.get("source_composed_evidence") != candidate.get("primitive_or_template_only"),
        f"{prefix}_source_composed_primitive_state_ambiguous",
        failures,
    )
    validate_role_evidence(candidate, prefix, failures)
    mc202_producer_fix_routing.validate_candidate_fix_route(candidate, prefix, failures)
    check(len(str(candidate.get("candidate_sha256", ""))) == 64, f"{prefix}_candidate_sha_invalid", failures)
    check(len(str(candidate.get("review_sha256", ""))) == 64, f"{prefix}_review_sha_invalid", failures)


def validate_structured_listening_review_queue(
    report: dict[str, Any],
    candidates: list[Any],
    failures: list[str],
) -> None:
    queue = report.get("structured_listening_review_queue")
    if not isinstance(queue, list):
        failures.append("structured_listening_review_queue_missing")
        return
    check(
        report.get("structured_listening_review_queue_count") == len(queue),
        "structured_listening_review_queue_count_mismatch",
        failures,
    )
    expected_ids = {
        str(candidate.get("case_id"))
        for candidate in candidates
        if isinstance(candidate, dict)
        and (
            candidate.get("human_verdict") == "unverified"
            or candidate.get("demo_readiness") == "unverified"
        )
        and "human_listening"
        in string_list(
            object_or_empty(candidate.get("mc202_producer_fix_route")).get(
                "proposed_fix_categories"
            )
        )
    }
    queue_ids = {
        str(entry.get("case_id"))
        for entry in queue
        if isinstance(entry, dict)
    }
    check(bool(queue), "structured_listening_review_queue_empty", failures)
    check(queue_ids == expected_ids, "structured_listening_review_queue_case_ids_stale", failures)
    candidate_by_id = {
        str(candidate.get("case_id")): candidate
        for candidate in candidates
        if isinstance(candidate, dict)
    }
    for index, entry in enumerate(queue):
        validate_structured_listening_review_queue_entry(
            entry,
            index,
            candidate_by_id,
            failures,
        )
    label_corpus = object_or_empty(report.get("structured_listening_label_corpus"))
    check(
        label_corpus.get("matched_label_count")
        == sum(
            1
            for entry in queue
            if object_or_empty(entry.get("listening_label_resolution")).get("label_id")
        ),
        "structured_listening_label_match_count_stale",
        failures,
    )
    check(
        label_corpus.get("resolved_queue_count")
        == sum(
            1
            for entry in queue
            if object_or_empty(entry.get("listening_label_resolution")).get("status")
            == "resolved"
        ),
        "structured_listening_label_resolved_count_stale",
        failures,
    )


def validate_structured_listening_review_queue_entry(
    entry: Any,
    index: int,
    candidate_by_id: dict[str, Any],
    failures: list[str],
) -> None:
    if not isinstance(entry, dict):
        failures.append(f"structured_listening_review_queue_{index}_not_object")
        return
    prefix = f"structured_listening_review_queue_{index}"
    case_id = str(entry.get("case_id", ""))
    candidate = object_or_empty(candidate_by_id.get(case_id))
    check(case_id in candidate_by_id, f"{prefix}_case_id_unknown", failures)
    for field in (
        "queue_id",
        "source_family",
        "source",
        "candidate",
        "candidate_sha256",
        "review",
        "review_sha256",
        "review_prompt",
        "review_metrics",
        "mc202_role",
        "mc202_role_reason",
        "producer_fix_categories",
        "producer_fix_reason",
        "listening_label_resolution",
        "why_human_review_required",
        "allowed_human_verdicts",
    ):
        check(bool(entry.get(field)), f"{prefix}_{field}_missing", failures)
    check(entry.get("quality_proof") is False, f"{prefix}_claims_quality", failures)
    check(
        entry.get("automated_musical_approval") is False,
        f"{prefix}_claims_automated_approval",
        failures,
    )
    check(str(entry.get("candidate")) == str(candidate.get("candidate")), f"{prefix}_candidate_path_stale", failures)
    check(str(entry.get("review")) == str(candidate.get("review")), f"{prefix}_review_path_stale", failures)
    for field in ("candidate", "review", "review_prompt", "review_metrics"):
        check(Path(str(entry.get(field, ""))).is_file(), f"{prefix}_{field}_missing_on_disk", failures)
    check(
        str(entry.get("source_family")) == str(candidate.get("source_family")),
        f"{prefix}_source_family_stale",
        failures,
    )
    check(len(str(entry.get("candidate_sha256", ""))) == 64, f"{prefix}_candidate_sha_invalid", failures)
    check(len(str(entry.get("review_sha256", ""))) == 64, f"{prefix}_review_sha_invalid", failures)
    categories = string_list(entry.get("producer_fix_categories"))
    check("human_listening" in categories, f"{prefix}_human_listening_category_missing", failures)
    verdicts = string_list(entry.get("allowed_human_verdicts"))
    check("keep" in verdicts, f"{prefix}_keep_verdict_missing", failures)
    check(
        "technically_ok_but_musically_weak" in verdicts,
        f"{prefix}_weak_verdict_missing",
        failures,
    )
    check("reject" in verdicts, f"{prefix}_reject_verdict_missing", failures)
    resolution = object_or_empty(entry.get("listening_label_resolution"))
    check(resolution.get("quality_proof") is False, f"{prefix}_label_resolution_claims_quality", failures)
    check(
        resolution.get("automated_musical_approval") is False,
        f"{prefix}_label_resolution_claims_automated_approval",
        failures,
    )
    if resolution.get("status") == "resolved":
        check(
            resolution.get("human_verdict") in {"pass", "weak", "fail"},
            f"{prefix}_label_resolution_verdict_invalid",
            failures,
        )
        check(bool(resolution.get("label_id")), f"{prefix}_label_resolution_label_id_missing", failures)


def validate_role_evidence(candidate: dict[str, Any], prefix: str, failures: list[str]) -> None:
    role = object_or_empty(candidate.get("mc202_role_evidence"))
    source_family = str(candidate.get("source_family"))
    check(role.get("source_family") == source_family, f"{prefix}_mc202_role_source_family_mismatch", failures)
    check(role.get("result") == "pass", f"{prefix}_mc202_role_evidence_not_pass", failures)
    check(role.get("source_derived") is True, f"{prefix}_mc202_role_not_source_derived", failures)
    check(role.get("quality_proof") is False, f"{prefix}_mc202_role_claims_quality", failures)
    check(role.get("failure_codes") == [], f"{prefix}_mc202_role_failure_codes", failures)
    check(
        isinstance(role.get("musician_reason"), str) and bool(role["musician_reason"]),
        f"{prefix}_mc202_role_musician_reason_missing",
        failures,
    )
    if source_family == "sparse_bass_pressure":
        check(role.get("role") == "bass_pressure", f"{prefix}_mc202_sparse_role_not_bass_pressure", failures)
        metrics = object_or_empty(candidate.get("metrics"))
        check(
            number(metrics.get("bass_movement_source_derived")) >= 1.0,
            f"{prefix}_mc202_sparse_bass_movement_not_source_derived",
            failures,
        )
        check(
            number(metrics.get("sparse_bass_movement_static_distance_hz")) >= 1.75,
            f"{prefix}_mc202_sparse_bass_distance_too_low",
            failures,
        )
        check(
            number(metrics.get("sparse_bass_movement_frequency_span_hz")) >= 17.0,
            f"{prefix}_mc202_sparse_bass_span_too_low",
            failures,
        )
    elif source_family == "tonal_hook":
        check(
            role.get("role") == "hook_restraint_stab_answer",
            f"{prefix}_mc202_tonal_role_not_answer_stab",
            failures,
        )
    elif source_family in {"dense_break", "non_dense_break"}:
        check(role.get("role") == "pressure_answer", f"{prefix}_mc202_dense_role_not_pressure_answer", failures)
    else:
        check(False, f"{prefix}_mc202_unsupported_source_family", failures)


def validate_all_source_composed_candidates(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    professional = object_or_empty(report.get("professional_pack"))
    gate = object_or_empty(professional.get("mc202_source_composed_review_gate"))
    check(gate.get("source_composed_case_count") == 3, "pack_gate_source_composed_count_mismatch", failures)
    check(gate.get("non_dense_break_case_count", 0) >= 2, "pack_gate_non_dense_break_count_too_low", failures)
    candidates = report.get("review_candidates")
    if not isinstance(candidates, list):
        failures.append("review_candidates_missing")
        return failures
    for index, candidate in enumerate(candidates):
        if not isinstance(candidate, dict):
            failures.append(f"candidate_{index}_not_object")
            continue
        check(
            candidate.get("source_composed_evidence") is True,
            f"candidate_{index}_source_composed_missing",
            failures,
        )
        check(
            candidate.get("primitive_or_template_only") is False,
            f"candidate_{index}_primitive_template_leaked",
            failures,
        )
    blockers = report.get("blockers")
    if isinstance(blockers, list):
        check(
            all(
                not isinstance(blocker, dict)
                or blocker.get("code") != "primitive_or_template_candidate_blocks_promotion"
                for blocker in blockers
            ),
            "primitive_template_blocker_stale",
            failures,
        )
    return failures


def run_mutation_fixtures(report: dict[str, Any]) -> None:
    fixtures = []
    mutated = json.loads(json.dumps(report))
    mutated["quality_claim_allowed"] = True
    fixtures.append(("quality_claim", mutated, "quality_claim_allowed"))

    mutated = json.loads(json.dumps(report))
    mutated["producer_grade_promotion_result"] = "ready_for_human_promotion"
    fixtures.append(("premature_promotion", mutated, "producer_grade_promotion_not_blocked"))

    mutated = json.loads(json.dumps(report))
    mutated["review_candidates"][0]["human_verdict"] = "pass"
    fixtures.append(("stale_human_verdict", mutated, "candidate_0_human_verdict_not_unverified"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_review_queue"] = []
    fixtures.append(("missing_review_queue", mutated, "structured_listening_review_queue_empty"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_review_queue_count"] = 999
    fixtures.append(("stale_review_queue_count", mutated, "structured_listening_review_queue_count_mismatch"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_review_queue"][0]["quality_proof"] = True
    fixtures.append(("review_queue_claims_quality", mutated, "structured_listening_review_queue_0_claims_quality"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_label_corpus"]["matched_label_count"] = 999
    fixtures.append(("stale_label_match_count", mutated, "structured_listening_label_match_count_stale"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_review_queue"][0]["listening_label_resolution"]["quality_proof"] = True
    fixtures.append(("label_resolution_claims_quality", mutated, "structured_listening_review_queue_0_label_resolution_claims_quality"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_review_queue"][0]["review_prompt"] = "/tmp/riotbox-missing-review-prompt.md"
    fixtures.append(("review_queue_missing_prompt", mutated, "structured_listening_review_queue_0_review_prompt_missing_on_disk"))

    mutated = json.loads(json.dumps(report))
    mutated["structured_listening_review_queue"][0]["case_id"] = "missing_case"
    fixtures.append(("review_queue_stale_case", mutated, "structured_listening_review_queue_case_ids_stale"))

    mutated = json.loads(json.dumps(report))
    mutated["real_source_listening_scaffold"]["primitive_controls_are_product_output"] = True
    fixtures.append(("primitive_control_leak", mutated, "primitive_controls_are_product_output"))

    mutated = json.loads(json.dumps(report))
    for candidate in mutated["review_candidates"]:
        if candidate["source_family"] == "dense_break":
            candidate["source_composed_evidence"] = False
    fixtures.append(("dense_not_source_composed", mutated, "candidate_dense_break_source_composed_missing"))

    mutated = json.loads(json.dumps(report))
    for candidate in mutated["review_candidates"]:
        if candidate["source_family"] == "tonal_hook":
            candidate["primitive_or_template_only"] = True
            candidate["source_composed_evidence"] = False
    fixtures.append(("primitive_without_blocker", mutated, "primitive_template_blocker_missing"))

    mutated = json.loads(json.dumps(report))
    for candidate in mutated["review_candidates"]:
        if candidate["source_family"] == "tonal_hook":
            candidate["primitive_or_template_only"] = False
            candidate["source_composed_evidence"] = False
    fixtures.append(("ambiguous_candidate_state", mutated, "candidate_1_source_composed_primitive_state_ambiguous"))

    mutated = json.loads(json.dumps(report))
    mutated["review_candidates"][0].pop("mc202_role_evidence")
    fixtures.append(("missing_role_evidence", mutated, "candidate_0_mc202_role_evidence_missing"))

    mutated = json.loads(json.dumps(report))
    mutated["review_candidates"][0].pop("mc202_producer_fix_route")
    fixtures.append(("missing_fix_route", mutated, "candidate_0_mc202_producer_fix_route_missing"))

    mutated = json.loads(json.dumps(report))
    mutated["mc202_producer_fix_candidate_count"] = 0
    fixtures.append(("stale_fix_count", mutated, "mc202_fix_candidate_count_mismatch"))

    mutated = json.loads(json.dumps(report))
    mutated["mc202_producer_fix_candidates"][0]["quality_proof"] = True
    fixtures.append(("fix_candidate_claims_quality", mutated, "mc202_fix_candidate_0_claims_quality"))

    mutated = json.loads(json.dumps(report))
    mutated["mc202_producer_fix_candidates"][0]["category"] = "generic_quality"
    fixtures.append(("fix_candidate_bad_category", mutated, "mc202_fix_candidate_0_category_invalid"))

    mutated = json.loads(json.dumps(report))
    mutated["mc202_producer_fix_summary"]["candidate_count"] = 0
    fixtures.append(("stale_fix_summary", mutated, "mc202_fix_summary_candidate_count_stale"))

    mutated = json.loads(json.dumps(report))
    mutated["review_candidates"][0]["source_family"] = "unknown_family"
    fixtures.append(("unsupported_source_family", mutated, "candidate_0_mc202_unsupported_source_family"))

    mutated = json.loads(json.dumps(report))
    mutated["review_candidates"][0]["mc202_role_evidence"]["source_family"] = "stale_family"
    fixtures.append(("role_source_family_mismatch", mutated, "candidate_0_mc202_role_source_family_mismatch"))

    mutated = json.loads(json.dumps(report))
    for index, candidate in enumerate(mutated["review_candidates"]):
        if candidate["source_family"] == "sparse_bass_pressure":
            candidate["mc202_role_evidence"]["role"] = "pressure_answer"
            fixtures.append(
                ("sparse_wrong_role", mutated, f"candidate_{index}_mc202_sparse_role_not_bass_pressure")
            )
            break

    mutated = json.loads(json.dumps(report))
    for index, candidate in enumerate(mutated["review_candidates"]):
        if candidate["source_family"] == "tonal_hook":
            candidate["mc202_role_evidence"]["role"] = "bass_pressure"
            fixtures.append(("tonal_wrong_role", mutated, f"candidate_{index}_mc202_tonal_role_not_answer_stab"))
            break

    for name, fixture, expected in fixtures:
        failures = validate_report(fixture)
        if expected not in failures:
            raise SystemExit(f"mutation {name} expected {expected}, got {failures}")


def write_reports(output: Path, report: dict[str, Any]) -> None:
    output.mkdir(parents=True, exist_ok=True)
    (output / "mc202-producer-grade-closeout.json").write_text(json.dumps(report, indent=2) + "\n")
    (output / "mc202-producer-grade-closeout.md").write_text(markdown_report(report))


def markdown_report(report: dict[str, Any]) -> str:
    lines = [
        "# MC-202 Producer-Grade Closeout",
        "",
        f"- Ticket: `{report['ticket']}`",
        f"- Parent quality ticket: `{report['parent_ticket']}`",
        f"- Technical closeout: `{report['technical_closeout_result']}`",
        f"- Producer-grade promotion: `{report['producer_grade_promotion_result']}`",
        f"- Quality claim allowed: `{str(report['quality_claim_allowed']).lower()}`",
        "",
        "## Candidate Status",
        "",
    ]
    for candidate in report["review_candidates"]:
        role = candidate["mc202_role_evidence"]
        lines.extend(
            [
                f"### `{candidate['case_id']}`",
                "",
                f"- Source family: `{candidate['source_family']}`",
                f"- MC-202 role evidence: `{role['role']}` / `{role['result']}`",
                f"- Musician role reason: {role['musician_reason']}",
                f"- Producer fix route: `{candidate['mc202_producer_fix_route']['proposed_next_fix_category']}`",
                f"- Producer fix reason: {candidate['mc202_producer_fix_route']['musician_fix_reason']}",
                f"- WAV: `{candidate['candidate']}`",
                f"- Source-composed evidence: `{str(candidate['source_composed_evidence']).lower()}`",
                f"- Primitive/template only: `{str(candidate['primitive_or_template_only']).lower()}`",
                f"- Human verdict: `{candidate['human_verdict']}`",
                f"- Demo readiness: `{candidate['demo_readiness']}`",
                f"- Not demo-ready: {candidate['not_demo_worthy_reason']}",
                "",
            ]
        )
    lines.extend(["## Structured Listening Review Queue", ""])
    for entry in report["structured_listening_review_queue"]:
        lines.extend(
            [
                f"### `{entry['queue_id']}`",
                "",
                f"- Case: `{entry['case_id']}` / `{entry['source_family']}`",
                f"- Candidate WAV: `{entry['candidate']}`",
                f"- Review JSON: `{entry['review']}`",
                f"- Review prompt: `{entry['review_prompt']}`",
                f"- Review metrics: `{entry['review_metrics']}`",
                f"- MC-202 role: `{entry['mc202_role']}`",
                f"- Why human review is required: {entry['why_human_review_required']}",
                "",
            ]
        )
    lines.extend(["## Producer Fix Candidates", ""])
    for fix in report["mc202_producer_fix_candidates"]:
        lines.extend(
            [
                f"### `{fix['candidate_id']}`",
                "",
                f"- Category: `{fix['category']}`",
                f"- Cases: `{', '.join(fix['case_ids'])}`",
                f"- Primary cases: `{', '.join(fix['primary_case_ids']) or 'none'}`",
                f"- Source families: `{', '.join(fix['source_families'])}`",
                f"- Software next step: {fix['software_next_step']}",
                f"- Musician payoff: {fix['musician_payoff']}",
                f"- Artifacts: `{', '.join(fix['artifact_refs'])}`",
                "",
            ]
        )
    lines.extend(["## Blockers", ""])
    for blocker in report["blockers"]:
        lines.append(f"- `{blocker['code']}`: {blocker['reason']}")
    lines.extend(["", "## Musician Summary", "", report["musician_summary"], ""])
    return "\n".join(lines)


def musician_summary(blockers: list[dict[str, Any]]) -> str:
    primitive_blocked = any(
        blocker.get("code") == "primitive_or_template_candidate_blocks_promotion"
        for blocker in blockers
    )
    if blockers:
        if primitive_blocked:
            return (
                "MC-202 is technically reviewable across dense and non-dense sources, "
                "but it is not demo-ready and must not claim producer-grade quality "
                "until structured listening records a human pass. Primitive/template "
                "cases stay fix targets, not musical proof."
            )
        return (
            "MC-202 is technically reviewable across dense and non-dense sources, "
            "but it is not demo-ready and must not claim producer-grade quality "
            "until structured listening records a human pass."
        )
    return "MC-202 has no closeout blockers."


def primitive_or_template_case_ids(candidates: list[Any]) -> list[str]:
    return [
        str(candidate.get("case_id"))
        for candidate in candidates
        if isinstance(candidate, dict) and candidate.get("primitive_or_template_only") is True
    ]


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def list_field(value: dict[str, Any], field: str, path: Path) -> list[dict[str, Any]]:
    raw = value.get(field)
    require(isinstance(raw, list), f"{path}: {field} must be array")
    for index, item in enumerate(raw):
        require(isinstance(item, dict), f"{path}: {field}[{index}] must be object")
    return raw


def object_field(value: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    raw = value.get(field)
    require(isinstance(raw, dict), f"{path}: {field} must be object")
    return raw


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def number(value: Any) -> float:
    return float(value) if isinstance(value, (int, float)) else 0.0


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if isinstance(item, str)]


def required_string(value: dict[str, Any], field: str, path: Path) -> str:
    raw = value.get(field)
    require(isinstance(raw, str) and bool(raw.strip()), f"{path}: {field} must be non-empty string")
    return raw


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
