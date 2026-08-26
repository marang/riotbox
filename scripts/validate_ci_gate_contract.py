#!/usr/bin/env python3
"""Validate the small PR/broad CI split without executing audio generators."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


RECIPE = re.compile(r"^([A-Za-z0-9_-]+)(?:\s[^:]*)?:\s*$")
JUST_CALL = re.compile(r"(?:^|\s)just\s+([A-Za-z0-9_-]+)(?:\s|$)")
GUARD = "scripts/require_broad_audio_qa_access.sh"
EXACT_MIX_CHECK = (
    "scripts/validate_first_playable_jam_probe.sh --exact-mix-dir "
    "artifacts/audio_qa/local-dense-break-live-path-smoke"
)
REDUNDANT_SOURCE_GENERATORS = {
    "pro-pressure-source-matrix-smoke",
    "professional-source-wav-pack-smoke",
    "edge-source-professional-diagnostics-smoke",
    "non-dense-professional-proof-pack-smoke",
    "professional-output-listening-pack-smoke",
    "destructive-variation-professional-smoke",
    "mc202-producer-grade-closeout-smoke",
}
REGISTERED_SOURCE_GENERATORS = REDUNDANT_SOURCE_GENERATORS | {
    "professional-output-suite-smoke",
    "mc202-real-source-listening-pack-smoke",
}
REGISTERED_SOURCE_COMMANDS = {
    "scripts/generate_dense_break_performance_pack.py",
    "scripts/generate_edge_source_professional_diagnostics.py",
    "scripts/generate_mc202_real_source_listening_pack.py",
    "scripts/generate_non_dense_professional_proof_pack.py",
    "scripts/generate_professional_output_listening_pack.py",
    "scripts/generate_professional_output_suite.py",
    "scripts/generate_professional_source_wav_pack.py",
    "scripts/validate_pro_pressure_source_matrix.py",
}


def parse_recipes(text: str) -> dict[str, list[str]]:
    recipes: dict[str, list[str]] = {}
    active: str | None = None
    for line in text.splitlines():
        match = RECIPE.match(line)
        if match:
            active = match.group(1)
            recipes[active] = []
        elif active is not None:
            if line and not line[0].isspace():
                active = None
            else:
                recipes[active].append(line.strip())
    return recipes


def calls(body: list[str]) -> list[str]:
    return [match.group(1) for line in body if (match := JUST_CALL.search(line))]


def closure(recipes: dict[str, list[str]], root: str) -> set[str]:
    reached: set[str] = set()
    pending = [root]
    while pending:
        recipe = pending.pop()
        if recipe in reached:
            continue
        reached.add(recipe)
        pending.extend(calls(recipes.get(recipe, [])))
    return reached


def first_command(recipes: dict[str, list[str]], recipe: str) -> str:
    return next((line for line in recipes.get(recipe, []) if line and not line.startswith("#")), "")


def validate(text: str) -> list[str]:
    recipes = parse_recipes(text)
    failures: list[str] = []
    required = {
        "ci",
        "ci-broad",
        "audio-qa-pr",
        "_audio-qa-pr-unlocked",
        "audio-qa-ci",
        "_audio-qa-ci-unlocked",
    }
    missing = sorted(required - recipes.keys())
    if missing:
        failures.append(f"missing recipes: {', '.join(missing)}")
        return failures

    normal = closure(recipes, "ci")
    escaped = sorted(normal & {"ci-broad", "_ci-broad-extra", "audio-qa-ci", "_audio-qa-ci-unlocked"})
    if escaped:
        failures.append(f"normal ci reaches broad recipes: {', '.join(escaped)}")
    source_generators = sorted(normal & REGISTERED_SOURCE_GENERATORS)
    if source_generators:
        failures.append(f"normal ci reaches registered-source generators: {', '.join(source_generators)}")
    direct_source_commands = sorted(
        command
        for command in REGISTERED_SOURCE_COMMANDS
        if any(command in line for recipe in normal for line in recipes.get(recipe, []))
    )
    if direct_source_commands:
        failures.append(
            "normal ci directly invokes registered-source commands: "
            + ", ".join(direct_source_commands)
        )
    exact_mix_owners = sorted(
        recipe
        for recipe, body in recipes.items()
        if any(line == EXACT_MIX_CHECK for line in body)
    )
    if exact_mix_owners != ["_audio-qa-pr-unlocked"]:
        failures.append(
            "synthetic exact-mix validation must stay beside its PR artifact producer"
        )

    for recipe in ("ci-broad", "audio-qa-ci"):
        if first_command(recipes, recipe) != GUARD:
            failures.append(f"{recipe} must fail closed through {GUARD} before other commands")
    if calls(recipes["ci-broad"]) != ["ci", "_ci-broad-extra"]:
        failures.append("ci-broad must run normal ci exactly once before its broad extras")
    audio_qa_ci_commands = [line for line in recipes["audio-qa-ci"] if line]
    if audio_qa_ci_commands != [
        GUARD,
        "scripts/with_audio_qa_lock.sh broad-audio-qa just _audio-qa-ci-unlocked",
    ]:
        failures.append("audio-qa-ci must guard access and then acquire the broad lock exactly once")

    broad_calls = calls(recipes["_audio-qa-ci-unlocked"])
    if broad_calls.count("professional-output-suite-smoke") != 1:
        failures.append("broad audio QA must generate the professional output suite exactly once")
    redundant = sorted(set(broad_calls) & REDUNDANT_SOURCE_GENERATORS)
    if redundant:
        failures.append(f"broad audio QA repeats covered source generators: {', '.join(redundant)}")
    for required_reuse in (
        "_professional-output-suite-child-fixtures",
        "_mc202-producer-grade-closeout-from-existing",
    ):
        if broad_calls.count(required_reuse) != 1:
            failures.append(f"broad audio QA must invoke {required_reuse} exactly once")
    return failures


def run_fixtures(text: str) -> list[str]:
    failures: list[str] = []
    mutations = {
        "normal_calls_broad": text.replace(
            "    just sidecar-contract-fixtures\n",
            "    just sidecar-contract-fixtures\n    just audio-qa-ci\n",
            1,
        ),
        "broad_missing_guard": text.replace(
            "ci-broad:\n    scripts/require_broad_audio_qa_access.sh\n",
            "ci-broad:\n",
            1,
        ),
        "broad_skips_normal_ci": text.replace("    just ci\n", "", 1),
        "duplicate_source_pack": text.replace(
            "    just professional-output-suite-smoke\n",
            "    just professional-output-suite-smoke\n    just professional-source-wav-pack-smoke\n",
            1,
        ),
        "direct_source_command": text.replace(
            "_audio-qa-pr-unlocked:\n",
            "_audio-qa-pr-unlocked:\n"
            "    python3 scripts/generate_professional_output_suite.py\n",
            1,
        ),
        "broad_reuses_pr_artifact": text.replace(
            "    just source-timing-confirmation-probe\n",
            f"    {EXACT_MIX_CHECK}\n    just source-timing-confirmation-probe\n",
            1,
        ),
    }
    for name, mutation in mutations.items():
        if not validate(mutation):
            failures.append(f"mutation unexpectedly passed: {name}")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--justfile", type=Path, default=Path("Justfile"))
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    text = args.justfile.read_text()
    failures = validate(text)
    if args.fixtures:
        failures.extend(run_fixtures(text))
    if failures:
        print("invalid CI gate contract: " + "; ".join(failures), file=sys.stderr)
        return 1
    print("valid CI gate contract")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
