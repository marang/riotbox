#!/usr/bin/env python3
"""Render and route professional-output diagnostics for edge source families."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary, evidence_boundary_failure_codes
from route_weak_output_fixes import route_signals


SCHEMA = "riotbox.edge_source_professional_diagnostics.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-edge-source-professional-diagnostics")
DEFAULT_CORPUS = Path("docs/benchmarks/sound_excellence_source_corpus_v1.json")
CASES = [
    {
        "case_id": "pad_noise_fadapad_120",
        "source_family": "pad_noise",
        "corpus_case_id": "pad_noise_fadapad_120",
        "expected_timing_confidence": "degraded",
        "expected_grid_use": "unavailable",
        "expected_policy_risk": "pad/noise material must not be promoted as dense-break proof",
    },
    {
        "case_id": "bad_timing_beat20_128",
        "source_family": "bad_timing",
        "corpus_case_id": "bad_timing_beat20_128",
        "expected_timing_confidence": "candidate_ambiguous",
        "expected_grid_use": "manual_confirm_only",
        "expected_policy_risk": "ambiguous downbeat material must route to timing/UI confirmation before bar-locked moves",
    },
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--corpus", type=Path, default=DEFAULT_CORPUS)
    parser.add_argument("--date", default="local-edge-source-professional-diagnostics")
    parser.add_argument("--keep-output", action="store_true")
    parser.add_argument("--validate-report", type=Path)
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json(args.validate_report)
            failures = report_failure_codes(report)
            if failures:
                raise ValueError(", ".join(failures))
            print(f"valid edge-source professional diagnostics: {args.validate_report}")
            return 0

        repo = repo_root()
        output = resolve_repo_path(repo, args.output)
        corpus_path = resolve_repo_path(repo, args.corpus)
        ensure_safe_output(repo, output)
        if output.exists() and not args.keep_output:
            shutil.rmtree(output)
        output.mkdir(parents=True, exist_ok=True)

        corpus = read_json(corpus_path)
        cases = [render_case(repo, output, args.date, corpus, spec) for spec in CASES]
        failures = case_failure_codes(cases)
        report = {
            "schema": SCHEMA,
            "schema_version": 1,
            "result": "pass" if not failures else "fail",
            "agent_verdict": "agent_promising" if not failures else "agent_fail",
            "human_verdict": "unverified",
            "automated_musical_approval": False,
            "case_count": len(cases),
            "diagnostic_case_count": len(cases),
            "weak_routed_case_count": sum(
                1 for case in cases if case["musical_risk_status"] == "weak_routed"
            ),
            "source_families": sorted({case["source_family"] for case in cases}),
            "guarded_failure_classes": [
                "silence",
                "fallback_collapse",
                "identical_output",
                "missing_source_family_metadata",
                "bar_locked_policy_on_bad_timing",
            ],
            "cases": cases,
            "failure_codes": failures,
            "boundary": (
                "Edge-source diagnostics deliberately render pad/noise and bad-timing "
                "sources through the current professional-output path, then route the "
                "weak/risky result to concrete production fixes. Passing this report "
                "means the risk was detected and bounded, not that the sound is "
                "musically approved."
            ),
        }
        apply_evidence_boundary(
            report,
            evidence_role="diagnostic",
            source_backed=True,
            source_timing_backed=True,
            scripted_generation=True,
            notes=(
                "This is source-backed diagnostic coverage for edge families. "
                "It is not product-quality proof while timing and arrangement "
                "responses remain scripted."
            ),
        )
        validation_failures = report_failure_codes(report)
        if validation_failures:
            report["result"] = "fail"
            report["agent_verdict"] = "agent_fail"
            report["failure_codes"] = sorted(set(failures + validation_failures))
        write_reports(output, report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"edge-source professional diagnostics failed: {error}", file=sys.stderr)
        return 1

    if report["result"] != "pass":
        print(
            "edge-source professional diagnostics failed: "
            + ", ".join(report["failure_codes"]),
            file=sys.stderr,
        )
        return 1
    print(f"edge-source professional diagnostics written to {output}")
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
    corpus: dict[str, Any],
    spec: dict[str, Any],
) -> dict[str, Any]:
    case_dir = output / str(spec["case_id"])
    case_dir.mkdir(parents=True, exist_ok=True)
    corpus_entry = corpus_entry_for(corpus, str(spec["corpus_case_id"]))
    source = str(corpus_entry["source_path"])
    bpm = float(corpus_entry["bpm_hint"])

    source_timing_path = case_dir / "source-timing.json"
    run_or_exit(
        repo,
        [
            "cargo",
            "run",
            "-p",
            "riotbox-audio",
            "--bin",
            "source_timing_probe",
            "--",
            "--json",
            source,
        ],
        source_timing_path,
        stderr_path=case_dir / "source-timing.log",
    )
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/validate_source_timing_probe_json.py",
            str(source_timing_path),
        ],
        case_dir / "source-timing-validation.log",
    )
    render_dir = case_dir / "render"
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_dense_break_performance_pack.py",
            "--source",
            source,
            "--bpm",
            f"{bpm:.6f}",
            "--output",
            str(render_dir),
            "--date",
            f"{date}-{spec['case_id']}",
        ],
        case_dir / "render.log",
    )
    source_timing = read_json(source_timing_path)
    performance_report_path = render_dir / "performance-report.json"
    performance_report = read_json(performance_report_path)
    files = object_or_empty(performance_report.get("files"))
    proof = object_or_empty(performance_report.get("proof"))
    metrics = object_or_empty(performance_report.get("metrics"))
    source_policy = object_or_empty(performance_report.get("source_policy"))
    pressure_lift_policy = object_or_empty(source_policy.get("pressure_lift_policy"))
    source_window = render_dir / str(files.get("source_window", ""))
    rendered_audio = render_dir / str(files.get("full_performance", ""))
    weak_signals = weak_output_signals(
        str(spec["source_family"]),
        source_timing,
        pressure_lift_policy,
        proof,
    )
    route = route_signals(weak_signals, reason_tags_for_case(str(spec["source_family"])), [])
    case = {
        "case_id": spec["case_id"],
        "source_family": spec["source_family"],
        "corpus_case_id": spec["corpus_case_id"],
        "source": source,
        "bpm": bpm,
        "expected_policy_risk": spec["expected_policy_risk"],
        "human_verdict": "unverified",
        "automated_musical_approval": False,
        "musical_risk_status": "weak_routed",
        "source_timing": {
            "report": str(source_timing_path),
            "report_sha256": sha256_file(source_timing_path),
            "confidence_result": source_timing.get("confidence_result"),
            "grid_use": source_timing.get("grid_use"),
            "readiness": source_timing.get("readiness"),
            "cue": source_timing.get("cue"),
            "actionability": source_timing.get("actionability"),
            "warning_codes": string_list(source_timing.get("warning_codes")),
            "beat_status": source_timing.get("beat_status"),
            "downbeat_status": source_timing.get("downbeat_status"),
            "phrase_status": source_timing.get("phrase_status"),
            "alternate_downbeat_phase_count": number_or_none(
                source_timing.get("alternate_downbeat_phase_count")
            ),
        },
        "rendered_audio": str(rendered_audio),
        "rendered_audio_sha256": sha256_file(rendered_audio),
        "source_window": str(source_window),
        "source_window_sha256": sha256_file(source_window),
        "performance_report": str(performance_report_path),
        "performance_report_sha256": sha256_file(performance_report_path),
        "pressure_lift_policy": pressure_lift_policy,
        "proof": {
            "w30_to_source_rms_ratio": number(proof.get("w30_to_source_rms_ratio")),
            "full_to_source_rms_ratio": number(proof.get("full_to_source_rms_ratio")),
            "source_to_performance_correlation": number(
                proof.get("source_to_performance_correlation")
            ),
            "pressure_lift_policy_decision_count": number(
                proof.get("pressure_lift_policy_decision_count")
            ),
        },
        "metrics": {
            "full_performance_rms": number(
                object_or_empty(metrics.get("full_performance")).get("rms")
            ),
            "full_performance_peak_abs": number(
                object_or_empty(metrics.get("full_performance")).get("peak_abs")
            ),
            "source_window_rms": number(object_or_empty(metrics.get("source_window")).get("rms")),
        },
        "weak_output_signals": weak_signals,
        "proposed_next_fix_category": route["proposed_next_fix_category"],
        "proposed_fix_categories": route["proposed_fix_categories"],
        "routing_reasons": route["routing_reasons"],
        "guarded_failure_classes": [
            "silence",
            "fallback_collapse",
            "identical_output",
            "missing_source_family_metadata",
        ],
        "diagnostic_failure_codes": diagnostic_failure_codes(
            spec,
            source_timing,
            pressure_lift_policy,
            proof,
            metrics,
            rendered_audio,
            source_window,
        ),
    }
    return apply_evidence_boundary(
        case,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
    )


def corpus_entry_for(corpus: dict[str, Any], case_id: str) -> dict[str, Any]:
    for entry in list_or_empty(corpus.get("entries")):
        if isinstance(entry, dict) and entry.get("case_id") == case_id:
            return entry
    raise ValueError(f"missing corpus case: {case_id}")


def weak_output_signals(
    source_family: str,
    source_timing: dict[str, Any],
    pressure_lift_policy: dict[str, Any],
    proof: dict[str, Any],
) -> list[str]:
    signals = []
    confidence = str(source_timing.get("confidence_result") or "")
    grid_use = str(source_timing.get("grid_use") or "")
    warnings = string_list(source_timing.get("warning_codes"))
    if confidence in {"degraded", "candidate_ambiguous"}:
        signals.append(confidence)
    if grid_use in {"unavailable", "manual_confirm_only"}:
        signals.append(grid_use)
    signals.extend(warnings)
    policy_family = str(pressure_lift_policy.get("source_family") or "")
    if source_family == "pad_noise":
        if policy_family == "pad_noise":
            signals.append("pad_noise_policy_path")
            signals.append("pad_noise_texture_gate")
        else:
            signals.append("pad_noise_policy_not_applied")
            if policy_family == "dense_break":
                signals.append("pad_noise_misclassified_as_dense_break")
            signals.append("source_not_transformed_for_pad_noise")
    if source_family == "bad_timing":
        signals.append("bar_locked_policy_on_bad_timing")
        if source_timing.get("downbeat_status") == "ambiguous":
            signals.append("ambiguous_downbeat")
    if number(proof.get("source_to_performance_correlation")) >= 0.975:
        signals.append("fallback_collapse_or_identical_output")
    return sorted(set(signals))


def reason_tags_for_case(source_family: str) -> dict[str, str]:
    if source_family == "pad_noise":
        return {
            "hook_clarity": "weak",
            "source_character": "source_lost",
            "destructive_contrast": "weak",
        }
    return {
        "hook_clarity": "weak",
        "source_character": "source_transformed_but_present",
        "replay_value_after_eight_bars": "low",
    }


def diagnostic_failure_codes(
    spec: dict[str, Any],
    source_timing: dict[str, Any],
    pressure_lift_policy: dict[str, Any],
    proof: dict[str, Any],
    metrics: dict[str, Any],
    rendered_audio: Path,
    source_window: Path,
) -> list[str]:
    failures = []
    if not str(spec.get("source_family") or ""):
        failures.append("missing_source_family_metadata")
    if source_timing.get("confidence_result") != spec["expected_timing_confidence"]:
        failures.append("source_timing_confidence_unexpected")
    if source_timing.get("grid_use") != spec["expected_grid_use"]:
        failures.append("source_timing_grid_use_unexpected")
    if not pressure_lift_policy.get("source_family"):
        failures.append("pressure_lift_source_family_missing")
    if spec.get("source_family") == "pad_noise" and pressure_lift_policy.get("source_family") != "pad_noise":
        failures.append("pad_noise_policy_not_applied")
    full = object_or_empty(metrics.get("full_performance"))
    if number(full.get("rms")) <= 0.01:
        failures.append("rendered_audio_silent")
    if rendered_audio.is_file() and source_window.is_file() and sha256_file(rendered_audio) == sha256_file(source_window):
        failures.append("identical_output")
    if number(proof.get("source_to_performance_correlation")) >= 0.975:
        failures.append("fallback_or_identical_output_collapse")
    return failures


def case_failure_codes(cases: list[dict[str, Any]]) -> list[str]:
    failures = []
    expected_families = ["bad_timing", "pad_noise"]
    families = sorted(str(case.get("source_family", "")) for case in cases)
    if families != expected_families:
        failures.append("source_family_coverage_mismatch")
    for case in cases:
        for code in string_list(case.get("diagnostic_failure_codes")):
            failures.append(f"{case['case_id']}:{code}")
        if not case.get("proposed_fix_categories"):
            failures.append(f"{case['case_id']}:missing_fix_routing")
        if case.get("human_verdict") != "unverified":
            failures.append(f"{case['case_id']}:unexpected_human_verdict")
        if case.get("quality_proof") is not False:
            failures.append(f"{case['case_id']}:quality_proof_not_false")
    return failures


def report_failure_codes(report: dict[str, Any]) -> list[str]:
    failures = []
    failures.extend(evidence_boundary_failure_codes(report))
    if report.get("schema") != SCHEMA:
        failures.append("schema_mismatch")
    if report.get("schema_version") != 1:
        failures.append("schema_version_mismatch")
    if report.get("human_verdict") != "unverified":
        failures.append("unexpected_human_verdict")
    if report.get("automated_musical_approval") is not False:
        failures.append("automated_musical_approval_not_false")
    if report.get("quality_proof") is not False:
        failures.append("quality_proof_not_false")
    if report.get("scripted_generation") is not True:
        failures.append("scripted_generation_not_true")
    cases = list_or_empty(report.get("cases"))
    if len(cases) < 2:
        failures.append("case_coverage_too_small")
    for case in cases:
        case_id = str(case.get("case_id", "unknown"))
        if not case.get("source_family"):
            failures.append(f"{case_id}:missing_source_family_metadata")
        if not object_or_empty(case.get("source_timing")).get("confidence_result"):
            failures.append(f"{case_id}:source_timing_missing")
        if not object_or_empty(case.get("pressure_lift_policy")).get("source_family"):
            failures.append(f"{case_id}:pressure_lift_source_family_missing")
        if (
            case.get("source_family") == "pad_noise"
            and object_or_empty(case.get("pressure_lift_policy")).get("source_family")
            != "pad_noise"
        ):
            failures.append(f"{case_id}:pad_noise_policy_not_applied")
        metrics = object_or_empty(case.get("metrics"))
        proof = object_or_empty(case.get("proof"))
        if number(metrics.get("full_performance_rms")) <= 0.01:
            failures.append(f"{case_id}:rendered_audio_silent")
        if case.get("rendered_audio_sha256") == case.get("source_window_sha256"):
            failures.append(f"{case_id}:identical_output")
        if number(proof.get("source_to_performance_correlation")) >= 0.975:
            failures.append(f"{case_id}:fallback_or_identical_output_collapse")
        if not case.get("proposed_fix_categories"):
            failures.append(f"{case_id}:missing_fix_routing")
        if case.get("human_verdict") != "unverified":
            failures.append(f"{case_id}:unexpected_human_verdict")
        if case.get("automated_musical_approval") is not False:
            failures.append(f"{case_id}:automated_musical_approval_not_false")
        if case.get("quality_proof") is not False:
            failures.append(f"{case_id}:quality_proof_not_false")
    families = sorted(str(case.get("source_family", "")) for case in cases)
    if families != ["bad_timing", "pad_noise"]:
        failures.append("source_family_coverage_mismatch")
    return sorted(set(failures))


def run_or_exit(
    repo: Path,
    command: list[str],
    stdout_path: Path,
    *,
    stderr_path: Path | None = None,
) -> None:
    result = subprocess.run(
        command,
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stdout_path.write_text(result.stdout)
    if stderr_path is not None:
        stderr_path.parent.mkdir(parents=True, exist_ok=True)
        stderr_path.write_text(result.stderr)
    elif result.stderr:
        stdout_path.write_text(result.stdout + ("\n" if result.stdout else "") + result.stderr)
    if result.returncode != 0:
        raise ValueError(f"command failed; see {stdout_path}")


def read_json(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    if not isinstance(data, dict):
        raise ValueError(f"expected JSON object: {path}")
    return data


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def list_or_empty(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if isinstance(item, str) and item]


def number(value: Any) -> float:
    if isinstance(value, bool) or value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


def number_or_none(value: Any) -> float | None:
    if isinstance(value, bool) or value is None:
        return None
    if isinstance(value, (int, float)):
        return float(value)
    return None


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_reports(output: Path, report: dict[str, Any]) -> None:
    (output / "edge-source-professional-diagnostics.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    lines = [
        "# Edge Source Professional Diagnostics",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Weak routed cases: `{report['weak_routed_case_count']}/{report['case_count']}`",
        "",
        "## Cases",
        "",
    ]
    for case in report["cases"]:
        lines.extend(
            [
                f"### `{case['case_id']}`",
                "",
                f"- Source family: `{case['source_family']}`",
                f"- Source: `{case['source']}`",
                f"- Rendered WAV: `{case['rendered_audio']}`",
                f"- Timing: `{case['source_timing']['confidence_result']}` / `{case['source_timing']['grid_use']}`",
                f"- Pressure policy classified as: `{case['pressure_lift_policy'].get('source_family')}`",
                f"- Proposed next fix: `{case['proposed_next_fix_category']}`",
                f"- Weak signals: `{', '.join(case['weak_output_signals'])}`",
                "",
            ]
        )
    lines.extend(["## Boundary", "", report["boundary"], ""])
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    raise SystemExit(main())
