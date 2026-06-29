#!/usr/bin/env python3
"""Generate structured listening-review packs for professional output WAVs."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

from audio_qa_evidence_boundary import apply_evidence_boundary
from mc202_source_composed_review_gate import (
    MC202_GATE_FIELD,
    MC202_ROLE_FIELD,
    case_gate as mc202_source_composed_case_gate,
    pack_gate as mc202_source_composed_pack_gate,
    role_evidence_for_gate as mc202_role_evidence_for_gate,
)


SCHEMA = "riotbox.professional_output_listening_pack.v1"
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-professional-output-listening-pack")
DEFAULT_PROFESSIONAL_WAV_PACK = Path("artifacts/audio_qa/local-professional-source-wav-pack")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--professional-wav-pack", type=Path, default=DEFAULT_PROFESSIONAL_WAV_PACK)
    parser.add_argument("--date", default="local-professional-output-listening-pack")
    parser.add_argument("--label-created-at")
    parser.add_argument("--ticket", default="RIOTBOX-1197")
    parser.add_argument("--reuse-professional-wav-pack", action="store_true")
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

    if args.reuse_professional_wav_pack:
        require_professional_wav_pack(professional_wav_pack)
    else:
        render_professional_wav_pack(repo, professional_wav_pack, args.date)
    professional_report = read_json(professional_wav_pack / "professional-source-wav-pack.json")
    dense_case = render_dense_case(repo, output, args.date)
    cases = [dense_case] + [
        case_from_professional_report(professional_wav_pack, case)
        for case in professional_report["cases"]
    ]

    label_created_at = args.label_created_at or date.today().isoformat()
    review_cases = [
        create_review_pack(repo, output, args.ticket, case, label_created_at)
        for case in cases
    ]
    mc202_gate = mc202_source_composed_pack_gate(review_cases)
    report = {
        "schema": SCHEMA,
        "schema_version": 1,
        "result": "pass",
        "agent_verdict": "agent_promising",
        "human_verdict": "unverified",
        "case_count": len(review_cases),
        MC202_GATE_FIELD: mc202_gate,
        "cases": review_cases,
    }
    apply_evidence_boundary(
        report,
        evidence_role="listening_review_scaffold",
        source_backed=True,
        source_timing_backed=True,
        scripted_generation=True,
        notes=(
            "Listening pack scaffolds human review for scripted diagnostic "
            "candidate renders. It does not convert candidates into quality proof."
        ),
    )
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


def require_professional_wav_pack(output: Path) -> None:
    report_path = output / "professional-source-wav-pack.json"
    if not report_path.is_file():
        raise SystemExit(f"missing professional WAV pack report: {report_path}")
    report = read_json(report_path)
    if report.get("schema") != "riotbox.professional_source_wav_pack.v1":
        raise SystemExit(f"invalid professional WAV pack schema: {report_path}")
    if report.get("result") != "pass":
        raise SystemExit(f"professional WAV pack must pass before reuse: {report_path}")


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
    return apply_evidence_boundary(
        {
        "case_id": "dense_beat03_130",
        "source_family": "dense_break",
        "source": "data/test_audio/examples/Beat03_130BPM(Full).wav",
        "output": str(case_dir),
        "candidate": str(case_dir / report["files"]["full_performance"]),
        "source_report": str(case_dir / "performance-report.json"),
        "source_report_sha256": sha256_file(case_dir / "performance-report.json"),
        "expected": "Dense break should hit with clear chop hook, pressure lift, destructive stutter, and bigger restore.",
        },
        evidence_role="listening_review_scaffold",
        source_backed=bool(report.get("source_backed")),
        source_timing_backed=bool(report.get("source_timing_backed")),
        scripted_generation=bool(report.get("scripted_generation")),
    )


def case_from_professional_report(base: Path, case: dict) -> dict:
    case_dir = base / case["case_id"]
    return apply_evidence_boundary(
        {
        "case_id": case["case_id"],
        "source_family": case["source_family"],
        "source": case["source"],
        "output": str(case_dir),
        "candidate": str(case_dir / case["audio_files"]["full_performance"]),
        "source_report": str(case_dir / "performance-report.json"),
        "source_report_sha256": sha256_file(case_dir / "performance-report.json"),
        "expected": expected_for_family(case["source_family"]),
        },
        evidence_role="listening_review_scaffold",
        source_backed=bool(case.get("source_backed", True)),
        source_timing_backed=bool(case.get("source_timing_backed", True)),
        scripted_generation=bool(case.get("scripted_generation", True)),
    )


def expected_for_family(source_family: str) -> str:
    if source_family == "tonal_hook":
        return "Tonal source should become a playable W-30 chop hook with pressure support, not a static source copy."
    if source_family == "sparse_bass_pressure":
        return "Sparse source should gain low-end authority and MC-202/TR-909 pressure without masking the source."
    return "Review whether the generated audio has source character, hook, and stage-meaningful impact."


def create_review_pack(
    repo: Path,
    output: Path,
    ticket: str,
    case: dict,
    label_created_at: str,
) -> dict:
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
    source_report_data = read_json(Path(case["source_report"]))
    demo_readiness = demo_readiness_reasons(case, source_report_data)
    mc202_gate = mc202_source_composed_case_gate(case, source_report_data)
    mc202_role = mc202_role_evidence_for_gate(mc202_gate)
    review["demo_readiness"] = demo_readiness["demo_readiness"]
    review["demo_worthy_reason"] = demo_readiness["demo_worthy_reason"]
    review["not_demo_worthy_reason"] = demo_readiness["not_demo_worthy_reason"]
    review[MC202_GATE_FIELD] = mc202_gate
    review[MC202_ROLE_FIELD] = mc202_role
    review["audio_judge_label"] = build_audio_judge_label(
        output, case, label_created_at, mc202_gate, mc202_role
    )
    review_path.write_text(json.dumps(review, indent=2) + "\n")
    append_demo_readiness_to_prompt(review_dir / "prompt.md", demo_readiness)
    case_summary = {
        **case,
        **demo_readiness,
        "review": str(review_path),
        "review_sha256": sha256_file(review_path),
        "candidate_sha256": sha256_file(Path(case["candidate"])),
        "human_verdict": review["human_verdict"],
        "review_artifacts": review["artifacts"],
        MC202_GATE_FIELD: mc202_gate,
        MC202_ROLE_FIELD: mc202_role,
    }
    return apply_evidence_boundary(
        case_summary,
        evidence_role="listening_review_scaffold",
        source_backed=bool(case.get("source_backed")),
        source_timing_backed=bool(case.get("source_timing_backed")),
        scripted_generation=bool(case.get("scripted_generation")),
    )


def demo_readiness_reasons(case: dict, source_report: dict) -> dict[str, str]:
    proof = source_report.get("proof") if isinstance(source_report.get("proof"), dict) else {}
    strongest = str(proof.get("strongest_audible_element") or "unknown")
    survival = number(proof.get("rebuild_only_source_character_survival_score"))
    pressure = number(proof.get("pressure_to_hook_rms_ratio"))
    restore = number(proof.get("restore_to_pressure_rms_ratio"))
    family = str(case["source_family"])
    if family == "sparse_bass_pressure":
        worthy = (
            f"Worth review: `{strongest}` leads while source survival is "
            f"{survival:.2f} and bass pressure is the judgment target."
        )
    elif family == "tonal_hook":
        worthy = (
            f"Worth review: `{strongest}` leads while source survival is "
            f"{survival:.2f} and the W-30 chop should read as the hook."
        )
    else:
        worthy = (
            f"Worth review: `{strongest}` leads with pressure {pressure:.2f} "
            f"and restore {restore:.2f} against the hook."
        )
    not_ready = (
        f"Not demo-ready yet: `{family}` still has `human_verdict: unverified` "
        "and scripted diagnostics cannot claim product quality."
    )
    return {
        "demo_readiness": "unverified",
        "demo_worthy_reason": worthy,
        "not_demo_worthy_reason": not_ready,
    }


def append_demo_readiness_to_prompt(path: Path, reasons: dict[str, str]) -> None:
    if not path.is_file():
        return
    with path.open("a") as handle:
        handle.write("\n## Demo Readiness\n\n")
        handle.write(f"- {reasons['demo_worthy_reason']}\n")
        handle.write(f"- {reasons['not_demo_worthy_reason']}\n")


def build_audio_judge_label(
    output: Path,
    case: dict,
    label_created_at: str,
    mc202_gate: dict,
    mc202_role: dict,
) -> dict:
    source_report = Path(case["source_report"])
    source_report_data = read_json(source_report)
    files = source_report_data["files"]
    source_window = source_report.parent / files["source_window"]
    full_performance = Path(case["candidate"])
    agent_review = source_report.with_name("agent-review.json")
    require_file(source_report)
    require_file(agent_review)
    require_file(source_window)
    require_file(full_performance)
    tags = reason_tags_for_family(case["source_family"])
    return {
        "created_at": label_created_at,
        "source_family": case["source_family"],
        "source_id": case["case_id"],
        "review_pack_schema": SCHEMA,
        "review_pack_id": f"{output.name}:{case['case_id']}",
        "artifact_identity": {
            "performance_report_sha256": sha256_file(source_report),
            "agent_review_sha256": sha256_file(agent_review),
            "audio_sha256": {
                "source_window": sha256_file(source_window),
                "full_performance": sha256_file(full_performance),
            },
        },
        "artifact_paths": {
            "performance_report": str(source_report),
            "agent_review": str(agent_review),
            "audio": {
                "source_window": str(source_window),
                "full_performance": str(full_performance),
            },
        },
        "reason_tags": tags,
        MC202_GATE_FIELD: mc202_gate,
        MC202_ROLE_FIELD: mc202_role,
        "summary": default_label_summary(case["source_family"]),
    }


def reason_tags_for_family(source_family: str) -> dict:
    if source_family == "sparse_bass_pressure":
        return {
            "hook_clarity": "weak",
            "hardest_hit": "bass",
            "bass_pressure": "strong",
            "destructive_contrast": "present",
            "source_character": "source_transformed_but_present",
            "replay_value_after_eight_bars": "medium",
        }
    if source_family == "tonal_hook":
        return {
            "hook_clarity": "clear",
            "hardest_hit": "chop",
            "bass_pressure": "present",
            "destructive_contrast": "present",
            "source_character": "source_transformed_but_present",
            "replay_value_after_eight_bars": "high",
        }
    return {
        "hook_clarity": "clear",
        "hardest_hit": "break_transient",
        "bass_pressure": "present",
        "destructive_contrast": "strong",
        "source_character": "source_transformed_but_present",
        "replay_value_after_eight_bars": "high",
    }


def default_label_summary(source_family: str) -> str:
    if source_family == "sparse_bass_pressure":
        return "Professional sparse-bass output review with bass pressure as the main judgment target."
    if source_family == "tonal_hook":
        return "Professional tonal-hook output review with W-30 chop identity as the main judgment target."
    return "Professional dense-break output review with hook, pressure, destructive contrast, and restore impact as the judgment targets."


def require_file(path: Path) -> None:
    if not path.is_file():
        raise SystemExit(f"missing required review artifact: {path}")


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


def number(value: object) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    return 0.0


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
        lines.append(f"  - Demo reason: {case['demo_worthy_reason']}")
        lines.append(f"  - Not demo-ready: {case['not_demo_worthy_reason']}")
    lines.extend(["", "## Boundary", ""])
    lines.append("This pack prepares human review. It does not record a human musical pass.")
    (output / "README.md").write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    sys.exit(main())
