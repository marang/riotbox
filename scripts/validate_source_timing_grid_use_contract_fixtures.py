#!/usr/bin/env python3
"""Validate Source Timing grid_use contract parity across JSON surfaces."""

from __future__ import annotations

import copy
import json
import pathlib
import subprocess
import tempfile
from dataclasses import dataclass
from typing import Any


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROBE_FIXTURE = REPO_ROOT / "crates/riotbox-audio/tests/fixtures/source_timing_probe/probe_valid.json"
MANIFEST_FIXTURE = (
    REPO_ROOT / "crates/riotbox-audio/tests/fixtures/listening_manifest/manifest_valid_source_timing.json"
)
SUMMARY_FIXTURE = (
    REPO_ROOT
    / "crates/riotbox-app/tests/fixtures/observer_audio_correlation/summary_invalid_source_timing_grid_use.json"
)


@dataclass(frozen=True)
class GridUseCase:
    grid_use: str
    cue: str
    readiness: str
    requires_manual_confirm: bool
    primary_bpm: float | None
    bpm_agrees_with_grid: bool | None
    beat_status: str
    downbeat_status: str
    confidence_result: str
    drift_status: str
    phrase_status: str
    alternate_evidence_count: int
    grid_bpm_source: str
    grid_bpm_decision_reason: str
    source_timing_bpm_delta: float | None


CASES = [
    GridUseCase(
        grid_use="locked_grid",
        cue="grid locked",
        readiness="ready",
        requires_manual_confirm=False,
        primary_bpm=128.0,
        bpm_agrees_with_grid=True,
        beat_status="stable",
        downbeat_status="stable",
        confidence_result="candidate_cautious",
        drift_status="stable",
        phrase_status="stable",
        alternate_evidence_count=0,
        grid_bpm_source="source_timing",
        grid_bpm_decision_reason="source_timing_ready",
        source_timing_bpm_delta=0.0,
    ),
    GridUseCase(
        grid_use="short_loop_manual_confirm",
        cue="needs confirm",
        readiness="needs_review",
        requires_manual_confirm=True,
        primary_bpm=128.0,
        bpm_agrees_with_grid=True,
        beat_status="stable",
        downbeat_status="stable",
        confidence_result="candidate_cautious",
        drift_status="not_enough_material",
        phrase_status="not_enough_material",
        alternate_evidence_count=0,
        grid_bpm_source="source_timing",
        grid_bpm_decision_reason="source_timing_needs_review_manual_confirm",
        source_timing_bpm_delta=0.0,
    ),
    GridUseCase(
        grid_use="manual_confirm_only",
        cue="needs confirm",
        readiness="needs_review",
        requires_manual_confirm=True,
        primary_bpm=128.0,
        bpm_agrees_with_grid=True,
        beat_status="stable",
        downbeat_status="ambiguous",
        confidence_result="candidate_ambiguous",
        drift_status="stable",
        phrase_status="ambiguous_downbeat",
        alternate_evidence_count=2,
        grid_bpm_source="static_default",
        grid_bpm_decision_reason="source_timing_requires_manual_confirm",
        source_timing_bpm_delta=0.397,
    ),
    GridUseCase(
        grid_use="fallback_grid",
        cue="listen first",
        readiness="weak",
        requires_manual_confirm=False,
        primary_bpm=128.0,
        bpm_agrees_with_grid=True,
        beat_status="stable",
        downbeat_status="weak",
        confidence_result="candidate_cautious",
        drift_status="stable",
        phrase_status="stable",
        alternate_evidence_count=0,
        grid_bpm_source="static_default",
        grid_bpm_decision_reason="source_timing_not_ready",
        source_timing_bpm_delta=0.397,
    ),
    GridUseCase(
        grid_use="unavailable",
        cue="needs confirm",
        readiness="unavailable",
        requires_manual_confirm=True,
        primary_bpm=None,
        bpm_agrees_with_grid=None,
        beat_status="unavailable",
        downbeat_status="unavailable",
        confidence_result="degraded",
        drift_status="unavailable",
        phrase_status="unavailable",
        alternate_evidence_count=0,
        grid_bpm_source="static_default",
        grid_bpm_decision_reason="source_timing_missing_bpm",
        source_timing_bpm_delta=None,
    ),
]


