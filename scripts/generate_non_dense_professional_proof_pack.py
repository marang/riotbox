#!/usr/bin/env python3
"""Generate non-dense professional-output proof diagnostics."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

from audio_qa_evidence_boundary import apply_evidence_boundary


SCHEMA = "riotbox.non_dense_professional_proof_pack.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-non-dense-professional-proof-pack")
DEFAULT_SOURCE_WAV_PACK = Path("artifacts/audio_qa/local-professional-source-wav-pack")
CASES = [
    {
        "case_id": "tonal_rusharp_120",
        "source_family": "tonal_hook",
        "validator": "scripts/validate_tonal_hook_professional.py",
        "fixture_manifest": "scripts/fixtures/automated_musical_fitness/valid_tonal_hook_chop/manifest.json",
        "expected": (
            "Tonal material should keep a clear W-30 hook, transformed source "
            "presence, and generated support without copying the source."
        ),
        "guarded_failure_classes": [
            "hookless output",
            "fallback collapse",
            "source copy",
            "static bars",
            "weak generated support",
        ],
    },
    {
        "case_id": "sparse_kicksnr_120",
        "source_family": "sparse_bass_pressure",
        "validator": "scripts/validate_sparse_bass_pressure_professional.py",
        "fixture_manifest": "scripts/fixtures/automated_musical_fitness/valid_sparse_bass_pulse/manifest.json",
        "expected": (
            "Sparse drums should gain low-band pressure, MC-202 bass movement, "
            "TR-909 support, and grid-aligned source response without masking."
        ),
        "guarded_failure_classes": [
            "weak bass pressure",
            "fallback collapse",
            "static bars",
            "masked source response",
            "loose source-grid alignment",
        ],
    },
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-non-dense-professional-proof-pack")
    parser.add_argument("--professional-source-wav-pack", type=Path, default=DEFAULT_SOURCE_WAV_PACK)
    parser.add_argument("--reuse-professional-source-wav-pack", action="store_true")
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    output = resolve_repo_path(repo, args.output)
    source_wav_pack = resolve_repo_path(repo, args.professional_source_wav_pack)
    ensure_safe_output(repo, output)
    ensure_safe_output(repo, source_wav_pack)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    if not args.reuse_professional_source_wav_pack:
        render_source_wav_pack(repo, source_wav_pack, args.date)
    source_wav_report = read_json(source_wav_pack / "professional-source-wav-pack.json")
    cases = [build_case(repo, output, source_wav_report, source_wav_pack, case) for case in CASES]
    failures = report_failure_codes(source_wav_report, cases)
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass" if not failures else "fail",
        "agent_verdict": "agent_promising" if not failures else "agent_fail",
        "human_verdict": "unverified",
        "case_count": len(cases),
        "passed_case_count": sum(1 for case in cases if case["result"] == "pass"),
        "failed_case_count": sum(1 for case in cases if case["result"] != "pass"),
        "professional_source_wav_pack": str(source_wav_pack / "professional-source-wav-pack.json"),
        "professional_source_wav_pack_sha256": sha256_file(
            source_wav_pack / "professional-source-wav-pack.json"
        ),
        "cases": cases,
        "failure_codes": failures,
    }
    apply_evidence_boundary(
        report,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "Non-dense professional proof pack joins rendered tonal/sparse WAV "
            "diagnostics with source-family validator reports. It is not "
            "product-quality proof while render decisions remain scripted."
        ),
    )
    write_reports(output, report)
    if failures:
        print("non-dense professional proof pack failed: " + ", ".join(failures), file=sys.stderr)
        return 1
    print(f"non-dense professional proof pack written to {output}")
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


def render_source_wav_pack(repo: Path, output: Path, date: str) -> None:
    run_or_exit(
        repo,
        [
            sys.executable,
            "scripts/generate_professional_source_wav_pack.py",
            "--output",
            str(output),
            "--date",
            f"{date}-source-wav",
        ],
        output / "source-wav-render.log",
    )


def build_case(
    repo: Path,
    output: Path,
    source_wav_report: dict[str, Any],
    source_wav_pack: Path,
    spec: dict[str, Any],
) -> dict[str, Any]:
    case_id = str(spec["case_id"])
    case_dir = output / case_id
    case_dir.mkdir(parents=True, exist_ok=True)
    source_case = find_case(source_wav_report, case_id)
    validator_json = case_dir / "source-family-validator.json"
    validator_md = case_dir / "source-family-validator.md"
    run_or_exit(
        repo,
        [
            sys.executable,
            str(spec["validator"]),
            "--json-output",
            str(validator_json),
            "--markdown-output",
            str(validator_md),
            str(spec["fixture_manifest"]),
        ],
        case_dir / "source-family-validator.log",
    )
    validator_report = read_json(validator_json)
    fixture_manifest = repo / str(spec["fixture_manifest"])
    audio_path = source_wav_pack / case_id / source_case["audio_files"]["full_performance"]
    source_report_path = source_wav_pack / case_id / "performance-report.json"
    review_prompt = case_dir / "review-prompt.md"
    review_prompt.write_text(render_review_prompt(spec, source_case, validator_report, audio_path))
    failures = case_failure_codes(source_case, validator_report, audio_path, source_report_path)
    case = {
        "case_id": case_id,
        "source_family": spec["source_family"],
        "result": "pass" if not failures else "fail",
        "agent_verdict": "agent_promising" if not failures else "agent_fail",
        "human_verdict": "unverified",
        "source": source_case["source"],
        "rendered_audio": str(audio_path),
        "rendered_audio_sha256": sha256_file(audio_path) if audio_path.is_file() else None,
        "source_report": str(source_report_path),
        "source_report_sha256": sha256_file(source_report_path) if source_report_path.is_file() else None,
        "source_family_manifest": str(fixture_manifest),
        "source_family_manifest_sha256": sha256_file(fixture_manifest),
        "source_family_validator": str(validator_json),
        "source_family_validator_sha256": sha256_file(validator_json),
        "review_prompt": str(review_prompt),
        "review_prompt_sha256": sha256_file(review_prompt),
        "expected": spec["expected"],
        "guarded_failure_classes": spec["guarded_failure_classes"],
        "source_wav_metrics": source_case["metrics"],
        "source_family_metrics": validator_report["metrics"],
        "failure_codes": failures,
    }
    return apply_evidence_boundary(
        case,
        evidence_role="diagnostic",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
    )


def find_case(report: dict[str, Any], case_id: str) -> dict[str, Any]:
    for case in list_or_empty(report.get("cases")):
        if case.get("case_id") == case_id:
            return case
    raise ValueError(f"missing professional source WAV case: {case_id}")


def case_failure_codes(
    source_case: dict[str, Any],
    validator_report: dict[str, Any],
    audio_path: Path,
    source_report_path: Path,
) -> list[str]:
    failures = []
    if source_case.get("result") != "pass":
        failures.append("source_wav_case_not_passed")
    if validator_report.get("result") != "pass":
        failures.append("source_family_validator_not_passed")
    if source_case.get("human_verdict") != "unverified":
        failures.append("unexpected_source_wav_human_verdict")
    if validator_report.get("human_verdict") != "unverified":
        failures.append("unexpected_validator_human_verdict")
    if source_case.get("quality_proof") is not False:
        failures.append("source_wav_quality_proof_not_false")
    if validator_report.get("quality_proof") is not False:
        failures.append("validator_quality_proof_not_false")
    if not audio_path.is_file():
        failures.append("rendered_audio_missing")
    if not source_report_path.is_file():
        failures.append("source_report_missing")
    if validator_report.get("scripted_generation") is not True:
        failures.append("validator_scripted_generation_not_true")
    return failures


def report_failure_codes(source_wav_report: dict[str, Any], cases: list[dict[str, Any]]) -> list[str]:
    failures = []
    if source_wav_report.get("result") != "pass":
        failures.append("professional_source_wav_pack_not_passed")
    expected_families = ["sparse_bass_pressure", "tonal_hook"]
    families = sorted(str(case.get("source_family", "unknown")) for case in cases)
    if families != expected_families:
        failures.append("source_family_coverage_mismatch")
    for case in cases:
        if case["result"] != "pass":
            failures.append(f"{case['case_id']}:not_passed")
        for code in case["failure_codes"]:
            failures.append(f"{case['case_id']}:{code}")
    return failures


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


def render_review_prompt(
    spec: dict[str, Any],
    source_case: dict[str, Any],
    validator_report: dict[str, Any],
    audio_path: Path,
) -> str:
    return "\n".join(
        [
            f"# {spec['case_id']} Listening Prompt",
            "",
            f"- Source family: `{spec['source_family']}`",
            f"- Candidate: `{audio_path}`",
            f"- Human verdict: `unverified`",
            f"- Expected: {spec['expected']}",
            f"- Source WAV result: `{source_case['result']}`",
            f"- Source-family validator result: `{validator_report['result']}`",
            "",
            "Listen for hook/pressure identity, transformed source presence, "
            "fallback collapse, identical-output collapse, weak contrast, and "
            "whether the result would be worth triggering again.",
        ]
    ) + "\n"


def read_json(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text())
    if not isinstance(data, dict):
        raise ValueError(f"expected JSON object: {path}")
    return data


def list_or_empty(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_reports(output: Path, report: dict[str, Any]) -> None:
    (output / "non-dense-professional-proof-pack.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    lines = [
        "# Non-Dense Professional Proof Pack",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Evidence role: `{report['evidence_role']}`",
        f"- Quality proof: `{str(report['quality_proof']).lower()}`",
        f"- Cases: `{report['passed_case_count']}/{report['case_count']}` passing",
        "",
        "## Cases",
        "",
    ]
    for case in report["cases"]:
        lines.append(
            f"- `{case['case_id']}` `{case['source_family']}`: `{case['result']}` "
            f"audio `{case['rendered_audio']}` prompt `{case['review_prompt']}`"
        )
        if case["failure_codes"]:
            lines.append(f"  failure_codes: `{', '.join(case['failure_codes'])}`")
    lines.extend(
        [
            "",
            "## Boundary",
            "",
            "This pack extends professional-output diagnostics beyond dense "
            "breaks by joining rendered tonal/sparse WAV identity with "
            "source-family validator evidence and review prompts. It remains "
            "diagnostic evidence, not product-quality proof.",
        ]
    )
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
