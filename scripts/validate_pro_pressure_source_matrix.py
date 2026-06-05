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
            "Source-matrix renders the current scripted pro-pressure diagnostic "
            "pack across multiple sources; it is cross-source diagnostic "
            "evidence, not source-family quality proof."
        ),
    )
    write_reports(output, report)
    if failed:
        print(
            "pro-pressure source matrix failed: "
            + ", ".join(case["case_id"] for case in failed),
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
            },
            "metrics": {
                "chop_hook_dbfs": metrics["chop_hook"]["dbfs"],
                "pressure_lift_dbfs": metrics["pressure_lift"]["dbfs"],
                "dropout_stutter_dbfs": metrics["dropout_stutter"]["dbfs"],
                "restore_hit_dbfs": metrics["restore_hit"]["dbfs"],
                "full_performance_dbfs": metrics["full_performance"]["dbfs"],
                "full_performance_peak_abs": metrics["full_performance"]["peak_abs"],
            },
            "pressure_lift_policy": pressure_lift_policy,
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


def write_reports(output: Path, report: dict) -> None:
    (output / "source-matrix-report.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = [
        "# Pro-Pressure Source Matrix",
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
            f"- `{case['case_id']}`: `{case['result']}` "
            f"source `{case['source']}` output `{case['output']}`"
        )
        if case["failure_codes"]:
            lines.append(f"  failure_codes: `{', '.join(case['failure_codes'])}`")
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