def main() -> int:
    probe_base = read_json(PROBE_FIXTURE)
    manifest_base = read_json(MANIFEST_FIXTURE)
    summary_base = read_json(SUMMARY_FIXTURE)

    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = pathlib.Path(tmp)
        for case in CASES:
            validate_probe_case(probe_base, case, tmpdir)
            validate_manifest_case(manifest_base, case, tmpdir)
            validate_summary_case(summary_base, case, tmpdir)

        reject_mismatched_grid_use(
            build_probe(probe_base, CASES[1]),
            "grid_use must be",
            tmpdir / "probe_invalid_grid_use.json",
            ["python3", "scripts/validate_source_timing_probe_json.py"],
        )
        reject_mismatched_grid_use(
            build_manifest(manifest_base, CASES[1]),
            "source_timing grid_use must be",
            tmpdir / "manifest_invalid_grid_use.json",
            ["python3", "scripts/validate_listening_manifest_json.py"],
            nested=True,
        )
        reject_mismatched_grid_use(
            build_summary(summary_base, CASES[1]),
            "source_timing.grid_use must be",
            tmpdir / "summary_invalid_grid_use.json",
            ["python3", "scripts/validate_observer_audio_summary_json.py"],
            nested=True,
        )

    print(f"source timing grid_use contract fixtures ok: cases={len(CASES)} surfaces=3")
    return 0


def validate_probe_case(base: dict[str, Any], case: GridUseCase, tmpdir: pathlib.Path) -> None:
    path = tmpdir / f"probe_{case.grid_use}.json"
    write_json(path, build_probe(base, case))
    run_validator(["python3", "scripts/validate_source_timing_probe_json.py", str(path)])


def validate_manifest_case(base: dict[str, Any], case: GridUseCase, tmpdir: pathlib.Path) -> None:
    path = tmpdir / f"manifest_{case.grid_use}.json"
    write_json(path, build_manifest(base, case))
    run_validator(["python3", "scripts/validate_listening_manifest_json.py", str(path)])


def validate_summary_case(base: dict[str, Any], case: GridUseCase, tmpdir: pathlib.Path) -> None:
    path = tmpdir / f"summary_{case.grid_use}.json"
    write_json(path, build_summary(base, case))
    run_validator(["python3", "scripts/validate_observer_audio_summary_json.py", str(path)])


def build_probe(base: dict[str, Any], case: GridUseCase) -> dict[str, Any]:
    data = copy.deepcopy(base)
    apply_timing_fields(data, case)
    data["cue"] = case.cue
    data["grid_use"] = case.grid_use
    return data


def build_manifest(base: dict[str, Any], case: GridUseCase) -> dict[str, Any]:
    data = copy.deepcopy(base)
    data["grid_bpm_source"] = case.grid_bpm_source
    data["grid_bpm_decision_reason"] = case.grid_bpm_decision_reason
    data["source_timing_bpm_delta"] = case.source_timing_bpm_delta
    apply_timing_fields(data["source_timing"], case)
    data["source_timing"]["grid_use"] = case.grid_use
    data["source_timing"]["bpm_agrees_with_grid"] = case.bpm_agrees_with_grid
    return data


def build_summary(base: dict[str, Any], case: GridUseCase) -> dict[str, Any]:
    data = copy.deepcopy(base)
    output = data["output_path"]
    output["grid_bpm_source"] = case.grid_bpm_source
    output["grid_bpm_decision_reason"] = case.grid_bpm_decision_reason
    output["source_timing_bpm_delta"] = case.source_timing_bpm_delta
    apply_timing_fields(output["source_timing"], case)
    output["source_timing"]["cue"] = case.cue
    output["source_timing"]["grid_use"] = case.grid_use
    output["source_timing"]["bpm_agrees_with_grid"] = case.bpm_agrees_with_grid
    return data


def apply_timing_fields(target: dict[str, Any], case: GridUseCase) -> None:
    target["readiness"] = case.readiness
    target["requires_manual_confirm"] = case.requires_manual_confirm
    target["primary_bpm"] = case.primary_bpm
    target["beat_status"] = case.beat_status
    target["downbeat_status"] = case.downbeat_status
    target["confidence_result"] = case.confidence_result
    target["drift_status"] = case.drift_status
    target["phrase_status"] = case.phrase_status
    target["alternate_evidence_count"] = case.alternate_evidence_count
    target["warning_codes"] = []


def reject_mismatched_grid_use(
    data: dict[str, Any],
    expected_error: str,
    path: pathlib.Path,
    command_prefix: list[str],
    *,
    nested: bool = False,
) -> None:
    if nested:
        if "output_path" in data:
            data["output_path"]["source_timing"]["grid_use"] = "locked_grid"
        else:
            data["source_timing"]["grid_use"] = "locked_grid"
    else:
        data["grid_use"] = "locked_grid"
    write_json(path, data)
    result = subprocess.run(
        [*command_prefix, str(path)],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode == 0:
        raise SystemExit(f"expected mismatched grid_use fixture to fail: {path}")
    if expected_error not in result.stderr:
        raise SystemExit(
            f"expected {expected_error!r} in validator error for {path}, got:\n{result.stderr}"
        )


def run_validator(command: list[str]) -> None:
    subprocess.run(command, cwd=REPO_ROOT, check=True)


def read_json(path: pathlib.Path) -> dict[str, Any]:
    with path.open() as handle:
        return json.load(handle)


def write_json(path: pathlib.Path, data: dict[str, Any]) -> None:
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    raise SystemExit(main())
