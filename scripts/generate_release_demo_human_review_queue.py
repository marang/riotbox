#!/usr/bin/env python3
"""Generate a P023 human review queue from release-demo candidates."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from release_demo_human_review_queue_fixtures import validate_mutation_fixtures
import validate_source_family_release_demo_coverage as coverage


SCHEMA = "riotbox.release_demo_human_review_queue.v1"
DEFAULT_SOURCE_CORPUS = coverage.DEFAULT_SOURCE_CORPUS
DEFAULT_DEMO_BANK = coverage.DEFAULT_DEMO_BANK
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-release-demo-human-review-queue")
REQUIRED_REVIEW_QUESTIONS = [
    "What is the strongest audible element: kick, snare, bass, stab, chop, vocal, silence, restore, or none?",
    "Does the source character survive as transformed material, or did the candidate collapse into fallback/generic output?",
    "Does a memorable hook appear within the first two bars?",
    "Does the destructive contrast, dropout, stutter, or restore feel like a playable live gesture?",
    "Is the candidate demo-worthy for a musician, weak-but-fixable, or a rejection example?",
    "What concrete follow-up should happen next if the verdict is not pass?",
]
REQUIRED_QUEUE_CONTEXT_FIELDS = [
    "hook_within_two_bars",
    "strongest_audible_element",
    "source_character",
    "destructive_contrast",
    "bass_drum_pressure",
    "live_triggerability",
    "eight_bar_replay_value",
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-corpus", type=Path, default=DEFAULT_SOURCE_CORPUS)
    parser.add_argument("--demo-bank", type=Path, default=DEFAULT_DEMO_BANK)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-release-demo-human-review-queue")
    parser.add_argument("--validate-report", type=Path)
    parser.add_argument("--mutation-fixtures", action="store_true")
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json_object(args.validate_report)
        else:
            source_corpus = read_json_object(args.source_corpus)
            demo_bank = read_json_object(args.demo_bank)
            coverage_report = coverage.build_report(
                source_corpus,
                demo_bank,
                args.source_corpus,
                args.demo_bank,
            )
            coverage_failures = coverage.validate_report(coverage_report)
            if coverage_failures:
                raise ValueError(", ".join(coverage_failures))
            report = build_report(args, demo_bank, coverage_report)
            args.output.mkdir(parents=True, exist_ok=True)
            write_report(args.output, report)

        failures = validate_report(report)
        if failures:
            raise ValueError(", ".join(failures))
        if args.mutation_fixtures:
            failures = validate_mutation_fixtures(report, validate_report)
            if failures:
                raise ValueError(", ".join(failures))
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid release-demo human review queue: {error}", file=sys.stderr)
        return 1

    print("valid release-demo human review queue")
    return 0


def build_report(
    args: argparse.Namespace,
    demo_bank: dict[str, Any],
    coverage_report: dict[str, Any],
) -> dict[str, Any]:
    entries = list_field(demo_bank, "entries", args.demo_bank)
    family_lookup = demo_family_lookup(coverage_report)
    family_gaps = family_review_gaps(coverage_report)
    queue = [
        review_queue_entry(entry, family_lookup, family_gaps)
        for entry in entries
        if entry.get("human_verdict") == "unverified"
        or entry.get("demo_readiness") == "unverified"
    ]
    weak_or_failed = [
        {
            "entry_id": required_string(entry, "entry_id"),
            "source_family": str(entry.get("source_family")),
            "human_verdict": str(entry.get("human_verdict")),
            "demo_readiness": str(entry.get("demo_readiness")),
            "fix_categories": string_list(entry.get("fix_categories", [])),
            "reason": str(entry.get("demo_worthiness_note", "")),
        }
        for entry in entries
        if entry.get("human_verdict") in {"weak", "fail"}
    ]
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "created_at": args.date,
        "result": "pass",
        "phase": "P023",
        "source_files_required": False,
        "quality_claim_allowed": False,
        "human_verdict_boundary": (
            "This queue only prepares human listening work. It must not promote "
            "unverified, scripted, or diagnostic artifacts into product-quality "
            "proof."
        ),
        "demo_bank": str(args.demo_bank),
        "source_family_coverage": {
            "source_corpus": coverage_report["source_corpus"],
            "required_family_count": coverage_report["required_family_count"],
            "covered_demo_ready_family_count": coverage_report[
                "covered_demo_ready_family_count"
            ],
            "missing_human_verdict_families": coverage_report[
                "missing_human_verdict_families"
            ],
            "missing_demo_ready_families": coverage_report[
                "missing_demo_ready_families"
            ],
            "family_gaps": family_gaps,
            "blockers": coverage_report["blockers"],
        },
        "review_queue_count": len(queue),
        "review_queue": queue,
        "weak_or_failed_entries": weak_or_failed,
        "next_actions": next_actions(queue, family_gaps, weak_or_failed),
    }


def demo_family_lookup(coverage_report: dict[str, Any]) -> dict[str, str]:
    lookup: dict[str, str] = {}
    for family in coverage_report["families"]:
        source_family = str(family["source_family"])
        for alias in family["demo_bank_family_aliases"]:
            lookup[str(alias)] = source_family
    return lookup


def family_review_gaps(coverage_report: dict[str, Any]) -> list[dict[str, Any]]:
    gaps = []
    for family in coverage_report["families"]:
        gaps.append(
            {
                "source_family": family["source_family"],
                "status": family["status"],
                "missing_human_verdict": not family["human_verdict_entry_ids"],
                "missing_demo_ready_human_pass": not family["demo_ready_entry_ids"],
                "candidate_entry_ids": family["candidate_entry_ids"],
                "human_verdict_entry_ids": family["human_verdict_entry_ids"],
                "demo_ready_entry_ids": family["demo_ready_entry_ids"],
                "unverified_entry_ids": family["unverified_entry_ids"],
            }
        )
    return gaps


def review_queue_entry(
    entry: dict[str, Any],
    family_lookup: dict[str, str],
    family_gaps: list[dict[str, Any]],
) -> dict[str, Any]:
    demo_family = str(entry.get("source_family"))
    corpus_family = family_lookup.get(demo_family, demo_family)
    gap = next(
        (item for item in family_gaps if item["source_family"] == corpus_family),
        None,
    )
    missing_human = bool(gap and gap["missing_human_verdict"])
    missing_demo_ready = bool(gap and gap["missing_demo_ready_human_pass"])
    if missing_human:
        priority = "high"
        action = (
            "Record a structured human pass, weak, or fail verdict before this "
            "family can support release-ready claims."
        )
    elif missing_demo_ready:
        priority = "medium"
        action = (
            "Review whether this updated candidate can become human-pass and "
            "demo-ready, or route it to the next production fix."
        )
    else:
        priority = "low"
        action = (
            "Optional review candidate; keep it unverified unless a structured "
            "listener verdict is recorded."
        )
    summary = musical_summary(entry)
    blockers = review_blockers(entry, missing_human, missing_demo_ready)

    return {
        "entry_id": required_string(entry, "entry_id"),
        "review_priority": priority,
        "source_family": corpus_family,
        "demo_bank_source_family": demo_family,
        "source_path": required_string(entry, "source_path"),
        "rendered_wav": artifact_ref(entry, "rendered_wav"),
        "metrics": artifact_ref(entry, "metrics"),
        "review_prompt": artifact_ref(entry, "review_prompt"),
        "human_verdict": str(entry.get("human_verdict")),
        "demo_readiness": str(entry.get("demo_readiness")),
        "quality_claim": entry.get("quality_claim"),
        "demo_worthiness_note": required_string(entry, "demo_worthiness_note"),
        "demo_worthy_reason": required_string(entry, "demo_worthiness_note"),
        "not_demo_ready_reason": not_demo_ready_reason(corpus_family, blockers),
        "hook_within_two_bars": summary["hook_within_two_bars"],
        "strongest_audible_element": summary["hardest_audible_element"],
        "source_character": summary["source_character"],
        "destructive_contrast": summary["destructive_contrast"],
        "bass_drum_pressure": summary["bass_drum_pressure"],
        "live_triggerability": summary["live_triggerability"],
        "eight_bar_replay_value": summary["eight_bar_replay_value"],
        "review_blockers": blockers,
        "required_listening_questions": REQUIRED_REVIEW_QUESTIONS,
        "required_verdict_path": {
            "current_state": "human_verdict:unverified/demo_readiness:unverified",
            "pass": "Only after listening: set human_verdict=pass, demo_readiness=demo_ready, and document why it is musician-facing.",
            "weak": "Set human_verdict=weak, demo_readiness=not_demo_ready, and route concrete fix_categories.",
            "fail": "Set human_verdict=fail, demo_readiness=not_demo_ready, and preserve it only as failure evidence.",
            "release_ready_blocker": "Unverified candidates must not claim quality or support release-ready coverage.",
        },
        "next_review_action": action,
    }


def next_actions(
    queue: list[dict[str, Any]],
    family_gaps: list[dict[str, Any]],
    weak_or_failed: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    actions: list[dict[str, Any]] = []
    for family in family_gaps:
        if family["missing_human_verdict"]:
            actions.append(
                {
                    "category": "human_review",
                    "target": family["source_family"],
                    "action": "Review the high-priority unverified candidate and record pass, weak, or fail.",
                }
            )
        elif family["missing_demo_ready_human_pass"]:
            actions.append(
                {
                    "category": "demo_promotion",
                    "target": family["source_family"],
                    "action": "Review the strongest candidate for possible demo-ready promotion or fix routing.",
                }
            )
    if weak_or_failed:
        actions.append(
            {
                "category": "production_fix",
                "target": "weak_or_failed_demo_bank_entries",
                "action": "Use weak/fail entries as fix inputs before adding more demo-ready claims.",
            }
        )
    if queue:
        actions.append(
            {
                "category": "listening_pack",
                "target": "release_demo_candidates",
                "action": "Open the listed review prompts and rendered WAV refs; record structured human verdicts only after listening.",
            }
        )
    return actions


def validate_report(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    check(report.get("phase") == "P023", "phase_mismatch", failures)
    check(report.get("source_files_required") is False, "source_files_required_must_be_false", failures)
    check(report.get("quality_claim_allowed") is False, "quality_claim_must_be_false", failures)

    queue = report.get("review_queue")
    check(isinstance(queue, list) and bool(queue), "review_queue_missing", failures)
    if isinstance(queue, list):
        check(
            report.get("review_queue_count") == len(queue),
            "review_queue_count_mismatch",
            failures,
        )
        priorities = {entry.get("review_priority") for entry in queue if isinstance(entry, dict)}
        check("high" in priorities, "high_priority_review_missing", failures)
        for index, entry in enumerate(queue):
            validate_queue_entry(entry, index, failures)

    coverage_data = report.get("source_family_coverage")
    check(isinstance(coverage_data, dict), "source_family_coverage_missing", failures)
    if isinstance(coverage_data, dict):
        gaps = coverage_data.get("family_gaps")
        blockers = coverage_data.get("blockers")
        check(isinstance(gaps, list) and bool(gaps), "family_gaps_missing", failures)
        check(isinstance(blockers, list), "coverage_blockers_missing", failures)
        if isinstance(gaps, list):
            gap_families = {
                str(gap.get("source_family"))
                for gap in gaps
                if isinstance(gap, dict)
                and (gap.get("missing_human_verdict") or gap.get("missing_demo_ready_human_pass"))
            }
            for required in ["bad_timing", "pad_noise", "sparse_drums", "weak_source"]:
                check(required in gap_families, f"{required}_gap_missing", failures)

    check(
        isinstance(report.get("human_verdict_boundary"), str)
        and "must not promote" in report["human_verdict_boundary"],
        "human_verdict_boundary_missing",
        failures,
    )
    check(isinstance(report.get("next_actions"), list) and report["next_actions"], "next_actions_missing", failures)
    return failures


def validate_queue_entry(entry: Any, index: int, failures: list[str]) -> None:
    if not isinstance(entry, dict):
        failures.append(f"review_queue_{index}_not_object")
        return
    for field in [
        "entry_id",
        "review_priority",
        "source_family",
        "demo_bank_source_family",
        "source_path",
        "rendered_wav",
        "metrics",
        "review_prompt",
        "human_verdict",
        "demo_readiness",
        "quality_claim",
        "demo_worthy_reason",
        "not_demo_ready_reason",
        "hook_within_two_bars",
        "strongest_audible_element",
        "source_character",
        "destructive_contrast",
        "bass_drum_pressure",
        "live_triggerability",
        "eight_bar_replay_value",
        "review_blockers",
        "required_listening_questions",
        "required_verdict_path",
        "next_review_action",
    ]:
        if field not in entry:
            failures.append(f"review_queue_{index}_{field}_missing")
    check(
        entry.get("review_priority") in {"high", "medium", "low"},
        f"review_queue_{index}_priority_invalid",
        failures,
    )
    check(entry.get("human_verdict") == "unverified", f"review_queue_{index}_not_unverified", failures)
    check(entry.get("demo_readiness") == "unverified", f"review_queue_{index}_not_unverified_demo", failures)
    check(entry.get("quality_claim") is False, f"review_queue_{index}_claims_quality", failures)
    for field in REQUIRED_QUEUE_CONTEXT_FIELDS:
        check(
            isinstance(entry.get(field), str) and bool(entry[field].strip()),
            f"review_queue_{index}_{field}_missing",
            failures,
        )
    check(
        isinstance(entry.get("demo_worthy_reason"), str)
        and bool(entry["demo_worthy_reason"].strip()),
        f"review_queue_{index}_demo_worthy_reason_missing",
        failures,
    )
    check(
        isinstance(entry.get("not_demo_ready_reason"), str)
        and "human_verdict" in entry["not_demo_ready_reason"]
        and "unverified" in entry["not_demo_ready_reason"],
        f"review_queue_{index}_not_demo_ready_reason_missing",
        failures,
    )
    blockers = entry.get("review_blockers")
    check(
        isinstance(blockers, list)
        and "human_verdict_unverified" in blockers
        and "demo_readiness_unverified" in blockers
        and "quality_claim_blocked" in blockers,
        f"review_queue_{index}_review_blockers_missing",
        failures,
    )
    questions = entry.get("required_listening_questions")
    check(
        isinstance(questions, list)
        and questions == REQUIRED_REVIEW_QUESTIONS,
        f"review_queue_{index}_required_listening_questions_invalid",
        failures,
    )
    verdict_path = entry.get("required_verdict_path")
    check(isinstance(verdict_path, dict), f"review_queue_{index}_required_verdict_path_missing", failures)
    if isinstance(verdict_path, dict):
        check(
            verdict_path.get("current_state")
            == "human_verdict:unverified/demo_readiness:unverified",
            f"review_queue_{index}_stale_verdict_state",
            failures,
        )
        for verdict in ["pass", "weak", "fail", "release_ready_blocker"]:
            check(
                isinstance(verdict_path.get(verdict), str)
                and bool(verdict_path[verdict].strip()),
                f"review_queue_{index}_required_verdict_path_{verdict}_missing",
                failures,
            )
    for field in ["rendered_wav", "metrics", "review_prompt"]:
        artifact = entry.get(field)
        check(
            isinstance(artifact, dict)
            and isinstance(artifact.get("path"), str)
            and bool(artifact["path"])
            and isinstance(artifact.get("sha256"), str)
            and len(artifact["sha256"]) == 64,
            f"review_queue_{index}_{field}_invalid",
            failures,
        )


def write_report(output: Path, report: dict[str, Any]) -> None:
    (output / "release-demo-human-review-queue.json").write_text(json.dumps(report, indent=2) + "\n")
    (output / "release-demo-human-review-queue.md").write_text(markdown_report(report))


def markdown_report(report: dict[str, Any]) -> str:
    lines = [
        "# Release-Demo Human Review Queue",
        "",
        f"- Phase: `{report['phase']}`",
        f"- Queue entries: `{report['review_queue_count']}`",
        f"- Quality claim allowed: `{str(report['quality_claim_allowed']).lower()}`",
        "",
        "## Review Queue",
        "",
    ]
    for entry in report["review_queue"]:
        lines.extend(
            [
                f"### `{entry['entry_id']}`",
                "",
                f"- Priority: `{entry['review_priority']}`",
                f"- Source family: `{entry['source_family']}`",
                f"- Source: `{entry['source_path']}`",
                f"- Strongest audible element: {entry['strongest_audible_element']}",
                f"- Source character: {entry['source_character']}",
                f"- Demo-worthy reason: {entry['demo_worthy_reason']}",
                f"- Not demo-ready: {entry['not_demo_ready_reason']}",
                f"- WAV: `{entry['rendered_wav']['path']}`",
                f"- Metrics: `{entry['metrics']['path']}`",
                f"- Prompt: `{entry['review_prompt']['path']}`",
                f"- Blockers: `{', '.join(entry['review_blockers'])}`",
                f"- Action: {entry['next_review_action']}",
                "- Required listening questions:",
                *[f"  - {question}" for question in entry["required_listening_questions"]],
                "",
            ]
        )
    lines.extend(["## Family Gaps", ""])
    for gap in report["source_family_coverage"]["family_gaps"]:
        lines.append(
            f"- `{gap['source_family']}`: `{gap['status']}`, "
            f"missing human verdict `{str(gap['missing_human_verdict']).lower()}`, "
            f"missing demo-ready `{str(gap['missing_demo_ready_human_pass']).lower()}`"
        )
    lines.extend(["", "## Boundary", "", report["human_verdict_boundary"], ""])
    return "\n".join(lines)


def artifact_ref(entry: dict[str, Any], field: str) -> dict[str, str]:
    value = entry.get(field)
    require(isinstance(value, dict), f"{entry.get('entry_id')}: {field} must be object")
    return {
        "path": required_string(value, "path"),
        "sha256": required_string(value, "sha256"),
    }


def review_blockers(
    entry: dict[str, Any],
    missing_human: bool,
    missing_demo_ready: bool,
) -> list[str]:
    blockers = [
        "human_verdict_unverified",
        "demo_readiness_unverified",
        "quality_claim_blocked",
    ]
    if missing_human:
        blockers.append("source_family_human_verdict_missing")
    if missing_demo_ready:
        blockers.append("source_family_demo_ready_human_pass_missing")
    if entry.get("quality_claim") is not False:
        blockers.append("unexpected_quality_claim_state")
    return blockers


def not_demo_ready_reason(source_family: str, blockers: list[str]) -> str:
    blocker_text = ", ".join(blockers)
    return (
        f"{source_family} remains not demo-ready because human_verdict is "
        f"unverified, demo_readiness is unverified, and release-ready quality "
        f"claims are blocked until structured listening records pass, weak, "
        f"or fail. Active blockers: {blocker_text}."
    )


def musical_summary(entry: dict[str, Any]) -> dict[str, str]:
    value = entry.get("musical_summary")
    require(isinstance(value, dict), f"{entry.get('entry_id')}: musical_summary must be object")
    summary: dict[str, str] = {}
    for field in REQUIRED_QUEUE_CONTEXT_FIELDS:
        source_field = "hardest_audible_element" if field == "strongest_audible_element" else field
        summary[source_field] = required_string(value, source_field)
    return summary


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if isinstance(item, str) and item]


def required_string(data: dict[str, Any], field: str) -> str:
    value = data.get(field)
    require(isinstance(value, str) and bool(value), f"{field} must be non-empty string")
    return str(value)


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
