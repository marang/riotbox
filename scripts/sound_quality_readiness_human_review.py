#!/usr/bin/env python3
"""Human-review queue helpers for the P023 sound-quality readiness report."""

from __future__ import annotations

from collections import Counter
from pathlib import Path
from typing import Any


HUMAN_REVIEW_QUEUE_SCHEMA = "riotbox.release_demo_human_review_queue.v1"
DEFAULT_HUMAN_REVIEW_QUEUE = Path(
    "artifacts/audio_qa/local-release-demo-human-review-queue/release-demo-human-review-queue.json"
)
REQUIRED_REVIEW_BLOCKERS = {
    "human_verdict_unverified",
    "demo_readiness_unverified",
    "quality_claim_blocked",
}
REQUIRED_REVIEW_SOURCE_FAMILIES = {"pad_noise", "weak_source", "bad_timing"}
REQUIRED_VERDICT_CURRENT_STATE = "human_verdict:unverified/demo_readiness:unverified"


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
    priority_counts = Counter(
        str(entry.get("review_priority")) for entry in queue if isinstance(entry, dict)
    )
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


def validate_human_review_queue_section(
    review_queue: dict[str, Any],
    blockers: list[Any],
    failures: list[str],
) -> None:
    review_queue_available = review_queue.get("available")
    review_candidates = review_queue.get("candidates", [])
    review_count = review_queue.get("review_queue_count")

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
            and REQUIRED_REVIEW_SOURCE_FAMILIES.issubset(
                set(str(item) for item in review_queue.get("source_families", []))
            ),
            "human_review_queue_source_families_incomplete",
            failures,
        )
        review_blockers = set(str(item) for item in review_queue.get("review_blockers", []))
        check(
            REQUIRED_REVIEW_BLOCKERS.issubset(review_blockers),
            "human_review_queue_review_blockers_missing",
            failures,
        )
        if isinstance(review_candidates, list):
            for index, candidate in enumerate(review_candidates):
                validate_review_queue_candidate(candidate, index, failures)
    else:
        check(
            any(
                isinstance(blocker, dict)
                and blocker.get("code") == "human_review_queue_not_available"
                for blocker in blockers
            ),
            "missing_human_review_queue_blocker",
            failures,
        )


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
        and REQUIRED_REVIEW_BLOCKERS.issubset(set(str(item) for item in blockers)),
        f"human_review_queue_candidate_{index}_review_blockers_missing",
        failures,
    )
    check(
        candidate.get("required_verdict_current_state") == REQUIRED_VERDICT_CURRENT_STATE,
        f"human_review_queue_candidate_{index}_stale_verdict_state",
        failures,
    )
    check(
        isinstance(candidate.get("required_listening_questions"), list)
        and candidate.get("required_listening_question_count")
        == len(candidate["required_listening_questions"])
        and candidate.get("required_listening_question_count") >= 6,
        f"human_review_queue_candidate_{index}_listening_questions_incomplete",
        failures,
    )


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def queue_string_list(data: dict[str, Any], field: str, path: Path, index: int) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list), f"{path}: review_queue[{index}].{field} must be array")
    strings = [str(item) for item in value if isinstance(item, str) and item]
    require(
        len(strings) == len(value),
        f"{path}: review_queue[{index}].{field} values must be strings",
    )
    return strings


def required_queue_string(data: dict[str, Any], field: str, path: Path, index: int) -> str:
    value = data.get(field)
    require(
        isinstance(value, str) and bool(value),
        f"{path}: review_queue[{index}].{field} must be non-empty string",
    )
    return value


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
