#!/usr/bin/env python3
"""Validate Riotbox musical-pass gate policy fixtures."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


SCHEMA = "riotbox.musical_pass_gate_policy.v1"
REQUIRED_STATES = {
    "technical_fail",
    "technical_pass",
    "agent_fail",
    "agent_weak",
    "agent_promising",
    "human_musical_pass",
    "human_musical_fail",
    "calibrated_agent_musical_pass",
}
PASS_STATES = {"human_musical_pass", "calibrated_agent_musical_pass"}
NON_PASS_STATES = REQUIRED_STATES - PASS_STATES
REQUIRED_CALIBRATED_VERDICTS = {"pass", "weak", "fail"}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("policy", type=Path)
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()

    try:
        policy = read_json_object(args.policy)
        summary = validate_policy(policy, args.policy)
        if args.json_output:
            args.json_output.parent.mkdir(parents=True, exist_ok=True)
            args.json_output.write_text(json.dumps(summary, indent=2) + "\n")
    except (OSError, TypeError, ValueError) as error:
        print(f"invalid musical-pass gate policy: {error}", file=sys.stderr)
        return 1

    print(f"valid musical-pass gate policy: {args.policy}")
    return 0


def validate_policy(policy: dict[str, Any], path: Path) -> dict[str, Any]:
    require(policy.get("schema") == SCHEMA, f"{path}: schema must be {SCHEMA}")
    require(policy.get("schema_version") == 1, f"{path}: schema_version must be 1")
    states = policy.get("states")
    require(isinstance(states, dict), f"{path}: states must be object")
    missing = sorted(REQUIRED_STATES - set(states))
    extra = sorted(set(states) - REQUIRED_STATES)
    require(not missing, f"{path}: missing states: {', '.join(missing)}")
    require(not extra, f"{path}: unknown states: {', '.join(extra)}")

    for name in sorted(REQUIRED_STATES):
        validate_state(name, states[name], path)

    for name in sorted(NON_PASS_STATES):
        require(
            states[name]["may_claim_musical_pass"] is False,
            f"{path}: {name} must not claim musical pass",
        )
    for name in sorted(PASS_STATES):
        require(
            states[name]["may_claim_musical_pass"] is True,
            f"{path}: {name} must be an explicit musical-pass state",
        )

    calibrated = states["calibrated_agent_musical_pass"]
    validation = calibrated.get("judge_validation")
    require(isinstance(validation, dict), f"{path}: calibrated state needs judge_validation")
    require(
        int_field(validation, "minimum_label_count") >= 12,
        f"{path}: calibrated state needs at least 12 labels",
    )
    verdicts = set(string_list(validation, "required_verdicts", path, "judge_validation"))
    require(
        REQUIRED_CALIBRATED_VERDICTS <= verdicts,
        f"{path}: calibrated state must require pass, weak, and fail labels",
    )
    require(
        int_field(validation, "minimum_source_families") >= 2,
        f"{path}: calibrated state needs at least 2 source families",
    )
    require(
        validation.get("requires_confusion_examples") is True,
        f"{path}: calibrated state must require confusion examples",
    )
    require(
        validation.get("provider_boundary") == "offline_qa_only",
        f"{path}: calibrated provider boundary must stay offline_qa_only",
    )
    require(
        "human_verdict: unverified" in calibrated["blocked_claims"],
        f"{path}: calibrated state must block pretending to be a human verdict",
    )

    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "state_count": len(states),
        "musical_pass_states": sorted(PASS_STATES),
        "non_pass_states": sorted(NON_PASS_STATES),
        "minimum_calibrated_label_count": validation["minimum_label_count"],
    }


def validate_state(name: str, state: Any, path: Path) -> None:
    require(isinstance(state, dict), f"{path}: {name} must be object")
    require(isinstance(state.get("may_claim_musical_pass"), bool), f"{path}: {name} pass flag")
    for field in (
        "meaning",
        "required_artifacts",
        "allowed_pr_language",
        "blocked_claims",
        "next_action",
    ):
        require(field in state, f"{path}: {name} missing {field}")
    require(isinstance(state["meaning"], str) and state["meaning"], f"{path}: {name} meaning")
    for field in ("required_artifacts", "allowed_pr_language", "blocked_claims"):
        string_list(state, field, path, name)
    require(
        isinstance(state["next_action"], str) and state["next_action"],
        f"{path}: {name} next_action",
    )


def string_list(data: dict[str, Any], field: str, path: Path, prefix: str) -> list[str]:
    value = data.get(field)
    require(isinstance(value, list) and value, f"{path}: {prefix}.{field} must be non-empty array")
    for item in value:
        require(isinstance(item, str) and item, f"{path}: {prefix}.{field} values must be strings")
    return value


def int_field(data: dict[str, Any], field: str) -> int:
    value = data.get(field)
    if isinstance(value, bool) or not isinstance(value, int):
        return 0
    return value


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    sys.exit(main())
