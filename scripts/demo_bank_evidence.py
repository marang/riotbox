#!/usr/bin/env python3
"""Shared fixture/live evidence boundary for P023 demo-bank consumers."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Any

from degraded_product_review import (
    SCHEMA as DEGRADED_PRODUCT_REVIEW_SCHEMA,
    read_json_object as read_degraded_product_review,
    review_is_human_pass as degraded_product_review_is_human_pass,
    validate_artifact_identity as validate_degraded_product_artifact_identity,
)


LIVE_READINESS = "live_readiness"
FIXTURE_CALIBRATION = "fixture_calibration"
EVIDENCE_MODES = {LIVE_READINESS, FIXTURE_CALIBRATION}
HUMAN_REVIEW_EVIDENCE_SCHEMA = "riotbox.demo_bank_human_review_evidence.v1"
DEGRADED_OR_REJECT_EVIDENCE_SCHEMA = (
    "riotbox.demo_bank_degraded_or_reject_review_evidence.v1"
)
NEGATIVE_SUCCESS_FAMILIES = {"weak_source", "bad_timing"}
DUAL_PATH_SUCCESS_FAMILIES = {"pad_noise"}
DEMO_READY_SUCCESS = "demo_ready_human_pass"
DEGRADED_SUCCESS = "reviewed_degraded_or_reject"
DUAL_PATH_SUCCESS = "demo_ready_or_reviewed_degraded_or_reject"
LIVE_REVIEW_BANK_ROLE = "live_review"
HEX_64_RE = re.compile(r"^[0-9a-f]{64}$")


def success_requirement_for_family(source_family: str) -> str:
    if source_family in NEGATIVE_SUCCESS_FAMILIES:
        return DEGRADED_SUCCESS
    if source_family in DUAL_PATH_SUCCESS_FAMILIES:
        return DUAL_PATH_SUCCESS
    return DEMO_READY_SUCCESS


def resolve_demo_bank_path(
    evidence_mode: str,
    explicit_path: Path | None,
    fixture_default: Path,
) -> Path | None:
    if evidence_mode not in EVIDENCE_MODES:
        raise ValueError(f"unknown demo-bank evidence mode: {evidence_mode}")
    if explicit_path is not None:
        return explicit_path
    if evidence_mode == FIXTURE_CALIBRATION:
        return fixture_default
    return None


def human_verdict_is_eligible(entry: dict[str, Any], evidence_mode: str) -> bool:
    if evidence_mode == FIXTURE_CALIBRATION:
        return entry.get("human_verdict") in {"pass", "weak", "fail"}
    if entry.get("human_verdict") not in {"pass", "weak", "fail"}:
        return False
    evidence = entry.get("human_review_evidence")
    if not isinstance(evidence, dict):
        return False
    reviewer = str(evidence.get("reviewer") or "").strip()
    if (
        not reviewer
        or evidence.get("reviewer_kind") != "human"
        or reviewer_is_fixture(reviewer)
    ):
        return False
    review_path = str(evidence.get("review_path") or "").strip()
    review_sha256 = str(evidence.get("review_sha256") or "")
    if not (
        evidence.get("schema") == HUMAN_REVIEW_EVIDENCE_SCHEMA
        and review_path.endswith(".json")
        and HEX_64_RE.fullmatch(review_sha256) is not None
    ):
        return False
    path = Path(review_path)
    return path.is_file() and sha256_file(path) == review_sha256


def reviewer_is_fixture(reviewer: str) -> bool:
    normalized = reviewer.casefold().replace("_", "-")
    return any(
        marker in normalized
        for marker in ("fixture", "synthetic", "test-listener", "calibration")
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def degraded_or_reject_is_eligible(
    entry: dict[str, Any],
    evidence_mode: str,
) -> bool:
    if not human_verdict_is_eligible(entry, evidence_mode):
        return False
    outcome = entry.get("degraded_or_reject_evidence")
    structurally_eligible = (
        isinstance(outcome, dict)
        and entry.get("human_verdict") in {"weak", "fail"}
        and entry.get("demo_readiness") == "not_demo_ready"
        and outcome.get("schema") == DEGRADED_OR_REJECT_EVIDENCE_SCHEMA
        and outcome.get("outcome") in {"degraded", "unavailable", "reject"}
        and outcome.get("product_path_reviewed") is True
        and outcome.get("fallback_music_present") is False
        and isinstance(outcome.get("reason"), str)
        and bool(outcome["reason"].strip())
    )
    if not structurally_eligible:
        return False
    if evidence_mode == FIXTURE_CALIBRATION:
        return True

    human_evidence = entry.get("human_review_evidence")
    if not isinstance(human_evidence, dict):
        return False
    review_path = Path(str(human_evidence.get("review_path") or ""))
    try:
        review = read_degraded_product_review(review_path)
        validate_degraded_product_artifact_identity(review)
    except (OSError, TypeError, ValueError):
        return False
    human = review.get("human_review")
    proof = review.get("product_path_proof")
    return (
        review.get("schema") == DEGRADED_PRODUCT_REVIEW_SCHEMA
        and degraded_product_review_is_human_pass(review)
        and review.get("source_family") == entry.get("source_family")
        and review.get("outcome") == outcome.get("outcome")
        and review.get("reason") == outcome.get("reason")
        and isinstance(human, dict)
        and human.get("reviewer") == human_evidence.get("reviewer")
        and isinstance(proof, dict)
        and proof.get("fallback_music_present") is False
        and proof.get("generated_output_configured") is False
        and proof.get("confident_bar_locked_output_allowed") is False
    )


def evidence_context(
    evidence_mode: str,
    demo_bank_path: Path | None,
    entries: list[Any],
    declared_evidence_role: Any,
) -> dict[str, Any]:
    object_entries = [entry for entry in entries if isinstance(entry, dict)]
    bank_is_live = declared_evidence_role == LIVE_REVIEW_BANK_ROLE
    usable_entries = (
        object_entries
        if evidence_mode == FIXTURE_CALIBRATION or bank_is_live
        else []
    )
    eligible = [
        str(entry.get("entry_id") or "")
        for entry in usable_entries
        if human_verdict_is_eligible(entry, evidence_mode)
    ]
    ignored = [
        str(entry.get("entry_id") or "")
        for entry in object_entries
        if entry.get("human_verdict") in {"pass", "weak", "fail"}
        and (
            entry not in usable_entries
            or not human_verdict_is_eligible(entry, evidence_mode)
        )
    ]
    state = (
        "missing"
        if demo_bank_path is None
        else "available"
        if evidence_mode == FIXTURE_CALIBRATION or bank_is_live
        else "rejected_non_live_bank"
    )
    return {
        "mode": evidence_mode,
        "fixture_only": evidence_mode == FIXTURE_CALIBRATION,
        "demo_bank_state": state,
        "demo_bank_path": str(demo_bank_path) if demo_bank_path is not None else None,
        "demo_bank_sha256": (
            sha256_file(demo_bank_path)
            if demo_bank_path is not None and demo_bank_path.is_file()
            else None
        ),
        "declared_evidence_role": declared_evidence_role,
        "live_human_review_evidence_required": evidence_mode == LIVE_READINESS,
        "eligible_human_verdict_entry_ids": eligible,
        "eligible_human_verdict_count": len(eligible),
        "ignored_unproven_human_verdict_entry_ids": ignored,
        "missing_evidence_reason": (
            None
            if state == "available"
            else (
                "No explicit real demo-bank path was supplied for live readiness."
                if state == "missing"
                else "The supplied demo bank is not declared as live_review evidence."
            )
        ),
    }


def usable_entries(
    demo_bank: dict[str, Any],
    entries: list[Any],
    evidence_mode: str,
) -> list[Any]:
    if (
        evidence_mode == LIVE_READINESS
        and demo_bank.get("evidence_role") != LIVE_REVIEW_BANK_ROLE
    ):
        return []
    return entries


def context_bank_identity_is_current(context: dict[str, Any]) -> bool:
    path_value = context.get("demo_bank_path")
    if path_value is None:
        return context.get("demo_bank_sha256") is None
    if not isinstance(path_value, str):
        return False
    expected = context.get("demo_bank_sha256")
    path = Path(path_value)
    return (
        isinstance(expected, str)
        and HEX_64_RE.fullmatch(expected) is not None
        and path.is_file()
        and sha256_file(path) == expected
    )


def empty_demo_bank(schema: str) -> dict[str, Any]:
    return {
        "schema": schema,
        "schema_version": 1,
        "evidence_role": LIVE_REVIEW_BANK_ROLE,
        "entries": [],
    }
