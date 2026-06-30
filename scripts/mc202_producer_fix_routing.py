"""MC-202 producer-grade fix routing helpers."""

from __future__ import annotations

from typing import Any


MC202_FIX_CATEGORY_ORDER = [
    "bass_movement",
    "answer_bite",
    "hook_restraint",
    "source_selection",
    "mix_bus",
    "destructive_articulation",
    "human_listening",
]


def route_candidate(candidate: dict[str, Any]) -> dict[str, Any]:
    source_family = str(candidate["source_family"])
    metrics = object_or_empty(candidate.get("metrics"))
    categories: list[str] = []
    reasons: dict[str, list[str]] = {category: [] for category in MC202_FIX_CATEGORY_ORDER}

    if candidate["human_verdict"] == "unverified" or candidate["demo_readiness"] == "unverified":
        add_fix_category(
            categories,
            reasons,
            "human_listening",
            "candidate is still unverified by structured listening",
        )

    if source_family == "sparse_bass_pressure":
        if number(metrics.get("sparse_bass_movement_static_distance_hz")) < 2.25:
            add_fix_category(
                categories,
                reasons,
                "bass_movement",
                "sparse bass movement stays too close to the fixed contour",
            )
        if number(metrics.get("sparse_bass_movement_frequency_span_hz")) < 12.0:
            add_fix_category(
                categories,
                reasons,
                "bass_movement",
                "sparse bass movement is close to the minimum span",
            )
    elif source_family == "tonal_hook":
        add_fix_category(
            categories,
            reasons,
            "hook_restraint",
            "tonal source should be judged first on hook-safe restraint and stab answer",
        )
        if number(metrics.get("mc202_to_w30_rms_ratio")) < 0.20:
            add_fix_category(
                categories,
                reasons,
                "mix_bus",
                "MC-202 support sits close to the W-30 balance floor",
            )
        if number(metrics.get("pressure_low_band_lift_ratio")) < 2.20:
            add_fix_category(
                categories,
                reasons,
                "hook_restraint",
                "tonal pressure lift is close to the review floor",
            )
    elif source_family == "dense_break":
        if dense_answer_bite_below_floor(metrics):
            add_fix_category(
                categories,
                reasons,
                "answer_bite",
                "break pressure-answer bite is below the producer floor",
            )
        if number(metrics.get("pressure_lift_bar5_to_bar4_rms_ratio")) < 1.10:
            add_fix_category(
                categories,
                reasons,
                "destructive_articulation",
                "pressure lift is close to the live-gesture impact floor",
            )
    elif source_family == "non_dense_break":
        if answer_role_below_floor(metrics, min_scripted_distance=2.0):
            add_fix_category(
                categories,
                reasons,
                "answer_bite",
                "non-dense break answer role is below the producer floor",
            )
        if number(metrics.get("pressure_lift_bar5_to_bar4_rms_ratio")) < 1.10:
            add_fix_category(
                categories,
                reasons,
                "destructive_articulation",
                "pressure lift is close to the live-gesture impact floor",
            )
    else:
        add_fix_category(
            categories,
            reasons,
            "source_selection",
            "source family is unsupported for MC-202 producer routing",
        )

    if candidate.get("source_composed_evidence") is not True:
        add_fix_category(
            categories,
            reasons,
            "source_selection",
            "candidate lacks source-composed evidence",
        )
    if candidate.get("primitive_or_template_only") is True:
        add_fix_category(
            categories,
            reasons,
            "source_selection",
            "primitive/template-only MC-202 output must become implementation work",
        )

    gate_failures = [str(code) for code in candidate.get("gate_failure_codes", [])]
    role_failures = [
        str(code)
        for code in object_or_empty(candidate.get("mc202_role_evidence")).get("failure_codes", [])
    ]
    for code in gate_failures + role_failures:
        route_failure_code(code, categories, reasons)

    primary = next(
        (category for category in MC202_FIX_CATEGORY_ORDER if category in categories),
        "human_listening",
    )
    return {
        "proposed_next_fix_category": primary,
        "proposed_fix_categories": categories,
        "main_weakness": fix_weakness_label(primary),
        "musician_fix_reason": fix_musician_payoff(primary),
        "artifact_to_hear": candidate["candidate"],
        "routing_reasons": {
            category: reasons[category]
            for category in categories
            if reasons[category]
        },
        "matched_known_routing_signal": bool(categories),
        "evidence_role": "mc202_producer_fix_route",
        "quality_proof": False,
        "automated_musical_approval": False,
    }


