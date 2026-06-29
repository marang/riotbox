#!/usr/bin/env python3
"""Generate rendered weak professional-output examples for negative QA gates."""

from __future__ import annotations

import argparse
import json
import math
import shutil
import struct
import subprocess
import sys
import wave
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary


SCHEMA = "riotbox.rendered_weak_professional_outputs.v1"
SAMPLE_RATE = 44_100
CHANNELS = 2
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-rendered-weak-professional-outputs")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    output = resolve_repo_path(repo, args.output)
    ensure_safe_output(repo, output)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    cases = [generate_dense_flat_stutter_case(repo, output)]
    failed_cases = [case for case in cases if case["validator_result"] != "expected_fail"]
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failed_cases else "fail",
        "agent_verdict": "agent_promising" if not failed_cases else "agent_fail",
        "human_verdict": "unverified",
        "case_count": len(cases),
        "cases": cases,
    }
    apply_evidence_boundary(
        report,
        evidence_role="negative_diagnostic",
        source_backed=False,
        source_timing_backed=False,
        scripted_generation=True,
        notes=(
            "Rendered weak fixtures are synthetic negative diagnostics. They "
            "prove rejection of known weak shapes, not product quality."
        ),
    )
    write_reports(output, report)
    if failed_cases:
        print(
            "rendered weak professional outputs failed unexpectedly: "
            + ", ".join(case["case_id"] for case in failed_cases),
            file=sys.stderr,
        )
        return 1
    print(f"rendered weak professional outputs written to {output}")
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


def generate_dense_flat_stutter_case(repo: Path, output: Path) -> dict[str, Any]:
    case_dir = output / "dense_flat_stutter"
    case_dir.mkdir(parents=True, exist_ok=True)
    duration = 1.0
    source = render_pulse_train(duration, frequency=65.0, amplitude=0.10)
    weak_hook = render_pulse_train(duration, frequency=98.0, amplitude=0.040)
    weak_pressure = render_pulse_train(duration, frequency=55.0, amplitude=0.042)
    flat_stutter = render_pulse_train(duration, frequency=98.0, amplitude=0.038)
    weak_restore = render_pulse_train(duration, frequency=70.0, amplitude=0.039)
    rebuild_only = normalize_length(weak_hook + weak_pressure + flat_stutter + weak_restore)

    files = {
        "source_window": "00_source_window.wav",
        "chop_hook": "01_chop_hook.wav",
        "pressure_lift": "02_pressure_lift.wav",
        "dropout_stutter": "03_dropout_stutter.wav",
        "restore_hit": "04_restore_hit.wav",
        "rebuild_only_performance": "05_rebuild_only_performance.wav",
    }
    audio = {
        "source_window": source,
        "chop_hook": weak_hook,
        "pressure_lift": weak_pressure,
        "dropout_stutter": flat_stutter,
        "restore_hit": weak_restore,
        "rebuild_only_performance": rebuild_only,
    }
    for role, samples in audio.items():
        write_wav(case_dir / files[role], samples)

    metrics = {role: audio_metrics(samples) for role, samples in audio.items()}
    proof = {
        "dropout_to_stutter_rms_ratio": ratio(
            metrics["dropout_stutter"]["rms"] * 0.72,
            metrics["dropout_stutter"]["rms"],
        ),
        "dropout_silence_to_stutter_rms_ratio": ratio(
            metrics["dropout_stutter"]["rms"] * 0.72,
            metrics["dropout_stutter"]["rms"],
        ),
        "stutter_to_hook_transient_ratio": ratio(
            metrics["dropout_stutter"]["transient_score"],
            metrics["chop_hook"]["transient_score"],
        ),
        "restore_to_hook_transient_ratio": ratio(
            metrics["restore_hit"]["transient_score"],
            metrics["chop_hook"]["transient_score"],
        ),
        "restore_to_pressure_rms_ratio": ratio(
            metrics["restore_hit"]["rms"],
            metrics["pressure_lift"]["rms"],
        ),
        "restore_to_dropout_silence_rms_ratio": ratio(
            metrics["restore_hit"]["rms"],
            metrics["dropout_stutter"]["rms"] * 0.72,
        ),
        "max_adjacent_bar_correlation": 0.996,
        "source_to_performance_correlation": 0.93,
    }
    performance_report = {
        "schema": "riotbox.dense_break_performance_pack.v1",
        "schema_version": 1,
        "result": "pass",
        "agent_verdict": "agent_fail",
        "human_verdict": "unverified",
        "case_id": "dense_flat_stutter",
        "files": files,
        "proof": proof,
        "metrics": metrics,
        "failure_codes": [],
    }
    apply_evidence_boundary(
        performance_report,
        evidence_role="negative_diagnostic",
        source_backed=False,
        source_timing_backed=False,
        scripted_generation=True,
        notes=(
            "Synthetic weak dense-break-shaped report used only to prove "
            "negative destructive-variation rejection."
        ),
    )
    report_path = case_dir / "performance-report.json"
    report_path.write_text(json.dumps(performance_report, indent=2) + "\n")
    validation = run_destructive_validator(repo, report_path, case_dir / "validator.log")
    case_summary = {
        "case_id": "dense_flat_stutter",
        "source_family": "dense_break",
        "weakness": "flat stutter and weak restore from rendered WAV artifacts",
        "output": str(case_dir),
        "performance_report": str(report_path),
        "validator_result": validation["result"],
        "failure_codes": validation["failure_codes"],
        "metrics": {
            "dropout_stutter_rms": metrics["dropout_stutter"]["rms"],
            "restore_hit_rms": metrics["restore_hit"]["rms"],
            "dropout_silence_to_stutter_rms_ratio": proof[
                "dropout_silence_to_stutter_rms_ratio"
            ],
            "stutter_to_hook_transient_ratio": proof["stutter_to_hook_transient_ratio"],
            "restore_to_pressure_rms_ratio": proof["restore_to_pressure_rms_ratio"],
            "restore_to_dropout_silence_rms_ratio": proof[
                "restore_to_dropout_silence_rms_ratio"
            ],
        },
    }
    return apply_evidence_boundary(
        case_summary,
        evidence_role="negative_diagnostic",
        source_backed=False,
        source_timing_backed=False,
        scripted_generation=True,
    )


