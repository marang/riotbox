#!/usr/bin/env python3
"""Validate the Riotbox 10/10 sound-product readiness rubric."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.sound_product_readiness_rubric.v1"
REQUIRED_STATES = {
    "technical_pass",
    "diagnostic_evidence",
    "agent_promising",
    "human_weak",
    "human_pass",
    "demo_ready",
    "release_ready",
}
PASS_STATES = {"human_pass", "demo_ready", "release_ready"}
NON_PASS_STATES = REQUIRED_STATES - PASS_STATES
REQUIRED_DIMENSIONS = {
    "hook_within_two_bars",
    "hardest_audible_element",
    "source_character",
    "destructive_contrast",
    "bass_drum_pressure",
    "live_triggerability",
    "eight_bar_replay_value",
}
REQUIRED_FIX_CATEGORIES = {
    "source_selection",
    "chop_policy",
    "drum_pressure",
    "bass_movement",
    "mix_bus",
    "destructive_gesture",
    "fixture_threshold",
    "ui_cue",
}
REQUIRED_EVIDENCE_CLASSES = {
    "hardcoded_or_scripted",
    "smoke",
    "regression",
    "diagnostic",
    "automated_promising",
    "human_listening",
    "demo_curation",
    "release_gate",
}
REQUIRED_PHASE_LINKS = {"P021", "P022", "P023"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("rubric", type=Path)
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()

    try:
        rubric = read_json_object(args.rubric)
        summary = validate_rubric(rubric, args.rubric)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(summary, indent=2) + "\n")
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid sound-product readiness rubric: {error}", file=sys.stderr)
        return 1

    print(f"valid sound-product readiness rubric: {args.rubric}")
    return 0


def validate_rubric(rubric: dict[str, Any], path: Path) -> dict[str, Any]:
    require(rubric.get("schema") == SCHEMA, f"{path}: schema must be {SCHEMA}")
    require(rubric.get("schema_version") == 1, f"{path}: schema_version must be 1")
    require(
        rubric.get("hidden_taste_oracle_allowed") is False,
        f"{path}: hidden_taste_oracle_allowed must be false",
    )

    states = object_field(rubric, "states", path)
    missing_states = sorted(REQUIRED_STATES - set(states))
    extra_states = sorted(set(states) - REQUIRED_STATES)
    require(not missing_states, f"{path}: missing states: {', '.join(missing_states)}")
    require(not extra_states, f"{path}: unknown states: {', '.join(extra_states)}")
    for name in sorted(REQUIRED_STATES):
        validate_state(name, states[name], path)
    for name in sorted(NON_PASS_STATES):
        require(
            states[name]["may_claim_product_quality"] is False,
            f"{path}: {name} must not claim product quality",
        )
    for name in sorted(PASS_STATES):
        require(
            states[name]["may_claim_product_quality"] is True,
            f"{path}: {name} must be an explicit product-quality state",
        )

    dimensions = object_field(rubric, "musical_dimensions", path)
    missing_dimensions = sorted(REQUIRED_DIMENSIONS - set(dimensions))
    extra_dimensions = sorted(set(dimensions) - REQUIRED_DIMENSIONS)
    require(not missing_dimensions, f"{path}: missing musical dimensions: {', '.join(missing_dimensions)}")
    require(not extra_dimensions, f"{path}: unknown musical dimensions: {', '.join(extra_dimensions)}")
    for name in sorted(REQUIRED_DIMENSIONS):
        validate_dimension(name, dimensions[name], path)

    evidence = object_field(rubric, "evidence_classes", path)
    missing_evidence = sorted(REQUIRED_EVIDENCE_CLASSES - set(evidence))
    extra_evidence = sorted(set(evidence) - REQUIRED_EVIDENCE_CLASSES)
    require(not missing_evidence, f"{path}: missing evidence classes: {', '.join(missing_evidence)}")
    require(not extra_evidence, f"{path}: unknown evidence classes: {', '.join(extra_evidence)}")
    require(
        evidence["hardcoded_or_scripted"]["may_claim_quality_proof"] is False,
        f"{path}: hardcoded_or_scripted must not claim quality proof",
    )
    for name, item in evidence.items():
        validate_evidence_class(name, item, path)

    reporting = object_field(rubric, "reporting_contract", path)
    require(
        reporting.get("missing_human_verdict_label") == "human_verdict: unverified",
        f"{path}: missing human verdicts must be reported as human_verdict: unverified",
    )
    require(
        reporting.get("weak_human_verdict_blocks_demo_ready") is True,
        f"{path}: weak human verdict must block demo-ready",
    )
    require(
        set(string_list(reporting, "required_weak_output_fix_categories", path))
        == REQUIRED_FIX_CATEGORIES,
        f"{path}: weak-output fix categories must match RIOTBOX-1205",
    )

    links = object_field(rubric, "phase_links", path)
    require(set(links) == REQUIRED_PHASE_LINKS, f"{path}: phase links must be P021/P022/P023")
    for phase, details in links.items():
        validate_phase_link(phase, details, path)

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "state_count": len(states),
        "quality_states": sorted(PASS_STATES),
        "non_quality_states": sorted(NON_PASS_STATES),
        "musical_dimension_count": len(dimensions),
        "fix_categories": sorted(REQUIRED_FIX_CATEGORIES),
        "phase_links": sorted(REQUIRED_PHASE_LINKS),
    }


def validate_state(name: str, state: Any, path: Path) -> None:
    require(isinstance(state, dict), f"{path}: {name} must be object")
    require(isinstance(state.get("may_claim_product_quality"), bool), f"{path}: {name} quality flag")
    for field in (
        "meaning",
        "required_evidence",
        "allowed_language",
        "blocked_claims",
        "next_action",
    ):
        require(field in state, f"{path}: {name} missing {field}")
    require_non_empty_string(state["meaning"], f"{path}: {name}.meaning")
    for field in ("required_evidence", "allowed_language", "blocked_claims"):
        string_list(state, field, path, prefix=name)
    require_non_empty_string(state["next_action"], f"{path}: {name}.next_action")


def validate_dimension(name: str, dimension: Any, path: Path) -> None:
    require(isinstance(dimension, dict), f"{path}: {name} dimension must be object")
    for field in ("question", "ten_out_of_ten", "failure_modes", "evidence"):
        require(field in dimension, f"{path}: {name} missing {field}")
    require_non_empty_string(dimension["question"], f"{path}: {name}.question")
    require_non_empty_string(dimension["ten_out_of_ten"], f"{path}: {name}.ten_out_of_ten")
    string_list(dimension, "failure_modes", path, prefix=name)
    string_list(dimension, "evidence", path, prefix=name)


def validate_evidence_class(name: str, item: Any, path: Path) -> None:
    require(isinstance(item, dict), f"{path}: evidence {name} must be object")
    require(isinstance(item.get("may_claim_quality_proof"), bool), f"{path}: evidence {name} proof flag")
    require_non_empty_string(item.get("meaning"), f"{path}: evidence {name}.meaning")
    string_list(item, "allowed_use", path, prefix=f"evidence {name}")
    string_list(item, "blocked_claims", path, prefix=f"evidence {name}")


def validate_phase_link(phase: str, details: Any, path: Path) -> None:
    require(isinstance(details, dict), f"{path}: {phase} link must be object")
    require_non_empty_string(details.get("role"), f"{path}: {phase}.role")
    string_list(details, "required_outputs", path, prefix=phase)
    string_list(details, "blocked_shortcuts", path, prefix=phase)


def object_field(data: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    value = data.get(field)
    require(isinstance(value, dict) and value, f"{path}: {field} must be non-empty object")
    return value


def string_list(data: dict[str, Any], field: str, path: Path, prefix: str = "") -> list[str]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {prefix}.{field} must be non-empty array")
    for item in value:
        require(isinstance(item, str) and item, f"{path}: {prefix}.{field} values must be strings")
    return value


def require_non_empty_string(value: Any, message: str) -> None:
    require(isinstance(value, str) and value, message)


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
