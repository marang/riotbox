"""Fixture contract checks for release-demo human-review queue reports."""

from __future__ import annotations

import copy
from typing import Any, Callable


ValidateReport = Callable[[dict[str, Any]], list[str]]


def validate_mutation_fixtures(
    report: dict[str, Any],
    validate_report: ValidateReport,
) -> list[str]:
    failures = validate_fixture_contract(report)
    mutations = [
        (
            "quality_claim",
            "claims_quality",
            lambda candidate: candidate.update({"quality_claim": True}),
        ),
        (
            "missing_source_character",
            "source_character_missing",
            lambda candidate: candidate.update({"source_character": ""}),
        ),
        (
            "stale_verdict_state",
            "stale_verdict_state",
            lambda candidate: candidate["required_verdict_path"].update(
                {"current_state": "human_verdict:pass/demo_readiness:demo_ready"}
            ),
        ),
        (
            "incomplete_listening_questions",
            "required_listening_questions_invalid",
            lambda candidate: candidate["required_listening_questions"].pop(2),
        ),
    ]
    for name, expected, mutate in mutations:
        mutated = copy.deepcopy(report)
        queue = mutated.get("review_queue")
        if not isinstance(queue, list) or not queue or not isinstance(queue[0], dict):
            failures.append(f"{name}_fixture_queue_missing")
            continue
        mutate(queue[0])
        mutation_failures = validate_report(mutated)
        if not any(expected in failure for failure in mutation_failures):
            failures.append(f"{name}_fixture_did_not_fail_with_{expected}")
    return failures


