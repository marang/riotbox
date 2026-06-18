#!/usr/bin/env python3
"""Validate professional output listening-pack contracts."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.professional_output_listening_pack.v1"
MC202_GATE_FIELD = "mc202_source_composed_review_gate"
EXPECTED_FAMILIES = ["dense_break", "sparse_bass_pressure", "tonal_hook"]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("report", type=Path)
    parser.add_argument("--require-review-files", action="store_true")
    parser.add_argument("--mutation-fixtures", action="store_true")
    args = parser.parse_args()

    try:
        report = read_json_object(args.report)
        failures = validate_report(report, args.report, args.require_review_files)
        if failures:
            raise ValueError(", ".join(failures))
        if args.mutation_fixtures:
            run_mutation_fixtures(report, args.report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid professional output listening pack: {error}", file=sys.stderr)
        return 1

    print(f"valid professional output listening pack: {args.report}")
    return 0


def validate_report(report: dict[str, Any], path: Path, require_review_files: bool) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    check(report.get("agent_verdict") == "agent_promising", "agent_verdict_not_promising", failures)
    check(report.get("human_verdict") == "unverified", "human_verdict_not_unverified", failures)
    check(report.get("evidence_role") == "listening_review_scaffold", "evidence_role_mismatch", failures)
    check(report.get("source_backed") is True, "source_backed_not_true", failures)
    check(report.get("source_timing_backed") is True, "source_timing_backed_not_true", failures)
    check(report.get("scripted_generation") is True, "scripted_generation_not_true", failures)
    check(report.get("quality_proof") is False, "quality_proof_claimed", failures)
    cases = report.get("cases")
    check(isinstance(cases, list) and len(cases) == 3, "case_count_mismatch", failures)
    check(report.get("case_count") == 3, "case_count_field_mismatch", failures)
    if isinstance(cases, list):
        families = sorted(str(case.get("source_family")) for case in cases if isinstance(case, dict))
        check(families == EXPECTED_FAMILIES, "source_family_set_mismatch", failures)
        for index, case in enumerate(cases):
            validate_case(case, index, path, require_review_files, failures)
        check(
            any(
                isinstance(case, dict)
                and case.get("source_family") == "dense_break"
                and object_or_empty(case.get(MC202_GATE_FIELD)).get("family_kind") == "dense_break"
                and object_or_empty(case.get(MC202_GATE_FIELD)).get("source_composed_evidence") is True
                for case in cases
            ),
            "dense_break_source_composed_missing",
            failures,
        )
        check(
            any(
                isinstance(case, dict)
                and case.get("source_family") == "sparse_bass_pressure"
                and object_or_empty(case.get(MC202_GATE_FIELD)).get("family_kind") == "non_dense_break"
                and object_or_empty(case.get(MC202_GATE_FIELD)).get("source_composed_evidence") is True
                and number(object_or_empty(object_or_empty(case.get(MC202_GATE_FIELD)).get("metrics")).get("bass_movement_source_derived")) >= 1.0
                for case in cases
            ),
            "sparse_bass_pressure_source_composed_missing",
            failures,
        )
        check(
            any(
                isinstance(case, dict)
                and case.get("source_family") == "tonal_hook"
                and object_or_empty(case.get(MC202_GATE_FIELD)).get("family_kind") == "non_dense_break"
                and object_or_empty(case.get(MC202_GATE_FIELD)).get("source_composed_evidence") is True
                and number(object_or_empty(object_or_empty(case.get(MC202_GATE_FIELD)).get("metrics")).get("mc202_to_w30_rms_ratio")) >= 0.16
                for case in cases
            ),
            "tonal_hook_source_composed_missing",
            failures,
        )
    validate_pack_gate(object_or_empty(report.get(MC202_GATE_FIELD)), failures)
    return failures


def validate_pack_gate(gate: dict[str, Any], failures: list[str]) -> None:
    check(gate.get("result") == "pass", "pack_gate_not_pass", failures)
    check(gate.get("source_composed_case_count") == 3, "pack_gate_source_composed_count_mismatch", failures)
    check(gate.get("dense_break_case_count", 0) >= 1, "pack_gate_dense_break_missing", failures)
    check(gate.get("non_dense_break_case_count", 0) >= 2, "pack_gate_non_dense_break_missing", failures)
    check(gate.get("quality_proof") is False, "pack_gate_claims_quality", failures)


def validate_case(
    case: Any,
    index: int,
    report_path: Path,
    require_review_files: bool,
    failures: list[str],
) -> None:
    if not isinstance(case, dict):
        failures.append(f"case_{index}_not_object")
        return
    prefix = f"case_{index}"
    gate = object_or_empty(case.get(MC202_GATE_FIELD))
    check(case.get("human_verdict") == "unverified", f"{prefix}_human_verdict_not_unverified", failures)
    check(case.get("demo_readiness") == "unverified", f"{prefix}_demo_readiness_not_unverified", failures)
    check(str(case.get("demo_worthy_reason", "")).startswith("Worth review:"), f"{prefix}_demo_worthy_reason_missing", failures)
    check(str(case.get("not_demo_worthy_reason", "")).startswith("Not demo-ready yet:"), f"{prefix}_not_demo_worthy_reason_missing", failures)
    check(case.get("evidence_role") == "listening_review_scaffold", f"{prefix}_evidence_role_mismatch", failures)
    check(case.get("quality_proof") is False, f"{prefix}_claims_quality", failures)
    check(str(case.get("candidate", "")).endswith(".wav"), f"{prefix}_candidate_not_wav", failures)
    check(len(str(case.get("candidate_sha256", ""))) == 64, f"{prefix}_candidate_sha_invalid", failures)
    check(len(str(case.get("review_sha256", ""))) == 64, f"{prefix}_review_sha_invalid", failures)
    check(gate.get("promotion_blocked_until_human_pass") is True, f"{prefix}_promotion_not_blocked", failures)
    check(gate.get("source_composed_evidence") is True, f"{prefix}_source_composed_missing", failures)
    check(gate.get("primitive_or_template_only") is False, f"{prefix}_primitive_template_leaked", failures)
    if require_review_files:
        review = Path(str(case.get("review", "")))
        prompt = report_path.parent / "reviews" / str(case.get("case_id", "")) / "prompt.md"
        check(review.is_file(), f"{prefix}_review_file_missing", failures)
        check(prompt.is_file(), f"{prefix}_prompt_file_missing", failures)
        if prompt.is_file():
            check("Demo Readiness" in prompt.read_text(), f"{prefix}_prompt_demo_readiness_missing", failures)
        if review.is_file():
            review_data = read_json_object(review)
            review_gate = object_or_empty(review_data.get(MC202_GATE_FIELD))
            label_gate = object_or_empty(object_or_empty(review_data.get("audio_judge_label")).get(MC202_GATE_FIELD))
            check(review_data.get("demo_readiness") == "unverified", f"{prefix}_review_demo_readiness_not_unverified", failures)
            check(str(review_data.get("demo_worthy_reason", "")).startswith("Worth review:"), f"{prefix}_review_demo_worthy_reason_missing", failures)
            check(str(review_data.get("not_demo_worthy_reason", "")).startswith("Not demo-ready yet:"), f"{prefix}_review_not_demo_worthy_reason_missing", failures)
            check(review_gate.get("promotion_blocked_until_human_pass") is True, f"{prefix}_review_promotion_not_blocked", failures)
            check(review_gate.get("source_composed_evidence") is True, f"{prefix}_review_source_composed_missing", failures)
            check(review_gate.get("primitive_or_template_only") is False, f"{prefix}_review_primitive_template_leaked", failures)
            check(
                label_gate.get("source_composed_evidence") == review_gate.get("source_composed_evidence"),
                f"{prefix}_audio_judge_gate_mismatch",
                failures,
            )


def run_mutation_fixtures(report: dict[str, Any], path: Path) -> None:
    fixtures = []
    mutated = json.loads(json.dumps(report))
    mutated["human_verdict"] = "pass"
    fixtures.append(("human_verdict_claim", mutated, "human_verdict_not_unverified"))

    mutated = json.loads(json.dumps(report))
    mutated[MC202_GATE_FIELD]["source_composed_case_count"] = 2
    fixtures.append(("source_composed_count_stale", mutated, "pack_gate_source_composed_count_mismatch"))

    mutated = json.loads(json.dumps(report))
    for case in mutated["cases"]:
        if case["source_family"] == "tonal_hook":
            case[MC202_GATE_FIELD]["source_composed_evidence"] = False
            case[MC202_GATE_FIELD]["primitive_or_template_only"] = True
    fixtures.append(("tonal_regressed_to_primitive", mutated, "case_1_source_composed_missing"))

    for name, fixture, expected in fixtures:
        failures = validate_report(fixture, path, require_review_files=False)
        if expected not in failures:
            raise ValueError(f"mutation {name} expected {expected}, got {failures}")


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: JSON root must be object")
    return value


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


if __name__ == "__main__":
    raise SystemExit(main())
