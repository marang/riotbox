#!/usr/bin/env python3
"""Render the pro-pressure dense-break pack across multiple local sources."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

from audio_qa_evidence_boundary import apply_evidence_boundary
from route_weak_output_fixes import route_signals


SCHEMA = "riotbox.pro_pressure_source_matrix.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-pro-pressure-source-matrix")
DEFAULT_CASES = [
    ("beat03_130", "data/test_audio/examples/Beat03_130BPM(Full).wav", 130.0),
    ("beat08_128", "data/test_audio/examples/Beat08_128BPM(Full).wav", 128.0),
    ("beat20_128", "data/test_audio/examples/Beat20_128BPM(Full).wav", 128.0),
    ("dh_beatc_120", "data/test_audio/examples/DH_BeatC_120-01.wav", 120.0),
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-pro-pressure-source-matrix")
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    output = resolve_repo_path(repo, args.output)
    ensure_safe_output(repo, output)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    cases = []
    for case_id, source, bpm in DEFAULT_CASES:
        cases.append(render_case(repo, output, args.date, case_id, source, bpm))

    failed = [case for case in cases if case["result"] != "pass"]
    arrangements = arrangement_summary(cases)
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failed and not arrangements["failure_codes"] else "fail",
        "agent_verdict": (
            "agent_promising" if not failed and not arrangements["failure_codes"] else "agent_weak"
        ),
        "human_verdict": "unverified",
        "case_count": len(cases),
        "passed_case_count": len(cases) - len(failed),
        "failed_case_count": len(failed),
        "arrangement_summary": arrangements,
        "cases": cases,
    }
    apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "Source-matrix renders the current source-aware pro-pressure and "
            "arrangement diagnostic pack across multiple sources; it proves "
            "bounded arrangement-policy diversity, not source-family quality proof."
        ),
    )
    write_reports(output, report)
    if failed or arrangements["failure_codes"]:
        print(
            "pro-pressure source matrix failed: "
            + ", ".join([case["case_id"] for case in failed] + arrangements["failure_codes"]),
            file=sys.stderr,
        )
        return 1
    print(f"pro-pressure source matrix written to {output}")
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


def render_case(
    repo: Path,
    output: Path,
    date: str,
    case_id: str,
    source: str,
    bpm: float,
) -> dict:
    case_dir = output / case_id
    command = [
        sys.executable,
        "scripts/generate_dense_break_performance_pack.py",
        "--source",
        source,
        "--bpm",
        f"{bpm:.6f}",
        "--output",
        str(case_dir),
        "--date",
        f"{date}-{case_id}",
    ]
    result = subprocess.run(
        command,
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    (case_dir / "matrix-render.log").parent.mkdir(parents=True, exist_ok=True)
    (case_dir / "matrix-render.log").write_text(
        result.stdout + ("\n" if result.stdout and result.stderr else "") + result.stderr
    )

    report_path = case_dir / "performance-report.json"
    if report_path.is_file():
        case_report = json.loads(report_path.read_text())
        proof = case_report["proof"]
        metrics = case_report["metrics"]
        pressure_lift_policy = case_report["source_policy"]["pressure_lift_policy"]
        arrangement_policy = case_report["source_policy"]["arrangement_policy"]
        case_summary = {
            "case_id": case_id,
            "source": source,
            "bpm": bpm,
            "result": case_report["result"],
            "agent_verdict": case_report["agent_verdict"],
            "human_verdict": case_report["human_verdict"],
            "failure_codes": case_report["failure_codes"],
            "output": str(case_dir),
            "proof": {
                "w30_to_source_rms_ratio": proof["w30_to_source_rms_ratio"],
                "full_to_source_rms_ratio": proof["full_to_source_rms_ratio"],
                "hook_to_source_transient_ratio": proof["hook_to_source_transient_ratio"],
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
                "hook_chop_riff_velocity_span": proof[
                    "hook_chop_riff_velocity_span"
                ],
                "hook_chop_riff_reverse_count": proof[
                    "hook_chop_riff_reverse_count"
                ],
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
                "mix_treatment_source_derived": proof[
                    "mix_treatment_source_derived"
                ],
                "mix_treatment_candidate_count": proof[
                    "mix_treatment_candidate_count"
                ],
                "mix_treatment_fixed_distance": proof[
                    "mix_treatment_fixed_distance"
                ],
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
                "pressure_low_band_lift_ratio": proof["pressure_low_band_lift_ratio"],
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
                "arrangement_policy_decision_count": proof[
                    "arrangement_policy_decision_count"
                ],
                "arrangement_role_order_source_derived": proof[
                    "arrangement_role_order_source_derived"
                ],
                "arrangement_role_candidate_count": proof[
                    "arrangement_role_candidate_count"
                ],
                "arrangement_scripted_role_distance": proof[
                    "arrangement_scripted_role_distance"
                ],
                "arrangement_pressure_role_count": proof[
                    "arrangement_pressure_role_count"
                ],
                "arrangement_destructive_role_count": proof[
                    "arrangement_destructive_role_count"
                ],
                "arrangement_failure_count": proof["arrangement_failure_count"],
                "rebuild_only_to_full_rms_ratio": proof[
                    "rebuild_only_to_full_rms_ratio"
                ],
                "rebuild_only_to_source_rms_ratio": proof[
                    "rebuild_only_to_source_rms_ratio"
                ],
                "rebuild_only_to_source_correlation": proof[
                    "rebuild_only_to_source_correlation"
                ],
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
                "chop_hook_dbfs": metrics["chop_hook"]["dbfs"],
                "pressure_lift_dbfs": metrics["pressure_lift"]["dbfs"],
                "dropout_stutter_dbfs": metrics["dropout_stutter"]["dbfs"],
                "restore_hit_dbfs": metrics["restore_hit"]["dbfs"],
                "full_performance_dbfs": metrics["full_performance"]["dbfs"],
                "full_performance_peak_abs": metrics["full_performance"]["peak_abs"],
                "rebuild_only_performance_dbfs": metrics["rebuild_only_performance"]["dbfs"],
                "rebuild_only_performance_peak_abs": metrics[
                    "rebuild_only_performance"
                ]["peak_abs"],
            },
            "pressure_lift_policy": pressure_lift_policy,
            "hook_chop_policy": case_report["source_policy"]["hook_chop_policy"],
            "destructive_gesture_policy": case_report["source_policy"][
                "destructive_gesture_policy"
            ],
            "arrangement_policy": arrangement_policy,
            "arrangement_failure_codes": arrangement_failure_codes(arrangement_policy),
        }
        return apply_evidence_boundary(
            case_summary,
            evidence_role="diagnostic",
            source_backed=True,
            source_timing_backed=True,
            scripted_generation=True,
        )

    failure_summary = {
        "case_id": case_id,
        "source": source,
        "bpm": bpm,
        "result": "fail",
        "agent_verdict": "agent_fail",
        "human_verdict": "unverified",
        "failure_codes": ["render_failed_without_report"],
        "output": str(case_dir),
        "returncode": result.returncode,
    }
    return apply_evidence_boundary(
        failure_summary,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
    )


def arrangement_summary(cases: list[dict]) -> dict:
    signatures = sorted(
        {
            str(object_or_empty(case.get("arrangement_policy")).get("role_order_signature", "unknown"))
            for case in cases
            if case.get("result") == "pass"
        }
    )
    families = sorted(
        {
            str(object_or_empty(case.get("arrangement_policy")).get("source_family", "unknown"))
            for case in cases
            if case.get("result") == "pass"
        }
    )
    failures = []
    if len(signatures) < 2:
        failures.append("arrangement_role_order_collapsed_across_source_families")
    for case in cases:
        for code in case.get("arrangement_failure_codes", []):
            failures.append(f"{case['case_id']}:{code}")
    return {
        "unique_role_order_signature_count": len(signatures),
        "role_order_signatures": signatures,
        "source_families": families,
        "failure_codes": failures,
        "failure_routes": arrangement_failure_routes(failures),
    }


def arrangement_failure_codes(policy: dict) -> list[str]:
    roles = list(object_or_empty(policy).get("role_order") or [])
    failures = []
    if len(roles) != 8:
        failures.append("arrangement_role_order_not_8_bars")
    for role in ("hook", "chop", "pressure", "dropout", "restore"):
        if role not in roles:
            failures.append(f"arrangement_missing_{role}_role")
    if roles.count("pressure") < 2:
        failures.append("arrangement_pressure_lift_too_short")
    if roles[-2:] != ["dropout", "restore"]:
        failures.append("arrangement_destructive_restore_tail_missing")
    return failures


def object_or_empty(value: object) -> dict:
    return value if isinstance(value, dict) else {}


def arrangement_failure_routes(failures: list[str]) -> list[dict[str, object]]:
    routes = []
    for code in failures:
        route = route_signals([code], {}, [])
        routes.append(
            {
                "failure_code": code,
                "proposed_next_fix_category": route["proposed_next_fix_category"],
                "proposed_fix_categories": route["proposed_fix_categories"],
                "main_weakness": route["main_weakness"],
            }
        )
    return routes


def write_reports(output: Path, report: dict) -> None:
    (output / "source-matrix-report.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = [
        "# Pro-Pressure Source Matrix",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Cases: `{report['passed_case_count']}/{report['case_count']}` passing",
        f"- Arrangement signatures: `{report['arrangement_summary']['unique_role_order_signature_count']}`",
        "",
        "## Cases",
        "",
    ]
    for case in report["cases"]:
        lines.append(
            f"- `{case['case_id']}`: `{case['result']}` "
            f"source `{case['source']}` output `{case['output']}`"
        )
        if case["failure_codes"]:
            lines.append(f"  failure_codes: `{', '.join(case['failure_codes'])}`")
        if case.get("arrangement_policy"):
            lines.append(
                "  arrangement: "
                f"`{case['arrangement_policy']['role_order_signature']}` "
                f"shape `{case['arrangement_policy']['arrangement_shape']}`"
            )
    lines.extend(["", "## Arrangement Summary", ""])
    for signature in report["arrangement_summary"]["role_order_signatures"]:
        lines.append(f"- `{signature}`")
    if report["arrangement_summary"]["failure_codes"]:
        lines.append("")
        lines.append("Arrangement failure routes:")
        for route in report["arrangement_summary"]["failure_routes"]:
            lines.append(
                f"- `{route['failure_code']}` -> `{route['proposed_next_fix_category']}`"
            )
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "This matrix proves deterministic source robustness for the current "
            "pro-pressure render gates. It does not claim human musical pass.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
