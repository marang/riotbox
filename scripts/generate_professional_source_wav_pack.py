#!/usr/bin/env python3
"""Generate tonal and sparse professional-output WAV packs."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary, evidence_boundary_failure_codes


SCHEMA = "riotbox.professional_source_wav_pack.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-professional-source-wav-pack")
MIN_TONAL_W30_TO_SOURCE_RMS_RATIO = 0.20
MIN_PROFESSIONAL_W30_TO_SOURCE_RMS_RATIO = 0.22
MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES = 256.0
MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES = 512.0
MIN_HOOK_CHOP_RIFF_UNIQUE_SOURCE_OFFSET_COUNT = 3.0
MIN_HOOK_CHOP_RIFF_HIT_COUNT = 6.0
MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN = 0.20
MIN_HOOK_CHOP_RIFF_REVERSE_COUNT = 1.0
MIN_HOOK_CHOP_SOURCE_CHARACTER_SCORE_FLOOR = 0.60
MIN_HOOK_CHOP_SOURCE_CHARACTER_SCORE_SPAN = 0.10
MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES = 256.0
MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES = 512.0
MIN_MIX_TREATMENT_FIXED_DISTANCE = 0.08
MIN_MIX_TREATMENT_OUTPUT_CONTRAST_RATIO = 2.10
MIN_TAIL_SHAPE_FIXED_DISTANCE = 0.20
MIN_TAIL_SHAPE_OUTPUT_CONTRAST_RATIO = 3.00
MIN_STRONGEST_AUDIBLE_ELEMENT_SCORE = 1.00
MIN_STRONGEST_AUDIBLE_ELEMENT_MARGIN = 0.05
MIN_REBUILD_ONLY_SOURCE_SPECTRAL_SIMILARITY = 0.60
MIN_REBUILD_ONLY_SOURCE_TRANSIENT_RETENTION = 0.45
MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_SCORE = 0.70
MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN = 0.10
MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ = 1.25
MIN_SPARSE_BASS_MOVEMENT_FREQUENCY_SPAN_HZ = 8.0
MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO = 1.60
MIN_SPARSE_PRESSURE_LOW_BAND_SHARE = 0.20
MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO = 1.75
MIN_SPARSE_BASS_DOMINANCE_MARGIN = 0.08
ALLOWED_STRONGEST_AUDIBLE_ELEMENTS = {"kick", "snare", "bass", "stab", "silence", "restore"}
DEFAULT_CASES = [
    {
        "case_id": "tonal_rusharp_120",
        "source_family": "tonal_hook",
        "source": "data/test_audio/examples/DH_RushArp_120_A.wav",
        "bpm": 120.0,
    },
    {
        "case_id": "sparse_kicksnr_120",
        "source_family": "sparse_bass_pressure",
        "source": "data/test_audio/examples/DH_BeatC_KickSnr_120-01.wav",
        "bpm": 120.0,
    },
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-professional-source-wav-pack")
    parser.add_argument("--keep-output", action="store_true")
    parser.add_argument("--validate-report", type=Path)
    parser.add_argument("--require-artifacts", action="store_true")
    parser.add_argument("--mutation-fixtures", action="store_true")
    args = parser.parse_args()

    if args.validate_report:
        try:
            report = read_json(args.validate_report)
            failures = validate_report_failure_codes(
                report,
                require_artifacts=args.require_artifacts,
            )
            if failures:
                raise ValueError(", ".join(failures))
            if args.mutation_fixtures:
                run_mutation_fixtures(report)
        except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
            print(f"invalid professional source WAV pack: {error}", file=sys.stderr)
            return 1
        print(f"valid professional source WAV pack: {args.validate_report}")
        return 0

    repo = repo_root()
    output = resolve_repo_path(repo, args.output)
    ensure_safe_output(repo, output)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    cases = [render_case(repo, output, args.date, case) for case in DEFAULT_CASES]
    failed = [case for case in cases if case["result"] != "pass"]
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failed else "fail",
        "agent_verdict": "agent_promising" if not failed else "agent_weak",
        "human_verdict": "unverified",
        "case_count": len(cases),
        "passed_case_count": len(cases) - len(failed),
        "failed_case_count": len(failed),
        "cases": cases,
    }
    apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "Professional source WAV pack currently reuses the scripted dense-break "
            "performance generator for tonal/sparse sources. It is diagnostic "
            "coverage, not source-family quality proof."
        ),
    )
    validation_failures = validate_report_failure_codes(report, require_artifacts=False)
    if validation_failures:
        report["result"] = "fail"
        report["agent_verdict"] = "agent_weak"
        failed = cases
        report["failure_codes"] = validation_failures
    write_reports(output, report)
    if report["result"] != "pass":
        print(
            "professional source WAV pack failed: "
            + ", ".join(report.get("failure_codes") or [case["case_id"] for case in failed]),
            file=sys.stderr,
        )
        return 1
    print(f"professional source WAV pack written to {output}")
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


def render_case(repo: Path, output: Path, date: str, case: dict) -> dict:
    case_dir = output / case["case_id"]
    command = [
        sys.executable,
        "scripts/generate_dense_break_performance_pack.py",
        "--source",
        case["source"],
        "--bpm",
        f"{case['bpm']:.6f}",
        "--output",
        str(case_dir),
        "--date",
        f"{date}-{case['case_id']}",
    ]
    result = subprocess.run(
        command,
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    (case_dir / "professional-render.log").parent.mkdir(parents=True, exist_ok=True)
    (case_dir / "professional-render.log").write_text(
        result.stdout + ("\n" if result.stdout and result.stderr else "") + result.stderr
    )
    report_path = case_dir / "performance-report.json"
    if not report_path.is_file():
        failure_summary = {
            **case,
            "result": "fail",
            "agent_verdict": "agent_fail",
            "human_verdict": "unverified",
            "output": str(case_dir),
            "failure_codes": ["render_failed_without_report"],
            "returncode": result.returncode,
        }
        return apply_evidence_boundary(
            failure_summary,
            evidence_role="diagnostic",
            source_backed=True,
            source_timing_backed=True,
            scripted_generation=True,
        )

    source_report = json.loads(report_path.read_text())
    proof = source_report["proof"]
    metrics = source_report["metrics"]
    files = source_report["files"]
    pressure_lift_policy = source_report["source_policy"]["pressure_lift_policy"]
    arrangement_policy = source_report["source_policy"]["arrangement_policy"]
    source_report_failures = []
    if source_report["result"] != "pass":
        source_report_failures = [
            f"source_report_{code}" for code in source_report["failure_codes"]
        ]
    family_failures = source_report_failures + family_failure_codes(
        case["source_family"], proof, metrics
    )
    case_summary = {
        **case,
        "result": "pass" if not family_failures else "fail",
        "source_report_result": source_report["result"],
        "source_report_failure_codes": source_report["failure_codes"],
        "agent_verdict": "agent_promising" if not family_failures else "agent_fail",
        "human_verdict": "unverified",
        "output": str(case_dir),
        "audio_files": {
            "source_window": files["source_window"],
            "chop_hook": files["chop_hook"],
            "pressure_lift": files["pressure_lift"],
            "dropout_stutter": files["dropout_stutter"],
            "restore_hit": files["restore_hit"],
            "full_performance": files["full_performance"],
            "rebuild_only_performance": files["rebuild_only_performance"],
        },
        "proof": {
            "w30_to_source_rms_ratio": proof["w30_to_source_rms_ratio"],
            "full_to_source_rms_ratio": proof["full_to_source_rms_ratio"],
            "hook_to_source_transient_ratio": proof["hook_to_source_transient_ratio"],
            "pressure_low_band_lift_ratio": proof["pressure_low_band_lift_ratio"],
            "pressure_to_hook_rms_ratio": proof["pressure_to_hook_rms_ratio"],
            "stutter_to_hook_transient_ratio": proof["stutter_to_hook_transient_ratio"],
            "restore_to_pressure_rms_ratio": proof["restore_to_pressure_rms_ratio"],
            "source_to_performance_correlation": proof["source_to_performance_correlation"],
            "pressure_lift_policy_decision_count": proof[
                "pressure_lift_policy_decision_count"
            ],
            "pressure_lift_bar5_to_bar4_rms_ratio": proof[
                "pressure_lift_bar5_to_bar4_rms_ratio"
            ],
            "hook_chop_selection_source_derived": proof[
                "hook_chop_selection_source_derived"
            ],
            "hook_chop_selection_candidate_count": proof[
                "hook_chop_selection_candidate_count"
            ],
            "hook_chop_static_distance_frames": proof[
                "hook_chop_static_distance_frames"
            ],
            "hook_chop_offset_distance_frames": proof[
                "hook_chop_offset_distance_frames"
            ],
            "hook_chop_riff_unique_source_offset_count": proof[
                "hook_chop_riff_unique_source_offset_count"
            ],
            "hook_chop_riff_hit_pattern_source_derived": proof[
                "hook_chop_riff_hit_pattern_source_derived"
            ],
            "hook_chop_riff_hit_count": proof["hook_chop_riff_hit_count"],
            "hook_chop_riff_velocity_span": proof["hook_chop_riff_velocity_span"],
            "hook_chop_riff_reverse_count": proof["hook_chop_riff_reverse_count"],
            "hook_chop_source_character_score_floor": proof[
                "hook_chop_source_character_score_floor"
            ],
            "hook_chop_source_character_score_mean": proof[
                "hook_chop_source_character_score_mean"
            ],
            "hook_chop_source_character_score_span": proof[
                "hook_chop_source_character_score_span"
            ],
            "destructive_gesture_source_derived": proof[
                "destructive_gesture_source_derived"
            ],
            "destructive_gesture_candidate_count": proof[
                "destructive_gesture_candidate_count"
            ],
            "destructive_static_distance_frames": proof[
                "destructive_static_distance_frames"
            ],
            "destructive_offset_distance_frames": proof[
                "destructive_offset_distance_frames"
            ],
            "mix_treatment_source_derived": proof["mix_treatment_source_derived"],
            "mix_treatment_candidate_count": proof["mix_treatment_candidate_count"],
            "mix_treatment_fixed_distance": proof["mix_treatment_fixed_distance"],
            "mix_treatment_output_contrast_ratio": proof[
                "mix_treatment_output_contrast_ratio"
            ],
            "tail_shape_source_derived": proof["tail_shape_source_derived"],
            "tail_shape_candidate_count": proof["tail_shape_candidate_count"],
            "tail_shape_fixed_distance": proof["tail_shape_fixed_distance"],
            "tail_shape_output_contrast_ratio": proof[
                "tail_shape_output_contrast_ratio"
            ],
            "strongest_audible_element": proof["strongest_audible_element"],
            "strongest_audible_element_score": proof[
                "strongest_audible_element_score"
            ],
            "strongest_audible_element_margin": proof[
                "strongest_audible_element_margin"
            ],
            "strongest_audible_element_candidate_count": proof[
                "strongest_audible_element_candidate_count"
            ],
            "bass_movement_source_derived": proof["bass_movement_source_derived"],
            "sparse_bass_movement_static_distance_hz": proof[
                "sparse_bass_movement_static_distance_hz"
            ],
            "sparse_bass_movement_frequency_span_hz": proof[
                "sparse_bass_movement_frequency_span_hz"
            ],
            "sparse_pressure_low_band_share": proof[
                "sparse_pressure_low_band_share"
            ],
            "sparse_pressure_low_to_mid_ratio": proof[
                "sparse_pressure_low_to_mid_ratio"
            ],
            "arrangement_policy_decision_count": proof["arrangement_policy_decision_count"],
            "arrangement_role_order_source_derived": proof[
                "arrangement_role_order_source_derived"
            ],
            "arrangement_role_candidate_count": proof["arrangement_role_candidate_count"],
            "arrangement_scripted_role_distance": proof[
                "arrangement_scripted_role_distance"
            ],
            "arrangement_pressure_role_count": proof["arrangement_pressure_role_count"],
            "arrangement_destructive_role_count": proof["arrangement_destructive_role_count"],
            "arrangement_failure_count": proof["arrangement_failure_count"],
            "rebuild_only_to_full_rms_ratio": proof["rebuild_only_to_full_rms_ratio"],
            "rebuild_only_to_source_rms_ratio": proof["rebuild_only_to_source_rms_ratio"],
            "rebuild_only_to_source_correlation": proof["rebuild_only_to_source_correlation"],
            "source_on_to_rebuild_only_correlation": proof[
                "source_on_to_rebuild_only_correlation"
            ],
            "rebuild_only_source_spectral_similarity": proof[
                "rebuild_only_source_spectral_similarity"
            ],
            "rebuild_only_source_transient_retention": proof[
                "rebuild_only_source_transient_retention"
            ],
            "rebuild_only_source_character_survival_score": proof[
                "rebuild_only_source_character_survival_score"
            ],
            "rebuild_only_source_character_survival_margin": proof[
                "rebuild_only_source_character_survival_margin"
            ],
        },
        "metrics": {
            "full_performance_rms": metrics["full_performance"]["rms"],
            "full_performance_dbfs": metrics["full_performance"]["dbfs"],
            "full_performance_peak_abs": metrics["full_performance"]["peak_abs"],
            "chop_hook_dbfs": metrics["chop_hook"]["dbfs"],
            "pressure_lift_dbfs": metrics["pressure_lift"]["dbfs"],
            "restore_hit_dbfs": metrics["restore_hit"]["dbfs"],
            "rebuild_only_performance_rms": metrics["rebuild_only_performance"]["rms"],
            "rebuild_only_performance_dbfs": metrics["rebuild_only_performance"]["dbfs"],
            "rebuild_only_performance_peak_abs": metrics["rebuild_only_performance"]["peak_abs"],
        },
        "pressure_lift_policy": pressure_lift_policy,
        "hook_chop_policy": source_report["source_policy"]["hook_chop_policy"],
        "destructive_gesture_policy": source_report["source_policy"][
            "destructive_gesture_policy"
        ],
        "mix_treatment_policy": source_report["source_policy"]["mix_treatment_policy"],
        "tail_shape_policy": source_report["source_policy"]["tail_shape_policy"],
        "arrangement_policy": arrangement_policy,
        "failure_codes": family_failures,
    }
    return apply_evidence_boundary(
        case_summary,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
    )


def family_failure_codes(source_family: str, proof: dict, metrics: dict) -> list[str]:
    full = metrics["full_performance"]
    failures = []
    if full["rms"] < 0.12:
        failures.append("full_performance_too_quiet")
    if full["peak_abs"] > 0.985:
        failures.append("full_performance_near_clipping")
    if not 0.20 <= proof["source_to_performance_correlation"] <= 0.80:
        failures.append("source_not_transformed_but_present")
    if proof["w30_to_source_rms_ratio"] < 0.18:
        failures.append("w30_source_chop_too_weak")
    if proof["pressure_low_band_lift_ratio"] < 1.16:
        failures.append("pressure_lift_lacks_low_band_support")
    if proof["stutter_to_hook_transient_ratio"] < 0.58:
        failures.append("stutter_lacks_hook_contrast")
    if proof["restore_to_pressure_rms_ratio"] < 1.12:
        failures.append("restore_not_bigger_than_pressure")

    if source_family == "tonal_hook":
        if proof["w30_to_source_rms_ratio"] < MIN_TONAL_W30_TO_SOURCE_RMS_RATIO:
            failures.append("tonal_w30_source_chop_too_weak")
        if proof["hook_to_source_transient_ratio"] < 1.0:
            failures.append("tonal_hook_lacks_source_transient")
        if proof["pressure_to_hook_rms_ratio"] < 1.05:
            failures.append("tonal_pressure_support_too_buried")
    elif source_family == "sparse_bass_pressure":
        if proof["pressure_to_hook_rms_ratio"] < 1.30:
            failures.append("sparse_pressure_not_stronger_than_hook")
        if proof["full_to_source_rms_ratio"] < 1.0:
            failures.append("sparse_full_performance_not_assertive")
    else:
        failures.append("unsupported_source_family")
    return failures


def validate_report_failure_codes(
    report: dict[str, Any],
    *,
    require_artifacts: bool = False,
) -> list[str]:
    failures = []
    failures.extend(evidence_boundary_failure_codes(report))
    if report.get("schema") != SCHEMA:
        failures.append("schema_mismatch")
    if report.get("schema_version") != 1:
        failures.append("schema_version_mismatch")
    if report.get("result") != "pass":
        failures.append("result_not_pass")
    if report.get("agent_verdict") != "agent_promising":
        failures.append("agent_verdict_not_promising")
    if report.get("human_verdict") != "unverified":
        failures.append("human_verdict_not_unverified")
    if report.get("evidence_role") != "diagnostic":
        failures.append("evidence_role_mismatch")
    if report.get("source_backed") is not True:
        failures.append("source_backed_not_true")
    if report.get("source_timing_backed") is not True:
        failures.append("source_timing_backed_not_true")
    if report.get("scripted_generation") is not True:
        failures.append("scripted_generation_not_true")
    if report.get("quality_proof") is not False:
        failures.append("quality_proof_claimed")
    cases = list_or_empty(report.get("cases"))
    if report.get("case_count") != 2 or len(cases) != 2:
        failures.append("case_count_mismatch")
    if report.get("passed_case_count") != 2:
        failures.append("passed_case_count_mismatch")
    families = sorted(str(case.get("source_family", "")) for case in cases if isinstance(case, dict))
    if families != ["sparse_bass_pressure", "tonal_hook"]:
        failures.append("source_family_coverage_mismatch")
    for index, case in enumerate(cases):
        validate_report_case(case, index, require_artifacts, failures)
    if not any(is_tonal_professional_case(case) for case in cases if isinstance(case, dict)):
        failures.append("tonal_hook_professional_case_missing")
    if not any(is_sparse_professional_case(case) for case in cases if isinstance(case, dict)):
        failures.append("sparse_bass_pressure_professional_case_missing")
    return sorted(set(failures))


def validate_report_case(
    case: Any,
    index: int,
    require_artifacts: bool,
    failures: list[str],
) -> None:
    if not isinstance(case, dict):
        failures.append(f"case_{index}_not_object")
        return
    prefix = str(case.get("case_id") or f"case_{index}")
    proof = object_or_empty(case.get("proof"))
    metrics = object_or_empty(case.get("metrics"))
    pressure_policy = object_or_empty(case.get("pressure_lift_policy"))
    arrangement_policy = object_or_empty(case.get("arrangement_policy"))
    audio_files = object_or_empty(case.get("audio_files"))

    if case.get("result") != "pass":
        failures.append(f"{prefix}:result_not_pass")
    if case.get("human_verdict") != "unverified":
        failures.append(f"{prefix}:human_verdict_not_unverified")
    if case.get("evidence_role") != "diagnostic":
        failures.append(f"{prefix}:evidence_role_mismatch")
    if case.get("source_backed") is not True:
        failures.append(f"{prefix}:source_backed_not_true")
    if case.get("source_timing_backed") is not True:
        failures.append(f"{prefix}:source_timing_backed_not_true")
    if case.get("scripted_generation") is not True:
        failures.append(f"{prefix}:scripted_generation_not_true")
    if case.get("quality_proof") is not False:
        failures.append(f"{prefix}:quality_proof_claimed")
    if pressure_policy.get("source_aware") is not True:
        failures.append(f"{prefix}:pressure_lift_policy_not_source_aware")
    if pressure_policy.get("source_family") != case.get("source_family"):
        failures.append(f"{prefix}:pressure_lift_source_family_mismatch")
    if arrangement_policy.get("source_aware") is not True:
        failures.append(f"{prefix}:arrangement_policy_not_source_aware")
    if arrangement_policy.get("source_family") != case.get("source_family"):
        failures.append(f"{prefix}:arrangement_source_family_mismatch")
    if not isinstance(arrangement_policy.get("role_order_signature"), str):
        failures.append(f"{prefix}:arrangement_role_order_signature_missing")
    if number(proof.get("pressure_lift_policy_decision_count")) < 12.0:
        failures.append(f"{prefix}:pressure_lift_policy_decision_count_too_low")
    if number(proof.get("arrangement_policy_decision_count")) < 8.0:
        failures.append(f"{prefix}:arrangement_policy_decision_count_too_low")
    if number(proof.get("arrangement_pressure_role_count")) < 2.0:
        failures.append(f"{prefix}:arrangement_pressure_role_count_too_low")
    if number(proof.get("arrangement_destructive_role_count")) < 2.0:
        failures.append(f"{prefix}:arrangement_destructive_role_count_too_low")
    if number(proof.get("arrangement_failure_count")) != 0.0:
        failures.append(f"{prefix}:arrangement_failure_count_nonzero")
    if number(proof.get("pressure_lift_bar5_to_bar4_rms_ratio")) < 1.02:
        failures.append(f"{prefix}:pressure_lift_bar5_to_bar4_too_weak")
    if number(proof.get("rebuild_only_to_full_rms_ratio")) < 0.42:
        failures.append(f"{prefix}:rebuild_only_to_full_too_weak")
    if number(proof.get("rebuild_only_to_source_rms_ratio")) < 0.30:
        failures.append(f"{prefix}:rebuild_only_to_source_too_weak")
    if number(proof.get("rebuild_only_to_source_correlation")) > 0.92:
        failures.append(f"{prefix}:rebuild_only_too_source_masked")
    if number(proof.get("source_on_to_rebuild_only_correlation")) > 0.995:
        failures.append(f"{prefix}:source_layer_toggle_did_not_change_output")
    if number(metrics.get("full_performance_peak_abs")) > 0.985:
        failures.append(f"{prefix}:full_performance_near_clipping")
    if number(metrics.get("rebuild_only_performance_peak_abs")) > 0.985:
        failures.append(f"{prefix}:rebuild_only_performance_near_clipping")
    if number(proof.get("arrangement_role_order_source_derived")) < 1.0:
        failures.append(f"{prefix}:arrangement_role_order_not_source_derived")
    if number(proof.get("arrangement_role_candidate_count")) < 6.0:
        failures.append(f"{prefix}:arrangement_role_candidate_count_too_low")
    if number(proof.get("arrangement_scripted_role_distance")) < 1.0:
        failures.append(f"{prefix}:arrangement_role_order_too_scripted")
    validate_common_source_character(case, prefix, proof, failures)
    if require_artifacts:
        output = Path(str(case.get("output", "")))
        for role in ("full_performance", "rebuild_only_performance"):
            if not str(audio_files.get(role, "")).endswith(".wav"):
                failures.append(f"{prefix}:{role}_not_wav")
        for relative in (
            audio_files.get("full_performance"),
            audio_files.get("rebuild_only_performance"),
            "performance-report.json",
        ):
            if not relative or not (output / str(relative)).is_file():
                failures.append(f"{prefix}:artifact_missing_{relative}")


def validate_common_source_character(
    case: dict[str, Any],
    prefix: str,
    proof: dict[str, Any],
    failures: list[str],
) -> None:
    if number(proof.get("mix_treatment_source_derived")) < 1.0:
        failures.append(f"{prefix}:mix_treatment_not_source_derived")
    if number(proof.get("mix_treatment_candidate_count")) < 6.0:
        failures.append(f"{prefix}:mix_treatment_candidate_count_too_low")
    if number(proof.get("mix_treatment_fixed_distance")) < MIN_MIX_TREATMENT_FIXED_DISTANCE:
        failures.append(f"{prefix}:mix_treatment_too_fixed")
    if number(proof.get("mix_treatment_output_contrast_ratio")) < MIN_MIX_TREATMENT_OUTPUT_CONTRAST_RATIO:
        failures.append(f"{prefix}:mix_treatment_output_contrast_too_low")
    if number(proof.get("tail_shape_source_derived")) < 1.0:
        failures.append(f"{prefix}:tail_shape_not_source_derived")
    if number(proof.get("tail_shape_candidate_count")) < 6.0:
        failures.append(f"{prefix}:tail_shape_candidate_count_too_low")
    if number(proof.get("tail_shape_fixed_distance")) < MIN_TAIL_SHAPE_FIXED_DISTANCE:
        failures.append(f"{prefix}:tail_shape_collapsed_to_fixed_recipe")
    if number(proof.get("tail_shape_output_contrast_ratio")) < MIN_TAIL_SHAPE_OUTPUT_CONTRAST_RATIO:
        failures.append(f"{prefix}:tail_shape_output_contrast_too_low")
    if proof.get("strongest_audible_element") not in ALLOWED_STRONGEST_AUDIBLE_ELEMENTS:
        failures.append(f"{prefix}:strongest_audible_element_missing")
    if number(proof.get("strongest_audible_element_score")) < MIN_STRONGEST_AUDIBLE_ELEMENT_SCORE:
        failures.append(f"{prefix}:strongest_audible_element_too_weak")
    if number(proof.get("strongest_audible_element_margin")) < MIN_STRONGEST_AUDIBLE_ELEMENT_MARGIN:
        failures.append(f"{prefix}:strongest_audible_element_ambiguous")
    if number(proof.get("strongest_audible_element_candidate_count")) < 5.0:
        failures.append(f"{prefix}:strongest_audible_element_candidate_count_too_low")
    if number(proof.get("rebuild_only_source_spectral_similarity")) < MIN_REBUILD_ONLY_SOURCE_SPECTRAL_SIMILARITY:
        failures.append(f"{prefix}:rebuild_only_source_spectral_character_lost")
    if number(proof.get("rebuild_only_source_transient_retention")) < MIN_REBUILD_ONLY_SOURCE_TRANSIENT_RETENTION:
        failures.append(f"{prefix}:rebuild_only_source_transient_character_lost")
    if number(proof.get("rebuild_only_source_character_survival_score")) < MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_SCORE:
        failures.append(f"{prefix}:rebuild_only_source_character_not_surviving")
    if number(proof.get("rebuild_only_source_character_survival_margin")) < MIN_REBUILD_ONLY_SOURCE_CHARACTER_SURVIVAL_MARGIN:
        failures.append(f"{prefix}:rebuild_only_source_character_margin_too_low")
    if case.get("source_family") == "tonal_hook":
        validate_tonal_case(prefix, proof, failures)
    if case.get("source_family") == "sparse_bass_pressure":
        validate_sparse_case(prefix, proof, failures)


def validate_tonal_case(prefix: str, proof: dict[str, Any], failures: list[str]) -> None:
    if number(proof.get("hook_chop_selection_source_derived")) < 1.0:
        failures.append(f"{prefix}:hook_chop_selection_not_source_derived")
    if number(proof.get("hook_chop_static_distance_frames")) < MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES:
        failures.append(f"{prefix}:hook_chop_selection_collapsed_to_static_first_bar")
    if number(proof.get("hook_chop_offset_distance_frames")) < MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES:
        failures.append(f"{prefix}:hook_chop_selection_offsets_too_close")
    if number(proof.get("w30_to_source_rms_ratio")) < MIN_PROFESSIONAL_W30_TO_SOURCE_RMS_RATIO:
        failures.append(f"{prefix}:tonal_w30_source_chop_too_weak")
    if number(proof.get("hook_chop_riff_unique_source_offset_count")) < MIN_HOOK_CHOP_RIFF_UNIQUE_SOURCE_OFFSET_COUNT:
        failures.append(f"{prefix}:hook_chop_riff_source_offsets_too_narrow")
    if number(proof.get("hook_chop_riff_hit_pattern_source_derived")) < 1.0:
        failures.append(f"{prefix}:hook_chop_riff_pattern_not_source_derived")
    if number(proof.get("hook_chop_riff_hit_count")) < MIN_HOOK_CHOP_RIFF_HIT_COUNT:
        failures.append(f"{prefix}:hook_chop_riff_pattern_too_sparse")
    if number(proof.get("hook_chop_riff_velocity_span")) < MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN:
        failures.append(f"{prefix}:hook_chop_riff_velocity_too_flat")
    if number(proof.get("hook_chop_riff_reverse_count")) < MIN_HOOK_CHOP_RIFF_REVERSE_COUNT:
        failures.append(f"{prefix}:hook_chop_riff_reverse_missing")
    if number(proof.get("hook_chop_source_character_score_floor")) < MIN_HOOK_CHOP_SOURCE_CHARACTER_SCORE_FLOOR:
        failures.append(f"{prefix}:hook_chop_source_character_too_weak")
    if number(proof.get("hook_chop_source_character_score_span")) < MIN_HOOK_CHOP_SOURCE_CHARACTER_SCORE_SPAN:
        failures.append(f"{prefix}:hook_chop_source_character_too_flat")
    if number(proof.get("destructive_gesture_source_derived")) < 1.0:
        failures.append(f"{prefix}:destructive_gesture_not_source_derived")
    if number(proof.get("destructive_static_distance_frames")) < MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES:
        failures.append(f"{prefix}:destructive_gesture_collapsed_to_fixed_choice")
    if number(proof.get("destructive_offset_distance_frames")) < MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES:
        failures.append(f"{prefix}:destructive_gesture_offsets_too_close")


def validate_sparse_case(prefix: str, proof: dict[str, Any], failures: list[str]) -> None:
    if number(proof.get("bass_movement_source_derived")) < 1.0:
        failures.append(f"{prefix}:sparse_bass_movement_not_source_derived")
    if number(proof.get("sparse_bass_movement_static_distance_hz")) < MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ:
        failures.append(f"{prefix}:sparse_bass_movement_collapsed_to_fixed_contour")
    if number(proof.get("sparse_bass_movement_frequency_span_hz")) < MIN_SPARSE_BASS_MOVEMENT_FREQUENCY_SPAN_HZ:
        failures.append(f"{prefix}:sparse_bass_movement_frequency_span_too_narrow")
    if number(proof.get("pressure_low_band_lift_ratio")) < MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO:
        failures.append(f"{prefix}:sparse_pressure_lift_lacks_low_band_support")
    if number(proof.get("sparse_pressure_low_band_share")) < MIN_SPARSE_PRESSURE_LOW_BAND_SHARE:
        failures.append(f"{prefix}:sparse_pressure_low_band_share_too_low")
    if number(proof.get("sparse_pressure_low_to_mid_ratio")) < MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO:
        failures.append(f"{prefix}:sparse_pressure_reads_as_midrange_phrase")
    if proof.get("strongest_audible_element") != "bass":
        failures.append(f"{prefix}:sparse_bass_not_strongest")
    if number(proof.get("strongest_audible_element_margin")) < MIN_SPARSE_BASS_DOMINANCE_MARGIN:
        failures.append(f"{prefix}:sparse_bass_dominance_margin_too_low")


def is_tonal_professional_case(case: dict[str, Any]) -> bool:
    proof = object_or_empty(case.get("proof"))
    return (
        case.get("source_family") == "tonal_hook"
        and number(proof.get("hook_chop_selection_source_derived")) >= 1.0
        and number(proof.get("hook_chop_static_distance_frames")) >= MIN_HOOK_CHOP_STATIC_DISTANCE_FRAMES
        and number(proof.get("hook_chop_offset_distance_frames")) >= MIN_HOOK_CHOP_OFFSET_DISTANCE_FRAMES
        and number(proof.get("w30_to_source_rms_ratio")) >= MIN_PROFESSIONAL_W30_TO_SOURCE_RMS_RATIO
        and number(proof.get("hook_chop_riff_unique_source_offset_count")) >= MIN_HOOK_CHOP_RIFF_UNIQUE_SOURCE_OFFSET_COUNT
        and number(proof.get("hook_chop_riff_hit_pattern_source_derived")) >= 1.0
        and number(proof.get("hook_chop_riff_hit_count")) >= MIN_HOOK_CHOP_RIFF_HIT_COUNT
        and number(proof.get("hook_chop_riff_velocity_span")) >= MIN_HOOK_CHOP_RIFF_VELOCITY_SPAN
        and number(proof.get("hook_chop_riff_reverse_count")) >= MIN_HOOK_CHOP_RIFF_REVERSE_COUNT
        and number(proof.get("hook_chop_source_character_score_floor")) >= MIN_HOOK_CHOP_SOURCE_CHARACTER_SCORE_FLOOR
        and number(proof.get("hook_chop_source_character_score_span")) >= MIN_HOOK_CHOP_SOURCE_CHARACTER_SCORE_SPAN
        and number(proof.get("destructive_gesture_source_derived")) >= 1.0
        and number(proof.get("destructive_static_distance_frames")) >= MIN_DESTRUCTIVE_STATIC_DISTANCE_FRAMES
        and number(proof.get("destructive_offset_distance_frames")) >= MIN_DESTRUCTIVE_OFFSET_DISTANCE_FRAMES
    )


def is_sparse_professional_case(case: dict[str, Any]) -> bool:
    proof = object_or_empty(case.get("proof"))
    return (
        case.get("source_family") == "sparse_bass_pressure"
        and number(proof.get("bass_movement_source_derived")) >= 1.0
        and number(proof.get("sparse_bass_movement_static_distance_hz")) >= MIN_SPARSE_BASS_MOVEMENT_STATIC_DISTANCE_HZ
        and number(proof.get("sparse_bass_movement_frequency_span_hz")) >= MIN_SPARSE_BASS_MOVEMENT_FREQUENCY_SPAN_HZ
        and number(proof.get("pressure_low_band_lift_ratio")) >= MIN_SPARSE_PRESSURE_LOW_BAND_LIFT_RATIO
        and number(proof.get("sparse_pressure_low_band_share")) >= MIN_SPARSE_PRESSURE_LOW_BAND_SHARE
        and number(proof.get("sparse_pressure_low_to_mid_ratio")) >= MIN_SPARSE_PRESSURE_LOW_TO_MID_RATIO
        and proof.get("strongest_audible_element") == "bass"
        and number(proof.get("strongest_audible_element_margin")) >= MIN_SPARSE_BASS_DOMINANCE_MARGIN
    )


def run_mutation_fixtures(report: dict[str, Any]) -> None:
    fixtures = []
    mutated = json.loads(json.dumps(report))
    mutated["human_verdict"] = "pass"
    fixtures.append(("human_verdict_claim", mutated, "human_verdict_not_unverified"))

    mutated = json.loads(json.dumps(report))
    mutated["quality_proof"] = True
    mutated["evidence_boundary"]["quality_proof"] = True
    fixtures.append(("quality_claim", mutated, "quality_proof_claimed"))

    mutated = json.loads(json.dumps(report))
    mutated["cases"][0]["source_family"] = "dense_break"
    fixtures.append(("source_family_coverage", mutated, "source_family_coverage_mismatch"))

    mutated = mutate_case_proof(report, "tonal_hook", "hook_chop_selection_source_derived", 0.0)
    fixtures.append(("tonal_hook_not_source_derived", mutated, "hook_chop_selection_not_source_derived"))

    mutated = mutate_case_proof(report, "tonal_hook", "hook_chop_static_distance_frames", 0.0)
    fixtures.append(
        (
            "tonal_hook_static",
            mutated,
            "hook_chop_selection_collapsed_to_static_first_bar",
        )
    )

    mutated = mutate_case_proof(report, "tonal_hook", "hook_chop_riff_unique_source_offset_count", 1.0)
    fixtures.append(
        ("tonal_hook_narrow_offsets", mutated, "hook_chop_riff_source_offsets_too_narrow")
    )

    mutated = mutate_case_proof(report, "tonal_hook", "destructive_static_distance_frames", 0.0)
    fixtures.append(
        (
            "tonal_destructive_static",
            mutated,
            "destructive_gesture_collapsed_to_fixed_choice",
        )
    )

    mutated = mutate_case_proof(report, "sparse_bass_pressure", "bass_movement_source_derived", 0.0)
    fixtures.append(("sparse_bass_not_source_derived", mutated, "sparse_bass_movement_not_source_derived"))

    mutated = mutate_case_proof(report, "sparse_bass_pressure", "sparse_bass_movement_static_distance_hz", 0.0)
    fixtures.append(
        (
            "sparse_bass_static",
            mutated,
            "sparse_bass_movement_collapsed_to_fixed_contour",
        )
    )

    mutated = mutate_case_proof(report, "sparse_bass_pressure", "pressure_low_band_lift_ratio", 0.0)
    fixtures.append(
        ("sparse_pressure_weak", mutated, "sparse_pressure_lift_lacks_low_band_support")
    )

    mutated = mutate_case_proof(report, "sparse_bass_pressure", "sparse_pressure_low_band_share", 0.0)
    fixtures.append(
        ("sparse_low_band_share_weak", mutated, "sparse_pressure_low_band_share_too_low")
    )

    mutated = mutate_case_proof(report, "sparse_bass_pressure", "sparse_pressure_low_to_mid_ratio", 0.0)
    fixtures.append(
        ("sparse_midrange_phrase", mutated, "sparse_pressure_reads_as_midrange_phrase")
    )

    mutated = mutate_case_proof(report, "sparse_bass_pressure", "strongest_audible_element_margin", 0.0)
    fixtures.append(
        ("sparse_bass_ambiguous", mutated, "sparse_bass_dominance_margin_too_low")
    )

    for name, fixture, expected in fixtures:
        failures = validate_report_failure_codes(fixture, require_artifacts=False)
        if not any(expected in failure for failure in failures):
            raise ValueError(f"mutation {name} expected {expected}, got {failures}")


def mutate_case_proof(
    report: dict[str, Any],
    source_family: str,
    key: str,
    value: Any,
) -> dict[str, Any]:
    mutated = json.loads(json.dumps(report))
    for case in mutated["cases"]:
        if case.get("source_family") == source_family:
            case["proof"][key] = value
            return mutated
    raise ValueError(f"missing source family in mutation fixture: {source_family}")


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"expected JSON object: {path}")
    return value


def list_or_empty(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def write_reports(output: Path, report: dict) -> None:
    (output / "professional-source-wav-pack.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = [
        "# Professional Source WAV Pack",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Cases: `{report['passed_case_count']}/{report['case_count']}` passing",
        "",
        "## Cases",
        "",
    ]
    for case in report["cases"]:
        lines.append(
            f"- `{case['case_id']}` `{case['source_family']}`: `{case['result']}` "
            f"source `{case['source']}` output `{case['output']}`"
        )
        if case["failure_codes"]:
            lines.append(f"  failure_codes: `{', '.join(case['failure_codes'])}`")
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "This pack writes audible WAV artifacts and family-aware deterministic proof. "
            "It does not claim human musical pass.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
