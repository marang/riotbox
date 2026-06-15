"""MC-202 source-composed listening-review and promotion gate helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any


MC202_GATE_FIELD = "mc202_source_composed_review_gate"
CASE_GATE_SCHEMA = "riotbox.mc202_source_composed_review_gate.v1"
PACK_GATE_SCHEMA = "riotbox.mc202_source_composed_pack_gate.v1"
MIN_MC202_TO_W30_RMS_RATIO = 0.16
MIN_PRESSURE_LOW_BAND_LIFT_RATIO = 1.50
MIN_PRESSURE_LIFT_BAR_MOVEMENT_RATIO = 1.03
MIN_SPARSE_BASS_STATIC_DISTANCE_HZ = 1.25
MIN_SPARSE_BASS_SPAN_HZ = 8.0


def case_gate(case: dict[str, Any], source_report: dict[str, Any]) -> dict[str, Any]:
    proof = source_report.get("proof") if isinstance(source_report.get("proof"), dict) else {}
    metrics = source_report.get("metrics") if isinstance(source_report.get("metrics"), dict) else {}
    mc202_metrics = metrics.get("mc202") if isinstance(metrics.get("mc202"), dict) else {}
    family = str(case["source_family"])
    family_kind = "dense_break" if family == "dense_break" else "non_dense_break"
    mc202_to_w30 = number(proof.get("mc202_to_w30_rms_ratio"))
    pressure_lift = number(proof.get("pressure_low_band_lift_ratio"))
    pressure_bar_movement = number(proof.get("pressure_lift_bar5_to_bar4_rms_ratio"))
    decision_count = number(proof.get("pressure_lift_policy_decision_count"))
    arrangement_source_derived = number(proof.get("arrangement_role_order_source_derived"))
    scripted_distance = number(proof.get("arrangement_scripted_role_distance"))
    sparse_source_derived = number(proof.get("bass_movement_source_derived"))
    sparse_static_distance = number(proof.get("sparse_bass_movement_static_distance_hz"))
    sparse_span = number(proof.get("sparse_bass_movement_frequency_span_hz"))
    common_pressure = (
        mc202_to_w30 >= MIN_MC202_TO_W30_RMS_RATIO
        and pressure_lift >= MIN_PRESSURE_LOW_BAND_LIFT_RATIO
        and pressure_bar_movement >= MIN_PRESSURE_LIFT_BAR_MOVEMENT_RATIO
        and decision_count >= 6.0
        and arrangement_source_derived >= 1.0
        and scripted_distance >= 2.0
    )
    sparse_bass_movement = (
        family == "sparse_bass_pressure"
        and sparse_source_derived >= 1.0
        and sparse_static_distance >= MIN_SPARSE_BASS_STATIC_DISTANCE_HZ
        and sparse_span >= MIN_SPARSE_BASS_SPAN_HZ
    )
    source_composed = common_pressure and (
        family != "sparse_bass_pressure" or sparse_bass_movement
    )
    failure_codes = failure_codes_for(
        family=family,
        mc202_to_w30=mc202_to_w30,
        pressure_lift=pressure_lift,
        pressure_bar_movement=pressure_bar_movement,
        decision_count=decision_count,
        arrangement_source_derived=arrangement_source_derived,
        scripted_distance=scripted_distance,
        sparse_bass_movement=sparse_bass_movement,
    )
    return {
        "schema": CASE_GATE_SCHEMA,
        "result": "pass" if source_composed else "fail",
        "source_family": family,
        "family_kind": family_kind,
        "source_composed_evidence": source_composed,
        "primitive_or_template_only": not source_composed,
        "quality_proof": False,
        "human_verdict": "unverified",
        "demo_readiness": "unverified",
        "promotion_blocked_until_human_pass": True,
        "template_only_blocks_promotion": True,
        "failure_codes": failure_codes,
        "metrics": {
            "mc202_rms": number(mc202_metrics.get("rms")),
            "mc202_low_band_rms": number(mc202_metrics.get("low_band_rms")),
            "mc202_to_w30_rms_ratio": mc202_to_w30,
            "pressure_low_band_lift_ratio": pressure_lift,
            "pressure_lift_bar5_to_bar4_rms_ratio": pressure_bar_movement,
            "pressure_lift_policy_decision_count": decision_count,
            "arrangement_role_order_source_derived": arrangement_source_derived,
            "arrangement_scripted_role_distance": scripted_distance,
            "bass_movement_source_derived": sparse_source_derived,
            "sparse_bass_movement_static_distance_hz": sparse_static_distance,
            "sparse_bass_movement_frequency_span_hz": sparse_span,
        },
    }


def pack_gate(cases: list[dict[str, Any]]) -> dict[str, Any]:
    source_cases = [
        case for case in cases
        if case.get(MC202_GATE_FIELD, {}).get("source_composed_evidence") is True
    ]
    dense_cases = [
        case for case in source_cases
        if case[MC202_GATE_FIELD].get("family_kind") == "dense_break"
    ]
    non_dense_cases = [
        case for case in source_cases
        if case[MC202_GATE_FIELD].get("family_kind") == "non_dense_break"
    ]
    unverified = all(
        case.get("human_verdict") == "unverified"
        and case.get("demo_readiness") == "unverified"
        and case.get("quality_proof") is False
        for case in cases
    )
    failure_codes = []
    if not dense_cases:
        failure_codes.append("mc202_dense_break_review_candidate_missing")
    if not non_dense_cases:
        failure_codes.append("mc202_non_dense_review_candidate_missing")
    if not unverified:
        failure_codes.append("mc202_review_pack_claims_quality_without_human_verdict")
    return {
        "schema": PACK_GATE_SCHEMA,
        "result": "pass" if not failure_codes else "fail",
        "case_count": len(cases),
        "source_composed_case_count": len(source_cases),
        "dense_break_case_count": len(dense_cases),
        "non_dense_break_case_count": len(non_dense_cases),
        "human_verdict": "unverified",
        "quality_proof": False,
        "promotion_contract": (
            "MC-202 source-composed review candidates may enter human listening "
            "queues, but demo-bank promotion requires structured human pass/weak/fail "
            "and template-only candidates must not promote."
        ),
        "failure_codes": failure_codes,
    }


def validate_promotion_gate(gate: dict[str, Any], path: Path) -> None:
    require(gate.get("schema") == CASE_GATE_SCHEMA, f"{path}: missing MC-202 source-composed review gate")
    require(
        gate.get("source_composed_evidence") is True,
        f"{path}: MC-202 source-composed evidence is required for demo-bank promotion",
    )
    require(
        gate.get("primitive_or_template_only") is False,
        f"{path}: primitive/template-only MC-202 output cannot be promoted",
    )
    require(
        gate.get("quality_proof") is False,
        f"{path}: MC-202 gate must not claim quality proof before promotion",
    )
    require(
        gate.get("template_only_blocks_promotion") is True,
        f"{path}: MC-202 template-only promotion blocker missing",
    )
    require(isinstance(gate.get("source_family"), str) and gate["source_family"], f"{path}: MC-202 gate source_family missing")


def failure_codes_for(
    *,
    family: str,
    mc202_to_w30: float,
    pressure_lift: float,
    pressure_bar_movement: float,
    decision_count: float,
    arrangement_source_derived: float,
    scripted_distance: float,
    sparse_bass_movement: bool,
) -> list[str]:
    failures = []
    if mc202_to_w30 < MIN_MC202_TO_W30_RMS_RATIO:
        failures.append("mc202_support_too_weak")
    if pressure_lift < MIN_PRESSURE_LOW_BAND_LIFT_RATIO:
        failures.append("mc202_pressure_lift_too_weak")
    if pressure_bar_movement < MIN_PRESSURE_LIFT_BAR_MOVEMENT_RATIO:
        failures.append("mc202_pressure_lift_static")
    if decision_count < 6.0:
        failures.append("mc202_pressure_policy_candidate_count_too_low")
    if arrangement_source_derived < 1.0:
        failures.append("mc202_arrangement_not_source_derived")
    if scripted_distance < 2.0:
        failures.append("mc202_arrangement_too_close_to_scripted_template")
    if family == "sparse_bass_pressure" and not sparse_bass_movement:
        failures.append("mc202_sparse_bass_movement_not_source_composed")
    return failures


def number(value: Any) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)
