#!/usr/bin/env python3
"""Generate tonal and sparse professional-output WAV packs."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

from audio_qa_evidence_boundary import apply_evidence_boundary


SCHEMA = "riotbox.professional_source_wav_pack.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-professional-source-wav-pack")
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
    args = parser.parse_args()

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
    write_reports(output, report)
    if failed:
        print(
            "professional source WAV pack failed: "
            + ", ".join(case["case_id"] for case in failed),
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
    family_failures = family_failure_codes(case["source_family"], proof, metrics)
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
            "bass_movement_source_derived": proof["bass_movement_source_derived"],
            "sparse_bass_movement_static_distance_hz": proof[
                "sparse_bass_movement_static_distance_hz"
            ],
            "sparse_bass_movement_frequency_span_hz": proof[
                "sparse_bass_movement_frequency_span_hz"
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
