#!/usr/bin/env python3
"""Validate the generated weak-output fix routing smoke artifacts."""

from __future__ import annotations

import argparse
import copy
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable

from route_weak_output_fixes import read_json_object, validate_routing_report


DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-weak-output-fix-routing")
REQUIRED_FIX_CATEGORIES = {
    "source_selection",
    "chop_policy",
    "bass_movement",
    "mix_bus",
    "destructive_gesture",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--report-json", type=Path)
    parser.add_argument("--report-markdown", type=Path)
    args = parser.parse_args()

    report_json = args.report_json or args.output / "weak-output-fix-routing.json"
    report_markdown = args.report_markdown or args.output / "weak-output-fix-routing.md"

    try:
        report = read_json_object(report_json)
        markdown = report_markdown.read_text()
        failures = validate_routing_report(report)
        if failures:
            raise ValueError(f"{report_json}: {', '.join(failures)}")
        validate_positive_contract(report)
        validate_markdown(markdown)
        validate_mutation_fixtures(report)
        validate_subprocess_fixtures(args.output)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid weak-output fix routing smoke: {error}", file=sys.stderr)
        return 1

    print(f"valid weak-output fix routing smoke: {report_json}")
    return 0


def validate_positive_contract(report: dict[str, Any]) -> None:
    require(report.get("schema") == "riotbox.weak_output_fix_routing.v1", "schema mismatch")
    require(report.get("result") == "pass", "result must be pass")
    require(report.get("agent_verdict") == "agent_promising", "agent verdict mismatch")
    require(report.get("human_verdict") == "unverified", "human verdict must be unverified")
    require(report.get("evidence_role") == "diagnostic", "evidence role mismatch")
    require(report.get("quality_proof") is False, "report must not claim quality")
    require(
        report.get("automated_musical_approval") is False,
        "report must not claim automated approval",
    )
    cases = list_field(report, "cases")
    require(report.get("case_count") == 6, "case count mismatch")
    require(report.get("routed_case_count") == 6, "routed case count mismatch")
    require(
        REQUIRED_FIX_CATEGORIES.issubset(set(string_list(report.get("fix_categories")))),
        "required fix categories missing",
    )
    for case in cases:
        require(isinstance(case, dict), "cases must be objects")
        for field in [
            "artifact_to_hear",
            "strongest_audible_element",
            "main_weakness",
            "proposed_next_fix_category",
            "musician_fix_reason",
        ]:
            require(isinstance(case.get(field), str) and bool(case[field]), f"{field} missing")
        require(case.get("matched_known_routing_signal") is True, "routing signal missing")
        require(string_list(case.get("proposed_fix_categories")), "case fix categories missing")
        require(case.get("quality_proof") is False, "case claims quality")
        require(case.get("automated_musical_approval") is False, "case claims approval")
    require_case(
        cases,
        "rendered_dense_flat_stutter",
        "destructive_gesture",
        "dropout_not_contrasting_with_stutter",
    )
    require_case(cases, "tonal_hookless_chop", "chop_policy", "w30_hook_not_dominant")
    require_case(cases, "sparse_bass_pressure_weak", "bass_movement", "mc202_bass_pressure_too_weak")
    require_case(cases, "automated_source_masked", "mix_bus", "source_first_generated_support_masks_source")
    dense_human = case_by_id(cases, "human_dense_fail_source_and_gesture")
    require(dense_human.get("proposed_next_fix_category") == "source_selection", "human dense route mismatch")
    require(nested_object(dense_human, "reason_tags").get("source_character") == "source_lost", "human dense reason missing")
    require("source character lost" in string_list(dense_human.get("avoid")), "human dense avoid missing")
    source_reasons = string_list(nested_object(dense_human, "routing_reasons").get("source_selection"))
    require(
        "source_character=source_lost: Human label says source is lost." in source_reasons,
        "human dense source-character routing reason missing",
    )
    require(
        "avoid=source character lost: Avoid-list calls out lost source character." in source_reasons,
        "human dense avoid routing reason missing",
    )
    validate_production_fix_candidates(report)


def validate_production_fix_candidates(report: dict[str, Any]) -> None:
    candidates = list_field(report, "production_fix_candidates")
    require(report.get("production_fix_candidate_count", 0) >= 5, "candidate count too low")
    for candidate in candidates:
        require(isinstance(candidate, dict), "production fix candidates must be objects")
        for field in ["candidate_id", "category", "software_next_step", "musician_payoff"]:
            require(isinstance(candidate.get(field), str) and bool(candidate[field]), f"{field} missing")
        require(isinstance(candidate.get("score"), int) and candidate["score"] >= 1, "score invalid")
        require(string_list(candidate.get("case_ids")), "candidate case ids missing")
        require(string_list(candidate.get("artifact_refs")), "candidate artifact refs missing")
        require(candidate.get("evidence_role") == "production_fix_candidate", "candidate evidence role mismatch")
        require(candidate.get("quality_proof") is False, "candidate claims quality")
        require(candidate.get("automated_musical_approval") is False, "candidate claims approval")
    require_candidate(candidates, "chop_policy", "agent_dense_weak_hook_and_pressure", "tonal_hookless_chop")
    require_candidate(candidates, "bass_movement", "sparse_bass_pressure_weak")
    require_candidate(candidates, "destructive_gesture", "rendered_dense_flat_stutter")
    require_candidate(candidates, "source_selection", "human_dense_fail_source_and_gesture")
    require_candidate(candidates, "mix_bus", "automated_source_masked")
    summary = nested_object(report, "production_fix_summary")
    require(summary.get("candidate_count") == report.get("production_fix_candidate_count"), "summary candidate count stale")
    require(string_list(summary.get("categories")) == [candidate["category"] for candidate in candidates], "summary categories stale")
    recurring = string_list(summary.get("recurring_fix_categories"))
    for category in ["chop_policy", "bass_movement", "destructive_gesture"]:
        require(category in recurring, f"recurring category missing: {category}")
    for candidate in candidates:
        category = candidate["category"]
        require(
            nested_object(summary, "case_counts_by_category").get(category) == len(candidate["case_ids"]),
            f"summary case count stale: {category}",
        )
        require(
            nested_object(summary, "primary_case_counts_by_category").get(category)
            == len(candidate["primary_case_ids"]),
            f"summary primary count stale: {category}",
        )
    require(summary.get("quality_proof") is False, "summary claims quality")
    require(summary.get("automated_musical_approval") is False, "summary claims approval")


def validate_markdown(markdown: str) -> None:
    for needle in ["Production Fix Candidates", "Recurring fix categories"]:
        require(needle in markdown, f"markdown missing {needle}")


def validate_mutation_fixtures(report: dict[str, Any]) -> None:
    expect_failure(
        report,
        "stale_candidate_count",
        lambda data: set_field(data, "production_fix_candidate_count", 999),
        "production_fix_candidate_count_mismatch",
    )
    expect_failure(
        report,
        "unknown_candidate_case",
        lambda data: list_field(data["production_fix_candidates"][0], "case_ids").append("missing_case"),
        "unknown_case_missing_case",
    )
    expect_failure(
        report,
        "stale_summary_count",
        lambda data: set_field(nested_object(data, "production_fix_summary"), "candidate_count", 0),
        "production_fix_summary_candidate_count_stale",
    )
    expect_failure(
        report,
        "duplicate_category_candidate",
        duplicate_first_candidate_category,
        "duplicate_category",
    )


def validate_subprocess_fixtures(output: Path) -> None:
    generated = output / "generated-weak-source-character"
    if generated.exists():
        shutil.rmtree(generated)
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        failed = run(
            [
                sys.executable,
                "scripts/generate_dense_break_performance_pack.py",
                "--output",
                str(generated),
                "--date",
                "weak-output-fix-routing-source-character",
                "--weak-source-character-fixture",
            ],
            check=False,
        )
        require(failed.returncode != 0, "weak source-character generation unexpectedly passed")
        require("rebuild_only_source_character_not_surviving" in failed.combined, "weak source-character failure code missing")
        manifest = {
            "schema": "riotbox.weak_output_fix_routing.v1",
            "schema_version": 1,
            "entries": [
                {
                    "case_id": "generated_weak_source_character",
                    "kind": "dense_performance_report",
                    "path": str(generated / "performance-report.json"),
                    "expected_next_fix_category": "source_selection",
                }
            ],
        }
        manifest_path = tmp_path / "manifest.json"
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
        routing_output = tmp_path / "routing"
        run(
            [
                sys.executable,
                "scripts/route_weak_output_fixes.py",
                "--manifest",
                str(manifest_path),
                "--output",
                str(routing_output),
            ]
        )
        routed = read_json_object(routing_output / "weak-output-fix-routing.json")
        case = list_field(routed, "cases")[0]
        require(routed.get("result") == "pass", "generated weak routing did not pass")
        require(case.get("proposed_next_fix_category") == "source_selection", "generated weak route mismatch")
        require(case.get("matched_known_routing_signal") is True, "generated weak routing signal missing")
        require("source_selection" in string_list(case.get("proposed_fix_categories")), "generated weak category missing")
        require("rebuild_only_source_character_not_surviving" in string_list(case.get("failure_codes")), "generated weak code missing")
        require("source window" in str(case.get("musician_fix_reason")), "generated weak musician reason missing")
    with tempfile.TemporaryDirectory() as tmp:
        failed = run(
            [
                sys.executable,
                "scripts/route_weak_output_fixes.py",
                "--manifest",
                "scripts/fixtures/weak_output_fix_routing/invalid_unknown_manifest.json",
                "--output",
                str(Path(tmp) / "unknown"),
            ],
            check=False,
        )
        require(failed.returncode != 0, "unknown weak-output route fixture unexpectedly passed")
        require("unknown_professional_failure_unknown_failure_route" in failed.combined, "unknown route failure code missing")


def expect_failure(
    report: dict[str, Any],
    fixture_name: str,
    mutate: Callable[[dict[str, Any]], object],
    expected_code: str,
) -> None:
    mutated = copy.deepcopy(report)
    mutate(mutated)
    failures = validate_routing_report(mutated)
    require(failures, f"{fixture_name}: mutation unexpectedly passed")
    require(
        any(expected_code in failure for failure in failures),
        f"{fixture_name}: expected {expected_code}, got {', '.join(failures)}",
    )


def duplicate_first_candidate_category(data: dict[str, Any]) -> None:
    candidates = list_field(data, "production_fix_candidates")
    duplicate = copy.deepcopy(candidates[0])
    duplicate["candidate_id"] = "p023_fix_duplicate_chop_policy"
    candidates.append(duplicate)
    data["production_fix_candidate_count"] += 1
    summary = nested_object(data, "production_fix_summary")
    summary["candidate_count"] += 1
    list_field(summary, "categories").append("chop_policy")


def require_case(cases: list[Any], case_id: str, category: str, failure_code: str) -> None:
    case = case_by_id(cases, case_id)
    require(case.get("proposed_next_fix_category") == category, f"{case_id} category mismatch")
    require(failure_code in string_list(case.get("failure_codes")), f"{case_id} failure code missing")


def require_candidate(candidates: list[Any], category: str, primary_case: str, secondary_case: str | None = None) -> None:
    candidate = next(
        (
            item
            for item in candidates
            if isinstance(item, dict) and item.get("category") == category
        ),
        None,
    )
    require(isinstance(candidate, dict), f"candidate missing: {category}")
    require(primary_case in string_list(candidate.get("primary_case_ids")), f"{category} primary case missing")
    if secondary_case:
        require(secondary_case in string_list(candidate.get("case_ids")), f"{category} secondary case missing")


def case_by_id(cases: list[Any], case_id: str) -> dict[str, Any]:
    case = next(
        (item for item in cases if isinstance(item, dict) and item.get("case_id") == case_id),
        None,
    )
    require(isinstance(case, dict), f"case missing: {case_id}")
    return case


def nested_object(data: dict[str, Any], field: str) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{field} must be object")
    return value


def list_field(data: dict[str, Any], field: str) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list), f"{field} must be list")
    return value


def string_list(value: Any) -> list[str]:
    return [str(item) for item in value] if isinstance(value, list) else []


def set_field(data: dict[str, Any], field: str, value: Any) -> None:
    data[field] = value


class RunResult:
    def __init__(self, completed: subprocess.CompletedProcess[str]) -> None:
        self.returncode = completed.returncode
        self.combined = f"{completed.stdout}\n{completed.stderr}"


def run(argv: list[str], *, check: bool = True) -> RunResult:
    completed = subprocess.run(argv, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if check and completed.returncode != 0:
        raise ValueError(f"{' '.join(argv)} failed: {completed.stdout}\n{completed.stderr}")
    return RunResult(completed)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
