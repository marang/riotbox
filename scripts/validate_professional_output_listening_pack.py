#!/usr/bin/env python3
"""Validate professional output listening-pack contracts."""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.professional_output_listening_pack.v1"
MC202_GATE_FIELD = "mc202_source_composed_review_gate"
EXPECTED_FAMILIES = ["dense_break", "sparse_bass_pressure", "tonal_hook"]
PRESENTATION_SAFETY_SCHEMA = "riotbox.audio_presentation_true_peak_safety.v1"


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
    if case.get("source_family") == "dense_break":
        validate_presentation_safety(
            object_or_empty(case.get("presentation_safety")),
            prefix,
            failures,
        )
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
            if case.get("source_family") == "dense_break":
                review_safety = object_or_empty(review_data.get("presentation_safety"))
                validate_presentation_safety(
                    review_safety,
                    f"{prefix}_review",
                    failures,
                )
                check(
                    review_safety == case.get("presentation_safety"),
                    f"{prefix}_review_presentation_safety_mismatch",
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

    mutated = json.loads(json.dumps(report))
    dense = next(case for case in mutated["cases"] if case["source_family"] == "dense_break")
    dense["presentation_safety"]["result"] = "fail"
    dense["presentation_safety"]["maximum_post_gain_true_peak_dbtp"] = 0.5
    fixtures.append(
        ("dense_true_peak_unsafe", mutated, "case_0_presentation_safety_not_passed")
    )

    mutated = json.loads(json.dumps(report))
    dense = next(case for case in mutated["cases"] if case["source_family"] == "dense_break")
    dense["presentation_safety"]["coverage"].pop("03_dropout_stutter.wav")
    fixtures.append(
        (
            "dense_true_peak_incomplete_coverage",
            mutated,
            "case_0_presentation_safety_coverage_mismatch",
        )
    )

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


def validate_presentation_safety(
    safety: dict[str, Any],
    prefix: str,
    failures: list[str],
) -> None:
    check(
        safety.get("schema") == PRESENTATION_SAFETY_SCHEMA,
        f"{prefix}_presentation_safety_schema_mismatch",
        failures,
    )
    check(
        safety.get("result") == "pass",
        f"{prefix}_presentation_safety_not_passed",
        failures,
    )
    check(
        safety.get("estimator") == "conservative_four_x_bandlimited_fft_v1",
        f"{prefix}_presentation_safety_estimator_mismatch",
        failures,
    )
    check(
        safety.get("oversample_factor") == 4,
        f"{prefix}_presentation_safety_oversample_mismatch",
        failures,
    )
    check(
        safety.get("schema_version") == 1,
        f"{prefix}_presentation_safety_version_mismatch",
        failures,
    )
    maximum = finite_number(safety.get("max_allowed_true_peak_dbtp"))
    target = finite_number(safety.get("normalization_target_true_peak_dbtp"))
    measured = finite_number(safety.get("maximum_post_gain_true_peak_dbtp"))
    gain = finite_number(safety.get("uniform_gain"))
    gain_db = finite_number(safety.get("uniform_gain_db"))
    check(maximum == -1.0, f"{prefix}_presentation_safety_limit_mismatch", failures)
    check(target == -1.2, f"{prefix}_presentation_safety_target_mismatch", failures)
    check(
        measured is not None and maximum is not None and measured <= maximum,
        f"{prefix}_presentation_true_peak_unsafe",
        failures,
    )
    check(
        gain is not None and 0.0 < gain <= 1.0,
        f"{prefix}_presentation_safety_gain_invalid",
        failures,
    )
    check(
        gain is not None
        and gain_db is not None
        and math.isclose(gain_db, 20.0 * math.log10(gain), abs_tol=1e-9),
        f"{prefix}_presentation_safety_gain_db_mismatch",
        failures,
    )
    expected_coverage = {
        "00_source_window.wav": "source_window",
        "01_chop_hook.wav": "chop_hook",
        "02_pressure_lift.wav": "pressure_lift",
        "03_dropout_stutter.wav": "dropout_stutter",
        "04_restore_hit.wav": "restore_hit",
        "05_rebuild_only_performance.wav": "rebuild_only_performance",
    }
    coverage = object_or_empty(safety.get("coverage"))
    check(
        coverage == expected_coverage,
        f"{prefix}_presentation_safety_coverage_mismatch",
        failures,
    )
    pre_gain = object_or_empty(safety.get("pre_gain_true_peak_dbtp"))
    post_gain = object_or_empty(safety.get("post_gain_true_peak_dbtp"))
    measurement_keys = set(expected_coverage.values()) | {"source_layered_reference"}
    check(
        set(pre_gain) == measurement_keys and set(post_gain) == measurement_keys,
        f"{prefix}_presentation_safety_measurements_incomplete",
        failures,
    )
    for name in sorted(measurement_keys):
        pre = finite_number(pre_gain.get(name))
        post = finite_number(post_gain.get(name))
        check(
            pre is not None
            and post is not None
            and gain_db is not None
            and math.isclose(post, pre + gain_db, abs_tol=1e-8),
            f"{prefix}_presentation_safety_{name}_gain_mismatch",
            failures,
        )
        check(
            post is not None and maximum is not None and post <= maximum,
            f"{prefix}_presentation_safety_{name}_unsafe",
            failures,
        )
    check(safety.get("quality_proof") is False, f"{prefix}_presentation_claims_quality", failures)
    check(
        safety.get("human_verdict") == "unverified",
        f"{prefix}_presentation_claims_human_verdict",
        failures,
    )


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def finite_number(value: Any) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    result = float(value)
    return result if math.isfinite(result) else None


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


if __name__ == "__main__":
    raise SystemExit(main())
