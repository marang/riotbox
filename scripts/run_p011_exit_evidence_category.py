#!/usr/bin/env python3
"""Run bounded P011 exit-evidence proof commands for one manifest category."""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

from validate_p011_exit_evidence_manifest import (
    DEFAULT_MANIFEST,
    load_just_recipes,
    validate_manifest,
)


def main() -> int:
    args = parse_args()
    try:
        manifest = json.loads(args.manifest.read_text())
        validate_manifest(manifest, load_just_recipes())
        commands = commands_for_category(manifest, args.category)
    except (OSError, ValueError, TypeError) as error:
        print(f"invalid P011 evidence category gate: {error}", file=sys.stderr)
        return 1

    for index, command in enumerate(commands, 1):
        print(
            f"p011 evidence category {args.category}: "
            f"command {index}/{len(commands)}: {command}"
        )
        if args.dry_run:
            continue
        result = subprocess.run(shlex.split(command), check=False)
        if result.returncode != 0:
            return result.returncode

    print(f"p011 evidence category {args.category}: ok ({len(commands)} commands)")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Execute one bounded category from the P011 exit evidence manifest."
    )
    parser.add_argument(
        "category",
        help="manifest category id, e.g. replay, or 'all' for every category",
    )
    parser.add_argument(
        "manifest",
        nargs="?",
        type=Path,
        default=DEFAULT_MANIFEST,
        help=f"manifest path, defaults to {DEFAULT_MANIFEST}",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print the commands that would run after validation",
    )
    return parser.parse_args()


def commands_for_category(manifest: dict[str, Any], category_id: str) -> list[str]:
    if category_id == "all":
        all_proofs: list[dict[str, Any]] = []
        for category in manifest["categories"]:
            if category["status"] != "bounded_supported":
                continue
            all_proofs.extend(category["proofs"])
        return unique_commands(all_proofs)

    for category in manifest["categories"]:
        if category["id"] == category_id:
            return unique_commands(category["proofs"])
    known = ", ".join(category["id"] for category in manifest["categories"])
    raise ValueError(f"unknown category {category_id!r}; known categories: {known}")


def unique_commands(proofs: list[dict[str, Any]]) -> list[str]:
    commands: list[str] = []
    seen: set[str] = set()
    for proof in proofs:
        command = proof["command"]
        if command in seen:
            continue
        seen.add(command)
        commands.append(command)
    if not commands:
        raise ValueError("category contains no proof commands")
    return commands


if __name__ == "__main__":
    raise SystemExit(main())