def validate_fixture_contract(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    queue = report.get("review_queue")
    check(isinstance(queue, list) and len(queue) >= 5, "fixture_review_queue_too_small", failures)
    entries = {
        str(entry.get("entry_id")): entry
        for entry in queue
        if isinstance(entry, dict)
    } if isinstance(queue, list) else {}
    check_candidate(
        entries,
        "pad-noise-fadapad-unverified-candidate",
        priority="high",
        source_family="pad_noise",
        source_character_any=["pad/noise", "texture"],
        strongest_any=["texture", "gate", "silence"],
        demo_reason_any=["playable"],
        not_ready_any=["human_verdict is unverified"],
        blockers=["source_family_human_verdict_missing"],
        failures=failures,
    )
    check_candidate(
        entries,
        "bad-timing-beat20-unverified-candidate",
        priority="high",
        source_family="bad_timing",
        source_character_any=["Beat20"],
        strongest_any=[],
        demo_reason_any=[],
        not_ready_any=[],
        blockers=["source_family_demo_ready_human_pass_missing"],
        failures=failures,
    )
    check_candidate(
        entries,
        "weak-source-beat20-rejection-unverified-candidate",
        priority="high",
        source_family="weak_source",
        source_character_any=["source-character failure"],
        strongest_any=[],
        demo_reason_any=[],
        not_ready_any=["quality claims are blocked"],
        blockers=[],
        failures=failures,
    )
    check_candidate(
        entries,
        "sparse-bass-pressure-updated-unverified-candidate",
        priority="medium",
        source_family="sparse_drums",
        source_character_any=[],
        strongest_any=["bass", "kick"],
        demo_reason_any=[],
        not_ready_any=[],
        blockers=["source_family_demo_ready_human_pass_missing"],
        failures=failures,
    )
    for entry_id, entry in entries.items():
        check(entry.get("human_verdict") == "unverified", f"{entry_id}_fixture_not_unverified", failures)
        check(entry.get("demo_readiness") == "unverified", f"{entry_id}_fixture_not_unverified_demo", failures)
        check(entry.get("quality_claim") is False, f"{entry_id}_fixture_claims_quality", failures)
        check(
            isinstance(entry.get("demo_worthy_reason"), str)
            and len(entry["demo_worthy_reason"]) > 20,
            f"{entry_id}_fixture_demo_reason_too_short",
            failures,
        )
        check(
            text_contains_any(entry.get("not_demo_ready_reason"), ["unverified"]),
            f"{entry_id}_fixture_not_ready_reason_missing_unverified",
            failures,
        )
        verdict_path = entry.get("required_verdict_path")
        check(
            isinstance(verdict_path, dict)
            and text_contains_any(verdict_path.get("release_ready_blocker"), ["must not claim quality"]),
            f"{entry_id}_fixture_release_blocker_missing_quality_boundary",
            failures,
        )
    gaps = object_list(report.get("source_family_coverage"), "family_gaps")
    check_gap(gaps, "weak_source", missing_human=True, missing_demo_ready=True, failures=failures)
    check_gap(gaps, "sparse_drums", missing_human=False, missing_demo_ready=True, failures=failures)
    actions = object_list(report, "next_actions")
    check(
        any(action.get("category") == "listening_pack" for action in actions),
        "fixture_listening_pack_action_missing",
        failures,
    )
    return failures


def check_candidate(
    entries: dict[str, dict[str, Any]],
    entry_id: str,
    *,
    priority: str,
    source_family: str,
    source_character_any: list[str],
    strongest_any: list[str],
    demo_reason_any: list[str],
    not_ready_any: list[str],
    blockers: list[str],
    failures: list[str],
) -> None:
    entry = entries.get(entry_id)
    if entry is None:
        failures.append(f"{entry_id}_fixture_missing")
        return
    check(entry.get("review_priority") == priority, f"{entry_id}_fixture_priority_mismatch", failures)
    check(entry.get("source_family") == source_family, f"{entry_id}_fixture_source_family_mismatch", failures)
    if source_character_any:
        check(
            text_contains_any(entry.get("source_character"), source_character_any),
            f"{entry_id}_fixture_source_character_mismatch",
            failures,
        )
    if strongest_any:
        check(
            text_contains_any(entry.get("strongest_audible_element"), strongest_any),
            f"{entry_id}_fixture_strongest_element_mismatch",
            failures,
        )
    if demo_reason_any:
        check(
            text_contains_any(entry.get("demo_worthy_reason"), demo_reason_any),
            f"{entry_id}_fixture_demo_reason_mismatch",
            failures,
        )
    if not_ready_any:
        check(
            text_contains_any(entry.get("not_demo_ready_reason"), not_ready_any),
            f"{entry_id}_fixture_not_ready_reason_mismatch",
            failures,
        )
    entry_blockers = set(str(blocker) for blocker in entry.get("review_blockers", []))
    for blocker in blockers:
        check(blocker in entry_blockers, f"{entry_id}_fixture_{blocker}_missing", failures)
    verdict_path = entry.get("required_verdict_path")
    check(
        isinstance(verdict_path, dict)
        and verdict_path.get("current_state") == "human_verdict:unverified/demo_readiness:unverified",
        f"{entry_id}_fixture_current_state_mismatch",
        failures,
    )
    questions = entry.get("required_listening_questions")
    check(isinstance(questions, list) and len(questions) == 6, f"{entry_id}_fixture_questions_missing", failures)


def check_gap(
    gaps: list[dict[str, Any]],
    family: str,
    *,
    missing_human: bool,
    missing_demo_ready: bool,
    failures: list[str],
) -> None:
    gap = next((item for item in gaps if item.get("source_family") == family), None)
    if gap is None:
        failures.append(f"{family}_fixture_gap_missing")
        return
    check(gap.get("missing_human_verdict") is missing_human, f"{family}_fixture_missing_human_mismatch", failures)
    check(
        gap.get("missing_demo_ready_human_pass") is missing_demo_ready,
        f"{family}_fixture_missing_demo_ready_mismatch",
        failures,
    )


def object_list(data: Any, field: str) -> list[dict[str, Any]]:
    if not isinstance(data, dict):
        return []
    value = data.get(field)
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def text_contains_any(value: Any, needles: list[str]) -> bool:
    text = str(value)
    lowered = text.lower()
    return any(needle in text or needle.lower() in lowered for needle in needles)


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)