def dense_answer_bite_below_floor(metrics: dict[str, Any]) -> bool:
    return (
        number(metrics.get("dense_answer_bite_source_derived")) < 1.0
        or number(metrics.get("dense_answer_bite_scripted_role_distance")) < 3.0
        or number(metrics.get("dense_answer_bite_stab_score")) < 1.65
        or number(metrics.get("dense_answer_bite_stab_margin")) < 0.15
        or number(metrics.get("dense_answer_bite_pressure_snap_ratio")) < 1.06
        or number(metrics.get("dense_answer_bite_score")) < 1.0
    )


def answer_role_below_floor(
    metrics: dict[str, Any],
    *,
    min_scripted_distance: float,
) -> bool:
    return (
        number(metrics.get("pressure_lift_policy_decision_count")) < 6.0
        or number(metrics.get("arrangement_role_order_source_derived")) < 1.0
        or number(metrics.get("arrangement_scripted_role_distance")) < min_scripted_distance
        or number(metrics.get("mc202_to_w30_rms_ratio")) < 0.16
        or number(metrics.get("pressure_low_band_lift_ratio")) < 1.50
    )


def build_fix_candidates(review_candidates: list[dict[str, Any]]) -> list[dict[str, Any]]:
    fix_candidates = []
    for category in MC202_FIX_CATEGORY_ORDER:
        category_cases = [
            candidate
            for candidate in review_candidates
            if category in candidate["mc202_producer_fix_route"]["proposed_fix_categories"]
        ]
        if not category_cases:
            continue
        primary_cases = [
            candidate
            for candidate in category_cases
            if candidate["mc202_producer_fix_route"]["proposed_next_fix_category"] == category
        ]
        score = len(primary_cases) * 2 + len(category_cases)
        fix_candidates.append(
            {
                "candidate_id": f"p023_mc202_fix_{category}",
                "category": category,
                "score": score,
                "primary_case_count": len(primary_cases),
                "case_count": len(category_cases),
                "case_ids": [str(candidate["case_id"]) for candidate in category_cases],
                "primary_case_ids": [str(candidate["case_id"]) for candidate in primary_cases],
                "source_families": sorted(
                    {str(candidate["source_family"]) for candidate in category_cases}
                ),
                "artifact_refs": sorted(
                    {str(candidate["candidate"]) for candidate in category_cases}
                ),
                "software_next_step": fix_software_next_step(category),
                "musician_payoff": fix_musician_payoff(category),
                "routing_reasons": {
                    str(candidate["case_id"]): candidate["mc202_producer_fix_route"][
                        "routing_reasons"
                    ].get(category, [])
                    for candidate in category_cases
                    if candidate["mc202_producer_fix_route"]["routing_reasons"].get(category)
                },
                "evidence_role": "mc202_producer_fix_candidate",
                "quality_proof": False,
                "automated_musical_approval": False,
            }
        )
    return sorted(
        fix_candidates,
        key=lambda candidate: (
            -candidate["score"],
            MC202_FIX_CATEGORY_ORDER.index(candidate["category"]),
        ),
    )


