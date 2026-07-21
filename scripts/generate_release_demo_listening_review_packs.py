#!/usr/bin/env python3
"""Generate local listening-review packs from the release-demo review queue."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

from generate_release_demo_human_review_queue import (
    REQUIRED_REVIEW_QUESTIONS,
    SCHEMA as QUEUE_SCHEMA,
    validate_report as validate_queue_report,
)
from listening_review_workflow import SCHEMA as LISTENING_REVIEW_SCHEMA
from listening_review_workflow import validate_review


SUMMARY_SCHEMA = "riotbox.release_demo_listening_review_packs.v1"
DEFAULT_QUEUE = Path("artifacts/audio_qa/local-release-demo-human-review-queue/release-demo-human-review-queue.json")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-release-demo-listening-review-packs")
DEFAULT_TICKET = "P023-release-demo-review"
UNVERIFIED_VERDICT_STATE = "human_verdict:unverified/demo_readiness:unverified"
SLUG_RE = re.compile(r"[^a-zA-Z0-9_.-]+")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--queue", type=Path, default=DEFAULT_QUEUE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--ticket", default=DEFAULT_TICKET)
    parser.add_argument("--date", default="local-release-demo-listening-review-packs")
    parser.add_argument("--validate-report", type=Path)
    args = parser.parse_args()

    try:
        if args.validate_report:
            summary = read_json_object(args.validate_report)
            failures = validate_summary(summary)
            if failures:
                raise ValueError(", ".join(failures))
            print(f"valid release-demo listening-review pack summary: {args.validate_report}")
            return 0

        queue = read_json_object(args.queue)
        failures = validate_queue_report(queue)
        if failures:
            raise ValueError(f"{args.queue}: {', '.join(failures)}")
        summary = build_packs(queue, args)
        failures = validate_summary(summary)
        if failures:
            raise ValueError(", ".join(failures))
        write_summary(args.output, summary)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid release-demo listening-review packs: {error}", file=sys.stderr)
        return 1

    print(f"release-demo listening-review packs written to {args.output}")
    return 0


def build_packs(queue: dict[str, Any], args: argparse.Namespace) -> dict[str, Any]:
    require(queue.get("schema") == QUEUE_SCHEMA, f"{args.queue}: schema must be {QUEUE_SCHEMA}")
    output = args.output
    evidence_context = object_field(queue, "demo_bank_evidence", args.queue)
    output.mkdir(parents=True, exist_ok=True)
    pack_entries = []
    for candidate in list_field(queue, "review_queue", args.queue):
        pack = build_pack(candidate, args.ticket, args.queue)
        pack_dir = output / safe_pack_dir(pack["entry_id"])
        pack_dir.mkdir(parents=True, exist_ok=True)
        review_path = pack_dir / "review.json"
        metrics_path = pack_dir / "metrics.json"
        prompt_path = pack_dir / "prompt.md"
        readme_path = pack_dir / "README.md"
        write_json(review_path, pack["review"])
        write_json(metrics_path, pack["metrics"])
        prompt_path.write_text(render_prompt(pack["review"], candidate))
        readme_path.write_text(render_readme(pack["review"]))
        validate_review(pack["review"], allow_unverified=True)
        pack_entries.append(
            {
                "entry_id": pack["entry_id"],
                "review_priority": pack["review_priority"],
                "source_family": pack["source_family"],
                "review_pack": str(pack_dir),
                "review_json": str(review_path),
                "metrics_json": str(metrics_path),
                "prompt_markdown": str(prompt_path),
                "human_verdict": "unverified",
                "demo_readiness": "unverified",
                "quality_claim": False,
            }
        )
    return {
        "schema": SUMMARY_SCHEMA,
        "schema_version": 1,
        "created_at": args.date,
        "result": "pass",
        "phase": "P023",
        "queue": str(args.queue),
        "ticket": args.ticket,
        "evidence_mode": evidence_context.get("mode"),
        "fixture_only": evidence_context.get("fixture_only"),
        "demo_bank_state": evidence_context.get("demo_bank_state"),
        "demo_bank_path": evidence_context.get("demo_bank_path"),
        "demo_bank_sha256": evidence_context.get("demo_bank_sha256"),
        "review_pack_count": len(pack_entries),
        "quality_claim_allowed": False,
        "human_verdict_boundary": (
            "Generated packs are local listening-review handoff artifacts. "
            "They keep human_verdict unverified until a listener records a verdict."
        ),
        "packs": pack_entries,
    }


def build_pack(candidate: Any, ticket: str, queue_path: Path) -> dict[str, Any]:
    require(isinstance(candidate, dict), f"{queue_path}: review_queue entries must be objects")
    entry_id = required_string(candidate, "entry_id", queue_path)
    require(candidate.get("human_verdict") == "unverified", f"{entry_id}: human_verdict must be unverified")
    require(candidate.get("demo_readiness") == "unverified", f"{entry_id}: demo_readiness must be unverified")
    require(candidate.get("quality_claim") is False, f"{entry_id}: quality_claim must be false")
    verdict_path = object_field(candidate, "required_verdict_path", queue_path)
    require(
        verdict_path.get("current_state") == UNVERIFIED_VERDICT_STATE,
        f"{entry_id}: required verdict state is stale",
    )
    rendered_wav = artifact_ref(candidate, "rendered_wav", queue_path)
    metrics = artifact_ref(candidate, "metrics", queue_path)
    review_prompt = artifact_ref(candidate, "review_prompt", queue_path)
    source_path = required_string(candidate, "source_path", queue_path)
    questions = string_list(candidate.get("required_listening_questions"))
    require(questions == REQUIRED_REVIEW_QUESTIONS, f"{entry_id}: required listening questions changed")
    metadata = {
        "entry_id": entry_id,
        "review_priority": required_string(candidate, "review_priority", queue_path),
        "source_family": required_string(candidate, "source_family", queue_path),
        "demo_bank_source_family": required_string(candidate, "demo_bank_source_family", queue_path),
        "rendered_wav": rendered_wav,
        "metrics": metrics,
        "review_prompt": review_prompt,
        "review_blockers": string_list(candidate.get("review_blockers")),
        "required_verdict_current_state": str(verdict_path["current_state"]),
        "human_verdict": "unverified",
        "demo_readiness": "unverified",
        "quality_claim": False,
        "demo_worthy_reason": required_string(candidate, "demo_worthy_reason", queue_path),
        "not_demo_ready_reason": required_string(candidate, "not_demo_ready_reason", queue_path),
        "required_listening_questions": questions,
        "next_review_action": required_string(candidate, "next_review_action", queue_path),
    }
    review = {
        "schema": LISTENING_REVIEW_SCHEMA,
        "schema_version": 1,
        "ticket": ticket,
        "pr": None,
        "command": f"generated from {queue_path}",
        "source_file": source_path,
        "seed_or_config": entry_id,
        "technical_status": "release_demo_queue_pass",
        "automated_musical_fitness_status": "unverified",
        "human_verdict": "unverified",
        "strongest_element": "none",
        "source_recognition": "unverified",
        "hook_after_two_bars": "unverified",
        "failure_reason": "",
        "preferred_direction": "",
        "avoid": [],
        "concrete_follow_up": "",
        "reviewer": None,
        "demo_readiness_consequence": "unverified_until_human_verdict",
        "expected_audible_behavior": expected_audible_behavior(candidate),
        "artifacts": {
            "candidate_audio": [rendered_wav["path"]],
            "source_audio": source_path,
            "metrics_json": "metrics.json",
            "prompt_markdown": "prompt.md",
            "queue_metrics": metrics,
            "queue_review_prompt": review_prompt,
        },
        "release_demo_review": metadata,
    }
    metrics_payload = {
        "schema": "riotbox.release_demo_listening_review_pack_metrics.v1",
        "schema_version": 1,
        "entry_id": entry_id,
        "source_file": file_record(Path(source_path)),
        "candidate_audio": [file_record(Path(rendered_wav["path"]))],
        "queue_metrics": metrics,
        "queue_review_prompt": review_prompt,
    }
    return {
        "entry_id": entry_id,
        "review_priority": metadata["review_priority"],
        "source_family": metadata["source_family"],
        "review": review,
        "metrics": metrics_payload,
    }


def expected_audible_behavior(candidate: dict[str, Any]) -> str:
    return (
        f"Judge {candidate['entry_id']} for {candidate['source_family']}: "
        f"{candidate['strongest_audible_element']} "
        f"{candidate['source_character']} "
        f"{candidate['demo_worthy_reason']}"
    )


def render_prompt(review: dict[str, Any], candidate: dict[str, Any]) -> str:
    metadata = object_field(review, "release_demo_review", Path("<review>"))
    lines = [
        "# Release-Demo Listening Review Pack",
        "",
        f"- Candidate: `{metadata['entry_id']}`",
        f"- Priority: `{metadata['review_priority']}`",
        f"- Source family: `{metadata['source_family']}`",
        f"- Current verdict state: `{metadata['required_verdict_current_state']}`",
        "- Human verdict: `unverified`",
        "- Quality claim: `false`",
        "",
        "## Listen",
        "",
        f"- Source: `{review['source_file']}`",
        f"- Candidate WAV: `{metadata['rendered_wav']['path']}`",
        f"- Metrics: `{metadata['metrics']['path']}`",
        f"- Existing review prompt: `{metadata['review_prompt']['path']}`",
        "",
        "## What To Judge",
        "",
        f"- Strongest audible element: {candidate['strongest_audible_element']}",
        f"- Source character: {candidate['source_character']}",
        f"- Demo-worthy reason: {metadata['demo_worthy_reason']}",
        f"- Not demo-ready yet: {metadata['not_demo_ready_reason']}",
        "",
        "## Required Listening Questions",
        "",
    ]
    lines.extend(f"{index}. {question}" for index, question in enumerate(metadata["required_listening_questions"], 1))
    lines.extend(
        [
            "",
            "## Verdict Path",
            "",
            "- Record `keep` only when this is genuinely demo-worthy after listening.",
            "- Record `technically_ok_but_musically_weak` when the path works but needs a concrete production fix.",
            "- Record `reject` when the candidate should remain failure evidence.",
            "- Keep this pack unverified until a human has listened.",
            "",
        ]
    )
    return "\n".join(lines)


def render_readme(review: dict[str, Any]) -> str:
    metadata = object_field(review, "release_demo_review", Path("<review>"))
    return (
        "# Release-Demo Listening Review Pack\n\n"
        "This local pack was generated from the P023 release-demo human-review "
        "queue. It is a handoff artifact only; it does not claim product quality.\n\n"
        f"- Candidate: `{metadata['entry_id']}`\n"
        "- Prompt: `prompt.md`\n"
        "- Structured verdict: `review.json`\n"
        "- Metrics: `metrics.json`\n"
    )


def validate_summary(summary: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(summary.get("schema") == SUMMARY_SCHEMA, "schema_mismatch", failures)
    check(summary.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(summary.get("result") == "pass", "result_not_pass", failures)
    check(summary.get("phase") == "P023", "phase_mismatch", failures)
    check(summary.get("quality_claim_allowed") is False, "quality_claim_allowed_must_be_false", failures)
    check(
        summary.get("evidence_mode") in {"fixture_calibration", "live_readiness"},
        "evidence_mode_invalid",
        failures,
    )
    check(isinstance(summary.get("fixture_only"), bool), "fixture_only_invalid", failures)
    check(
        summary.get("demo_bank_state")
        in {"available", "missing", "rejected_non_live_bank"},
        "demo_bank_state_invalid",
        failures,
    )
    check(
        isinstance(summary.get("demo_bank_sha256"), str)
        and len(summary["demo_bank_sha256"]) == 64,
        "demo_bank_sha256_invalid",
        failures,
    )
    packs = summary.get("packs")
    check(isinstance(packs, list) and bool(packs), "packs_missing", failures)
    if isinstance(packs, list):
        check(summary.get("review_pack_count") == len(packs), "review_pack_count_mismatch", failures)
        check(any(pack.get("review_priority") == "high" for pack in packs if isinstance(pack, dict)), "high_priority_pack_missing", failures)
        for index, pack in enumerate(packs):
            validate_summary_pack(pack, index, failures)
    return failures


def validate_summary_pack(pack: Any, index: int, failures: list[str]) -> None:
    if not isinstance(pack, dict):
        failures.append(f"pack_{index}_not_object")
        return
    for field in [
        "entry_id",
        "review_priority",
        "source_family",
        "review_pack",
        "review_json",
        "metrics_json",
        "prompt_markdown",
        "human_verdict",
        "demo_readiness",
    ]:
        check(isinstance(pack.get(field), str) and bool(pack[field]), f"pack_{index}_{field}_missing", failures)
    check(pack.get("human_verdict") == "unverified", f"pack_{index}_human_verdict_not_unverified", failures)
    check(pack.get("demo_readiness") == "unverified", f"pack_{index}_demo_readiness_not_unverified", failures)
    check(pack.get("quality_claim") is False, f"pack_{index}_quality_claim_not_false", failures)


def write_summary(output: Path, summary: dict[str, Any]) -> None:
    write_json(output / "release-demo-listening-review-packs.json", summary)
    (output / "release-demo-listening-review-packs.md").write_text(render_summary(summary))


def render_summary(summary: dict[str, Any]) -> str:
    lines = [
        "# Release-Demo Listening Review Packs",
        "",
        f"- Phase: `{summary['phase']}`",
        f"- Pack count: `{summary['review_pack_count']}`",
        f"- Quality claim allowed: `{str(summary['quality_claim_allowed']).lower()}`",
        f"- Evidence mode: `{summary['evidence_mode']}`",
        f"- Fixture only: `{str(summary['fixture_only']).lower()}`",
        f"- Demo-bank state: `{summary['demo_bank_state']}`",
        "",
        "## Packs",
        "",
    ]
    for pack in summary["packs"]:
        lines.extend(
            [
                f"### `{pack['entry_id']}`",
                "",
                f"- Priority: `{pack['review_priority']}`",
                f"- Source family: `{pack['source_family']}`",
                f"- Review JSON: `{pack['review_json']}`",
                f"- Prompt: `{pack['prompt_markdown']}`",
                "",
            ]
        )
    lines.extend(["## Boundary", "", summary["human_verdict_boundary"], ""])
    return "\n".join(lines)


def artifact_ref(entry: dict[str, Any], field: str, path: Path) -> dict[str, str]:
    value = object_field(entry, field, path)
    return {
        "path": required_string(value, "path", path),
        "sha256": required_string(value, "sha256", path),
    }


def file_record(path: Path) -> dict[str, Any]:
    return {
        "path": str(path),
        "exists": path.is_file(),
        "bytes": path.stat().st_size if path.is_file() else None,
    }


def safe_pack_dir(value: str) -> str:
    return SLUG_RE.sub("-", value).strip("-") or "candidate"


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict), f"{path}: {field} must be object")
    return value


def required_string(data: dict[str, Any], field: str, path: Path) -> str:
    value = data.get(field)
    require(isinstance(value, str) and bool(value.strip()), f"{path}: {field} must be string")
    return value


def string_list(value: Any) -> list[str]:
    require(isinstance(value, list), "expected string list")
    result = []
    for item in value:
        require(isinstance(item, str) and bool(item), "expected non-empty string list values")
        result.append(item)
    return result


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
