"""Shared evidence-boundary contract for Riotbox audio QA reports."""

from __future__ import annotations

from typing import Any


SCHEMA = "riotbox.audio_qa_evidence_boundary.v1"
ALLOWED_EVIDENCE_ROLES = {
    "diagnostic",
    "negative_diagnostic",
    "listening_review_scaffold",
    "suite_diagnostic",
}


def apply_evidence_boundary(
    report: dict[str, Any],
    *,
    evidence_role: str,
    source_backed: bool,
    source_timing_backed: bool,
    scripted_generation: bool,
    human_verdict: str | None = None,
    quality_proof: bool = False,
    notes: str | None = None,
) -> dict[str, Any]:
    """Attach and validate the shared evidence boundary.

    The fields are duplicated top-level for jq/report consumers and nested under
    `evidence_boundary` for a stable cross-language JSON contract.
    """

    boundary = build_evidence_boundary(
        evidence_role=evidence_role,
        source_backed=source_backed,
        source_timing_backed=source_timing_backed,
        scripted_generation=scripted_generation,
        human_verdict=human_verdict or str(report.get("human_verdict", "unverified")),
        quality_proof=quality_proof,
        notes=notes,
    )
    validate_evidence_boundary(boundary)
    report["evidence_boundary"] = boundary
    for key in (
        "evidence_role",
        "source_backed",
        "source_timing_backed",
        "scripted_generation",
        "quality_proof",
    ):
        report[key] = boundary[key]
    return report


def build_evidence_boundary(
    *,
    evidence_role: str,
    source_backed: bool,
    source_timing_backed: bool,
    scripted_generation: bool,
    human_verdict: str,
    quality_proof: bool = False,
    notes: str | None = None,
) -> dict[str, Any]:
    boundary: dict[str, Any] = {
        "schema": SCHEMA,
        "schema_version": 1,
        "evidence_role": evidence_role,
        "source_backed": source_backed,
        "source_timing_backed": source_timing_backed,
        "scripted_generation": scripted_generation,
        "quality_proof": quality_proof,
        "human_verdict": human_verdict,
    }
    if notes:
        boundary["notes"] = notes
    return boundary


def extract_evidence_boundary(report: dict[str, Any]) -> dict[str, Any]:
    boundary = report.get("evidence_boundary")
    if isinstance(boundary, dict):
        return dict(boundary)
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "evidence_role": report.get("evidence_role"),
        "source_backed": report.get("source_backed"),
        "source_timing_backed": report.get("source_timing_backed"),
        "scripted_generation": report.get("scripted_generation"),
        "quality_proof": report.get("quality_proof"),
        "human_verdict": report.get("human_verdict"),
    }


def evidence_boundary_failure_codes(
    report: dict[str, Any],
    *,
    require_present: bool = True,
) -> list[str]:
    boundary = report.get("evidence_boundary")
    if require_present and not isinstance(boundary, dict):
        return ["evidence_boundary_missing"]
    try:
        validate_evidence_boundary(extract_evidence_boundary(report))
    except ValueError as error:
        return [str(error)]
    return []


def validate_evidence_boundary(boundary: dict[str, Any]) -> None:
    if boundary.get("schema") != SCHEMA:
        raise ValueError("evidence_boundary_schema_mismatch")
    if boundary.get("schema_version") != 1:
        raise ValueError("evidence_boundary_schema_version_mismatch")
    if boundary.get("evidence_role") not in ALLOWED_EVIDENCE_ROLES:
        raise ValueError("evidence_role_invalid")
    for key in (
        "source_backed",
        "source_timing_backed",
        "scripted_generation",
        "quality_proof",
    ):
        if not isinstance(boundary.get(key), bool):
            raise ValueError(f"{key}_not_boolean")
    if boundary.get("quality_proof") is not False:
        raise ValueError("quality_proof_true_not_allowed")
    if boundary.get("scripted_generation") and boundary.get("quality_proof"):
        raise ValueError("scripted_generation_claims_quality_proof")
    if boundary.get("human_verdict") != "unverified":
        raise ValueError("evidence_boundary_human_verdict_not_unverified")