def build_fix_summary(
    review_candidates: list[dict[str, Any]], fix_candidates: list[dict[str, Any]]
) -> dict[str, Any]:
    case_counts = {category: 0 for category in MC202_FIX_CATEGORY_ORDER}
    primary_counts = {category: 0 for category in MC202_FIX_CATEGORY_ORDER}
    for candidate in review_candidates:
        route = candidate["mc202_producer_fix_route"]
        for category in route["proposed_fix_categories"]:
            case_counts[category] += 1
        primary_counts[str(route["proposed_next_fix_category"])] += 1
    categories = [str(candidate["category"]) for candidate in fix_candidates]
    top = fix_candidates[0] if fix_candidates else None
    return {
        "candidate_count": len(fix_candidates),
        "categories": categories,
        "recurring_fix_categories": [
            category
            for category in MC202_FIX_CATEGORY_ORDER
            if case_counts[category] >= 2
        ],
        "case_ref_count": sum(len(candidate["case_ids"]) for candidate in fix_candidates),
        "primary_case_ref_count": sum(
            len(candidate["primary_case_ids"]) for candidate in fix_candidates
        ),
        "case_counts_by_category": {
            category: case_counts[category]
            for category in categories
        },
        "primary_case_counts_by_category": {
            category: primary_counts[category]
            for category in categories
        },
        "top_candidate_category": str(top["category"]) if top else "none",
        "quality_proof": False,
        "automated_musical_approval": False,
    }


def validate_candidate_fix_route(
    candidate: dict[str, Any],
    prefix: str,
    failures: list[str],
) -> None:
    route = object_or_empty(candidate.get("mc202_producer_fix_route"))
    categories = string_list(route.get("proposed_fix_categories"))
    primary = str(route.get("proposed_next_fix_category", ""))
    check(primary in MC202_FIX_CATEGORY_ORDER, f"{prefix}_mc202_fix_primary_invalid", failures)
    check(categories, f"{prefix}_mc202_fix_categories_missing", failures)
    check(primary in categories, f"{prefix}_mc202_fix_primary_not_listed", failures)
    for category in categories:
        check(category in MC202_FIX_CATEGORY_ORDER, f"{prefix}_mc202_fix_category_invalid", failures)
    check(
        route.get("artifact_to_hear") == candidate.get("candidate"),
        f"{prefix}_mc202_fix_artifact_mismatch",
        failures,
    )
    check(
        route.get("matched_known_routing_signal") is True,
        f"{prefix}_mc202_fix_no_known_signal",
        failures,
    )
    check(
        route.get("evidence_role") == "mc202_producer_fix_route",
        f"{prefix}_mc202_fix_evidence_role_invalid",
        failures,
    )
    check(route.get("quality_proof") is False, f"{prefix}_mc202_fix_claims_quality", failures)
    check(
        route.get("automated_musical_approval") is False,
        f"{prefix}_mc202_fix_claims_automated_approval",
        failures,
    )
    check(
        isinstance(route.get("musician_fix_reason"), str) and bool(route["musician_fix_reason"]),
        f"{prefix}_mc202_fix_musician_reason_missing",
        failures,
    )
    check(
        isinstance(route.get("main_weakness"), str) and bool(route["main_weakness"]),
        f"{prefix}_mc202_fix_main_weakness_missing",
        failures,
    )
    reasons = object_or_empty(route.get("routing_reasons"))
    check(
        all(category in reasons and reasons[category] for category in categories),
        f"{prefix}_mc202_fix_routing_reasons_missing",
        failures,
    )


