#!/usr/bin/env python3
"""Validate Riotbox 20/10 sound-product future ideas."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.sound_product_2010_future_ideas.v1"
OWNER_TICKET = "RIOTBOX-1210"
RELEASE_BOUNDARY = "non_blocking_for_1_0"
REQUIRED_IDEAS = {
    "producer_loop_take_selection",
    "source_to_performance_memory",
    "live_resampling_self_abuse",
    "stage_impact_macros",
    "taste_aware_demo_generation",
    "set_level_performance_memory",
    "ecosystem_surfaces_preserve_gates",
}
PRODUCT_SPINE = {
    "source_graph",
    "session_model",
    "action_lexicon",
    "queue_commit",
    "audio_engine",
    "audio_qa",
    "demo_bank",
    "external_surfaces",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("ideas", type=Path)
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()

    try:
        ideas = read_json_object(args.ideas)
        summary = validate_ideas(ideas, args.ideas)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(summary, indent=2) + "\n")
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid 20/10 future ideas: {error}", file=sys.stderr)
        return 1

    print(f"valid 20/10 future ideas: {args.ideas}")
    return 0


def validate_ideas(manifest: dict[str, Any], path: Path) -> dict[str, Any]:
    require(manifest.get("schema") == SCHEMA, f"{path}: schema must be {SCHEMA}")
    require(manifest.get("schema_version") == 1, f"{path}: schema_version must be 1")
    require(manifest.get("owner_ticket") == OWNER_TICKET, f"{path}: owner_ticket must be {OWNER_TICKET}")
    require(
        manifest.get("release_gate_boundary") == RELEASE_BOUNDARY,
        f"{path}: release_gate_boundary must be {RELEASE_BOUNDARY}",
    )
    require(
        manifest.get("hidden_taste_oracle_allowed") is False,
        f"{path}: hidden_taste_oracle_allowed must be false",
    )
    require(
        manifest.get("promotion_requires_linear_issue") is True,
        f"{path}: promotion_requires_linear_issue must be true",
    )

    ideas = list_field(manifest, "ideas", path)
    seen_ids: set[str] = set()
    touched_spine: set[str] = set()

    for index, idea in enumerate(ideas):
        require(isinstance(idea, dict), f"{path}: ideas[{index}] must be object")
        idea_id = validate_idea(idea, path, index)
        require(idea_id not in seen_ids, f"{path}: duplicate idea_id {idea_id}")
        seen_ids.add(idea_id)
        touched_spine.update(idea["product_spine_improved"])

    missing = sorted(REQUIRED_IDEAS - seen_ids)
    extra = sorted(seen_ids - REQUIRED_IDEAS)
    require(not missing, f"{path}: missing required ideas: {', '.join(missing)}")
    require(not extra, f"{path}: unknown ideas: {', '.join(extra)}")
    require("source_graph" in touched_spine, f"{path}: future ideas must improve Source Graph")
    require("session_model" in touched_spine, f"{path}: future ideas must improve Session model")
    require("audio_qa" in touched_spine, f"{path}: future ideas must improve audio QA")

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "idea_count": len(ideas),
        "release_blocking_count": 0,
        "product_spine": sorted(touched_spine),
        "required_ideas": sorted(REQUIRED_IDEAS),
    }


def validate_idea(idea: dict[str, Any], path: Path, index: int) -> str:
    prefix = f"{path}: ideas[{index}]"
    idea_id = non_empty_string(idea.get("idea_id"), f"{prefix}.idea_id")
    non_empty_string(idea.get("title"), f"{prefix}.title")
    non_empty_string(idea.get("musical_payoff"), f"{prefix}.musical_payoff")
    non_empty_string(idea.get("promotion_condition"), f"{prefix}.promotion_condition")
    boundary = non_empty_string(idea.get("release_1_0_boundary"), f"{prefix}.release_1_0_boundary")

    require(idea.get("release_blocking") is False, f"{prefix}: 20/10 ideas must not be release_blocking")
    require(
        "non-blocking" in boundary.lower(),
        f"{prefix}.release_1_0_boundary must explicitly say non-blocking",
    )

    spine = string_list(idea, "product_spine_improved", prefix)
    unknown_spine = sorted(set(spine) - PRODUCT_SPINE)
    require(not unknown_spine, f"{prefix}.product_spine_improved unknown: {', '.join(unknown_spine)}")
    require(spine, f"{prefix}.product_spine_improved must be non-empty")
    string_list(idea, "replay_realtime_risk", prefix)
    string_list(idea, "evidence_needed", prefix)
    return idea_id


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def list_field(data: dict[str, Any], field: str, path: Path) -> list[Any]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {field} must be non-empty array")
    return value


def string_list(data: dict[str, Any], field: str, prefix: str) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{prefix}.{field} must be non-empty array")
    for item in value:
        require(isinstance(item, str) and item, f"{prefix}.{field} values must be strings")
    return value


def non_empty_string(value: Any, message: str) -> str:
    require(isinstance(value, str) and value.strip(), message)
    return str(value)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
