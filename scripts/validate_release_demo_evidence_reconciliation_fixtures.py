#!/usr/bin/env python3
"""Mutation fixtures for release-demo evidence reconciliation."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable

import demo_bank_evidence as evidence
import release_demo_evidence_reconciliation as reconciliation


FIXTURE_BANK = Path("scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json")


def main() -> int:
    try:
        bank = read_json(FIXTURE_BANK)
        prepare_complete_fixture_bank(bank)
        with tempfile.TemporaryDirectory(prefix="riotbox-release-evidence-") as tmp_value:
            tmp = Path(tmp_value)
            bank_path = tmp / "demo-bank.json"
            contract_path = tmp / "reconciliation.json"
            write_json(bank_path, bank)
            contract = fixture_contract(reconciliation.sha256_file(bank_path))
            write_json(contract_path, contract)

            summary = reconciliation.summarize(
                contract,
                contract_path,
                bank,
                bank_path,
                list(bank["entries"]),
                evidence.FIXTURE_CALIBRATION,
            )
            require(summary.get("result") == "pass", "fixture reconciliation did not pass")
            require(summary.get("quality_proof") is False, "fixture reconciliation claimed quality")
            require(
                summary.get("superseded_entry_ids") == ["dense-break-beat03-human-fail"],
                "superseded fixture entry changed",
            )

            expect_failure(
                contract,
                lambda data: data["supersessions"][0].update(
                    {"successor_entry_id": "tonal-hook-rusharp-human-pass"}
                ),
                contract_path,
                bank,
                bank_path,
                "source family mismatch",
            )
            expect_failure(
                contract,
                lambda data: data["demo_bank"].update({"sha256": "0" * 64}),
                contract_path,
                bank,
                bank_path,
                "demo bank sha256 does not match",
            )
            live_contract = copy.deepcopy(contract)
            live_contract["evidence_mode"] = evidence.LIVE_READINESS
            expect_failure(
                live_contract,
                lambda _data: None,
                contract_path,
                bank,
                bank_path,
                "quality entry lacks current human-review evidence",
                evidence_mode=evidence.LIVE_READINESS,
            )
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid release-demo evidence reconciliation fixtures: {error}", file=sys.stderr)
        return 1

    print("valid release-demo evidence reconciliation fixtures")
    return 0


def prepare_complete_fixture_bank(bank: dict[str, Any]) -> None:
    update_entry(
        bank,
        "sparse-bass-pressure-human-weak",
        human_verdict="pass",
        demo_readiness="demo_ready",
        fix_categories=[],
    )
    for entry_id, family, outcome in (
        ("pad-noise-fadapad-unverified-candidate", "pad_noise", "unavailable"),
        ("bad-timing-beat20-unverified-candidate", "bad_timing", "degraded"),
        ("weak-source-beat20-rejection-unverified-candidate", "weak_source", "degraded"),
    ):
        update_entry(
            bank,
            entry_id,
            source_family=family,
            human_verdict="fail",
            demo_readiness="not_demo_ready",
            fix_categories=["source_selection"],
            degraded_or_reject_evidence={
                "schema": "riotbox.demo_bank_degraded_or_reject_review_evidence.v1",
                "outcome": outcome,
                "product_path_reviewed": True,
                "fallback_music_present": False,
                "reason": "Fixture-calibration product handling stayed honest.",
            },
        )


def fixture_contract(bank_sha256: str) -> dict[str, Any]:
    return {
        "schema": reconciliation.SCHEMA,
        "schema_version": 1,
        "status": "frozen",
        "evidence_mode": evidence.FIXTURE_CALIBRATION,
        "demo_bank": {
            "sha256": bank_sha256,
            "evidence_role": evidence.FIXTURE_CALIBRATION,
        },
        "supersessions": [
            {
                "superseded_entry_id": "dense-break-beat03-human-fail",
                "successor_entry_id": "dense-break-beat03-human-pass",
                "negative_evidence_retained": True,
                "reason": "Fixture supersession exercises lifecycle validation.",
            }
        ],
        "quality_evidence": {
            "positive_entry_ids": [
                "dense-break-beat03-human-pass",
                "sparse-bass-pressure-human-weak",
                "tonal-hook-rusharp-human-pass",
            ],
            "reviewed_negative_entry_ids": [
                "bad-timing-beat20-unverified-candidate",
                "pad-noise-fadapad-unverified-candidate",
                "weak-source-beat20-rejection-unverified-candidate",
            ],
            "fixture_or_scripted_quality_proof_allowed": False,
        },
    }


def expect_failure(
    contract: dict[str, Any],
    mutate: Callable[[dict[str, Any]], object],
    contract_path: Path,
    bank: dict[str, Any],
    bank_path: Path,
    expected: str,
    *,
    evidence_mode: str = evidence.FIXTURE_CALIBRATION,
) -> None:
    candidate = copy.deepcopy(contract)
    mutate(candidate)
    try:
        reconciliation.summarize(
            candidate,
            contract_path,
            bank,
            bank_path,
            list(bank["entries"]),
            evidence_mode,
        )
    except ValueError as error:
        require(expected in str(error), f"expected {expected}, got {error}")
        return
    raise ValueError(f"mutation unexpectedly passed: {expected}")


def update_entry(bank: dict[str, Any], entry_id: str, **fields: Any) -> None:
    entry = next(
        (
            item
            for item in bank.get("entries", [])
            if isinstance(item, dict) and item.get("entry_id") == entry_id
        ),
        None,
    )
    require(isinstance(entry, dict), f"missing fixture entry {entry_id}")
    entry.update(fields)


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