def validate_fix_candidates(
    fix_candidates: list[Any],
    review_candidates: list[Any],
    summary: dict[str, Any],
    failures: list[str],
) -> None:
    review_ids = {
        str(candidate.get("case_id"))
        for candidate in review_candidates
        if isinstance(candidate, dict)
    }
    categories: list[str] = []
    category_counts = {category: 0 for category in MC202_FIX_CATEGORY_ORDER}
    primary_counts = {category: 0 for category in MC202_FIX_CATEGORY_ORDER}
    for candidate in review_candidates:
        if not isinstance(candidate, dict):
            continue
        route = object_or_empty(candidate.get("mc202_producer_fix_route"))
        for category in string_list(route.get("proposed_fix_categories")):
            if category in category_counts:
                category_counts[category] += 1
        primary = str(route.get("proposed_next_fix_category", ""))
        if primary in primary_counts:
            primary_counts[primary] += 1

    for index, candidate in enumerate(fix_candidates):
        if not isinstance(candidate, dict):
            failures.append(f"mc202_fix_candidate_{index}_not_object")
            continue
        prefix = f"mc202_fix_candidate_{index}"
        category = str(candidate.get("category", ""))
        categories.append(category)
        check(category in MC202_FIX_CATEGORY_ORDER, f"{prefix}_category_invalid", failures)
        check(
            candidate.get("candidate_id") == f"p023_mc202_fix_{category}",
            f"{prefix}_candidate_id_invalid",
            failures,
        )
        case_ids = string_list(candidate.get("case_ids"))
        primary_case_ids = string_list(candidate.get("primary_case_ids"))
        check(case_ids, f"{prefix}_case_ids_missing", failures)
        check(all(case_id in review_ids for case_id in case_ids), f"{prefix}_unknown_case", failures)
        check(
            all(case_id in case_ids for case_id in primary_case_ids),
            f"{prefix}_primary_case_not_in_cases",
            failures,
        )
        check(
            candidate.get("case_count") == len(case_ids),
            f"{prefix}_case_count_mismatch",
            failures,
        )
        check(
            candidate.get("primary_case_count") == len(primary_case_ids),
            f"{prefix}_primary_case_count_mismatch",
            failures,
        )
        check(string_list(candidate.get("artifact_refs")), f"{prefix}_artifact_refs_missing", failures)
        check(
            isinstance(candidate.get("software_next_step"), str) and bool(candidate["software_next_step"]),
            f"{prefix}_software_next_step_missing",
            failures,
        )
        check(
            isinstance(candidate.get("musician_payoff"), str) and bool(candidate["musician_payoff"]),
            f"{prefix}_musician_payoff_missing",
            failures,
        )
        check(
            candidate.get("evidence_role") == "mc202_producer_fix_candidate",
            f"{prefix}_evidence_role_invalid",
            failures,
        )
        check(candidate.get("quality_proof") is False, f"{prefix}_claims_quality", failures)
        check(
            candidate.get("automated_musical_approval") is False,
            f"{prefix}_claims_automated_approval",
            failures,
        )

    check(
        summary.get("candidate_count") == len(fix_candidates),
        "mc202_fix_summary_candidate_count_stale",
        failures,
    )
    check(summary.get("categories") == categories, "mc202_fix_summary_categories_stale", failures)
    check(
        summary.get("case_ref_count")
        == sum(
            len(string_list(candidate.get("case_ids")))
            for candidate in fix_candidates
            if isinstance(candidate, dict)
        ),
        "mc202_fix_summary_case_ref_count_stale",
        failures,
    )
    check(
        summary.get("primary_case_ref_count")
        == sum(
            len(string_list(candidate.get("primary_case_ids")))
            for candidate in fix_candidates
            if isinstance(candidate, dict)
        ),
        "mc202_fix_summary_primary_ref_count_stale",
        failures,
    )
    case_counts = object_or_empty(summary.get("case_counts_by_category"))
    primary_summary = object_or_empty(summary.get("primary_case_counts_by_category"))
    for category in categories:
        check(
            case_counts.get(category) == category_counts.get(category),
            "mc202_fix_summary_case_counts_stale",
            failures,
        )
        check(
            primary_summary.get(category) == primary_counts.get(category),
            "mc202_fix_summary_primary_counts_stale",
            failures,
        )
    if fix_candidates:
        check(
            summary.get("top_candidate_category") == fix_candidates[0].get("category"),
            "mc202_fix_summary_top_candidate_stale",
            failures,
        )
    check(summary.get("quality_proof") is False, "mc202_fix_summary_claims_quality", failures)
    check(
        summary.get("automated_musical_approval") is False,
        "mc202_fix_summary_claims_automated_approval",
        failures,
    )


