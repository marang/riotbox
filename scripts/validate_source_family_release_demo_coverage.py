#!/usr/bin/env python3
"""Validate P023 source-family release-demo coverage."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.source_family_release_demo_coverage.v1"
SOURCE_CORPUS_SCHEMA = "riotbox.sound_excellence_source_corpus.v1"
DEMO_BANK_SCHEMA = "riotbox.release_grade_demo_bank.v1"
DEFAULT_SOURCE_CORPUS = Path("docs/benchmarks/sound_excellence_source_corpus_v1.json")
DEFAULT_DEMO_BANK = Path("scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json")
CORPUS_TO_DEMO_FAMILIES = {
    "dense_break": {"dense_break"},
    "sparse_drums": {"sparse_bass_pressure"},
    "tonal_riff": {"tonal_hook"},
    "pad_noise": {"tonal_pad"},
    "weak_source": {"other"},
    "bad_timing": {"other"},
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-corpus", type=Path, default=DEFAULT_SOURCE_CORPUS)
    parser.add_argument("--demo-bank", type=Path, default=DEFAULT_DEMO_BANK)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    parser.add_argument("--validate-report", type=Path)
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json_object(args.validate_report)
        else:
            source_corpus = read_json_object(args.source_corpus)
            demo_bank = read_json_object(args.demo_bank)
            report = build_report(source_corpus, demo_bank, args.source_corpus, args.demo_bank)
            if args.json_output:
                args.json_output.parent.mkdir(parents=True, exist_ok=True)
                args.json_output.write_text(json.dumps(report, indent=2) + "\n")
            if args.markdown_output:
                args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
                args.markdown_output.write_text(markdown_report(report))

        failures = validate_report(report)
        if failures:
            raise ValueError(", ".join(failures))
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid source-family release-demo coverage: {error}", file=sys.stderr)
        return 1

    print("valid source-family release-demo coverage")
    return 0


def build_report(
    source_corpus: dict[str, Any],
    demo_bank: dict[str, Any],
    source_corpus_path: Path,
    demo_bank_path: Path,
) -> dict[str, Any]:
    require(
        source_corpus.get("schema") == SOURCE_CORPUS_SCHEMA,
        f"{source_corpus_path}: schema must be {SOURCE_CORPUS_SCHEMA}",
    )
    require(
        demo_bank.get("schema") == DEMO_BANK_SCHEMA,
        f"{demo_bank_path}: schema must be {DEMO_BANK_SCHEMA}",
    )
    required_families = string_list(source_corpus, "required_source_families", source_corpus_path)
    corpus_entries = list_field(source_corpus, "entries", source_corpus_path)
    demo_entries = list_field(demo_bank, "entries", demo_bank_path)

    families = [
        family_coverage(family, corpus_entries, demo_entries)
        for family in required_families
    ]
    blockers = coverage_blockers(families)
    release_readiness = "release_ready" if not blockers else "blocked"
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "source_files_required": False,
        "release_readiness": release_readiness,
        "quality_claim_allowed": release_readiness == "release_ready",
        "human_verdict_boundary": (
            "A source family is release-demo covered only when it has a "
            "demo-ready human-pass entry. Candidates and weak/fail human "
            "verdicts are useful evidence but block release-ready claims."
        ),
        "source_corpus": str(source_corpus_path),
        "demo_bank": str(demo_bank_path),
        "required_family_count": len(families),
        "covered_demo_ready_family_count": sum(
            1 for family in families if family["status"] == "demo_ready_covered"
        ),
        "missing_demo_candidate_families": [
            family["source_family"] for family in families if not family["candidate_entry_ids"]
        ],
        "missing_human_verdict_families": [
            family["source_family"] for family in families if not family["human_verdict_entry_ids"]
        ],
        "missing_demo_ready_families": [
            family["source_family"] for family in families if not family["demo_ready_entry_ids"]
        ],
        "families": families,
        "blockers": blockers,
    }


def family_coverage(
    source_family: str,
    corpus_entries: list[Any],
    demo_entries: list[Any],
) -> dict[str, Any]:
    aliases = CORPUS_TO_DEMO_FAMILIES.get(source_family, {source_family})
    matching_entries = [
        entry
        for entry in demo_entries
        if isinstance(entry, dict) and entry.get("source_family") in aliases
    ]
    candidate_ids = entry_ids(matching_entries)
    human_entries = [
        entry for entry in matching_entries if entry.get("human_verdict") in {"pass", "weak", "fail"}
    ]
    demo_ready_entries = [
        entry
        for entry in matching_entries
        if entry.get("human_verdict") == "pass" and entry.get("demo_readiness") == "demo_ready"
    ]
    if demo_ready_entries:
        status = "demo_ready_covered"
    elif human_entries:
        status = "human_verdict_non_demo"
    elif matching_entries:
        status = "candidate_only"
    else:
        status = "missing_candidate"
    return {
        "source_family": source_family,
        "demo_bank_family_aliases": sorted(aliases),
        "corpus_case_ids": [
            str(entry.get("case_id"))
            for entry in corpus_entries
            if isinstance(entry, dict) and entry.get("source_family") == source_family
        ],
        "candidate_entry_ids": candidate_ids,
        "human_verdict_entry_ids": entry_ids(human_entries),
        "demo_ready_entry_ids": entry_ids(demo_ready_entries),
        "unverified_entry_ids": entry_ids(
            [entry for entry in matching_entries if entry.get("human_verdict") == "unverified"]
        ),
        "status": status,
    }


def coverage_blockers(families: list[dict[str, Any]]) -> list[dict[str, Any]]:
    blockers = []
    for family in families:
        source_family = family["source_family"]
        if not family["candidate_entry_ids"]:
            blockers.append(
                {
                    "code": "source_family_demo_candidate_missing",
                    "source_family": source_family,
                    "severity": "release_blocking",
                    "reason": "No demo-bank candidate exists for this P023 source family.",
                }
            )
        if not family["human_verdict_entry_ids"]:
            blockers.append(
                {
                    "code": "source_family_human_verdict_missing",
                    "source_family": source_family,
                    "severity": "release_blocking",
                    "reason": "No pass, weak, or fail human verdict exists for this source family.",
                }
            )
        if not family["demo_ready_entry_ids"]:
            blockers.append(
                {
                    "code": "source_family_demo_ready_human_pass_missing",
                    "source_family": source_family,
                    "severity": "release_blocking",
                    "reason": "No demo-ready human-pass entry exists for this source family.",
                }
            )
    return blockers


def validate_report(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    check(report.get("source_files_required") is False, "source_files_required_must_be_false", failures)
    families = report.get("families")
    blockers = report.get("blockers")
    check(isinstance(families, list) and bool(families), "families_missing", failures)
    check(isinstance(blockers, list), "blockers_must_be_array", failures)
    if isinstance(families, list):
        for index, family in enumerate(families):
            validate_family(family, index, failures)
    release_readiness = report.get("release_readiness")
    check(release_readiness in {"blocked", "release_ready"}, "release_readiness_invalid", failures)
    if blockers:
        check(release_readiness == "blocked", "blockers_require_blocked_readiness", failures)
        check(report.get("quality_claim_allowed") is False, "blocked_report_claims_quality", failures)
    else:
        check(release_readiness == "release_ready", "unblocked_report_must_be_release_ready", failures)
        check(report.get("quality_claim_allowed") is True, "release_ready_must_allow_quality_claim", failures)
    return failures


def validate_family(family: Any, index: int, failures: list[str]) -> None:
    if not isinstance(family, dict):
        failures.append(f"families_{index}_not_object")
        return
    for field in [
        "source_family",
        "demo_bank_family_aliases",
        "corpus_case_ids",
        "candidate_entry_ids",
        "human_verdict_entry_ids",
        "demo_ready_entry_ids",
        "unverified_entry_ids",
        "status",
    ]:
        if field not in family:
            failures.append(f"families_{index}_{field}_missing")
    status = family.get("status")
    check(
        status in {
            "demo_ready_covered",
            "human_verdict_non_demo",
            "candidate_only",
            "missing_candidate",
        },
        f"families_{index}_status_invalid",
        failures,
    )
    for field in [
        "demo_bank_family_aliases",
        "corpus_case_ids",
        "candidate_entry_ids",
        "human_verdict_entry_ids",
        "demo_ready_entry_ids",
        "unverified_entry_ids",
    ]:
        values = family.get(field)
        check(
            isinstance(values, list) and all(isinstance(item, str) and item for item in values),
            f"families_{index}_{field}_invalid",
            failures,
        )


def markdown_report(report: dict[str, Any]) -> str:
    lines = [
        "# Source-Family Release-Demo Coverage",
        "",
        f"- Release readiness: `{report['release_readiness']}`",
        f"- Quality claim allowed: `{str(report['quality_claim_allowed']).lower()}`",
        f"- Covered demo-ready families: `{report['covered_demo_ready_family_count']}/{report['required_family_count']}`",
        f"- Source files required: `{str(report['source_files_required']).lower()}`",
        "",
        "## Families",
        "",
    ]
    for family in report["families"]:
        lines.extend(
            [
                f"### `{family['source_family']}`",
                "",
                f"- Status: `{family['status']}`",
                f"- Demo aliases: `{', '.join(family['demo_bank_family_aliases'])}`",
                f"- Corpus cases: `{', '.join(family['corpus_case_ids'])}`",
                f"- Candidate entries: `{', '.join(family['candidate_entry_ids']) or 'none'}`",
                f"- Human verdict entries: `{', '.join(family['human_verdict_entry_ids']) or 'none'}`",
                f"- Demo-ready entries: `{', '.join(family['demo_ready_entry_ids']) or 'none'}`",
                "",
            ]
        )
    lines.extend(["## Blockers", ""])
    if report["blockers"]:
        for blocker in report["blockers"]:
            lines.append(
                f"- `{blocker['source_family']}` / `{blocker['code']}`: {blocker['reason']}"
            )
    else:
        lines.append("- none")
    lines.extend(["", "## Boundary", "", report["human_verdict_boundary"], ""])
    return "\n".join(lines)


def entry_ids(entries: list[Any]) -> list[str]:
    return [
        str(entry.get("entry_id"))
        for entry in entries
        if isinstance(entry, dict) and isinstance(entry.get("entry_id"), str) and entry["entry_id"]
    ]


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def string_list(data: dict[str, Any], field: str, path: Path) -> list[str]:
    value = list_field(data, field, path)
    require(
        all(isinstance(item, str) and item for item in value),
        f"{path}: {field} must contain non-empty strings",
    )
    return [str(item) for item in value]


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
