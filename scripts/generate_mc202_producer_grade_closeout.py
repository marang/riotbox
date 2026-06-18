#!/usr/bin/env python3
"""Generate the RIOTBOX-1279 MC-202 producer-grade closeout gate."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from mc202_source_composed_review_gate import MC202_GATE_FIELD, pack_gate


SCHEMA = "riotbox.mc202_producer_grade_closeout.v1"
PACK_SCHEMA = "riotbox.professional_output_listening_pack.v1"
REAL_SOURCE_SCHEMA = "riotbox.mc202_real_source_listening_pack.v1"
DEFAULT_PROFESSIONAL_PACK = Path(
    "artifacts/audio_qa/local-professional-output-listening-pack/"
    "professional-output-listening-pack.json"
)
DEFAULT_REAL_SOURCE_PACK = Path(
    "artifacts/audio_qa/local-mc202-real-source-listening-pack/"
    "mc202-real-source-listening-pack.json"
)
DEFAULT_OUTPUT = Path("artifacts/audio_qa/local-mc202-producer-grade-closeout")
TICKET = "RIOTBOX-1279"
PARENT_TICKET = "RIOTBOX-1264"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--professional-pack", type=Path, default=DEFAULT_PROFESSIONAL_PACK)
    parser.add_argument("--real-source-pack", type=Path, default=DEFAULT_REAL_SOURCE_PACK)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--date", default="local-mc202-producer-grade-closeout")
    parser.add_argument("--validate-report", type=Path)
    parser.add_argument("--mutation-fixtures", action="store_true")
    args = parser.parse_args()

    try:
        if args.validate_report:
            report = read_json_object(args.validate_report)
        else:
            professional_pack = read_json_object(args.professional_pack)
            real_source_pack = read_json_object(args.real_source_pack)
            report = build_report(args, professional_pack, real_source_pack)
            write_reports(args.output, report)
        failures = validate_report(report)
        if failures:
            raise ValueError(", ".join(failures))
        if args.mutation_fixtures:
            run_mutation_fixtures(report)
    except (OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"invalid MC-202 producer-grade closeout: {error}", file=sys.stderr)
        return 1

    print("valid MC-202 producer-grade closeout")
    return 0


def build_report(
    args: argparse.Namespace,
    professional_pack: dict[str, Any],
    real_source_pack: dict[str, Any],
) -> dict[str, Any]:
    require(
        professional_pack.get("schema") == PACK_SCHEMA,
        f"{args.professional_pack}: schema must be {PACK_SCHEMA}",
    )
    require(
        real_source_pack.get("schema") == REAL_SOURCE_SCHEMA,
        f"{args.real_source_pack}: schema must be {REAL_SOURCE_SCHEMA}",
    )
    professional_cases = list_field(professional_pack, "cases", args.professional_pack)
    real_source_cases = list_field(real_source_pack, "cases", args.real_source_pack)
    gate = pack_gate(professional_cases)
    candidates = [candidate_summary(case) for case in professional_cases]
    source_scaffold = real_source_scaffold_summary(
        args.real_source_pack,
        real_source_pack,
        real_source_cases,
    )
    blockers = closeout_blockers(gate, candidates, source_scaffold)
    technical_result = "pass" if gate["result"] == "pass" and not source_scaffold["failure_codes"] else "fail"
    promotion_result = "blocked_for_human_promotion" if blockers else "ready_for_human_promotion"
    parent_state = "keep_open" if blockers else "ready_to_close"
    return {
        "schema": SCHEMA,
        "schema_version": 1,
        "ticket": TICKET,
        "parent_ticket": PARENT_TICKET,
        "phase": "P023",
        "created_at": args.date,
        "result": "pass" if technical_result == "pass" else "fail",
        "technical_closeout_result": technical_result,
        "producer_grade_promotion_result": promotion_result,
        "quality_claim_allowed": False,
        "demo_bank_promotion_allowed": False,
        "parent_ticket_state": parent_state,
        "human_verdict_boundary": (
            "MC-202 candidates are source-composed and reviewable, but producer-grade "
            "quality and demo-bank promotion require structured human pass/weak/fail "
            "verdicts. Unverified or primitive/template-only evidence must not claim "
            "product quality."
        ),
        "professional_pack": {
            "path": str(args.professional_pack),
            "schema": professional_pack["schema"],
            "human_verdict": str(professional_pack.get("human_verdict")),
            "quality_proof": professional_pack.get("quality_proof"),
            "mc202_source_composed_review_gate": gate,
        },
        "real_source_listening_scaffold": source_scaffold,
        "review_candidates": candidates,
        "blockers": blockers,
        "next_actions": next_actions(blockers, candidates),
        "musician_summary": musician_summary(blockers),
    }


def candidate_summary(case: dict[str, Any]) -> dict[str, Any]:
    gate = object_field(case, MC202_GATE_FIELD, Path("<professional-pack>"))
    return {
        "case_id": required_string(case, "case_id", Path("<professional-pack>")),
        "source_family": required_string(case, "source_family", Path("<professional-pack>")),
        "candidate": required_string(case, "candidate", Path("<professional-pack>")),
        "candidate_sha256": required_string(case, "candidate_sha256", Path("<professional-pack>")),
        "review": required_string(case, "review", Path("<professional-pack>")),
        "review_sha256": required_string(case, "review_sha256", Path("<professional-pack>")),
        "human_verdict": str(case.get("human_verdict")),
        "demo_readiness": str(case.get("demo_readiness")),
        "quality_proof": case.get("quality_proof"),
        "demo_worthy_reason": required_string(case, "demo_worthy_reason", Path("<professional-pack>")),
        "not_demo_worthy_reason": required_string(case, "not_demo_worthy_reason", Path("<professional-pack>")),
        "source_composed_evidence": gate.get("source_composed_evidence") is True,
        "primitive_or_template_only": gate.get("primitive_or_template_only") is True,
        "promotion_blocked_until_human_pass": gate.get("promotion_blocked_until_human_pass") is True,
        "template_only_blocks_promotion": gate.get("template_only_blocks_promotion") is True,
        "gate_result": str(gate.get("result")),
        "gate_failure_codes": list(gate.get("failure_codes", [])),
        "metrics": object_field(gate, "metrics", Path("<professional-pack>")),
    }


def real_source_scaffold_summary(
    path: Path,
    report: dict[str, Any],
    cases: list[dict[str, Any]],
) -> dict[str, Any]:
    failure_codes: list[str] = []
    if report.get("human_verdict") != "unverified":
        failure_codes.append("real_source_pack_human_verdict_not_unverified")
    if report.get("quality_proof") is not False:
        failure_codes.append("real_source_pack_claims_quality")
    dense_count = sum(1 for case in cases if case.get("source_family") == "dense_break")
    non_dense_count = len(cases) - dense_count
    if dense_count < 1:
        failure_codes.append("real_source_dense_case_missing")
    if non_dense_count < 1:
        failure_codes.append("real_source_non_dense_case_missing")
    for index, case in enumerate(cases):
        if case.get("quality_proof") is not False:
            failure_codes.append(f"real_source_case_{index}_claims_quality")
        if case.get("human_verdict") != "unverified":
            failure_codes.append(f"real_source_case_{index}_human_verdict_not_unverified")
        control = object_or_empty(case.get("primitive_ab_control"))
        if control.get("product_fallback_allowed") is not False:
            failure_codes.append(f"real_source_case_{index}_allows_product_fallback")
        if control.get("control_kind") != "primitive_renderer_non_product_control":
            failure_codes.append(f"real_source_case_{index}_control_kind_invalid")
    return {
        "path": str(path),
        "schema": report["schema"],
        "case_count": len(cases),
        "dense_case_count": dense_count,
        "non_dense_case_count": non_dense_count,
        "human_verdict": str(report.get("human_verdict")),
        "quality_proof": report.get("quality_proof"),
        "primitive_controls_are_product_output": False,
        "failure_codes": failure_codes,
    }


def closeout_blockers(
    gate: dict[str, Any],
    candidates: list[dict[str, Any]],
    source_scaffold: dict[str, Any],
) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    if gate["result"] != "pass":
        blockers.append(
            {
                "code": "mc202_source_composed_pack_gate_failed",
                "severity": "release_blocking",
                "reason": "Dense and non-dense MC-202 source-composed review coverage is required.",
                "failure_codes": gate["failure_codes"],
            }
        )
    if source_scaffold["failure_codes"]:
        blockers.append(
            {
                "code": "mc202_real_source_scaffold_invalid",
                "severity": "release_blocking",
                "reason": "Real-source listening scaffold must stay unverified and keep primitive controls out of product output.",
                "failure_codes": source_scaffold["failure_codes"],
            }
        )
    unverified = [
        candidate["case_id"]
        for candidate in candidates
        if candidate["human_verdict"] == "unverified"
        or candidate["demo_readiness"] == "unverified"
    ]
    if unverified:
        blockers.append(
            {
                "code": "structured_human_verdict_missing",
                "severity": "producer_grade_blocking",
                "case_ids": unverified,
                "reason": "Producer-grade closeout needs structured listener verdicts before quality or demo-bank promotion.",
            }
        )
    primitive_or_template = primitive_or_template_case_ids(candidates)
    if primitive_or_template:
        blockers.append(
            {
                "code": "primitive_or_template_candidate_blocks_promotion",
                "severity": "producer_grade_blocking",
                "case_ids": primitive_or_template,
                "reason": "Primitive/template-only MC-202 output may be reviewed as a blocker, not promoted.",
            }
        )
    return blockers


def next_actions(blockers: list[dict[str, Any]], candidates: list[dict[str, Any]]) -> list[dict[str, str]]:
    actions: list[dict[str, str]] = []
    if any(blocker["code"] == "structured_human_verdict_missing" for blocker in blockers):
        actions.append(
            {
                "category": "human_listening",
                "target": "mc202_source_composed_review_candidates",
                "software_benefit": "Keeps demo-bank promotion deterministic and hash-backed.",
                "musician_payoff": "The musician hears the exact candidate before it can be called producer-grade.",
            }
        )
    if any(blocker["code"] == "primitive_or_template_candidate_blocks_promotion" for blocker in blockers):
        actions.append(
            {
                "category": "production_fix",
                "target": "primitive_or_template_mc202_candidates",
                "software_benefit": "Routes non-source-composed cases to implementation work instead of hiding them as pass evidence.",
                "musician_payoff": "Weak or template-like bass parts become explicit fix work, not fake confidence.",
            }
        )
    if candidates:
        actions.append(
            {
                "category": "review_pack",
                "target": "candidate_wavs_and_prompts",
                "software_benefit": "Preserves exact WAV, metrics, prompt, and source-composed gate identities.",
                "musician_payoff": "Listening decisions can focus on hook, pressure, source character, and replay value.",
            }
        )
    return actions


def validate_report(report: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    check(report.get("schema") == SCHEMA, "schema_mismatch", failures)
    check(report.get("schema_version") == 1, "schema_version_mismatch", failures)
    check(report.get("ticket") == TICKET, "ticket_mismatch", failures)
    check(report.get("parent_ticket") == PARENT_TICKET, "parent_ticket_mismatch", failures)
    check(report.get("phase") == "P023", "phase_mismatch", failures)
    check(report.get("result") == "pass", "result_not_pass", failures)
    check(report.get("technical_closeout_result") == "pass", "technical_closeout_not_pass", failures)
    check(
        report.get("producer_grade_promotion_result") == "blocked_for_human_promotion",
        "producer_grade_promotion_not_blocked",
        failures,
    )
    check(report.get("quality_claim_allowed") is False, "quality_claim_allowed", failures)
    check(report.get("demo_bank_promotion_allowed") is False, "demo_bank_promotion_allowed", failures)
    check(report.get("parent_ticket_state") == "keep_open", "parent_ticket_not_kept_open", failures)
    boundary = str(report.get("human_verdict_boundary", ""))
    check("must not claim product quality" in boundary, "human_verdict_boundary_missing", failures)

    professional = object_or_empty(report.get("professional_pack"))
    gate = object_or_empty(professional.get("mc202_source_composed_review_gate"))
    check(gate.get("result") == "pass", "pack_gate_not_pass", failures)
    check(gate.get("quality_proof") is False, "pack_gate_claims_quality", failures)
    check(gate.get("dense_break_case_count", 0) >= 1, "dense_break_case_missing", failures)
    check(gate.get("non_dense_break_case_count", 0) >= 1, "non_dense_break_case_missing", failures)

    scaffold = object_or_empty(report.get("real_source_listening_scaffold"))
    check(scaffold.get("case_count", 0) >= 2, "real_source_case_count_too_low", failures)
    check(scaffold.get("dense_case_count", 0) >= 1, "real_source_dense_missing", failures)
    check(scaffold.get("non_dense_case_count", 0) >= 1, "real_source_non_dense_missing", failures)
    check(scaffold.get("quality_proof") is False, "real_source_claims_quality", failures)
    check(scaffold.get("primitive_controls_are_product_output") is False, "primitive_controls_are_product_output", failures)
    check(scaffold.get("failure_codes") == [], "real_source_scaffold_failures", failures)

    candidates = report.get("review_candidates")
    check(isinstance(candidates, list) and len(candidates) >= 2, "review_candidates_missing", failures)
    if isinstance(candidates, list):
        families = {str(candidate.get("source_family")) for candidate in candidates if isinstance(candidate, dict)}
        check("dense_break" in families, "candidate_dense_break_missing", failures)
        check(
            bool(families & {"sparse_bass_pressure", "tonal_hook", "non_dense_break"}),
            "candidate_non_dense_missing",
            failures,
        )
        check(
            any(
                isinstance(candidate, dict)
                and candidate.get("source_family") == "dense_break"
                and candidate.get("source_composed_evidence") is True
                for candidate in candidates
            ),
            "candidate_dense_break_source_composed_missing",
            failures,
        )
        check(
            any(
                isinstance(candidate, dict)
                and candidate.get("source_family") != "dense_break"
                and candidate.get("source_composed_evidence") is True
                for candidate in candidates
            ),
            "candidate_non_dense_source_composed_missing",
            failures,
        )
        for index, candidate in enumerate(candidates):
            validate_candidate(candidate, index, failures)
    blockers = report.get("blockers")
    check(isinstance(blockers, list) and bool(blockers), "blockers_missing", failures)
    if isinstance(blockers, list):
        codes = {str(blocker.get("code")) for blocker in blockers if isinstance(blocker, dict)}
        check("structured_human_verdict_missing" in codes, "human_verdict_blocker_missing", failures)
        primitive_case_ids = primitive_or_template_case_ids(candidates if isinstance(candidates, list) else [])
        if primitive_case_ids:
            check(
                "primitive_or_template_candidate_blocks_promotion" in codes,
                "primitive_template_blocker_missing",
                failures,
            )
        else:
            check(
                "primitive_or_template_candidate_blocks_promotion" not in codes,
                "primitive_template_blocker_stale",
                failures,
            )
    check(isinstance(report.get("next_actions"), list) and report["next_actions"], "next_actions_missing", failures)
    summary = str(report.get("musician_summary", ""))
    check("not demo-ready" in summary and "human pass" in summary, "musician_summary_missing", failures)
    return failures


def validate_candidate(candidate: Any, index: int, failures: list[str]) -> None:
    if not isinstance(candidate, dict):
        failures.append(f"candidate_{index}_not_object")
        return
    prefix = f"candidate_{index}"
    for field in (
        "case_id",
        "source_family",
        "candidate",
        "candidate_sha256",
        "review",
        "review_sha256",
        "human_verdict",
        "demo_readiness",
        "quality_proof",
        "source_composed_evidence",
        "primitive_or_template_only",
        "promotion_blocked_until_human_pass",
        "template_only_blocks_promotion",
    ):
        check(field in candidate, f"{prefix}_{field}_missing", failures)
    check(candidate.get("human_verdict") == "unverified", f"{prefix}_human_verdict_not_unverified", failures)
    check(candidate.get("demo_readiness") == "unverified", f"{prefix}_demo_readiness_not_unverified", failures)
    check(candidate.get("quality_proof") is False, f"{prefix}_claims_quality", failures)
    check(candidate.get("promotion_blocked_until_human_pass") is True, f"{prefix}_promotion_not_blocked", failures)
    check(candidate.get("template_only_blocks_promotion") is True, f"{prefix}_template_blocker_missing", failures)
    check(
        candidate.get("source_composed_evidence") != candidate.get("primitive_or_template_only"),
        f"{prefix}_source_composed_primitive_state_ambiguous",
        failures,
    )
    check(len(str(candidate.get("candidate_sha256", ""))) == 64, f"{prefix}_candidate_sha_invalid", failures)
    check(len(str(candidate.get("review_sha256", ""))) == 64, f"{prefix}_review_sha_invalid", failures)


def run_mutation_fixtures(report: dict[str, Any]) -> None:
    fixtures = []
    mutated = json.loads(json.dumps(report))
    mutated["quality_claim_allowed"] = True
    fixtures.append(("quality_claim", mutated, "quality_claim_allowed"))

    mutated = json.loads(json.dumps(report))
    mutated["producer_grade_promotion_result"] = "ready_for_human_promotion"
    fixtures.append(("premature_promotion", mutated, "producer_grade_promotion_not_blocked"))

    mutated = json.loads(json.dumps(report))
    mutated["review_candidates"][0]["human_verdict"] = "pass"
    fixtures.append(("stale_human_verdict", mutated, "candidate_0_human_verdict_not_unverified"))

    mutated = json.loads(json.dumps(report))
    mutated["real_source_listening_scaffold"]["primitive_controls_are_product_output"] = True
    fixtures.append(("primitive_control_leak", mutated, "primitive_controls_are_product_output"))

    mutated = json.loads(json.dumps(report))
    for candidate in mutated["review_candidates"]:
        if candidate["source_family"] == "dense_break":
            candidate["source_composed_evidence"] = False
    fixtures.append(("dense_not_source_composed", mutated, "candidate_dense_break_source_composed_missing"))

    mutated = json.loads(json.dumps(report))
    for candidate in mutated["review_candidates"]:
        if candidate["source_family"] == "tonal_hook":
            candidate["primitive_or_template_only"] = True
            candidate["source_composed_evidence"] = False
    fixtures.append(("primitive_without_blocker", mutated, "primitive_template_blocker_missing"))

    mutated = json.loads(json.dumps(report))
    for candidate in mutated["review_candidates"]:
        if candidate["source_family"] == "tonal_hook":
            candidate["primitive_or_template_only"] = False
            candidate["source_composed_evidence"] = False
    fixtures.append(("ambiguous_candidate_state", mutated, "candidate_1_source_composed_primitive_state_ambiguous"))

    for name, fixture, expected in fixtures:
        failures = validate_report(fixture)
        if expected not in failures:
            raise SystemExit(f"mutation {name} expected {expected}, got {failures}")


def write_reports(output: Path, report: dict[str, Any]) -> None:
    output.mkdir(parents=True, exist_ok=True)
    (output / "mc202-producer-grade-closeout.json").write_text(json.dumps(report, indent=2) + "\n")
    (output / "mc202-producer-grade-closeout.md").write_text(markdown_report(report))


def markdown_report(report: dict[str, Any]) -> str:
    lines = [
        "# MC-202 Producer-Grade Closeout",
        "",
        f"- Ticket: `{report['ticket']}`",
        f"- Parent quality ticket: `{report['parent_ticket']}`",
        f"- Technical closeout: `{report['technical_closeout_result']}`",
        f"- Producer-grade promotion: `{report['producer_grade_promotion_result']}`",
        f"- Quality claim allowed: `{str(report['quality_claim_allowed']).lower()}`",
        "",
        "## Candidate Status",
        "",
    ]
    for candidate in report["review_candidates"]:
        lines.extend(
            [
                f"### `{candidate['case_id']}`",
                "",
                f"- Source family: `{candidate['source_family']}`",
                f"- WAV: `{candidate['candidate']}`",
                f"- Source-composed evidence: `{str(candidate['source_composed_evidence']).lower()}`",
                f"- Primitive/template only: `{str(candidate['primitive_or_template_only']).lower()}`",
                f"- Human verdict: `{candidate['human_verdict']}`",
                f"- Demo readiness: `{candidate['demo_readiness']}`",
                f"- Not demo-ready: {candidate['not_demo_worthy_reason']}",
                "",
            ]
        )
    lines.extend(["## Blockers", ""])
    for blocker in report["blockers"]:
        lines.append(f"- `{blocker['code']}`: {blocker['reason']}")
    lines.extend(["", "## Musician Summary", "", report["musician_summary"], ""])
    return "\n".join(lines)


def musician_summary(blockers: list[dict[str, Any]]) -> str:
    primitive_blocked = any(
        blocker.get("code") == "primitive_or_template_candidate_blocks_promotion"
        for blocker in blockers
    )
    if blockers:
        if primitive_blocked:
            return (
                "MC-202 is technically reviewable across dense and non-dense sources, "
                "but it is not demo-ready and must not claim producer-grade quality "
                "until structured listening records a human pass. Primitive/template "
                "cases stay fix targets, not musical proof."
            )
        return (
            "MC-202 is technically reviewable across dense and non-dense sources, "
            "but it is not demo-ready and must not claim producer-grade quality "
            "until structured listening records a human pass."
        )
    return "MC-202 has no closeout blockers."


def primitive_or_template_case_ids(candidates: list[Any]) -> list[str]:
    return [
        str(candidate.get("case_id"))
        for candidate in candidates
        if isinstance(candidate, dict) and candidate.get("primitive_or_template_only") is True
    ]


def read_json_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"{path}: JSON root must be object")
    return value


def list_field(value: dict[str, Any], field: str, path: Path) -> list[dict[str, Any]]:
    raw = value.get(field)
    require(isinstance(raw, list), f"{path}: {field} must be array")
    for index, item in enumerate(raw):
        require(isinstance(item, dict), f"{path}: {field}[{index}] must be object")
    return raw


def object_field(value: dict[str, Any], field: str, path: Path) -> dict[str, Any]:
    raw = value.get(field)
    require(isinstance(raw, dict), f"{path}: {field} must be object")
    return raw


def object_or_empty(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def required_string(value: dict[str, Any], field: str, path: Path) -> str:
    raw = value.get(field)
    require(isinstance(raw, str) and bool(raw.strip()), f"{path}: {field} must be non-empty string")
    return raw


def check(condition: bool, code: str, failures: list[str]) -> None:
    if not condition:
        failures.append(code)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


if __name__ == "__main__":
    raise SystemExit(main())