def add_fix_category(
    categories: list[str], reasons: dict[str, list[str]], category: str, reason: str
) -> None:
    if category not in MC202_FIX_CATEGORY_ORDER:
        raise ValueError(f"unsupported MC-202 fix category: {category}")
    if category not in categories:
        categories.append(category)
    reasons[category].append(reason)


def route_failure_code(
    code: str,
    categories: list[str],
    reasons: dict[str, list[str]],
) -> None:
    normalized = code.lower()
    if "bass" in normalized or "low_band" in normalized or "pressure_lift" in normalized:
        add_fix_category(categories, reasons, "bass_movement", f"{code}: bass/pressure failure")
    elif "answer" in normalized or "scripted" in normalized:
        add_fix_category(categories, reasons, "answer_bite", f"{code}: answer movement failure")
    elif "hook" in normalized or "tonal" in normalized:
        add_fix_category(categories, reasons, "hook_restraint", f"{code}: hook-restraint failure")
    elif "mix" in normalized or "w30" in normalized:
        add_fix_category(categories, reasons, "mix_bus", f"{code}: mix-bus failure")
    elif "source" in normalized or "template" in normalized or "primitive" in normalized:
        add_fix_category(categories, reasons, "source_selection", f"{code}: source evidence failure")
    else:
        add_fix_category(
            categories,
            reasons,
            "destructive_articulation",
            f"{code}: articulation/impact failure",
        )


def fix_weakness_label(category: str) -> str:
    return {
        "bass_movement": "bass pressure or low-end motion may need more authority",
        "answer_bite": "answer phrase may need sharper bite or less template feel",
        "hook_restraint": "hook-restraint answer may need better stay-out/stab judgement",
        "source_selection": "source evidence or source family may not justify the phrase",
        "mix_bus": "mix balance may bury the MC-202 role or mask the source",
        "destructive_articulation": "pressure lift or articulation may not change the room",
        "human_listening": "structured listening verdict is still missing",
    }[category]


def fix_software_next_step(category: str) -> str:
    return {
        "bass_movement": "Tune MC-202 low-band phrase movement and sparse pressure thresholds for the routed cases.",
        "answer_bite": "Tune answer-step placement, stab bite, gate snap, and scripted-distance rejection.",
        "hook_restraint": "Tune hook-restraint stay-out/stab-answer scoring so tonal sources keep their hook clear.",
        "source_selection": "Tighten source-family/source-evidence gating before promoting this MC-202 candidate.",
        "mix_bus": "Rebalance generated-support and source-first MC-202 contribution for the routed cases.",
        "destructive_articulation": "Strengthen pressure-lift articulation and live-gesture contrast around the MC-202 answer.",
        "human_listening": "Record structured listening verdicts against the exact candidate WAVs and review prompts.",
    }[category]


def fix_musician_payoff(category: str) -> str:
    return {
        "bass_movement": "Bass pressure should feel like it pushes the room instead of tracing a polite line.",
        "answer_bite": "The answer should cut back at the source with a memorable stab or shove.",
        "hook_restraint": "The MC-202 should leave the hook intact while adding tension in the gaps.",
        "source_selection": "The part should sound like it belongs to the chosen source, not a generic patch.",
        "mix_bus": "The musician should hear the MC-202 role without losing the source character.",
        "destructive_articulation": "The lift, cut, or answer should create a live-performance moment.",
        "human_listening": "A reviewer can decide from the actual WAV whether this is keeper material.",
    }[category]


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if isinstance(item, str)]


def number(value: Any) -> float:
    return float(value) if isinstance(value, (int, float)) else 0.0


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)
