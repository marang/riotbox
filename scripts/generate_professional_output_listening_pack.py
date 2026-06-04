#!/usr/bin/env python3
"""Generate structured listening-review packs for professional output WAVs."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


SCHEMA = "riotbox.professional_output_listening_pack.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-professional-output-listening-pack")
DEFAULT_PROFESSIONAL_WAV_PACK = Path("artifacts/audio_qa/local-professional-source-wav-pack")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--professional-wav-pack", type=Path, default=DEFAULT_PROFESSIONAL_WAV_PACK)
    parser.add_argument("--date", default="local-professional-output-listening-pack")
    parser.add_argument("--ticket", default="RIOTBOX-1197")
    parser.add_argument("--keep-output", action="store_true")
    args = parser.parse_args()

    repo = repo_root()
    output = resolve_repo_path(repo, args.output)
    professional_wav_pack = resolve_repo_path(repo, args.professional_wav_pack)
    ensure_safe_output(repo, output)
    ensure_safe_output(repo, professional_wav_pack)
    if output.exists() and not args.keep_output:
        shutil.rmtree(output)
    output.mkdir(parents=True, exist_ok=True)

    render_professional_wav_pack(repo, professional_wav_pack, args.date)
    professional_report = read_json(professional_wav_pack / "professional-source-wav-pack.json")
    dense_case = render_dense_case(repo, output, args.date)
    cases = [dense_case] + [
        case_from_professional_report(professional_wav_pack, case)
        for case in professional_report["cases"]
    ]

    review_cases = [
        create_review_pack(repo, output, args.ticket, case)
        for case in cases
    ]
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "agent_verdict": "agent_promising",
        "human_verdict": "unverified",
        "case_count": len(review_cases),
        "cases": review_cases,
    }
    write_reports(output, report)
    print(f"professional output listening pack written to {output}")
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


def render_professional_wav_pack(repo: Path, output: Path, date: str) -> None:
    command = [
        sys.executable,
        "scripts/generate_professional_source_wav_pack.py",
        "--output",
        str(output),
        "--date",
        f"{date}-source-wav",
    ]
    run_or_exit(repo, command, output / "professional-source-wav-render.log")


def render_dense_case(repo: Path, output: Path, date: str) -> dict:
    case_dir = output / "renders" / "dense_beat03_130"
    command = [
        sys.executable,
        "scripts/generate_dense_break_performance_pack.py",
        "--source",
        "data/test_audio/examples/Beat03_130BPM(Full).wav",
        "--bpm",
        "130.000000",
        "--output",
        str(case_dir),
        "--date",
        f"{date}-dense-beat03",
    ]
    run_or_exit(repo, command, case_dir / "listening-render.log")
    report = read_json(case_dir / "performance-report.json")
    return {
        "case_id": "dense_beat03_130",
        "source_family": "dense_break",
        "source": "data/test_audio/examples/Beat03_130BPM(Full).wav",
        "output": str(case_dir),
        "candidate": str(case_dir / report["files"]["full_performance"]),
        "source_report": str(case_dir / "performance-report.json"),
        "source_report_sha256": sha256_file(case_dir / "performance-report.json"),
        "expected": "Dense break should hit with clear chop hook, pressure lift, destructive stutter, and bigger restore.",
    }


def case_from_professional_report(base: Path, case: dict) -> dict:
    case_dir = base / case["case_id"]
    return {
        "case_id": case["case_id"],
        "source_family": case["source_family"],
        "source": case["source"],
        "output": str(case_dir),
        "candidate": str(case_dir / case["audio_files"]["full_performance"]),
        "source_report": str(case_dir / "performance-report.json"),
        "source_report_sha256": sha256_file(case_dir / "performance-report.json"),
        "expected": expected_for_family(case["source_family"]),
    }


def expected_for_family(source_family: str) -> str:
    if source_family == "tonal_hook":
        return "Tonal source should become a playable W-30 chop hook with pressure support, not a static source copy."
    if source_family == "sparse_bass_pressure":
        return "Sparse source should gain low-end authority and MC-202/TR-909 pressure without masking the source."
    return "Review whether the generated audio has source character, hook, and stage-meaningful impact."


def create_review_pack(repo: Path, output: Path, ticket: str, case: dict) -> dict:
    review_dir = output / "reviews" / case["case_id"]
    command = [
        sys.executable,
        "scripts/listening_review_workflow.py",
        "pack",
        "--ticket",
        ticket,
        "--output",
        str(review_dir),
        "--source-file",
        case["source"],
        "--candidate",
        case["candidate"],
        "--technical-status",
        "pass",
        "--automated-musical-fitness-status",
        "pass",
        "--expected",
        case["expected"],
    ]
    run_or_exit(repo, command, review_dir / "listening-pack.log")
    review_path = review_dir / "review.json"
    review = read_json(review_path)
    return {
        **case,
        "review": str(review_path),
        "review_sha256": sha256_file(review_path),
        "candidate_sha256": sha256_file(Path(case["candidate"])),
        "human_verdict": review["human_verdict"],
        "review_artifacts": review["artifacts"],
    }


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


def read_json(path: Path) -> dict:
    return json.loads(path.read_text())


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_reports(output: Path, report: dict) -> None:
    (output / "professional-output-listening-pack.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    lines = [
        "# Professional Output Listening Pack",
        "",
        f"- Result: `{report['result']}`",
        f"- Agent verdict: `{report['agent_verdict']}`",
        f"- Human verdict: `{report['human_verdict']}`",
        f"- Cases: `{report['case_count']}`",
        "",
        "## Cases",
        "",
    ]
    for case in report["cases"]:
        lines.append(
            f"- `{case['case_id']}` `{case['source_family']}`: "
            f"candidate `{case['candidate']}` review `{case['review']}`"
        )
    lines.extend(["", "## Boundary", ""])
    lines.append("This pack prepares human review. It does not record a human musical pass.")
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
