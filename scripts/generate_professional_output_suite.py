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
CHILDREN = {
    "dense_break": "riotbox.dense_break_performance_pack.v1",
    "pro_pressure_source_matrix": "riotbox.pro_pressure_source_matrix.v1",
    "professional_source_wav_pack": "riotbox.professional_source_wav_pack.v1",
    "professional_output_listening_pack": "riotbox.professional_output_listening_pack.v1",
    "destructive_variation": "riotbox.destructive_variation_professional.v1",
    "rendered_weak_professional_outputs": "riotbox.rendered_weak_professional_outputs.v1",
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
    listening = output / "professional-output-listening-pack"
    destructive = output / "destructive-variation"
    rendered_weak = output / "rendered-weak-professional-outputs"

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
    ]
    children = [summarize_child(child_id, path) for child_id, path in child_specs]
    identity = validate_listening_identity(
        output / "professional-output-listening-pack" / "professional-output-listening-pack.json"
    )
    failures = suite_failure_codes(children, identity)
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
        "evidence_boundary": boundary,
        "evidence_role": boundary.get("evidence_role"),
        "source_backed": boundary.get("source_backed"),
        "source_timing_backed": boundary.get("source_timing_backed"),
        "scripted_generation": boundary.get("scripted_generation"),
        "quality_proof": boundary.get("quality_proof"),
    }


def key_metrics(child_id: str, data: dict[str, Any]) -> dict[str, Any]:
    if child_id == "dense_break":
        proof = object_or_empty(data.get("proof"))
        metrics = object_or_empty(data.get("metrics"))
        full = object_or_empty(metrics.get("full_performance"))
        return {
            "full_to_source_rms_ratio": number(proof.get("full_to_source_rms_ratio")),
            "pressure_to_hook_rms_ratio": number(proof.get("pressure_to_hook_rms_ratio")),
            "restore_to_pressure_rms_ratio": number(proof.get("restore_to_pressure_rms_ratio")),
            "full_performance_peak_abs": number(full.get("peak_abs")),
        }
    if child_id == "pro_pressure_source_matrix":
        return {
            "case_count": int(number(data.get("case_count"))),
            "passed_case_count": int(number(data.get("passed_case_count"))),
            "failed_case_count": int(number(data.get("failed_case_count"))),
        }
    if child_id == "professional_source_wav_pack":
        cases = list_or_empty(data.get("cases"))
        peaks = [
            number(object_or_empty(case.get("metrics")).get("full_performance_peak_abs"))
            for case in cases
        ]
        return {
            "case_count": int(number(data.get("case_count"))),
            "passed_case_count": int(number(data.get("passed_case_count"))),
            "max_full_performance_peak_abs": max(peaks) if peaks else 0.0,
        }
    if child_id == "professional_output_listening_pack":
        return {
            "case_count": int(number(data.get("case_count"))),
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
            "stutter_to_hook_transient_ratio": number(
                metrics.get("stutter_to_hook_transient_ratio")
            ),
            "restore_to_pressure_rms_ratio": number(
                metrics.get("restore_to_pressure_rms_ratio")
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
        failures.extend(f"{case_id}:{code}" for code in case_failures)
        cases.append(
            {
                "case_id": case_id,
                "source_family": case.get("source_family"),
                "candidate": str(candidate),
                "review": str(review),
                "source_report": str(source_report),
                "failure_codes": case_failures,
            }
        )
    expected_families = ["dense_break", "sparse_bass_pressure", "tonal_hook"]
    families = sorted(str(case.get("source_family", "unknown")) for case in cases)
    if families != expected_families:
        failures.append("listening_source_family_coverage_mismatch")
    if int(number(report.get("case_count"))) != len(cases):
        failures.append("listening_case_count_mismatch")
    return {
        "result": "pass" if not failures else "fail",
        "case_count": len(cases),
        "source_families": families,
        "failure_codes": failures,
        "cases": cases,
    }


def suite_failure_codes(children: list[dict[str, Any]], identity: dict[str, Any]) -> list[str]:
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
        if not child.get("evidence_role"):
            failures.append(f"{child_id}:evidence_role_missing")
        for code in child["failure_codes"]:
            failures.append(f"{child_id}:{code}")
    if identity["result"] != "pass":
        failures.extend(f"listening_identity:{code}" for code in identity["failure_codes"])
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
