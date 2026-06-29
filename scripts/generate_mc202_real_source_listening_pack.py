#!/usr/bin/env python3
"""Generate MC-202 real-source listening packs for dense/non-dense proof."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import wave
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import (
    apply_evidence_boundary,
    evidence_boundary_failure_codes,
)


SCHEMA = "riotbox.mc202_real_source_listening_pack.v1"
DEFAULT_MANIFEST = Path("data/showcase_sources/local_listening_manifest.json")
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-mc202-real-source-listening-pack")
DEFAULT_DATE = "local-mc202-real-source-listening-pack"
DEFAULT_SOURCE_IDS = ("beat03_full", "dh_beatc_kicksnr", "dh_rusharp")
TICKET = "RIOTBOX-1278"


@dataclass(frozen=True)
class RenderCase:
    source: dict[str, Any]
    window: dict[str, Any]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default=DEFAULT_DATE)
    parser.add_argument("--source-id", action="append", default=[])
    parser.add_argument("--keep-output", action="store_true")
    parser.add_argument("--validate-report", type=Path)
    parser.add_argument("--mutation-fixtures", action="store_true")
    args = parser.parse_args()

    if args.validate_report:
        try:
            validate_report(read_json(args.validate_report))
        except ValueError as error:
            print(f"invalid MC-202 real-source listening pack: {error}", file=sys.stderr)
            return 1
        print(f"valid MC-202 real-source listening pack: {args.validate_report}")
        return 0

    repo = repo_root()
    manifest_path = resolve_repo_path(repo, args.manifest)
    output = resolve_repo_path(repo, args.output)
    ensure_safe_output(repo, output)
    manifest = load_manifest(manifest_path)
    source_ids = tuple(args.source_id) if args.source_id else DEFAULT_SOURCE_IDS
    cases = select_cases(repo, manifest, source_ids)

    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    rendered = [render_case(repo, output, args.date, case) for case in cases]
    report = build_report(manifest_path, rendered)
    validate_report(report)
    write_reports(output, report)

    if args.mutation_fixtures:
        run_mutation_fixtures(report)

    print(f"MC-202 real-source listening pack written to {output}")
    return 0


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return Path(result.stdout.strip())


def resolve_repo_path(repo: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo / path


def ensure_safe_output(repo: Path, output: Path) -> None:
    allowed = (repo / "artifacts" / "audio_qa").resolve()
    output_resolved = output.resolve()
    if allowed not in output_resolved.parents:
        raise SystemExit(f"refusing to write outside artifacts/audio_qa: {output}")


def load_manifest(path: Path) -> dict[str, Any]:
    data = read_json(path)
    if data.get("schema") != "riotbox.real_source_listening_showcase.v1":
        raise SystemExit(f"unsupported showcase manifest schema: {data.get('schema')}")
    if data.get("schema_version") != 1:
        raise SystemExit(f"unsupported showcase manifest version: {data.get('schema_version')}")
    return data


def select_cases(
    repo: Path,
    manifest: dict[str, Any],
    source_ids: tuple[str, ...],
) -> list[RenderCase]:
    by_id = {source["id"]: source for source in manifest.get("sources", [])}
    cases: list[RenderCase] = []
    for source_id in source_ids:
        source = by_id.get(source_id)
        if not source:
            raise SystemExit(f"missing source id in manifest: {source_id}")
        source_path = resolve_repo_path(repo, Path(source["path"]))
        if not source_path.is_file():
            raise SystemExit(f"missing source file: {source['path']}")
        windows = source.get("windows") or []
        if not windows:
            raise SystemExit(f"source has no windows: {source_id}")
        window = windows[0]
        if float(window["start_seconds"]) < 0.0 or float(window["duration_seconds"]) <= 0.0:
            raise SystemExit(f"invalid window for source: {source_id}")
        if int(window["bars"]) <= 0:
            raise SystemExit(f"window bars must be positive for source: {source_id}")
        cases.append(RenderCase(source=source, window=window))
    return cases


def render_case(repo: Path, output: Path, date: str, case: RenderCase) -> dict[str, Any]:
    source = case.source
    window = case.window
    case_id = f"{source['id']}_{window['id']}"
    case_dir = output / "cases" / case_id
    render_dir = case_dir / "render"
    review_dir = case_dir / "review"
    render_dir.mkdir(parents=True, exist_ok=True)

    source_path = resolve_repo_path(repo, Path(source["path"]))
    source_window = case_dir / "00_source_window.wav"
    extract_wav_window(
        source_path,
        source_window,
        float(window["start_seconds"]),
        float(window["duration_seconds"]),
    )

    command = [
        "cargo",
        "run",
        "-p",
        "riotbox-audio",
        "--bin",
        "feral_grid_pack",
        "--",
        "--source",
        str(source_path),
        "--output-dir",
        str(render_dir),
        "--date",
        f"{date}-{case_id}",
        "--bpm",
        f"{float(source['bpm']):.6f}",
        "--bars",
        str(int(window["bars"])),
        "--source-window-seconds",
        str(float(window["duration_seconds"])),
        "--source-start-seconds",
        str(float(window["start_seconds"])),
    ]
    run_or_exit(repo, command, case_dir / "render.log")

    render_manifest = read_json(render_dir / "manifest.json")
    review_candidate = render_dir / "05_riotbox_generated_support_mix.wav"
    review_command = [
        sys.executable,
        "scripts/listening_review_workflow.py",
        "pack",
        "--ticket",
        TICKET,
        "--output",
        str(review_dir),
        "--source-file",
        str(source_window),
        "--candidate",
        str(render_dir / "stems" / "03_mc202_bass_pressure.wav"),
        "--candidate",
        str(review_candidate),
        "--technical-status",
        "pass",
        "--automated-musical-fitness-status",
        "unverified",
        "--expected",
        expected_for_source(source, source_family(source["role"])),
    ]
    run_or_exit(repo, review_command, review_dir / "listening-pack.log")

    review = read_json(review_dir / "review.json")
    case_report = {
        "case_id": case_id,
        "source_id": source["id"],
        "source_role": source["role"],
        "source_family": source_family(source["role"]),
        "source_path": source["path"],
        "window_id": window["id"],
        "bpm": float(source["bpm"]),
        "bars": int(window["bars"]),
        "output": str(case_dir),
        "human_verdict": "unverified",
        "demo_readiness": "unverified",
        "quality_proof": False,
        "review": str(review_dir / "review.json"),
        "review_sha256": sha256_file(review_dir / "review.json"),
        "artifacts": {
            "source_window": artifact_record(source_window),
            "mc202_stem": artifact_record(render_dir / "stems" / "03_mc202_bass_pressure.wav"),
            "source_first_mix": artifact_record(render_dir / "04_riotbox_source_first_mix.wav"),
            "generated_support_mix": artifact_record(review_candidate),
            "render_manifest": artifact_record(render_dir / "manifest.json"),
        },
        "mc202_expression_summary": expression_summary(render_manifest),
        "selected_motif": selected_motif(render_manifest),
        "primitive_ab_control": primitive_ab_control(render_manifest),
        "review_prompt": str(review_dir / "prompt.md"),
        "review_artifacts": review["artifacts"],
    }
    case_report["mc202_role_evidence"] = mc202_role_evidence(case_report)
    return apply_evidence_boundary(
        case_report,
        evidence_role="listening_review_scaffold",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "This MC-202 real-source listening pack is review scaffolding. "
            "The primitive A/B control is non-product evidence and may not be "
            "used as fallback musical output."
        ),
    )


def source_family(role: str) -> str:
    if role == "dense_break":
        return "dense_break"
    if role == "tonal_hook":
        return "tonal_hook"
    if role in {"kick_snare_loop", "clean_rhythm"}:
        return "sparse_bass_pressure"
    return "non_dense_break"


def expected_for_source(source: dict[str, Any], family: str) -> str:
    role = role_for_source_family(family)
    return (
        "Review whether MC-202 has source-specific bass/answer pressure, whether "
        "the MC-202 stem is audible against the mix, and whether the primitive "
        "control remains a labeled non-product comparison instead of fallback music. "
        f"Role target: {role['role_label']} - {role['listening_focus']} "
        f"Source expectation: {source['expected_outcome']}"
    )


def mc202_role_evidence(case: dict[str, Any]) -> dict[str, Any]:
    family = str(case.get("source_family"))
    role = role_for_source_family(family)
    expression = object_or_empty(case.get("mc202_expression_summary"))
    motif = object_or_empty(case.get("selected_motif"))
    control = object_or_empty(case.get("primitive_ab_control"))
    failure_codes = []
    if expression.get("contour_origin") != "source_derived_contour":
        failure_codes.append("role_target_contour_not_source_derived")
    if number(motif.get("stem_rms")) <= 0.0005:
        failure_codes.append("role_target_mc202_stem_too_quiet")
    if control.get("ab_delta_passed") is not True:
        failure_codes.append("role_target_ab_delta_missing")
    if role["role"] == "unsupported_source_family":
        failure_codes.append("role_target_unsupported_source_family")
    return {
        "role": role["role"],
        "role_label": role["role_label"],
        "source_family": family,
        "result": "pass" if not failure_codes else "fail",
        "proof_scope": "listening_review_target",
        "source_derived": not failure_codes,
        "quality_proof": False,
        "human_verdict": "unverified",
        "failure_codes": failure_codes,
        "listening_focus": role["listening_focus"],
        "musician_question": role["musician_question"],
    }


def role_for_source_family(family: str) -> dict[str, str]:
    if family == "sparse_bass_pressure":
        return {
            "role": "bass_pressure",
            "role_label": "source-derived bass pressure",
            "listening_focus": "judge whether the MC-202 gives the source physical low-end movement.",
            "musician_question": "Does the MC-202 push bass pressure that belongs to this sparse source?",
        }
    if family == "tonal_hook":
        return {
            "role": "hook_restraint_stab_answer",
            "role_label": "hook-restraint/stab answer",
            "listening_focus": "judge whether the MC-202 answers the tonal hook without faking bass movement.",
            "musician_question": "Does the MC-202 leave room for the hook while adding a useful answer or stab?",
        }
    if family in {"dense_break", "non_dense_break"}:
        return {
            "role": "pressure_answer",
            "role_label": "pressure answer",
            "listening_focus": "judge whether the MC-202 reinforces the break with source-specific answer pressure.",
            "musician_question": "Does the MC-202 answer the break with useful pressure instead of a generic phrase?",
        }
    return {
        "role": "unsupported_source_family",
        "role_label": "unsupported source family",
        "listening_focus": "stop review until the source family has an explicit MC-202 role target.",
        "musician_question": "Is this source family mapped to a valid MC-202 musical job?",
    }


def expression_summary(manifest: dict[str, Any]) -> dict[str, Any]:
    metrics = manifest["metrics"]
    contour = metrics["mc202_source_contour"]
    timing = manifest["source_timing"]
    return {
        "source_timing_readiness": timing["readiness"],
        "source_timing_cue": timing["cue"],
        "source_timing_grid_use": timing["grid_use"],
        "source_timing_confidence_result": timing["confidence_result"],
        "grid_bpm_source": manifest["grid_bpm_source"],
        "grid_bpm_decision_reason": manifest["grid_bpm_decision_reason"],
        "contour_origin": contour["pattern_origin"],
        "contour_hint": contour["contour_hint"],
        "note_budget": contour["note_budget"],
        "touch_boost": contour["touch_boost"],
        "music_bus_boost": contour["music_bus_boost"],
        "low_band_energy_ratio": contour["low_band_energy_ratio"],
        "mid_band_energy_ratio": contour["mid_band_energy_ratio"],
        "high_band_energy_ratio": contour["high_band_energy_ratio"],
        "event_density_per_bar": contour["event_density_per_bar"],
        "decision_reason": contour["reason"],
    }


def selected_motif(manifest: dict[str, Any]) -> dict[str, Any]:
    metrics = manifest["metrics"]
    pressure = metrics["mc202_bass_pressure"]
    stem = metrics["mc202_bass_pressure_stem"]
    mix = metrics["all_lane_mix_movement"]
    return {
        "role": pressure["pressure_role"],
        "mode": pressure["mode"],
        "phrase_shape": pressure["phrase_shape"],
        "note_budget": pressure["note_budget"],
        "touch": pressure["touch"],
        "music_bus_level": pressure["music_bus_level"],
        "pressure_reinforcement_gain": pressure["pressure_reinforcement_gain"],
        "phrase_variation_applied": pressure["phrase_variation_applied"],
        "distinct_bar_profile_count": pressure["distinct_bar_profile_count"],
        "bar_similarity": pressure["bar_similarity"],
        "stem_rms": stem["signal"]["rms"],
        "stem_peak_abs": stem["signal"]["peak_abs"],
        "stem_low_band_rms": stem["low_band"]["rms"],
        "mix_contribution_ratio": mix["mc202_contribution_ratio"],
        "decision_reason": pressure["reason"],
    }


def primitive_ab_control(manifest: dict[str, Any]) -> dict[str, Any]:
    metrics = manifest["metrics"]
    pressure = metrics["mc202_bass_pressure"]
    contour = metrics["mc202_source_contour"]
    delta = float(contour["source_contour_delta_rms"])
    threshold = float(contour["min_required_delta_rms"])
    return {
        "control_kind": "primitive_renderer_non_product_control",
        "product_fallback_allowed": False,
        "primitive_pattern_origin": pressure["pattern_origin"],
        "source_contour_origin": contour["pattern_origin"],
        "control_audio_path": None,
        "control_audio_reason": (
            "The old primitive MC-202 path is not emitted as product audio. "
            "This pack compares the source-contoured MC-202 result against the "
            "non-product primitive/silent control through source_contour_delta_rms."
        ),
        "source_contour_delta_rms": delta,
        "min_required_delta_rms": threshold,
        "ab_delta_passed": delta >= threshold,
    }


def extract_wav_window(source: Path, target: Path, start_seconds: float, duration_seconds: float) -> None:
    with wave.open(str(source), "rb") as src:
        channels = src.getnchannels()
        sample_width = src.getsampwidth()
        sample_rate = src.getframerate()
        start = int(round(start_seconds * sample_rate))
        count = int(round(duration_seconds * sample_rate))
        src.setpos(start)
        frames = src.readframes(count)
    with wave.open(str(target), "wb") as dst:
        dst.setnchannels(channels)
        dst.setsampwidth(sample_width)
        dst.setframerate(sample_rate)
        dst.writeframes(frames)


def artifact_record(path: Path) -> dict[str, Any]:
    return {
        "path": str(path),
        "sha256": sha256_file(path),
        "bytes": path.stat().st_size,
    }


def build_report(manifest_path: Path, cases: list[dict[str, Any]]) -> dict[str, Any]:
    dense_count = sum(1 for case in cases if case["source_family"] == "dense_break")
    non_dense_count = len(cases) - dense_count
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "ticket": TICKET,
        "result": "pass",
        "agent_verdict": "agent_promising",
        "human_verdict": "unverified",
        "demo_readiness": "unverified",
        "manifest": str(manifest_path),
        "case_count": len(cases),
        "dense_case_count": dense_count,
        "non_dense_case_count": non_dense_count,
        "cases": cases,
    }
    return apply_evidence_boundary(
        report,
        evidence_role="listening_review_scaffold",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "RIOTBOX-1278 prepares dense and non-dense real-source MC-202 "
            "listening evidence. Human verdict remains required before any "
            "demo-ready or product-quality claim."
        ),
    )


def validate_report(report: dict[str, Any]) -> None:
    failures: list[str] = []
    require(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    require(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    require(report.get("ticket") == TICKET, "ticket_mismatch", failures)
    require(report.get("result") == "pass", "result_not_pass", failures)
    require(report.get("human_verdict") == "unverified", "human_verdict_not_unverified", failures)
    require(report.get("quality_proof") is False, "quality_proof_must_be_false", failures)
    failures.extend(evidence_boundary_failure_codes(report))

    cases = report.get("cases")
    if not isinstance(cases, list):
        failures.append("cases_not_array")
        cases = []
    dense_count = sum(1 for case in cases if case.get("source_family") == "dense_break")
    non_dense_count = len(cases) - dense_count
    require(report.get("case_count") == len(cases), "case_count_mismatch", failures)
    require(report.get("case_count", 0) >= 2, "case_count_too_low", failures)
    require(report.get("dense_case_count") == dense_count, "dense_case_count_mismatch", failures)
    require(report.get("non_dense_case_count") == non_dense_count, "non_dense_case_count_mismatch", failures)
    require(report.get("dense_case_count", 0) >= 1, "dense_case_missing", failures)
    require(report.get("non_dense_case_count", 0) >= 1, "non_dense_case_missing", failures)
    for index, case in enumerate(cases):
        validate_case(index, case, failures)
    if failures:
        raise ValueError(",".join(failures))


def validate_case(index: int, case: dict[str, Any], failures: list[str]) -> None:
    prefix = f"case_{index}"
    failures.extend(f"{prefix}_{code}" for code in evidence_boundary_failure_codes(case))
    require(case.get("human_verdict") == "unverified", f"{prefix}_human_verdict_not_unverified", failures)
    require(case.get("quality_proof") is False, f"{prefix}_quality_proof_must_be_false", failures)
    require(case.get("source_family") in {"dense_break", "sparse_bass_pressure", "tonal_hook", "non_dense_break"}, f"{prefix}_source_family_invalid", failures)

    expression = object_or_empty(case.get("mc202_expression_summary"))
    for key in (
        "source_timing_readiness",
        "source_timing_grid_use",
        "source_timing_confidence_result",
        "contour_origin",
        "contour_hint",
        "note_budget",
        "low_band_energy_ratio",
        "event_density_per_bar",
        "decision_reason",
    ):
        require(key in expression, f"{prefix}_expression_{key}_missing", failures)
    require(expression.get("contour_origin") == "source_derived_contour", f"{prefix}_expression_not_source_derived", failures)

    motif = object_or_empty(case.get("selected_motif"))
    for key in (
        "role",
        "mode",
        "phrase_shape",
        "note_budget",
        "stem_rms",
        "mix_contribution_ratio",
    ):
        require(key in motif, f"{prefix}_motif_{key}_missing", failures)
    require(number(motif.get("stem_rms")) > 0.0005, f"{prefix}_mc202_stem_too_quiet", failures)
    validate_role_evidence(prefix, case, failures)

    control = object_or_empty(case.get("primitive_ab_control"))
    require(control.get("control_kind") == "primitive_renderer_non_product_control", f"{prefix}_control_kind_invalid", failures)
    require(control.get("product_fallback_allowed") is False, f"{prefix}_control_allows_product_fallback", failures)
    require(control.get("ab_delta_passed") is True, f"{prefix}_control_delta_not_passed", failures)
    require(
        number(control.get("source_contour_delta_rms")) >= number(control.get("min_required_delta_rms")),
        f"{prefix}_control_delta_below_threshold",
        failures,
    )

    artifacts = object_or_empty(case.get("artifacts"))
    for role in ("source_window", "mc202_stem", "source_first_mix", "generated_support_mix", "render_manifest"):
        artifact = object_or_empty(artifacts.get(role))
        require(str(artifact.get("path", "")).strip() != "", f"{prefix}_{role}_path_missing", failures)
        require(len(str(artifact.get("sha256", ""))) == 64, f"{prefix}_{role}_sha_missing", failures)
        require(number(artifact.get("bytes")) > 0, f"{prefix}_{role}_empty", failures)


def validate_role_evidence(prefix: str, case: dict[str, Any], failures: list[str]) -> None:
    evidence = object_or_empty(case.get("mc202_role_evidence"))
    family = str(case.get("source_family"))
    require(evidence.get("source_family") == family, f"{prefix}_mc202_role_source_family_mismatch", failures)
    require(evidence.get("result") == "pass", f"{prefix}_mc202_role_result_not_pass", failures)
    require(
        evidence.get("proof_scope") == "listening_review_target",
        f"{prefix}_mc202_role_proof_scope_invalid",
        failures,
    )
    require(evidence.get("source_derived") is True, f"{prefix}_mc202_role_not_source_derived", failures)
    require(evidence.get("quality_proof") is False, f"{prefix}_mc202_role_claims_quality", failures)
    require(evidence.get("human_verdict") == "unverified", f"{prefix}_mc202_role_human_verdict_not_unverified", failures)
    require(evidence.get("failure_codes") == [], f"{prefix}_mc202_role_failure_codes", failures)
    for field in ("role_label", "listening_focus", "musician_question"):
        require(
            isinstance(evidence.get(field), str) and bool(evidence[field].strip()),
            f"{prefix}_mc202_role_{field}_missing",
            failures,
        )
    if family == "sparse_bass_pressure":
        require(evidence.get("role") == "bass_pressure", f"{prefix}_mc202_role_sparse_not_bass_pressure", failures)
    elif family == "tonal_hook":
        require(
            evidence.get("role") == "hook_restraint_stab_answer",
            f"{prefix}_mc202_role_tonal_not_answer_stab",
            failures,
        )
    elif family in {"dense_break", "non_dense_break"}:
        require(evidence.get("role") == "pressure_answer", f"{prefix}_mc202_role_dense_not_pressure_answer", failures)
    else:
        require(False, f"{prefix}_mc202_role_unsupported_source_family", failures)


def run_mutation_fixtures(report: dict[str, Any]) -> None:
    fixtures = []
    mutated = json.loads(json.dumps(report))
    mutated["quality_proof"] = True
    mutated["evidence_boundary"]["quality_proof"] = True
    fixtures.append(("quality_proof_true", mutated, "quality_proof_must_be_false"))

    mutated = json.loads(json.dumps(report))
    del mutated["cases"][0]["mc202_expression_summary"]["contour_hint"]
    fixtures.append(("missing_expression", mutated, "expression_contour_hint_missing"))

    mutated = json.loads(json.dumps(report))
    mutated["cases"][0]["primitive_ab_control"]["product_fallback_allowed"] = True
    fixtures.append(("fallback_allowed", mutated, "control_allows_product_fallback"))

    mutated = json.loads(json.dumps(report))
    mutated["cases"][0]["selected_motif"]["stem_rms"] = 0.0
    fixtures.append(("silent_mc202", mutated, "mc202_stem_too_quiet"))

    mutated = json.loads(json.dumps(report))
    del mutated["cases"][0]["mc202_role_evidence"]
    fixtures.append(("missing_role_evidence", mutated, "mc202_role_result_not_pass"))

    mutated = json.loads(json.dumps(report))
    mutated["cases"][0]["mc202_role_evidence"]["source_family"] = "stale_family"
    fixtures.append(("stale_role_family", mutated, "mc202_role_source_family_mismatch"))

    mutated = json.loads(json.dumps(report))
    mutated["cases"][0]["mc202_role_evidence"]["quality_proof"] = True
    fixtures.append(("role_claims_quality", mutated, "mc202_role_claims_quality"))

    mutated = json.loads(json.dumps(report))
    mutated["cases"][0]["mc202_role_evidence"]["proof_scope"] = "quality_proof"
    fixtures.append(("role_bad_proof_scope", mutated, "mc202_role_proof_scope_invalid"))

    mutated = json.loads(json.dumps(report))
    for index, case in enumerate(mutated["cases"]):
        if case["source_family"] == "dense_break":
            case["mc202_role_evidence"]["role"] = "bass_pressure"
            fixtures.append(("dense_wrong_role", mutated, f"case_{index}_mc202_role_dense_not_pressure_answer"))
            break

    mutated = json.loads(json.dumps(report))
    for index, case in enumerate(mutated["cases"]):
        if case["source_family"] == "sparse_bass_pressure":
            case["mc202_role_evidence"]["role"] = "pressure_answer"
            fixtures.append(("sparse_wrong_role", mutated, f"case_{index}_mc202_role_sparse_not_bass_pressure"))
            break

    mutated = json.loads(json.dumps(report))
    for index, case in enumerate(mutated["cases"]):
        if case["source_family"] == "tonal_hook":
            case["mc202_role_evidence"]["role"] = "bass_pressure"
            fixtures.append(("tonal_wrong_role", mutated, f"case_{index}_mc202_role_tonal_not_answer_stab"))
            break

    mutated = json.loads(json.dumps(report))
    mutated["case_count"] = 999
    fixtures.append(("stale_case_count", mutated, "case_count_mismatch"))

    for name, fixture, expected in fixtures:
        try:
            validate_report(fixture)
        except ValueError as error:
            if expected not in str(error):
                raise SystemExit(f"mutation {name} failed with wrong error: {error}") from error
            continue
        raise SystemExit(f"mutation {name} unexpectedly passed")


def require(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def object_or_empty(value: object) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def number(value: object) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def run_or_exit(repo: Path, command: list[str], log_path: Path) -> None:
    result = subprocess.run(
        command,
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(result.stdout + ("\n" if result.stdout and result.stderr else "") + result.stderr)
    if result.returncode != 0:
        raise SystemExit(f"command failed; see {log_path}")


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_reports(output: Path, report: dict[str, Any]) -> None:
    (output / "mc202-real-source-listening-pack.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    lines = [
        "# MC-202 Real-Source Listening Pack",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Cases: `{report['case_count']}`",
        f"- Dense / non-dense: `{report['dense_case_count']}` / `{report['non_dense_case_count']}`",
        "",
        "## Cases",
        "",
        "| Case | Family | Role target | MC-202 Stem | Mix | Review | Motif | A/B Control |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for case in report["cases"]:
        motif = case["selected_motif"]
        control = case["primitive_ab_control"]
        role = case["mc202_role_evidence"]
        lines.append(
            f"| `{case['case_id']}` | `{case['source_family']}` | "
            f"`{role['role']}` | "
            f"`{case['artifacts']['mc202_stem']['path']}` | "
            f"`{case['artifacts']['generated_support_mix']['path']}` | "
            f"`{case['review']}` | "
            f"`{motif['mode']}/{motif['phrase_shape']}/{motif['note_budget']}` | "
            f"`delta {control['source_contour_delta_rms']:.6f}` |"
        )
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "This pack prepares human listening review for MC-202 real-source behavior.",
            "The primitive A/B control is non-product evidence. It must not be used as fallback music.",
            "Every case remains `human_verdict: unverified` and `quality_proof: false` until reviewed.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
