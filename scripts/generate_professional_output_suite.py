#!/usr/bin/env python3
"""Generate one aggregate report for the professional-output QA suite."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import (
    apply_evidence_boundary,
    evidence_boundary_failure_codes,
    extract_evidence_boundary,
)


SCHEMA = "riotbox.professional_output_suite.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-professional-output-suite")
MIN_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.145
MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO = 0.08
MIN_FERAL_SOURCE_FIRST_MASKING_HEADROOM = 0.04
MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO = 0.46
MIN_FERAL_TR909_RENDERED_SUPPORT_CONTRIBUTION_RATIO = 0.050
MIN_FERAL_TR909_RENDERED_LOW_BAND_RMS = 0.0030
MIN_SOURCE_CHARACTER_WINDOW_RMS_RETENTION_RATIO = 0.98
MIN_SOURCE_CHARACTER_WINDOW_SEARCHED_CASE_COUNT = 3
MIN_SOURCE_CHARACTER_WINDOW_PROMOTED_CASE_COUNT = 1
CHILDREN = {
    "dense_break": "riotbox.dense_break_performance_pack.v1",
    "pro_pressure_source_matrix": "riotbox.pro_pressure_source_matrix.v1",
    "professional_source_wav_pack": "riotbox.professional_source_wav_pack.v1",
    "non_dense_professional_proof_pack": "riotbox.non_dense_professional_proof_pack.v1",
    "professional_output_listening_pack": "riotbox.professional_output_listening_pack.v1",
    "destructive_variation": "riotbox.destructive_variation_professional.v1",
    "rendered_weak_professional_outputs": "riotbox.rendered_weak_professional_outputs.v1",
    "edge_source_professional_diagnostics": "riotbox.edge_source_professional_diagnostics.v1",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-professional-output-suite")
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    output = resolve_repo_path(repo, args.output)
    ensure_safe_output(repo, output)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    render_children(repo, output, args.date)
    report = build_report(output)
    write_reports(output, report)

    if report["result"] != "pass":
        print(
            "professional output suite failed: " + ", ".join(report["failure_codes"]),
            file=sys.stderr,
        )
        return 1
    print(f"professional output suite written to {output}")
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


def render_children(repo: Path, output: Path, date: str) -> None:
    dense = output / "dense-break"
    pro_matrix = output / "pro-pressure-source-matrix"
    source_wav = output / "professional-source-wav-pack"
    non_dense = output / "non-dense-professional-proof-pack"
    listening = output / "professional-output-listening-pack"
    destructive = output / "destructive-variation"
    rendered_weak = output / "rendered-weak-professional-outputs"
    edge_source = output / "edge-source-professional-diagnostics"

    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_dense_break_performance_pack.py",
            "--output",
            str(dense),
            "--date",
            f"{date}-dense-break",
        ],
        dense / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/validate_pro_pressure_source_matrix.py",
            "--output",
            str(pro_matrix),
            "--date",
            f"{date}-source-matrix",
        ],
        pro_matrix / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_professional_source_wav_pack.py",
            "--output",
            str(source_wav),
            "--date",
            f"{date}-source-wav",
        ],
        source_wav / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_professional_output_listening_pack.py",
            "--output",
            str(listening),
            "--professional-wav-pack",
            str(source_wav),
            "--reuse-professional-wav-pack",
            "--date",
            f"{date}-listening",
            "--ticket",
            "RIOTBOX-1199",
        ],
        listening / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_non_dense_professional_proof_pack.py",
            "--output",
            str(non_dense),
            "--professional-source-wav-pack",
            str(source_wav),
            "--reuse-professional-source-wav-pack",
            "--date",
            f"{date}-non-dense",
        ],
        non_dense / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/validate_destructive_variation_professional.py",
            "--json-output",
            str(destructive / "destructive-variation.json"),
            "--markdown-output",
            str(destructive / "destructive-variation.md"),
            str(dense / "performance-report.json"),
        ],
        destructive / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_rendered_weak_professional_outputs.py",
            "--output",
            str(rendered_weak),
        ],
        rendered_weak / "suite-render.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_edge_source_professional_diagnostics.py",
            "--output",
            str(edge_source),
            "--date",
            f"{date}-edge-source",
        ],
        edge_source / "suite-render.log",
    )


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


def build_report(output: Path) -> dict[str, Any]:
    child_specs = [
        ("dense_break", output / "dense-break" / "performance-report.json"),
        (
            "pro_pressure_source_matrix",
            output / "pro-pressure-source-matrix" / "source-matrix-report.json",
        ),
        (
            "professional_source_wav_pack",
            output / "professional-source-wav-pack" / "professional-source-wav-pack.json",
        ),
        (
            "non_dense_professional_proof_pack",
            output
            / "non-dense-professional-proof-pack"
            / "non-dense-professional-proof-pack.json",
        ),
        (
            "professional_output_listening_pack",
            output / "professional-output-listening-pack" / "professional-output-listening-pack.json",
        ),
        (
            "destructive_variation",
            output / "destructive-variation" / "destructive-variation.json",
        ),
        (
            "rendered_weak_professional_outputs",
            output
            / "rendered-weak-professional-outputs"
            / "rendered-weak-professional-outputs.json",
        ),
        (
            "edge_source_professional_diagnostics",
            output
            / "edge-source-professional-diagnostics"
            / "edge-source-professional-diagnostics.json",
        ),
    ]
    children = [summarize_child(child_id, path) for child_id, path in child_specs]
    identity = validate_listening_identity(
        output / "professional-output-listening-pack" / "professional-output-listening-pack.json"
    )
    feral_mix_balance = feral_mix_balance_summary(output)
    source_character_window_selection = source_character_window_selection_summary(output)
    tr909_rendered_drum_pressure = tr909_rendered_drum_pressure_summary(output)
    failures = suite_failure_codes(
        children,
        identity,
        feral_mix_balance,
        source_character_window_selection,
        tr909_rendered_drum_pressure,
    )
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failures else "fail",
        "agent_verdict": "agent_promising" if not failures else "agent_fail",
        "human_verdict": suite_human_verdict(children),
        "child_report_count": len(children),
        "passed_child_report_count": sum(1 for child in children if child["result"] == "pass"),
        "failed_child_report_count": sum(1 for child in children if child["result"] != "pass"),
        "children": children,
        "listening_identity": identity,
        "feral_mix_balance": feral_mix_balance,
        "source_character_window_selection": source_character_window_selection,
        "tr909_rendered_drum_pressure": tr909_rendered_drum_pressure,
        "failure_codes": failures,
    }
    return apply_evidence_boundary(
        report,
        evidence_role="suite_diagnostic",
        source_backed=False,
        source_timing_backed=False,
        scripted_generation=True,
        notes=(
            "Suite aggregates professional-output diagnostics and enforces "
            "evidence-boundary claims. It is not product-quality proof."
        ),
    )


def summarize_child(child_id: str, path: Path) -> dict[str, Any]:
    if not path.is_file():
        missing = {
            "id": child_id,
            "schema": None,
            "result": "fail",
            "agent_verdict": "agent_fail",
            "human_verdict": "unverified",
            "report": str(path),
            "report_sha256": None,
            "failure_codes": ["child_report_missing"],
            "key_metrics": {},
        }
        return apply_evidence_boundary(
            missing,
            evidence_role="diagnostic",
            source_backed=False,
            source_timing_backed=False,
            scripted_generation=True,
        )
    data = read_json(path)
    expected_schema = CHILDREN[child_id]
    failures = [str(code) for code in object_or_empty(data.get("failure_codes")).keys()]
    if isinstance(data.get("failure_codes"), list):
        failures = [str(code) for code in data["failure_codes"]]
    if data.get("schema") != expected_schema:
        failures.append("child_report_schema_mismatch")
    failures.extend(evidence_boundary_failure_codes(data))
    boundary = extract_evidence_boundary(data)
    fallback_selection_strategy_paths = selection_strategy_paths(data, "report")
    return {
        "id": child_id,
        "schema": data.get("schema"),
        "result": data.get("result", "fail"),
        "agent_verdict": data.get("agent_verdict", "agent_fail"),
        "human_verdict": data.get("human_verdict", "unverified"),
        "report": str(path),
        "report_sha256": sha256_file(path),
        "failure_codes": failures,
        "key_metrics": key_metrics(child_id, data),
        "fallback_selection_strategy_count": len(fallback_selection_strategy_paths),
        "fallback_selection_strategy_paths": fallback_selection_strategy_paths,
        "evidence_boundary": boundary,
        "evidence_role": boundary.get("evidence_role"),
        "source_backed": boundary.get("source_backed"),
        "source_timing_backed": boundary.get("source_timing_backed"),
        "scripted_generation": boundary.get("scripted_generation"),
        "quality_proof": boundary.get("quality_proof"),
    }


def strongest_audible_element_key_metrics(
    cases: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "strongest_audible_elements": sorted(
            {
                str(proof.get("strongest_audible_element"))
                for case in cases
                if (proof := object_or_empty(case.get("proof"))).get(
                    "strongest_audible_element"
                )
            }
        ),
        "min_strongest_audible_element_score": min(
            (
                number(
                    object_or_empty(case.get("proof")).get(
                        "strongest_audible_element_score"
                    )
                )
                for case in cases
            ),
            default=0.0,
        ),
        "min_strongest_audible_element_margin": min(
            (
                number(
                    object_or_empty(case.get("proof")).get(
                        "strongest_audible_element_margin"
                    )
                )
                for case in cases
            ),
            default=0.0,
        ),
    }


def selection_strategy_paths(value: Any, path: str) -> list[str]:
    paths: list[str] = []
    if isinstance(value, dict):
        for key, child in value.items():
            child_path = f"{path}.{key}"
            if key == "selection_strategy" and isinstance(child, str):
                if child.startswith("fallback-") or child == "fallback":
                    paths.append(child_path)
            paths.extend(selection_strategy_paths(child, child_path))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            paths.extend(selection_strategy_paths(child, f"{path}[{index}]"))
    return paths


def rebuild_only_source_character_key_metrics(
    cases: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "min_rebuild_only_source_spectral_similarity": min(
            (
                number(
                    object_or_empty(case.get("proof")).get(
                        "rebuild_only_source_spectral_similarity"
                    )
                )
                for case in cases
            ),
            default=0.0,
        ),
        "min_rebuild_only_source_transient_retention": min(
            (
                number(
                    object_or_empty(case.get("proof")).get(
                        "rebuild_only_source_transient_retention"
                    )
                )
                for case in cases
            ),
            default=0.0,
        ),
        "min_rebuild_only_source_character_survival_score": min(
            (
                number(
                    object_or_empty(case.get("proof")).get(
                        "rebuild_only_source_character_survival_score"
                    )
                )
                for case in cases
            ),
            default=0.0,
        ),
        "min_rebuild_only_source_character_survival_margin": min(
            (
                number(
                    object_or_empty(case.get("proof")).get(
                        "rebuild_only_source_character_survival_margin"
                    )
                )
                for case in cases
            ),
            default=0.0,
        ),
    }


def key_metrics(child_id: str, data: dict[str, Any]) -> dict[str, Any]:
    if child_id == "dense_break":
        proof = object_or_empty(data.get("proof"))
        metrics = object_or_empty(data.get("metrics"))
        rebuild_only = object_or_empty(metrics.get("rebuild_only_performance"))
        return {
            "full_to_source_rms_ratio": number(proof.get("full_to_source_rms_ratio")),
            "w30_to_source_rms_ratio": number(proof.get("w30_to_source_rms_ratio")),
            "hook_chop_w30_to_source_margin": number(
                proof.get("hook_chop_w30_to_source_margin")
            ),
            "pressure_to_hook_rms_ratio": number(proof.get("pressure_to_hook_rms_ratio")),
            "restore_to_pressure_rms_ratio": number(proof.get("restore_to_pressure_rms_ratio")),
            "rebuild_only_to_full_rms_ratio": number(
                proof.get("rebuild_only_to_full_rms_ratio")
            ),
            "rebuild_only_to_source_correlation": number(
                proof.get("rebuild_only_to_source_correlation")
            ),
            "rebuild_only_source_spectral_similarity": number(
                proof.get("rebuild_only_source_spectral_similarity")
            ),
            "rebuild_only_source_transient_retention": number(
                proof.get("rebuild_only_source_transient_retention")
            ),
            "rebuild_only_source_character_survival_score": number(
                proof.get("rebuild_only_source_character_survival_score")
            ),
            "rebuild_only_source_character_survival_margin": number(
                proof.get("rebuild_only_source_character_survival_margin")
            ),
            "hook_chop_selection_source_derived": number(
                proof.get("hook_chop_selection_source_derived")
            ),
            "hook_chop_static_distance_frames": number(
                proof.get("hook_chop_static_distance_frames")
            ),
            "hook_chop_offset_distance_frames": number(
                proof.get("hook_chop_offset_distance_frames")
            ),
            "hook_chop_riff_unique_source_offset_count": number(
                proof.get("hook_chop_riff_unique_source_offset_count")
            ),
            "hook_chop_riff_hit_pattern_source_derived": number(
                proof.get("hook_chop_riff_hit_pattern_source_derived")
            ),
            "hook_chop_riff_hit_count": number(
                proof.get("hook_chop_riff_hit_count")
            ),
            "hook_chop_riff_velocity_span": number(
                proof.get("hook_chop_riff_velocity_span")
            ),
            "hook_chop_riff_reverse_count": number(
                proof.get("hook_chop_riff_reverse_count")
            ),
            "hook_chop_source_character_score_floor": number(
                proof.get("hook_chop_source_character_score_floor")
            ),
            "hook_chop_source_character_score_span": number(
                proof.get("hook_chop_source_character_score_span")
            ),
            "destructive_gesture_source_derived": number(
                proof.get("destructive_gesture_source_derived")
            ),
            "destructive_static_distance_frames": number(
                proof.get("destructive_static_distance_frames")
            ),
            "destructive_offset_distance_frames": number(
                proof.get("destructive_offset_distance_frames")
            ),
            "dense_destructive_pressure_lift_ratio": number(
                proof.get("pressure_lift_bar5_to_bar4_rms_ratio")
            ),
            "arrangement_role_order_source_derived": number(
                proof.get("arrangement_role_order_source_derived")
            ),
            "arrangement_role_candidate_count": number(
                proof.get("arrangement_role_candidate_count")
            ),
            "arrangement_scripted_role_distance": number(
                proof.get("arrangement_scripted_role_distance")
            ),
            "dense_answer_bite_source_derived": number(
                proof.get("dense_answer_bite_source_derived")
            ),
            "dense_answer_bite_scripted_role_distance": number(
                proof.get("dense_answer_bite_scripted_role_distance")
            ),
            "dense_answer_bite_stab_score": number(
                proof.get("dense_answer_bite_stab_score")
            ),
            "dense_answer_bite_stab_margin": number(
                proof.get("dense_answer_bite_stab_margin")
            ),
            "dense_answer_bite_pressure_snap_ratio": number(
                proof.get("dense_answer_bite_pressure_snap_ratio")
            ),
            "dense_answer_bite_score": number(
                proof.get("dense_answer_bite_score")
            ),
            "mix_treatment_source_derived": number(
                proof.get("mix_treatment_source_derived")
            ),
            "mix_treatment_fixed_distance": number(
                proof.get("mix_treatment_fixed_distance")
            ),
            "mix_treatment_output_contrast_ratio": number(
                proof.get("mix_treatment_output_contrast_ratio")
            ),
            "tail_shape_source_derived": number(
                proof.get("tail_shape_source_derived")
            ),
            "tail_shape_fixed_distance": number(
                proof.get("tail_shape_fixed_distance")
            ),
            "tail_shape_output_contrast_ratio": number(
                proof.get("tail_shape_output_contrast_ratio")
            ),
            "strongest_audible_element": str(
                proof.get("strongest_audible_element") or ""
            ),
            "strongest_audible_element_score": number(
                proof.get("strongest_audible_element_score")
            ),
            "strongest_audible_element_margin": number(
                proof.get("strongest_audible_element_margin")
            ),
            "dense_break_physical_drum_pressure_score": number(
                proof.get("dense_break_physical_drum_pressure_score")
            ),
            "dense_break_snare_pressure_margin": number(
                proof.get("dense_break_snare_pressure_margin")
            ),
            "dense_break_pressure_transient_to_hook_ratio": number(
                proof.get("dense_break_pressure_transient_to_hook_ratio")
            ),
            "rebuild_only_performance_peak_abs": number(rebuild_only.get("peak_abs")),
        }
    if child_id == "pro_pressure_source_matrix":
        arrangement = object_or_empty(data.get("arrangement_summary"))
        cases = list_or_empty(data.get("cases"))
        return {
            "case_count": int(number(data.get("case_count"))),
            "passed_case_count": int(number(data.get("passed_case_count"))),
            "failed_case_count": int(number(data.get("failed_case_count"))),
            "arrangement_unique_role_order_signature_count": int(
                number(arrangement.get("unique_role_order_signature_count"))
            ),
            "arrangement_role_order_signatures": sorted(
                str(signature)
                for signature in list_or_empty(arrangement.get("role_order_signatures"))
            ),
            "min_rebuild_only_to_full_rms_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "rebuild_only_to_full_rms_ratio"
                        )
                    )
                    for case in cases
                ),
                default=0.0,
            ),
            "max_rebuild_only_to_source_correlation": max(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "rebuild_only_to_source_correlation"
                        )
                    )
                    for case in cases
                ),
                default=0.0,
            ),
            **rebuild_only_source_character_key_metrics(cases),
            "min_dense_hook_chop_static_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_static_distance_frames"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_offset_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_offset_distance_frames"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_riff_unique_source_offset_count": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_riff_unique_source_offset_count"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_riff_hit_pattern_source_derived": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_riff_hit_pattern_source_derived"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_riff_hit_count": min(
                (
                    number(object_or_empty(case.get("proof")).get("hook_chop_riff_hit_count"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_riff_velocity_span": min(
                (
                    number(object_or_empty(case.get("proof")).get("hook_chop_riff_velocity_span"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_riff_reverse_count": min(
                (
                    number(object_or_empty(case.get("proof")).get("hook_chop_riff_reverse_count"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_source_character_score_floor": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_source_character_score_floor"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_source_character_score_span": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_source_character_score_span"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_w30_to_source_rms_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("w30_to_source_rms_ratio"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_hook_chop_w30_to_source_margin": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_w30_to_source_margin"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_destructive_static_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "destructive_static_distance_frames"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_dense_destructive_offset_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "destructive_offset_distance_frames"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "dense_break"
                ),
                default=0.0,
            ),
            "min_source_derived_arrangement_role_candidate_count": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "arrangement_role_candidate_count"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    in ("dense_break", "tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_arrangement_scripted_role_distance": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "arrangement_scripted_role_distance"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    in ("dense_break", "tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_mix_treatment_fixed_distance": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "mix_treatment_fixed_distance"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    in ("dense_break", "tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_mix_treatment_output_contrast_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "mix_treatment_output_contrast_ratio"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    in ("dense_break", "tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_tail_shape_fixed_distance": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "tail_shape_fixed_distance"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    in ("dense_break", "tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_tail_shape_output_contrast_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "tail_shape_output_contrast_ratio"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    in ("dense_break", "tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            **strongest_audible_element_key_metrics(cases),
            "min_sparse_bass_movement_static_distance_hz": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "sparse_bass_movement_static_distance_hz"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "min_sparse_bass_movement_frequency_span_hz": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "sparse_bass_movement_frequency_span_hz"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "min_sparse_pressure_low_band_lift_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("pressure_low_band_lift_ratio"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "min_sparse_pressure_low_band_share": min(
                (
                    number(object_or_empty(case.get("proof")).get("sparse_pressure_low_band_share"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "min_sparse_pressure_low_to_mid_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("sparse_pressure_low_to_mid_ratio"))
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "min_sparse_bass_dominance_margin": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "strongest_audible_element_margin"
                        )
                    )
                    for case in cases
                    if object_or_empty(case.get("pressure_lift_policy")).get("source_family")
                    == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
        }
    if child_id == "professional_source_wav_pack":
        cases = list_or_empty(data.get("cases"))
        peaks = [
            number(object_or_empty(case.get("metrics")).get("rebuild_only_performance_peak_abs"))
            for case in cases
        ]
        return {
            "case_count": int(number(data.get("case_count"))),
            "passed_case_count": int(number(data.get("passed_case_count"))),
            "max_rebuild_only_performance_peak_abs": max(peaks) if peaks else 0.0,
            "min_rebuild_only_to_full_rms_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "rebuild_only_to_full_rms_ratio"
                        )
                    )
                    for case in cases
                ),
                default=0.0,
            ),
            **rebuild_only_source_character_key_metrics(cases),
            "tonal_hook_chop_static_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_static_distance_frames"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_offset_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_offset_distance_frames"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_riff_unique_source_offset_count": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_riff_unique_source_offset_count"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_riff_hit_pattern_source_derived": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_riff_hit_pattern_source_derived"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_riff_hit_count": min(
                (
                    number(object_or_empty(case.get("proof")).get("hook_chop_riff_hit_count"))
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_riff_velocity_span": min(
                (
                    number(object_or_empty(case.get("proof")).get("hook_chop_riff_velocity_span"))
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_riff_reverse_count": min(
                (
                    number(object_or_empty(case.get("proof")).get("hook_chop_riff_reverse_count"))
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_source_character_score_floor": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_source_character_score_floor"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_source_character_score_span": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_source_character_score_span"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_w30_to_source_rms_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("w30_to_source_rms_ratio"))
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_chop_w30_to_source_margin": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "hook_chop_w30_to_source_margin"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_hook_restraint_pressure_lift_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "pressure_low_band_lift_ratio"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_mix_bus_mc202_to_w30_rms_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "mc202_to_w30_rms_ratio"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_destructive_static_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "destructive_static_distance_frames"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "tonal_destructive_offset_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "destructive_offset_distance_frames"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "tonal_hook"
                ),
                default=0.0,
            ),
            "min_source_derived_arrangement_role_candidate_count": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "arrangement_role_candidate_count"
                        )
                    )
                    for case in cases
                    if case.get("source_family") in ("tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_arrangement_scripted_role_distance": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "arrangement_scripted_role_distance"
                        )
                    )
                    for case in cases
                    if case.get("source_family") in ("tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_mix_treatment_fixed_distance": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "mix_treatment_fixed_distance"
                        )
                    )
                    for case in cases
                    if case.get("source_family") in ("tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_mix_treatment_output_contrast_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "mix_treatment_output_contrast_ratio"
                        )
                    )
                    for case in cases
                    if case.get("source_family") in ("tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_tail_shape_fixed_distance": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "tail_shape_fixed_distance"
                        )
                    )
                    for case in cases
                    if case.get("source_family") in ("tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            "min_source_derived_tail_shape_output_contrast_ratio": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "tail_shape_output_contrast_ratio"
                        )
                    )
                    for case in cases
                    if case.get("source_family") in ("tonal_hook", "sparse_bass_pressure")
                ),
                default=0.0,
            ),
            **strongest_audible_element_key_metrics(cases),
            "sparse_bass_movement_static_distance_hz": max(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "sparse_bass_movement_static_distance_hz"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "sparse_bass_movement_frequency_span_hz": max(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "sparse_bass_movement_frequency_span_hz"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "sparse_pressure_low_band_lift_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("pressure_low_band_lift_ratio"))
                    for case in cases
                    if case.get("source_family") == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "sparse_pressure_low_band_share": min(
                (
                    number(object_or_empty(case.get("proof")).get("sparse_pressure_low_band_share"))
                    for case in cases
                    if case.get("source_family") == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "sparse_pressure_low_to_mid_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("sparse_pressure_low_to_mid_ratio"))
                    for case in cases
                    if case.get("source_family") == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
            "sparse_bass_dominance_margin": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "strongest_audible_element_margin"
                        )
                    )
                    for case in cases
                    if case.get("source_family") == "sparse_bass_pressure"
                ),
                default=0.0,
            ),
        }
    if child_id == "professional_output_listening_pack":
        cases = list_or_empty(data.get("cases"))
        mc202_gate = object_or_empty(data.get("mc202_source_composed_review_gate"))
        return {
            "case_count": int(number(data.get("case_count"))),
            "source_families": sorted(
                str(case.get("source_family", "unknown"))
                for case in cases
            ),
            "demo_reason_count": sum(
                1
                for case in cases
                if str(case.get("demo_worthy_reason", "")).startswith("Worth review:")
            ),
            "not_demo_ready_reason_count": sum(
                1
                for case in cases
                if str(case.get("not_demo_worthy_reason", "")).startswith(
                    "Not demo-ready yet:"
                )
            ),
            "mc202_source_composed_gate_result": str(mc202_gate.get("result")),
            "mc202_source_composed_case_count": int(number(mc202_gate.get("source_composed_case_count"))),
            "mc202_dense_break_case_count": int(number(mc202_gate.get("dense_break_case_count"))),
            "mc202_non_dense_break_case_count": int(number(mc202_gate.get("non_dense_break_case_count"))),
            "mc202_quality_proof": mc202_gate.get("quality_proof"),
        }
    if child_id == "non_dense_professional_proof_pack":
        return {
            "case_count": int(number(data.get("case_count"))),
            "passed_case_count": int(number(data.get("passed_case_count"))),
            "source_families": sorted(
                str(case.get("source_family", "unknown"))
                for case in list_or_empty(data.get("cases"))
            ),
        }
    if child_id == "destructive_variation":
        metrics = object_or_empty(data.get("metrics"))
        return {
            "dropout_to_stutter_rms_ratio": number(
                metrics.get("dropout_to_stutter_rms_ratio")
            ),
            "dropout_silence_to_stutter_rms_ratio": number(
                metrics.get("dropout_silence_to_stutter_rms_ratio")
            ),
            "stutter_to_hook_transient_ratio": number(
                metrics.get("stutter_to_hook_transient_ratio")
            ),
            "restore_to_pressure_rms_ratio": number(
                metrics.get("restore_to_pressure_rms_ratio")
            ),
            "restore_to_dropout_silence_rms_ratio": number(
                metrics.get("restore_to_dropout_silence_rms_ratio")
            ),
        }
    if child_id == "rendered_weak_professional_outputs":
        return {
            "case_count": int(number(data.get("case_count"))),
            "expected_fail_count": sum(
                1
                for case in list_or_empty(data.get("cases"))
                if case.get("validator_result") == "expected_fail"
            ),
        }
    if child_id == "edge_source_professional_diagnostics":
        promotion_summary = object_or_empty(data.get("source_selection_promotion_summary"))
        pad_cases = [
            case
            for case in list_or_empty(data.get("cases"))
            if case.get("source_family") == "pad_noise"
        ]
        return {
            "case_count": int(number(data.get("case_count"))),
            "weak_routed_case_count": int(number(data.get("weak_routed_case_count"))),
            "source_families": sorted(
                str(case.get("source_family", "unknown"))
                for case in list_or_empty(data.get("cases"))
            ),
            "pressure_policy_families": sorted(
                str(
                    object_or_empty(case.get("pressure_lift_policy")).get(
                        "source_family", "unknown"
                    )
                )
                for case in list_or_empty(data.get("cases"))
            ),
            "weak_output_signals": sorted(
                {
                    str(signal)
                    for case in list_or_empty(data.get("cases"))
                    for signal in list_or_empty(case.get("weak_output_signals"))
                }
            ),
            "proposed_fix_categories": sorted(
                {
                    str(category)
                    for case in list_or_empty(data.get("cases"))
                    for category in list_or_empty(case.get("proposed_fix_categories"))
                }
            ),
            "source_selection_promotion_blocked_case_count": int(
                number(promotion_summary.get("blocked_case_count"))
            ),
            "source_selection_promotion_allowed": bool(
                promotion_summary.get("promotion_allowed")
            ),
            "source_selection_blocked_source_families": sorted(
                str(family)
                for family in list_or_empty(
                    promotion_summary.get("blocked_source_families")
                )
            ),
            "source_selection_promotion_blockers": sorted(
                str(blocker)
                for blocker in list_or_empty(promotion_summary.get("blockers"))
            ),
            **strongest_audible_element_key_metrics(list_or_empty(data.get("cases"))),
            **rebuild_only_source_character_key_metrics(list_or_empty(data.get("cases"))),
            "pad_noise_texture_source_derived_count": sum(
                1
                for case in pad_cases
                if number(object_or_empty(case.get("proof")).get("pad_noise_texture_source_derived"))
                >= 1.0
            ),
            "min_pad_noise_texture_candidate_count": min(
                (
                    number(object_or_empty(case.get("proof")).get("pad_noise_texture_candidate_count"))
                    for case in pad_cases
                ),
                default=0.0,
            ),
            "min_pad_noise_texture_gate_static_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "pad_noise_texture_gate_static_distance_frames"
                        )
                    )
                    for case in pad_cases
                ),
                default=0.0,
            ),
            "min_pad_noise_texture_stab_static_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "pad_noise_texture_stab_static_distance_frames"
                        )
                    )
                    for case in pad_cases
                ),
                default=0.0,
            ),
            "min_pad_noise_texture_gate_stab_distance_frames": min(
                (
                    number(
                        object_or_empty(case.get("proof")).get(
                            "pad_noise_texture_gate_stab_distance_frames"
                        )
                    )
                    for case in pad_cases
                ),
                default=0.0,
            ),
            "min_pad_noise_texture_transient_ratio": min(
                (
                    number(object_or_empty(case.get("proof")).get("pad_noise_texture_transient_ratio"))
                    for case in pad_cases
                ),
                default=0.0,
            ),
        }
    return {}


def validate_listening_identity(listening_report: Path) -> dict[str, Any]:
    if not listening_report.is_file():
        return {
            "result": "fail",
            "case_count": 0,
            "failure_codes": ["listening_report_missing"],
            "cases": [],
        }
    report = read_json(listening_report)
    cases = []
    failures = []
    for case in list_or_empty(report.get("cases")):
        case_id = str(case.get("case_id", "unknown"))
        case_failures = []
        candidate = Path(str(case.get("candidate", "")))
        review = Path(str(case.get("review", "")))
        source_report = Path(str(case.get("source_report", "")))
        if not candidate.is_file():
            case_failures.append("candidate_file_missing")
        elif sha256_file(candidate) != case.get("candidate_sha256"):
            case_failures.append("candidate_hash_mismatch")
        if not review.is_file():
            case_failures.append("review_file_missing")
        elif sha256_file(review) != case.get("review_sha256"):
            case_failures.append("review_hash_mismatch")
        if not source_report.is_file():
            case_failures.append("source_report_missing")
        elif sha256_file(source_report) != case.get("source_report_sha256"):
            case_failures.append("source_report_hash_mismatch")
        if case.get("human_verdict") != "unverified":
            case_failures.append("unexpected_human_verdict")
        if case.get("demo_readiness") != "unverified":
            case_failures.append("unexpected_demo_readiness")
        if not str(case.get("demo_worthy_reason", "")).startswith("Worth review:"):
            case_failures.append("demo_worthy_reason_missing")
        if not str(case.get("not_demo_worthy_reason", "")).startswith(
            "Not demo-ready yet:"
        ):
            case_failures.append("not_demo_worthy_reason_missing")
        mc202_gate = object_or_empty(case.get("mc202_source_composed_review_gate"))
        source_composed = mc202_gate.get("source_composed_evidence") is True
        template_only = mc202_gate.get("primitive_or_template_only") is True
        template_blocked = (
            mc202_gate.get("promotion_blocked_until_human_pass") is True
            and mc202_gate.get("template_only_blocks_promotion") is True
        )
        if not source_composed and not (template_only and template_blocked):
            case_failures.append("mc202_source_composed_evidence_missing")
        if template_only and not template_blocked:
            case_failures.append("mc202_template_only_not_blocked")
        if mc202_gate.get("promotion_blocked_until_human_pass") is not True:
            case_failures.append("mc202_promotion_boundary_missing")
        failures.extend(f"{case_id}:{code}" for code in case_failures)
        cases.append(
            {
                "case_id": case_id,
                "source_family": case.get("source_family"),
                "candidate": str(candidate),
                "review": str(review),
                "source_report": str(source_report),
                "demo_readiness": case.get("demo_readiness"),
                "demo_worthy_reason": case.get("demo_worthy_reason"),
                "not_demo_worthy_reason": case.get("not_demo_worthy_reason"),
                "mc202_source_composed_review_gate": mc202_gate,
                "failure_codes": case_failures,
            }
        )
    expected_families = ["dense_break", "sparse_bass_pressure", "tonal_hook"]
    families = sorted(str(case.get("source_family", "unknown")) for case in cases)
    if families != expected_families:
        failures.append("listening_source_family_coverage_mismatch")
    if int(number(report.get("case_count"))) != len(cases):
        failures.append("listening_case_count_mismatch")
    mc202_pack_gate = object_or_empty(report.get("mc202_source_composed_review_gate"))
    if mc202_pack_gate.get("result") != "pass":
        failures.append("mc202_pack_gate_not_pass")
    if int(number(mc202_pack_gate.get("dense_break_case_count"))) < 1:
        failures.append("mc202_dense_break_review_candidate_missing")
    if int(number(mc202_pack_gate.get("non_dense_break_case_count"))) < 1:
        failures.append("mc202_non_dense_review_candidate_missing")
    if mc202_pack_gate.get("quality_proof") is not False:
        failures.append("mc202_pack_gate_claims_quality_proof")
    return {
        "result": "pass" if not failures else "fail",
        "case_count": len(cases),
        "source_families": families,
        "mc202_source_composed_review_gate": mc202_pack_gate,
        "failure_codes": failures,
        "cases": cases,
    }


def feral_mix_balance_summary(output: Path) -> dict[str, Any]:
    cases = []
    for manifest_path in sorted(output.rglob("manifest.json")):
        data = read_json(manifest_path)
        mix_balance = object_or_empty(object_or_empty(data.get("metrics")).get("mix_balance"))
        if not mix_balance:
            continue
        source_first_value = mix_balance.get("source_first_generated_to_source_rms_ratio")
        support_value = mix_balance.get("support_generated_to_source_rms_ratio")
        source_first = number(source_first_value)
        support = number(support_value)
        cases.append(
            {
                "manifest": str(manifest_path.relative_to(output)),
                "source_first_generated_to_source_rms_ratio": source_first,
                "support_generated_to_source_rms_ratio": support,
                "source_first_masking_headroom": (
                    MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
                    - source_first
                ),
                "has_required_mix_balance_fields": is_number(source_first_value)
                and is_number(support_value),
            }
        )
    failures = []
    if not cases:
        failures.append("feral_mix_balance_missing")
    if any(not case["has_required_mix_balance_fields"] for case in cases):
        failures.append("feral_mix_balance_fields_missing")
    if any(
        case["source_first_generated_to_source_rms_ratio"]
        > MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        for case in cases
    ):
        failures.append("feral_source_first_generated_support_masks_source")
    if any(
        case["source_first_masking_headroom"]
        < MIN_FERAL_SOURCE_FIRST_MASKING_HEADROOM
        for case in cases
    ):
        failures.append("feral_source_first_masking_headroom_too_low")
    if any(
        case["support_generated_to_source_rms_ratio"]
        < MIN_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO
        for case in cases
    ):
        failures.append("feral_generated_support_too_buried")
    if any(
        case["support_generated_to_source_rms_ratio"]
        > MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO
        for case in cases
    ):
        failures.append("feral_generated_support_masks_source")
    return {
        "result": "pass" if not failures else "fail",
        "case_count": len(cases),
        "min_support_generated_to_source_rms_ratio": min(
            (case["support_generated_to_source_rms_ratio"] for case in cases),
            default=0.0,
        ),
        "max_support_generated_to_source_rms_ratio": max(
            (case["support_generated_to_source_rms_ratio"] for case in cases),
            default=0.0,
        ),
        "max_source_first_generated_to_source_rms_ratio": max(
            (case["source_first_generated_to_source_rms_ratio"] for case in cases),
            default=0.0,
        ),
        "min_source_first_masking_headroom": min(
            (case["source_first_masking_headroom"] for case in cases),
            default=0.0,
        ),
        "thresholds": {
            "min_support_generated_to_source_rms_ratio": MIN_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
            "max_source_first_generated_to_source_rms_ratio": MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            "min_source_first_masking_headroom": MIN_FERAL_SOURCE_FIRST_MASKING_HEADROOM,
            "max_support_generated_to_source_rms_ratio": MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        },
        "failure_codes": failures,
        "cases": cases,
    }


def source_character_window_selection_summary(output: Path) -> dict[str, Any]:
    cases = []
    failures = []
    for manifest_path in sorted(output.rglob("manifest.json")):
        data = read_json(manifest_path)
        selection = object_or_empty(
            object_or_empty(data.get("metrics")).get("source_character_window_selection")
        )
        if not selection:
            continue
        requested_score = selection.get("requested_head_score")
        selected_score = selection.get("selected_score")
        scanned_count = selection.get("scanned_candidate_count")
        reason = str(selection.get("reason") or "")
        requested_rms = selection.get("requested_head_rms")
        selected_rms = selection.get("selected_rms")
        retention_ratio = selection.get("rms_retention_ratio")
        min_retention_ratio = selection.get("min_rms_retention_ratio")
        search_duration = selection.get("search_duration_seconds")
        selected_duration = selection.get("selected_duration_seconds")
        has_required_fields = (
            is_number(selection.get("requested_start_seconds"))
            and is_number(selection.get("selected_start_seconds"))
            and is_number(selected_duration)
            and is_number(selection.get("search_start_seconds"))
            and is_number(search_duration)
            and is_number(requested_score)
            and is_number(selected_score)
            and is_number(selection.get("score_lift"))
            and is_number(requested_rms)
            and is_number(selected_rms)
            and is_number(retention_ratio)
            and is_number(min_retention_ratio)
            and is_number(scanned_count)
            and bool(reason)
        )
        cases.append(
            {
                "manifest": str(manifest_path.relative_to(output)),
                "requested_start_seconds": number(
                    selection.get("requested_start_seconds")
                ),
                "selected_start_seconds": number(selection.get("selected_start_seconds")),
                "selected_duration_seconds": number(
                    selection.get("selected_duration_seconds")
                ),
                "requested_head_score": number(requested_score),
                "selected_score": number(selected_score),
                "score_lift": number(selection.get("score_lift")),
                "requested_head_rms": number(requested_rms),
                "selected_rms": number(selected_rms),
                "rms_retention_ratio": number(retention_ratio),
                "min_rms_retention_ratio": number(min_retention_ratio),
                "search_start_seconds": number(selection.get("search_start_seconds")),
                "search_duration_seconds": number(search_duration),
                "scanned_candidate_count": int(number(scanned_count)),
                "reason": reason,
                "has_required_source_character_window_fields": has_required_fields,
            }
        )
    if not cases:
        failures.append("source_character_window_selection_missing")
    if any(
        not case["has_required_source_character_window_fields"] for case in cases
    ):
        failures.append("source_character_window_selection_fields_missing")
    if any(case["selected_score"] < case["requested_head_score"] for case in cases):
        failures.append("source_character_window_selection_score_regressed")
    if any(case["scanned_candidate_count"] < 1 for case in cases):
        failures.append("source_character_window_selection_not_scanned")
    if any(
        case["rms_retention_ratio"] + 1e-6 < case["min_rms_retention_ratio"]
        for case in cases
        if case["reason"] == "source_character_window_promoted"
    ):
        failures.append("source_character_window_selection_rms_retention_too_low")
    if any(
        case["reason"]
        not in {"requested_source_window_kept", "source_character_window_promoted"}
        for case in cases
    ):
        failures.append("source_character_window_selection_unknown_reason")
    searched_cases = [
        case
        for case in cases
        if case["scanned_candidate_count"] >= 2
        and case["search_duration_seconds"] > case["selected_duration_seconds"] + 1e-6
    ]
    promoted_cases = [
        case for case in cases if case["reason"] == "source_character_window_promoted"
    ]
    if len(searched_cases) < MIN_SOURCE_CHARACTER_WINDOW_SEARCHED_CASE_COUNT:
        failures.append("source_character_window_selection_search_coverage_too_low")
    if len(promoted_cases) < MIN_SOURCE_CHARACTER_WINDOW_PROMOTED_CASE_COUNT:
        failures.append("source_character_window_selection_promoted_count_too_low")
    return {
        "result": "pass" if not failures else "fail",
        "case_count": len(cases),
        "searched_case_count": len(searched_cases),
        "promoted_case_count": len(promoted_cases),
        "min_required_rms_retention_ratio": MIN_SOURCE_CHARACTER_WINDOW_RMS_RETENTION_RATIO,
        "min_observed_rms_retention_ratio": min(
            (case["rms_retention_ratio"] for case in cases), default=0.0
        ),
        "max_selected_start_seconds": max(
            (case["selected_start_seconds"] for case in cases), default=0.0
        ),
        "max_score_lift": max((case["score_lift"] for case in cases), default=0.0),
        "failure_codes": failures,
        "cases": cases,
    }


def tr909_rendered_drum_pressure_summary(output: Path) -> dict[str, Any]:
    cases = []
    failures = []
    for manifest_path in sorted(output.rglob("manifest.json")):
        data = read_json(manifest_path)
        pressure = object_or_empty(
            object_or_empty(data.get("metrics")).get("tr909_rendered_drum_pressure")
        )
        if not pressure:
            continue
        applied = pressure.get("applied")
        contribution = pressure.get("support_mix_tr909_contribution_ratio")
        source_first = pressure.get("source_first_generated_to_source_rms_ratio")
        support = pressure.get("support_generated_to_source_rms_ratio")
        headroom = pressure.get("source_first_masking_headroom")
        low_band = pressure.get("tr909_low_band_rms")
        min_contribution = pressure.get("min_required_support_mix_tr909_contribution_ratio")
        min_low_band = pressure.get("min_required_tr909_low_band_rms")
        full_low = pressure.get("full_mix_low_band_rms")
        grid_hit = pressure.get("tr909_source_grid_hit_ratio")
        role = str(pressure.get("source_evidence_role") or "")
        origin = str(pressure.get("pattern_origin") or "")
        has_required_fields = (
            applied is True
            and origin == "source_derived"
            and bool(role)
            and is_number(contribution)
            and is_number(source_first)
            and is_number(support)
            and is_number(headroom)
            and is_number(low_band)
            and is_number(min_contribution)
            and is_number(min_low_band)
            and is_number(full_low)
            and is_number(grid_hit)
        )
        cases.append(
            {
                "manifest": str(manifest_path.relative_to(output)),
                "applied": applied is True,
                "pattern_origin": origin,
                "source_evidence_role": role,
                "support_mix_tr909_contribution_ratio": number(contribution),
                "source_first_generated_to_source_rms_ratio": number(source_first),
                "support_generated_to_source_rms_ratio": number(support),
                "source_first_masking_headroom": number(headroom),
                "tr909_low_band_rms": number(low_band),
                "min_required_support_mix_tr909_contribution_ratio": number(
                    min_contribution
                ),
                "min_required_tr909_low_band_rms": number(min_low_band),
                "full_mix_low_band_rms": number(full_low),
                "tr909_source_grid_hit_ratio": number(grid_hit),
                "has_required_tr909_rendered_drum_pressure_fields": has_required_fields,
            }
        )
    if not cases:
        failures.append("tr909_rendered_drum_pressure_missing")
    if any(
        not case["has_required_tr909_rendered_drum_pressure_fields"] for case in cases
    ):
        failures.append("tr909_rendered_drum_pressure_fields_missing")
    if any(case["pattern_origin"] != "source_derived" for case in cases):
        failures.append("tr909_rendered_drum_pressure_not_source_derived")
    if any(
        case["support_mix_tr909_contribution_ratio"]
        < case["min_required_support_mix_tr909_contribution_ratio"]
        for case in cases
    ):
        failures.append("tr909_rendered_drum_pressure_too_buried")
    if any(
        case["tr909_low_band_rms"] < case["min_required_tr909_low_band_rms"]
        for case in cases
    ):
        failures.append("tr909_rendered_drum_pressure_low_band_too_weak")
    if any(
        case["source_first_generated_to_source_rms_ratio"]
        > MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO
        for case in cases
    ):
        failures.append("tr909_rendered_drum_pressure_masks_source_first")
    if any(
        case["support_generated_to_source_rms_ratio"]
        > MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO
        for case in cases
    ):
        failures.append("tr909_rendered_drum_pressure_support_masks_source")
    return {
        "result": "pass" if not failures else "fail",
        "case_count": len(cases),
        "min_support_mix_tr909_contribution_ratio": min(
            (case["support_mix_tr909_contribution_ratio"] for case in cases),
            default=0.0,
        ),
        "min_tr909_low_band_rms": min(
            (case["tr909_low_band_rms"] for case in cases), default=0.0
        ),
        "min_required_support_mix_tr909_contribution_ratio": min(
            (
                case["min_required_support_mix_tr909_contribution_ratio"]
                for case in cases
            ),
            default=0.0,
        ),
        "min_required_tr909_low_band_rms": min(
            (case["min_required_tr909_low_band_rms"] for case in cases),
            default=0.0,
        ),
        "max_source_first_generated_to_source_rms_ratio": max(
            (case["source_first_generated_to_source_rms_ratio"] for case in cases),
            default=0.0,
        ),
        "max_support_generated_to_source_rms_ratio": max(
            (case["support_generated_to_source_rms_ratio"] for case in cases),
            default=0.0,
        ),
        "min_source_first_masking_headroom": min(
            (case["source_first_masking_headroom"] for case in cases),
            default=0.0,
        ),
        "thresholds": {
            "min_support_mix_tr909_contribution_ratio": MIN_FERAL_TR909_RENDERED_SUPPORT_CONTRIBUTION_RATIO,
            "min_tr909_low_band_rms": MIN_FERAL_TR909_RENDERED_LOW_BAND_RMS,
            "max_source_first_generated_to_source_rms_ratio": MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            "max_support_generated_to_source_rms_ratio": MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
        },
        "failure_codes": failures,
        "cases": cases,
    }


def suite_failure_codes(
    children: list[dict[str, Any]],
    identity: dict[str, Any],
    feral_mix_balance: dict[str, Any],
    source_character_window_selection: dict[str, Any],
    tr909_rendered_drum_pressure: dict[str, Any],
) -> list[str]:
    failures = []
    for child in children:
        child_id = child["id"]
        expected_schema = CHILDREN[child_id]
        if child["schema"] != expected_schema:
            failures.append(f"{child_id}:schema_mismatch")
        if child["result"] != "pass":
            failures.append(f"{child_id}:not_passed")
        if child["agent_verdict"] != "agent_promising":
            failures.append(f"{child_id}:agent_not_promising")
        if child["human_verdict"] != "unverified":
            failures.append(f"{child_id}:unexpected_human_verdict")
        if child.get("quality_proof") is not False:
            failures.append(f"{child_id}:quality_proof_not_false")
        if child.get("scripted_generation") is not True:
            failures.append(f"{child_id}:scripted_generation_not_true")
        if int(number(child.get("fallback_selection_strategy_count"))) != 0:
            failures.append(f"{child_id}:fallback_selection_strategy_present")
        if not child.get("evidence_role"):
            failures.append(f"{child_id}:evidence_role_missing")
        for code in child["failure_codes"]:
            failures.append(f"{child_id}:{code}")
    if identity["result"] != "pass":
        failures.extend(f"listening_identity:{code}" for code in identity["failure_codes"])
    if feral_mix_balance["result"] != "pass":
        failures.extend(
            f"feral_mix_balance:{code}" for code in feral_mix_balance["failure_codes"]
        )
    if source_character_window_selection["result"] != "pass":
        failures.extend(
            f"source_character_window_selection:{code}"
            for code in source_character_window_selection["failure_codes"]
        )
    if tr909_rendered_drum_pressure["result"] != "pass":
        failures.extend(
            f"tr909_rendered_drum_pressure:{code}"
            for code in tr909_rendered_drum_pressure["failure_codes"]
        )
    return failures


def suite_human_verdict(children: list[dict[str, Any]]) -> str:
    verdicts = {str(child.get("human_verdict", "unverified")) for child in children}
    return "unverified" if verdicts == {"unverified"} else "mixed"


def read_json(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    if not isinstance(data, dict):
        raise ValueError(f"expected JSON object: {path}")
    return data


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_or_empty(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def is_number(value: Any) -> bool:
    return not isinstance(value, bool) and isinstance(value, (int, float))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_reports(output: Path, report: dict[str, Any]) -> None:
    (output / "professional-output-suite.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = [
        "# Professional Output Suite",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Scripted generation: `{str(report['scripted_generation']).lower()}`",
        f"- Child reports: `{report['passed_child_report_count']}/{report['child_report_count']}` passing",
        "",
        "## Child Reports",
        "",
    ]
    for child in report["children"]:
        lines.append(
            f"- `{child['id']}`: `{child['result']}` "
            f"schema `{child['schema']}` evidence `{child['evidence_role']}` "
            f"quality_proof `{str(child['quality_proof']).lower()}` "
            f"report `{child['report']}`"
        )
        if child["failure_codes"]:
            lines.append(f"  failure_codes: `{', '.join(child['failure_codes'])}`")
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "This suite proves the current deterministic professional-output "
            "reports are present, fresh, hash-bound, and passing together. It "
            "also proves scripted diagnostics do not claim quality proof. It "
            "does not claim product quality or a human musical pass.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
