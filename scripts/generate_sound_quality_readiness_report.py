#!/usr/bin/env python3
"""Generate the P023 sound-quality readiness report."""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.sound_quality_readiness_report.v1"
RUBRIC_SCHEMA = "riotbox.sound_product_readiness_rubric.v1"
SOURCE_CORPUS_SCHEMA = "riotbox.sound_excellence_source_corpus.v1"
DEMO_BANK_SCHEMA = "riotbox.release_grade_demo_bank.v1"
WEAK_ROUTING_SCHEMA = "riotbox.weak_output_fix_routing.v1"
PROFESSIONAL_SUITE_SCHEMA = "riotbox.professional_output_suite.v1"

DEFAULT_RUBRIC = Path("scripts/fixtures/sound_product_readiness_rubric/rubric_v1.json")
DEFAULT_SOURCE_CORPUS = Path("docs/benchmarks/sound_excellence_source_corpus_v1.json")
DEFAULT_DEMO_BANK = Path("scripts/fixtures/release_grade_demo_bank/demo_bank_v1.json")
DEFAULT_WEAK_ROUTING = Path("artifacts/audio_qa/local-weak-output-fix-routing/weak-output-fix-routing.json")
DEFAULT_PROFESSIONAL_SUITE = Path("artifacts/audio_qa/local-professional-output-suite/professional-output-suite.json")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-sound-quality-readiness-report")

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
    parser.add_argument("--rubric", type=Path, default=DEFAULT_RUBRIC)
    parser.add_argument("--source-corpus", type=Path, default=DEFAULT_SOURCE_CORPUS)
    parser.add_argument("--demo-bank", type=Path, default=DEFAULT_DEMO_BANK)
    parser.add_argument("--weak-routing", type=Path, default=DEFAULT_WEAK_ROUTING)
    parser.add_argument("--professional-output-suite", type=Path, default=DEFAULT_PROFESSIONAL_SUITE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-sound-quality-readiness-report")
    parser.add_argument("--validate-report", type=Path)
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json_object(args.validate_report)
            failures = validate_report(report)
            if failures:
                raise ValueError(", ".join(failures))
            print(f"valid sound-quality readiness report: {args.validate_report}")
            return 0

        report = build_report(args)
        failures = validate_report(report)
        if failures:
            raise ValueError(", ".join(failures))
        args.output.mkdir(parents=True, exist_ok=True)
        write_report(args.output, report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid sound-quality readiness report: {error}", file=sys.stderr)
        return 1

    print(f"sound-quality readiness report written to {args.output}")
    return 0


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    rubric = read_json_object(args.rubric)
    source_corpus = read_json_object(args.source_corpus)
    demo_bank = read_json_object(args.demo_bank)
    weak_routing = read_optional_json_object(args.weak_routing)
    professional_suite = read_optional_json_object(args.professional_output_suite)

    require(rubric.get("schema") == RUBRIC_SCHEMA, f"{args.rubric}: schema must be {RUBRIC_SCHEMA}")
    require(
        source_corpus.get("schema") == SOURCE_CORPUS_SCHEMA,
        f"{args.source_corpus}: schema must be {SOURCE_CORPUS_SCHEMA}",
    )
    require(
        demo_bank.get("schema") == DEMO_BANK_SCHEMA,
        f"{args.demo_bank}: schema must be {DEMO_BANK_SCHEMA}",
    )

    source_families = source_family_coverage(source_corpus, demo_bank, args.source_corpus)
    demo_summary = demo_bank_summary(demo_bank, args.demo_bank)
    weak_summary = weak_routing_summary(weak_routing, args.weak_routing)
    suite_summary = professional_suite_summary(professional_suite, args.professional_output_suite)
    blockers = readiness_blockers(source_families, demo_summary, weak_summary, suite_summary)

    release_readiness = "release_ready" if not blockers else "blocked"
    quality_claim_allowed = release_readiness == "release_ready"
    next_fix_categories = sorted(set(weak_summary["fix_categories"]) | set(demo_summary["weak_fix_categories"]))
    if not next_fix_categories and blockers:
        next_fix_categories = ["source_selection", "fixture_threshold"]

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "created_at": args.date,
        "result": "pass",
        "phase": "P023",
        "release_readiness": release_readiness,
        "quality_claim_allowed": quality_claim_allowed,
        "human_verdict_boundary": "human verdicts are required for product-quality claims",
        "evidence_boundary": (
            "This report aggregates existing QA, corpus, demo-bank, and weak-output "
            "routing evidence. It is not a hidden taste oracle and it does not turn "
            "scripted or unverified diagnostics into product-quality proof."
        ),
        "rubric": {
            "path": str(args.rubric),
            "schema": rubric["schema"],
            "state_count": len(object_field(rubric, "states", args.rubric)),
            "quality_states": sorted(
                key for key, state in object_field(rubric, "states", args.rubric).items()
                if state.get("may_claim_product_quality") is True
            ),
            "musical_dimension_count": len(object_field(rubric, "musical_dimensions", args.rubric)),
        },
        "source_family_coverage": source_families,
        "demo_bank": demo_summary,
        "weak_output_routing": weak_summary,
        "professional_output_suite": suite_summary,
        "blockers": blockers,
        "next_actions": next_actions(source_families, demo_summary, weak_summary, suite_summary),
        "next_fix_categories": next_fix_categories,
        "musician_summary": musician_summary(blockers, next_fix_categories),
    }


def source_family_coverage(source_corpus: dict[str, Any], demo_bank: dict[str, Any], path: Path) -> dict[str, Any]:
    required = string_list_field(source_corpus, "required_source_families", path)
    corpus_entries = list_field(source_corpus, "entries", path)
    demo_entries = list_field(demo_bank, "entries", Path("demo_bank"))
    demo_families = {str(entry.get("source_family")) for entry in demo_entries if entry.get("demo_readiness") == "demo_ready"}
    all_demo_families = {str(entry.get("source_family")) for entry in demo_entries}

    families = []
    for family in required:
        mapped = CORPUS_TO_DEMO_FAMILIES.get(family, {family})
        has_any_candidate = bool(mapped & all_demo_families)
        has_demo_ready = bool(mapped & demo_families)
        corpus_case_ids = [
            str(entry.get("case_id"))
            for entry in corpus_entries
            if entry.get("source_family") == family
        ]
        families.append(
            {
                "source_family": family,
                "demo_bank_family_aliases": sorted(mapped),
                "corpus_case_ids": corpus_case_ids,
                "has_demo_candidate": has_any_candidate,
                "has_demo_ready_human_pass": has_demo_ready,
                "status": "covered" if has_demo_ready else ("candidate_only" if has_any_candidate else "missing"),
            }
        )

    missing_demo_ready = [item["source_family"] for item in families if not item["has_demo_ready_human_pass"]]
    return {
        "path": str(path),
        "required_source_families": required,
        "covered_demo_ready_families": [
            item["source_family"] for item in families if item["has_demo_ready_human_pass"]
        ],
        "missing_demo_ready_families": missing_demo_ready,
        "families": families,
    }


def demo_bank_summary(demo_bank: dict[str, Any], path: Path) -> dict[str, Any]:
    entries = list_field(demo_bank, "entries", path)
    verdict_counts = Counter(str(entry.get("human_verdict")) for entry in entries)
    readiness_counts = Counter(str(entry.get("demo_readiness")) for entry in entries)
    weak_fix_categories = sorted(
        {
            category
            for entry in entries
            if entry.get("human_verdict") in {"weak", "fail"}
            for category in list(entry.get("fix_categories", []))
            if isinstance(category, str) and category
        }
    )
    unverified = [
        str(entry.get("entry_id"))
        for entry in entries
        if entry.get("human_verdict") == "unverified" or entry.get("demo_readiness") == "unverified"
    ]
    weak_or_fail = [
        {
            "entry_id": str(entry.get("entry_id")),
            "human_verdict": str(entry.get("human_verdict")),
            "demo_readiness": str(entry.get("demo_readiness")),
            "fix_categories": list(entry.get("fix_categories", [])),
            "reason": str(entry.get("demo_worthiness_note", "")),
        }
        for entry in entries
        if entry.get("human_verdict") in {"weak", "fail"}
    ]
    return {
        "path": str(path),
        "entry_count": len(entries),
        "demo_ready_count": readiness_counts.get("demo_ready", 0),
        "human_verdict_counts": dict(sorted(verdict_counts.items())),
        "demo_readiness_counts": dict(sorted(readiness_counts.items())),
        "unverified_candidate_ids": unverified,
        "weak_or_fail_entries": weak_or_fail,
        "weak_fix_categories": weak_fix_categories,
    }


def weak_routing_summary(report: dict[str, Any] | None, path: Path) -> dict[str, Any]:
    if report is None:
        return {
            "path": str(path),
            "available": False,
            "result": "missing",
            "case_count": 0,
            "fix_categories": [],
            "production_fix_candidate_count": 0,
            "production_fix_candidates": [],
            "cases": [],
        }
    require(report.get("schema") == WEAK_ROUTING_SCHEMA, f"{path}: schema must be {WEAK_ROUTING_SCHEMA}")
    cases = list(report.get("cases", []))
    candidates = weak_routing_candidates(report, path)
    return {
        "path": str(path),
        "available": True,
        "result": str(report.get("result")),
        "case_count": int(report.get("case_count", len(cases))),
        "routed_case_count": int(report.get("routed_case_count", 0)),
        "quality_proof": report.get("quality_proof"),
        "automated_musical_approval": report.get("automated_musical_approval"),
        "fix_categories": list(report.get("fix_categories", [])),
        "production_fix_candidate_count": len(candidates),
        "production_fix_candidates": candidates,
        "cases": [
            {
                "case_id": str(case.get("case_id")),
                "artifact_to_hear": str(case.get("artifact_to_hear")),
                "proposed_next_fix_category": str(case.get("proposed_next_fix_category")),
                "musician_fix_reason": str(case.get("musician_fix_reason")),
            }
            for case in cases
        ],
    }


def weak_routing_candidates(report: dict[str, Any], path: Path) -> list[dict[str, str]]:
    raw_count = report.get("production_fix_candidate_count", 0)
    raw_candidates = report.get("production_fix_candidates", [])
    require(isinstance(raw_count, int), f"{path}: production_fix_candidate_count must be integer")
    require(isinstance(raw_candidates, list), f"{path}: production_fix_candidates must be array")
    require(raw_count == len(raw_candidates), f"{path}: production_fix_candidate_count mismatch")
    candidates = []
    for index, candidate in enumerate(raw_candidates):
        require(isinstance(candidate, dict), f"{path}: production_fix_candidates[{index}] must be object")
        candidates.append(
            {
                "candidate_id": required_string(candidate, "candidate_id", path, index),
                "category": required_string(candidate, "category", path, index),
                "software_next_step": required_string(candidate, "software_next_step", path, index),
                "musician_payoff": required_string(candidate, "musician_payoff", path, index),
            }
        )
    return candidates


def professional_suite_summary(report: dict[str, Any] | None, path: Path) -> dict[str, Any]:
    if report is None:
        return {
            "path": str(path),
            "available": False,
            "result": "missing",
            "human_verdict": "unverified",
            "scripted_generation": None,
            "quality_proof": None,
        }
    require(
        report.get("schema") == PROFESSIONAL_SUITE_SCHEMA,
        f"{path}: schema must be {PROFESSIONAL_SUITE_SCHEMA}",
    )
    return {
        "path": str(path),
        "available": True,
        "result": str(report.get("result")),
        "human_verdict": str(report.get("human_verdict")),
        "scripted_generation": report.get("scripted_generation"),
        "quality_proof": report.get("quality_proof"),
        "child_report_count": report.get("child_report_count"),
    }


def readiness_blockers(
    coverage: dict[str, Any],
    demo: dict[str, Any],
    weak: dict[str, Any],
    suite: dict[str, Any],
) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    missing_families = coverage["missing_demo_ready_families"]
    if missing_families:
        blockers.append(
            {
                "code": "source_family_demo_ready_coverage_missing",
                "severity": "release_blocking",
                "families": missing_families,
                "reason": "Every P023 source family needs demo-ready human-pass coverage before release-ready claims.",
            }
        )
    if demo["unverified_candidate_ids"]:
        blockers.append(
            {
                "code": "unverified_demo_candidates_present",
                "severity": "claim_blocking",
                "entries": demo["unverified_candidate_ids"],
                "reason": "Unverified candidates can be reviewed but must not be promoted as demo-ready.",
            }
        )
    if demo["weak_or_fail_entries"]:
        blockers.append(
            {
                "code": "weak_or_fail_demo_bank_entries_present",
                "severity": "production_blocking",
                "entries": [entry["entry_id"] for entry in demo["weak_or_fail_entries"]],
                "reason": "Weak/fail entries must route to production fixes before they can improve the demo bank.",
            }
        )
    if not weak["available"] or weak["result"] != "pass":
        blockers.append(
            {
                "code": "weak_output_routing_not_available",
                "severity": "actionability_blocking",
                "reason": "Weak outputs need concrete fix categories before P023 can move quickly.",
            }
        )
    if not suite["available"] or suite["result"] != "pass":
        blockers.append(
            {
                "code": "professional_output_suite_not_available",
                "severity": "evidence_blocking",
                "reason": "The P022 professional-output suite should be available as current diagnostic context.",
            }
        )
    if suite.get("scripted_generation") is True or suite.get("quality_proof") is False:
        blockers.append(
            {
                "code": "professional_suite_diagnostic_only",
                "severity": "claim_blocking",
                "reason": "The current professional-output suite remains scripted diagnostic evidence, not quality proof.",
            }
        )
    return blockers

def next_actions(
    coverage: dict[str, Any],
    demo: dict[str, Any],
    weak: dict[str, Any],
    suite: dict[str, Any],
) -> list[dict[str, Any]]:
    actions: list[dict[str, Any]] = []
    for family in coverage["missing_demo_ready_families"]:
        actions.append(
            {
                "category": "source_selection",
                "target": family,
                "action": "Create or promote a real-source candidate with structured human listening evidence.",
            }
        )
    for category in sorted(set(weak["fix_categories"]) | set(demo["weak_fix_categories"])):
        actions.append(
            {
                "category": category,
                "target": "weak output",
                "action": f"Implement the next bounded production fix for {category}.",
            }
        )
    if not suite["available"]:
        actions.append(
            {
                "category": "fixture_threshold",
                "target": "professional output suite",
                "action": "Generate the professional-output suite before using the readiness report for branch review.",
            }
        )
    return actions

def musician_summary(blockers: list[dict[str, Any]], fix_categories: list[str]) -> str:
    if not blockers:
        return "P023 release sound is ready for the covered scope."
    category_text = ", ".join(fix_categories) if fix_categories else "source selection and fixture thresholds"
    return (
        "Not release-ready yet: the current evidence still needs human/demo coverage "
        f"and concrete production fixes around {category_text}."
    )


def validate_report(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    release_readiness = report.get("release_readiness")
    check(release_readiness in {"blocked", "release_ready"}, "release_readiness_invalid", failures)
    blockers = list(report.get("blockers", []))
    missing_families = nested_list(report, "source_family_coverage", "missing_demo_ready_families")
    unverified = nested_list(report, "demo_bank", "unverified_candidate_ids")
    weak_entries = nested_list(report, "demo_bank", "weak_or_fail_entries")
    weak_available = nested_value(report, "weak_output_routing", "available")
    suite_available = nested_value(report, "professional_output_suite", "available")
    suite_scripted = nested_value(report, "professional_output_suite", "scripted_generation")
    suite_quality = nested_value(report, "professional_output_suite", "quality_proof")

    if release_readiness == "release_ready":
        check(not missing_families, "release_ready_without_required_coverage", failures)
        check(not unverified, "release_ready_with_unverified_candidates", failures)
        check(not weak_entries, "release_ready_with_weak_entries", failures)
        check(weak_available is True, "release_ready_without_weak_routing", failures)
        check(suite_available is True, "release_ready_without_professional_suite", failures)
        check(suite_scripted is not True, "release_ready_from_scripted_suite", failures)
        check(suite_quality is True, "release_ready_without_quality_proof", failures)

    if blockers:
        check(release_readiness == "blocked", "blockers_require_blocked_readiness", failures)
        check(report.get("quality_claim_allowed") is False, "blocked_report_claims_quality", failures)
    else:
        check(report.get("quality_claim_allowed") is True, "release_ready_must_allow_quality_claim", failures)

    fix_categories = report.get("next_fix_categories")
    check(isinstance(fix_categories, list) and bool(fix_categories), "next_fix_categories_missing", failures)
    check(isinstance(report.get("musician_summary"), str) and report["musician_summary"], "musician_summary_missing", failures)
    return failures


def write_report(output: Path, report: dict[str, Any]) -> None:
    (output / "sound-quality-readiness-report.json").write_text(json.dumps(report, indent=2) + "\n")
    (output / "sound-quality-readiness-report.md").write_text(markdown_report(report))


def markdown_report(report: dict[str, Any]) -> str:
    lines = [
        "# Sound Quality Readiness Report",
        "",
        f"- Phase: {report['phase']}",
        f"- Release readiness: {report['release_readiness']}",
        f"- Quality claim allowed: {str(report['quality_claim_allowed']).lower()}",
        f"- Musician summary: {report['musician_summary']}",
        "",
        "## Blockers",
        "",
    ]
    for blocker in report["blockers"]:
        lines.append(f"- `{blocker['code']}`: {blocker['reason']}")
    if not report["blockers"]:
        lines.append("- none")
    lines.extend(["", "## Next Actions", ""])
    for action in report["next_actions"]:
        lines.append(f"- `{action['category']}` / {action['target']}: {action['action']}")
    if not report["next_actions"]:
        lines.append("- none")
    lines.extend(["", "## Evidence Boundary", "", report["evidence_boundary"], ""])
    return "\n".join(lines)


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def read_optional_json_object(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    return read_json_object(path)


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict) and value, f"{path}: {field} must be non-empty object")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def string_list_field(data: dict[str, Any], field: str, path: Path) -> list[str]:
    value = list_field(data, field, path)
    require(all(isinstance(item, str) and item for item in value), f"{path}: {field} values must be strings")
    return [str(item) for item in value]


def required_string(data: dict[str, Any], field: str, path: Path, index: int) -> str:
    value = data.get(field)
    require(
        isinstance(value, str) and bool(value),
        f"{path}: production_fix_candidates[{index}].{field} must be non-empty string",
    )
    return value


def nested_value(report: dict[str, Any], object_name: str, field: str) -> Any:
    value = report.get(object_name)
    if not isinstance(value, dict):
        return None
    return value.get(field)


def nested_list(report: dict[str, Any], object_name: str, field: str) -> list[Any]:
    value = nested_value(report, object_name, field)
    return value if isinstance(value, list) else []


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
