#!/usr/bin/env python3
"""Create and record structured Riotbox listening-review verdicts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.listening_review.v1"
HUMAN_VERDICTS = {
    "keep",
    "reject",
    "technically_ok_but_musically_weak",
    "inconclusive",
    "unverified",
}
STRONGEST_ELEMENTS = {
    "kick",
    "snare",
    "bass",
    "stab",
    "chop",
    "vocal",
    "silence",
    "none",
}
SOURCE_RECOGNITION = {
    "source_clear",
    "source_transformed_but_present",
    "source_lost",
    "not_applicable",
    "unverified",
}
HOOK_VERDICTS = {
    "clear",
    "weak",
    "missing",
    "annoying",
    "inconclusive",
    "unverified",
}
DEMO_READINESS_CONSEQUENCES = {
    "unverified_until_human_verdict",
    "human_pass_allows_demo_ready_candidate",
    "human_weak_blocks_demo_ready_and_routes_fix",
    "human_fail_blocks_demo_ready_and_routes_fix",
    "inconclusive_blocks_demo_ready_until_relisten",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    pack = subparsers.add_parser("pack")
    pack.add_argument("--ticket", required=True)
    pack.add_argument("--output", type=Path, required=True)
    pack.add_argument("--pr", default="")
    pack.add_argument("--command-line", default="")
    pack.add_argument("--source-file", type=Path)
    pack.add_argument("--candidate", action="append", default=[])
    pack.add_argument("--seed-or-config", default="")
    pack.add_argument("--technical-status", default="unverified")
    pack.add_argument("--automated-musical-fitness-status", default="unverified")
    pack.add_argument("--expected", default="")

    record = subparsers.add_parser("record")
    record.add_argument("--review", type=Path, required=True)
    record.add_argument("--human-verdict", required=True)
    record.add_argument("--strongest-element", required=True)
    record.add_argument("--source-recognition", required=True)
    record.add_argument("--hook-after-two-bars", required=True)
    record.add_argument("--failure-reason", default="")
    record.add_argument("--preferred-direction", default="")
    record.add_argument("--avoid", default="")
    record.add_argument("--concrete-follow-up", default="")
    record.add_argument("--reviewer", default="")

    validate = subparsers.add_parser("validate")
    validate.add_argument("review", type=Path)

    args = parser.parse_args()
    if args.command == "pack":
        create_pack(args)
    elif args.command == "record":
        record_verdict(args)
    else:
        validate_review_file(args.review)
    return 0


def create_pack(args: argparse.Namespace) -> None:
    args.output.mkdir(parents=True, exist_ok=True)
    source_file = normalize_optional_path(args.source_file)
    candidates = [normalize_required_path(Path(path)) for path in args.candidate if path]
    review = {
        "schema": SCHEMA,
        "schema_version": 1,
        "ticket": args.ticket,
        "pr": empty_to_none(args.pr),
        "command": empty_to_none(args.command_line),
        "source_file": str(source_file) if source_file else None,
        "seed_or_config": empty_to_none(args.seed_or_config),
        "technical_status": args.technical_status,
        "automated_musical_fitness_status": args.automated_musical_fitness_status,
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
        "expected_audible_behavior": args.expected
        or "State the expected audible behavior before listening.",
        "artifacts": {
            "candidate_audio": [str(path) for path in candidates],
            "source_audio": str(source_file) if source_file else None,
            "metrics_json": "metrics.json",
            "prompt_markdown": "prompt.md",
        },
    }
    write_json(args.output / "review.json", review)
    write_json(args.output / "metrics.json", build_metrics(source_file, candidates))
    (args.output / "prompt.md").write_text(render_prompt(review))
    (args.output / "README.md").write_text(render_readme(review))
    validate_review(review, allow_unverified=True)
    print(f"listening review pack written: {args.output}")


def record_verdict(args: argparse.Namespace) -> None:
    review = json.loads(args.review.read_text())
    validate_review(review, allow_unverified=True)
    review["human_verdict"] = require_enum(
        "human_verdict",
        args.human_verdict,
        HUMAN_VERDICTS - {"unverified"},
    )
    review["strongest_element"] = require_enum(
        "strongest_element",
        args.strongest_element,
        STRONGEST_ELEMENTS,
    )
    review["source_recognition"] = require_enum(
        "source_recognition",
        args.source_recognition,
        SOURCE_RECOGNITION - {"unverified"},
    )
    review["hook_after_two_bars"] = require_enum(
        "hook_after_two_bars",
        args.hook_after_two_bars,
        HOOK_VERDICTS - {"unverified"},
    )
    review["failure_reason"] = args.failure_reason
    review["preferred_direction"] = args.preferred_direction
    review["avoid"] = split_csv(args.avoid)
    review["concrete_follow_up"] = args.concrete_follow_up
    review["reviewer"] = empty_to_none(args.reviewer)
    review["demo_readiness_consequence"] = demo_readiness_consequence(
        review["human_verdict"]
    )
    validate_review(review)
    write_json(args.review, review)
    summary_path = args.review.with_name("review-summary.md")
    summary_path.write_text(render_summary(review))
    print(f"listening review verdict recorded: {args.review}")


def validate_review_file(path: Path) -> None:
    review = json.loads(path.read_text())
    validate_review(review)
    print(f"valid {SCHEMA}: {path}")


def validate_review(review: dict[str, Any], allow_unverified: bool = False) -> None:
    required = {
        "schema",
        "schema_version",
        "ticket",
        "technical_status",
        "automated_musical_fitness_status",
        "human_verdict",
        "strongest_element",
        "source_recognition",
        "hook_after_two_bars",
        "failure_reason",
        "preferred_direction",
        "avoid",
        "concrete_follow_up",
        "expected_audible_behavior",
        "artifacts",
    }
    missing = sorted(required - set(review))
    if missing:
        raise SystemExit(f"review missing required fields: {', '.join(missing)}")
    if review["schema"] != SCHEMA:
        raise SystemExit(f"review schema must be {SCHEMA}")
    if review["schema_version"] != 1:
        raise SystemExit("review schema_version must be 1")
    verdicts = HUMAN_VERDICTS if allow_unverified else HUMAN_VERDICTS - {"unverified"}
    if review["human_verdict"] not in verdicts:
        raise SystemExit(f"invalid human_verdict: {review['human_verdict']}")
    require_enum("strongest_element", review["strongest_element"], STRONGEST_ELEMENTS)
    source_values = SOURCE_RECOGNITION if allow_unverified else SOURCE_RECOGNITION - {"unverified"}
    hook_values = HOOK_VERDICTS if allow_unverified else HOOK_VERDICTS - {"unverified"}
    require_enum("source_recognition", review["source_recognition"], source_values)
    require_enum("hook_after_two_bars", review["hook_after_two_bars"], hook_values)
    if not isinstance(review["avoid"], list):
        raise SystemExit("avoid must be an array")
    if not isinstance(review["artifacts"], dict):
        raise SystemExit("artifacts must be an object")
    if "demo_readiness_consequence" in review:
        require_enum(
            "demo_readiness_consequence",
            review["demo_readiness_consequence"],
            DEMO_READINESS_CONSEQUENCES,
        )


def build_metrics(source_file: Path | None, candidates: list[Path]) -> dict[str, Any]:
    return {
        "schema": "riotbox.listening_review.metrics.v1",
        "schema_version": 1,
        "source_file": file_record(source_file) if source_file else None,
        "candidate_audio": [file_record(path) for path in candidates],
        "candidate_count": len(candidates),
    }


def file_record(path: Path) -> dict[str, Any]:
    return {
        "path": str(path),
        "exists": path.is_file(),
        "bytes": path.stat().st_size if path.is_file() else None,
    }


def render_prompt(review: dict[str, Any]) -> str:
    candidates = review["artifacts"]["candidate_audio"]
    lines = [
        "# Riotbox Listening Review",
        "",
        f"- Ticket: `{review['ticket']}`",
        f"- PR: `{review.get('pr') or 'none'}`",
        f"- Technical status: `{review['technical_status']}`",
        "- Automated musical fitness status: "
        f"`{review['automated_musical_fitness_status']}`",
        "- Human verdict: `unverified`",
        "",
        "## Expected Audible Behavior",
        "",
        review["expected_audible_behavior"],
        "",
        "## Listen",
        "",
    ]
    if review["artifacts"].get("source_audio"):
        lines.append(f"- Source: `{review['artifacts']['source_audio']}`")
    if candidates:
        lines.extend(f"- Candidate: `{path}`" for path in candidates)
    else:
        lines.append("- Candidate: add candidate audio path before review.")
    lines.extend(
        [
            "",
            "## Questions",
            "",
            "1. What is the strongest element: kick, snare, bass, stab, chop, vocal, silence, or none?",
            "2. Is the source recognizable, transformed-but-present, lost, or not applicable?",
            "3. Is there a clear hook after two bars?",
            "4. What should Riotbox do more of, and what should it avoid?",
            "5. What concrete follow-up should be created if this is not keep-worthy?",
        ]
    )
    return "\n".join(lines) + "\n"


def render_readme(review: dict[str, Any]) -> str:
    return (
        "# Listening Review Pack\n\n"
        "This is a local human listening-review artifact. It complements "
        "automated musical fitness and must not be treated as hidden memory or "
        "CI-only truth.\n\n"
        f"- Ticket: `{review['ticket']}`\n"
        "- Prompt: `prompt.md`\n"
        "- Structured verdict: `review.json`\n"
        "- Compact metrics: `metrics.json`\n"
    )


def render_summary(review: dict[str, Any]) -> str:
    return (
        "# Listening Review Verdict\n\n"
        f"- Ticket: `{review['ticket']}`\n"
        f"- Human verdict: `{review['human_verdict']}`\n"
        f"- Technical status: `{review['technical_status']}`\n"
        "- Automated musical fitness status: "
        f"`{review['automated_musical_fitness_status']}`\n"
        f"- Strongest element: `{review['strongest_element']}`\n"
        f"- Source recognition: `{review['source_recognition']}`\n"
        f"- Hook after two bars: `{review['hook_after_two_bars']}`\n"
        f"- Failure reason: {review['failure_reason'] or 'none'}\n"
        f"- Preferred direction: {review['preferred_direction'] or 'none'}\n"
        f"- Avoid: {', '.join(review['avoid']) if review['avoid'] else 'none'}\n"
        f"- Concrete follow-up: {review['concrete_follow_up'] or 'none'}\n"
        "- Demo-readiness consequence: "
        f"`{review.get('demo_readiness_consequence', 'unknown')}`\n"
    )


def normalize_optional_path(path: Path | None) -> Path | None:
    if path is None or str(path) in {"", "."}:
        return None
    return normalize_required_path(path)


def normalize_required_path(path: Path) -> Path:
    return path.expanduser()


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")


def require_enum(name: str, value: str, allowed: set[str]) -> str:
    if value not in allowed:
        raise SystemExit(f"{name} must be one of: {', '.join(sorted(allowed))}")
    return value


def split_csv(value: str) -> list[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def empty_to_none(value: str) -> str | None:
    return value if value else None


def demo_readiness_consequence(human_verdict: str) -> str:
    return {
        "keep": "human_pass_allows_demo_ready_candidate",
        "technically_ok_but_musically_weak": "human_weak_blocks_demo_ready_and_routes_fix",
        "reject": "human_fail_blocks_demo_ready_and_routes_fix",
        "inconclusive": "inconclusive_blocks_demo_ready_until_relisten",
    }[human_verdict]


if __name__ == "__main__":
    raise SystemExit(main())