def run_destructive_validator(repo: Path, report_path: Path, log_path: Path) -> dict[str, Any]:
    output_json = report_path.with_name("destructive-validation.json")
    result = subprocess.run(
        [
            sys.executable,
            "scripts/validate_destructive_variation_professional.py",
            "--json-output",
            str(output_json),
            str(report_path),
        ],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    log_path.write_text(result.stdout + ("\n" if result.stdout and result.stderr else "") + result.stderr)
    if result.returncode == 0:
        return {"result": "unexpected_pass", "failure_codes": []}
    if not output_json.is_file():
        return {"result": "validator_failed_without_report", "failure_codes": []}
    data = json.loads(output_json.read_text())
    return {
        "result": "expected_fail" if data.get("result") == "fail" else "unexpected_pass",
        "failure_codes": data.get("failure_codes", []),
    }


def render_pulse_train(duration_seconds: float, frequency: float, amplitude: float) -> list[float]:
    frames = int(SAMPLE_RATE * duration_seconds)
    samples = []
    for index in range(frames):
        phase = (index / SAMPLE_RATE) * frequency
        envelope = 1.0 if int(phase * 4.0) % 8 == 0 else 0.35
        value = math.sin(phase * math.tau) * amplitude * envelope
        samples.append(value)
    return samples


def normalize_length(samples: list[float]) -> list[float]:
    peak = max(abs(value) for value in samples) or 1.0
    return [value * min(0.20 / peak, 1.0) for value in samples]


def audio_metrics(samples: list[float]) -> dict[str, float]:
    rms = math.sqrt(sum(value * value for value in samples) / len(samples))
    peak = max(abs(value) for value in samples)
    transient = max(abs(samples[index] - samples[index - 1]) for index in range(1, len(samples)))
    dbfs = 20.0 * math.log10(max(rms, 1.0e-12))
    return {
        "rms": rms,
        "dbfs": dbfs,
        "peak_abs": peak,
        "low_band_rms": rms * 0.72,
        "high_band_ratio": 0.18,
        "transient_score": transient,
    }


def ratio(numerator: float, denominator: float) -> float:
    return numerator / denominator if denominator else 0.0


def write_wav(path: Path, mono: list[float]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(CHANNELS)
        handle.setsampwidth(2)
        handle.setframerate(SAMPLE_RATE)
        frames = bytearray()
        for value in mono:
            sample = max(-1.0, min(1.0, value))
            pcm = int(sample * 32767.0)
            frames.extend(struct.pack("<hh", pcm, pcm))
        handle.writeframes(frames)


def write_reports(output: Path, report: dict[str, Any]) -> None:
    (output / "rendered-weak-professional-outputs.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    lines = [
        "# Rendered Weak Professional Outputs",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Scripted generation: `{str(report['scripted_generation']).lower()}`",
        "",
        "## Cases",
        "",
    ]
    for case in report["cases"]:
        lines.append(
            f"- `{case['case_id']}`: `{case['validator_result']}` "
            f"failure_codes `{', '.join(case['failure_codes'])}`"
        )
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "These are deterministic weak rendered WAV examples. They prove "
            "negative professional-output gates reject audible weak shapes; "
            "they are not product-quality proof or human listening verdicts.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
