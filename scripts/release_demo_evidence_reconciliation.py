#!/usr/bin/env python3
"""Validate explicit release-demo evidence supersession and quality ownership."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

import demo_bank_evidence as evidence
from exact_product_path_review_gate import validate_promotion_gate


SCHEMA = "riotbox.release_demo_evidence_reconciliation.v1"
REQUIRED_POSITIVE_FAMILIES = {"dense_break", "sparse_drums", "tonal_riff"}
REQUIRED_REVIEWED_NEGATIVE_FAMILIES = {"bad_timing", "pad_noise", "weak_source"}


def summarize(
    contract: dict[str, Any] | None,
    contract_path: Path | None,
    demo_bank: dict[str, Any],
    demo_bank_path: Path | None,
    entries: list[Any],
    evidence_mode: str,
) -> dict[str, Any]:
    if contract is None or contract_path is None:
        return {
            "path": str(contract_path) if contract_path is not None else None,
            "available": False,
            "result": "missing",
            "evidence_mode": evidence_mode,
            "demo_bank_path": str(demo_bank_path) if demo_bank_path is not None else None,
            "demo_bank_sha256": None,
            "scripted_generation": False,
            "quality_proof": False,
            "supersessions": [],
            "superseded_entry_ids": [],
            "quality_entry_ids": [],
            "reviewed_negative_entry_ids": [],
            "covered_positive_families": [],
            "covered_reviewed_negative_families": [],
        }

    require(contract.get("schema") == SCHEMA, f"{contract_path}: schema must be {SCHEMA}")
    require(contract.get("schema_version") == 1, f"{contract_path}: schema_version must be 1")
    require(contract.get("status") == "frozen", f"{contract_path}: status must be frozen")
    require(
        contract.get("evidence_mode") == evidence_mode,
        f"{contract_path}: evidence_mode must match report mode",
    )
    require(demo_bank_path is not None, f"{contract_path}: demo bank path is required")
    require(demo_bank_path.is_file(), f"{contract_path}: demo bank is missing")
    bank_hash = sha256_file(demo_bank_path)
    bank_ref = object_field(contract, "demo_bank", contract_path)
    require(
        bank_ref.get("sha256") == bank_hash,
        f"{contract_path}: demo bank sha256 does not match current bank",
    )
    require(
        bank_ref.get("evidence_role") == demo_bank.get("evidence_role"),
        f"{contract_path}: demo bank evidence role mismatch",
    )

    object_entries = [entry for entry in entries if isinstance(entry, dict)]
    entries_by_id = {
        str(entry.get("entry_id")): entry
        for entry in object_entries
        if isinstance(entry.get("entry_id"), str)
    }
    require(len(entries_by_id) == len(object_entries), f"{contract_path}: demo entry ids invalid")

    supersessions = list_field(contract, "supersessions", contract_path)
    normalized_supersessions: list[dict[str, str]] = []
    superseded_ids: set[str] = set()
    successor_ids: set[str] = set()
    for index, item in enumerate(supersessions):
        prefix = f"{contract_path}: supersessions[{index}]"
        require(isinstance(item, dict), f"{prefix} must be object")
        superseded_id = required_string(item, "superseded_entry_id", prefix)
        successor_id = required_string(item, "successor_entry_id", prefix)
        reason = required_string(item, "reason", prefix)
        require(item.get("negative_evidence_retained") is True, f"{prefix}: negative evidence must be retained")
        require(superseded_id not in superseded_ids, f"{prefix}: duplicate superseded entry")
        require(successor_id not in successor_ids, f"{prefix}: duplicate successor entry")
        old = required_entry(entries_by_id, superseded_id, prefix)
        new = required_entry(entries_by_id, successor_id, prefix)
        require(old.get("human_verdict") in {"weak", "fail"}, f"{prefix}: superseded entry must be weak/fail")
        require(old.get("demo_readiness") == "not_demo_ready", f"{prefix}: superseded entry must not be demo-ready")
        require(new.get("human_verdict") == "pass", f"{prefix}: successor must be a human pass")
        require(new.get("demo_readiness") == "demo_ready", f"{prefix}: successor must be demo-ready")
        require(old.get("source_family") == new.get("source_family"), f"{prefix}: source family mismatch")
        require(old.get("source_path") == new.get("source_path"), f"{prefix}: source identity mismatch")
        superseded_ids.add(superseded_id)
        successor_ids.add(successor_id)
        normalized_supersessions.append(
            {
                "superseded_entry_id": superseded_id,
                "successor_entry_id": successor_id,
                "source_family": str(new.get("source_family")),
                "reason": reason,
            }
        )

    quality = object_field(contract, "quality_evidence", contract_path)
    quality_ids = unique_string_list(quality, "positive_entry_ids", contract_path)
    reviewed_negative_ids = unique_string_list(
        quality,
        "reviewed_negative_entry_ids",
        contract_path,
    )
    require(not (set(quality_ids) & set(reviewed_negative_ids)), f"{contract_path}: quality and negative ids overlap")
    require(superseded_ids.isdisjoint(quality_ids), f"{contract_path}: superseded entry cannot carry quality")
    require(successor_ids.issubset(quality_ids), f"{contract_path}: every successor must be a quality entry")
    require(
        superseded_ids.isdisjoint(reviewed_negative_ids),
        f"{contract_path}: superseded failures cannot satisfy reviewed-negative families",
    )
    require(
        quality.get("fixture_or_scripted_quality_proof_allowed") is False,
        f"{contract_path}: fixture/scripted quality proof must be false",
    )

    positive_families: set[str] = set()
    for entry_id in quality_ids:
        entry = required_entry(entries_by_id, entry_id, str(contract_path))
        require(entry.get("human_verdict") == "pass", f"{contract_path}: quality entry must pass")
        require(entry.get("demo_readiness") == "demo_ready", f"{contract_path}: quality entry must be demo-ready")
        positive_families.add(canonical_family(str(entry.get("source_family") or "")))
        if evidence_mode == evidence.LIVE_READINESS:
            require(
                evidence.human_verdict_is_eligible(entry, evidence_mode),
                f"{contract_path}: quality entry lacks current human-review evidence",
            )
            gate = entry.get("exact_product_path_review_gate")
            require(isinstance(gate, dict), f"{contract_path}: quality entry lacks exact product-path gate")
            validate_promotion_gate(
                gate,
                contract_path,
                expected_source_family=str(entry.get("source_family")),
            )

    negative_families: set[str] = set()
    for entry_id in reviewed_negative_ids:
        entry = required_entry(entries_by_id, entry_id, str(contract_path))
        require(
            entry.get("human_verdict") in {"weak", "fail"}
            and entry.get("demo_readiness") == "not_demo_ready",
            f"{contract_path}: reviewed-negative entry must be weak/fail and not demo-ready",
        )
        outcome = entry.get("degraded_or_reject_evidence")
        require(
            isinstance(outcome, dict)
            and outcome.get("product_path_reviewed") is True
            and outcome.get("fallback_music_present") is False,
            f"{contract_path}: reviewed-negative entry lacks no-fallback product evidence",
        )
        negative_families.add(canonical_family(str(entry.get("source_family") or "")))
        if evidence_mode == evidence.LIVE_READINESS:
            require(
                evidence.degraded_or_reject_is_eligible(entry, evidence_mode),
                f"{contract_path}: negative entry lacks reviewed product-path evidence",
            )

    require(
        positive_families == REQUIRED_POSITIVE_FAMILIES,
        f"{contract_path}: positive family coverage must be {sorted(REQUIRED_POSITIVE_FAMILIES)}",
    )
    require(
        negative_families == REQUIRED_REVIEWED_NEGATIVE_FAMILIES,
        f"{contract_path}: reviewed-negative family coverage must be {sorted(REQUIRED_REVIEWED_NEGATIVE_FAMILIES)}",
    )

    live_quality = (
        evidence_mode == evidence.LIVE_READINESS
        and demo_bank.get("evidence_role") == evidence.LIVE_REVIEW_BANK_ROLE
    )
    return {
        "path": str(contract_path),
        "sha256": sha256_file(contract_path),
        "available": True,
        "result": "pass",
        "evidence_mode": evidence_mode,
        "demo_bank_path": str(demo_bank_path),
        "demo_bank_sha256": bank_hash,
        "scripted_generation": False,
        "quality_proof": live_quality,
        "supersessions": normalized_supersessions,
        "superseded_entry_ids": sorted(superseded_ids),
        "quality_entry_ids": quality_ids,
        "reviewed_negative_entry_ids": reviewed_negative_ids,
        "covered_positive_families": sorted(positive_families),
        "covered_reviewed_negative_families": sorted(negative_families),
    }


def canonical_family(source_family: str) -> str:
    return next(
        (
            family
            for family, aliases in evidence.CORPUS_TO_DEMO_FAMILIES.items()
            if source_family in aliases
        ),
        source_family,
    )


def summary_is_current(summary: dict[str, Any]) -> bool:
    path_value = summary.get("path")
    bank_path_value = summary.get("demo_bank_path")
    if not isinstance(path_value, str) or not isinstance(bank_path_value, str):
        return (
            summary.get("available") is False
            and summary.get("result") == "missing"
            and summary.get("quality_proof") is False
            and not summary.get("superseded_entry_ids")
        )
    path = Path(path_value)
    bank_path = Path(bank_path_value)
    if not path.is_file() or not bank_path.is_file():
        return False
    try:
        contract = read_json_object(path)
        demo_bank = read_json_object(bank_path)
        entries = demo_bank.get("entries")
        if not isinstance(entries, list):
            return False
        rebuilt = summarize(
            contract,
            path,
            demo_bank,
            bank_path,
            entries,
            str(summary.get("evidence_mode")),
        )
    except (OSError, TypeError, ValueError):
        return False
    return rebuilt == summary


def validate_summary(summary: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if not summary_is_current(summary):
        failures.append("release_demo_evidence_reconciliation_stale")
    if summary.get("quality_proof") is not True:
        return failures
    if not (summary.get("available") is True and summary.get("result") == "pass"):
        failures.append("release_demo_quality_proof_without_valid_reconciliation")
    if summary.get("evidence_mode") != evidence.LIVE_READINESS:
        failures.append("release_demo_quality_proof_not_live")
    if summary.get("scripted_generation") is not False:
        failures.append("release_demo_quality_proof_scripted")
    if set(string_values(summary.get("covered_positive_families"))) != REQUIRED_POSITIVE_FAMILIES:
        failures.append("release_demo_quality_proof_positive_families_incomplete")
    if (
        set(string_values(summary.get("covered_reviewed_negative_families")))
        != REQUIRED_REVIEWED_NEGATIVE_FAMILIES
    ):
        failures.append("release_demo_quality_proof_negative_families_incomplete")
    return failures


def markdown_lines(summary: dict[str, Any]) -> list[str]:
    return [
        "",
        "## Release-Demo Product Evidence",
        "",
        f"- Available: `{str(summary.get('available') is True).lower()}`",
        f"- Result: `{summary.get('result', 'missing')}`",
        f"- Quality proof: `{str(summary.get('quality_proof') is True).lower()}`",
        f"- Scripted generation: `{str(summary.get('scripted_generation') is True).lower()}`",
        "- Superseded negative entries: `"
        f"{', '.join(string_values(summary.get('superseded_entry_ids')))}`",
        "- Exact product human-pass entries: `"
        f"{', '.join(string_values(summary.get('quality_entry_ids')))}`",
    ]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def required_entry(entries: dict[str, dict[str, Any]], entry_id: str, prefix: str) -> dict[str, Any]:
    entry = entries.get(entry_id)
    require(entry is not None, f"{prefix}: missing demo entry {entry_id}")
    return entry


def string_values(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{path}: {field} must be object")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def unique_string_list(data: dict[str, Any], field: str, path: Path) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    require(all(isinstance(item, str) and item for item in value), f"{path}: {field} values must be strings")
    require(len(set(value)) == len(value), f"{path}: {field} values must be unique")
    return list(value)


def required_string(data: dict[str, Any], field: str, prefix: str) -> str:
    value = data.get(field)
    require(isinstance(value, str) and value.strip(), f"{prefix}.{field} must be non-empty string")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)
