#!/usr/bin/env python3
"""Prepare the one frozen Sparse A/B/C review set; never launch playback."""

from __future__ import annotations

import argparse
import copy
from dataclasses import dataclass
import json
from pathlib import Path
import re
import sys
from typing import Any

import listening_review_workflow as listening
import master_bus_limiter_calibration_runner as calibration
from master_bus_limiter_calibration_metrics import Version, measure_case
import master_bus_limiter_review_artifacts as artifact_io
from source_holdout_development_access import (
    PinnedStageARegistry, SourceIdentity, parse_strict_json_object,
    read_contained_regular_file, require, run_development_access_session,
)


REPO = Path(__file__).resolve().parents[1]
PROTOCOL = "docs/benchmarks/master_bus_limiter_review_artifact_protocol_v1.json"
PROTOCOL_SHA256 = "d7c66a7885133830c06251980075a3d3aeeb8e47a10b671017b42027e776b41d"
CASE_ID = "sparse_kicksnr_120"
EXPECTED_BASIS = {
    "protocol_path": "docs/benchmarks/master_bus_limiter_calibration_protocol_v2.json",
    "protocol_sha256": "e30a439dbc0d61245cb56ad099c4110481342a0bad0e4a3426125cfcaaeae9af",
    "report_path": "artifacts/development/riotbox-1501/calibration-v2/comparison.json",
    "report_sha256": "cfbe5eae3848503f8a906b4a4ea62cf6ad9cfebd25c7f6a4e583fdf3cb6f8ed2",
    "implementation_commit": "0bb7a17b72f15addb3da38748a13a36be9ea5371",
}
CAPTURE_SUMMARY = re.compile(
    r"capture 1 bar from source into W-30 path \| audio artifact written "
    r"captures/capture-([A-Za-z0-9]{6})\.wav \| promoted to pad bank-a/pad-01"
)


@dataclass(frozen=True)
class Preparation:
    contract: dict[str, Any]
    parent: dict[str, Any]
    case: dict[str, Any]
    historical_case: dict[str, Any]
    identities: list[SourceIdentity]
    graph: dict[str, Any]


def normalized_preparation(raw: dict[str, Any]) -> dict[str, Any]:
    """Normalize only the preregistered six-character capture token in a comparison copy."""
    value = copy.deepcopy(raw)
    actions = value.get("committed_actions")
    require(isinstance(actions, list) and len(actions) == 11, "expected eleven committed Sparse actions")
    action = actions[4]
    require(type(action.get("id")) is int and action["id"] == 5
            and action.get("command") == "PromoteCaptureToPad", "capture-summary action identity")
    summary = action.get("result", {}).get("summary")
    require(isinstance(summary, str), "missing capture result summary")
    matched = CAPTURE_SUMMARY.fullmatch(summary)
    require(matched is not None, "unexpected capture result summary")
    action["result"]["summary"] = summary[:matched.start(1)] + "FROZEN" + summary[matched.end(1):]
    return value


def validate_contract(contract: dict[str, Any], parent: dict[str, Any]) -> dict[str, Any]:
    require(contract["historical_basis"] == EXPECTED_BASIS, "parent V2 evidence binding")
    require(parent["schema"] == "riotbox.master_bus_limiter_calibration.v2"
            and parent["status"] == "accepted", "accepted parent protocol required")
    matches = [case for case in parent["cases"] if case["case_id"] == CASE_ID]
    require(len(matches) == 1, "unique parent Sparse case required")
    case = matches[0]
    require(contract["case_id"] == CASE_ID and contract["source_path"] == case["source_path"]
            and contract["source_sha256"] == case["sha256"] and case["sample_width_bits"] == 16,
            "exact Sparse source identity/native width")
    require(contract["condition"] == "stress_4x" and contract["frame_window"] == [0, 96000],
            "fixed condition/window required")
    identity = contract["full_render_identity"]
    require(identity["required_conditions"] == [name for name, _ in Version.V2.conditions]
            and identity["required_policies"] == ["A", "B", "C"], "fixed full-render comparison set")
    expected_budget = dict(parent["budget"])
    expected_budget.update(original_file_opens=1, source_pcm_decodes=1, runtime_mix_renders=3,
                           distinct_policy_outputs=9, derived_capture_wavs=1, comparison_wavs=3)
    require(contract["budget"] == expected_budget, "fixed artifact-phase access/output budget")
    require(contract["output_directory"] == "artifacts/development/riotbox-1501/review-artifacts-v1"
            and contract["access_log"] == "development-access.json"
            and contract["summary_report"] == "artifact-report.json"
            and contract["review_directory"] == "listening-review", "fixed new output identities")
    require(contract["platform"] == "linux" and contract["quality_proof"] is False
            and contract["production_policy_changes"] is False and contract["human_verdict"] == "unverified"
            and contract["review"]["playback_authorized"] is False, "artifact-only, no-playback boundary")
    return case


def validate_historical_case(contract: dict[str, Any], parent: dict[str, Any],
                             case: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    basis = contract["historical_basis"]
    require(report["schema"] == "riotbox.master_bus_limiter_calibration_report.v2"
            and report["status"] == "technical_comparison_complete_no_policy_selection"
            and report["protocol_sha256"] == basis["protocol_sha256"]
            and report["git_head"] == basis["implementation_commit"]
            and report["build"]["rustc_verbose_version"] == parent["build_rustc_verbose_version"],
            "historical V2 result/build identity")
    require([row["case_id"] for row in report["cases"]] == calibration.CASE_IDS,
            "historical V2 exact case order")
    prior = next(row for row in report["cases"] if row["case_id"] == CASE_ID)
    require(prior["protocol_version"] == "v2" and prior["frame_count"] == case["frame_count"]
            and prior["sample_rate_hz"] == 48000 and prior["channels"] == 2,
            "historical V2 render identity")
    access = prior["access"]
    require(access["source_path"] == case["source_path"] and access["actual_sha256"] == case["sha256"]
            and access["actual_source_format"]["sample_width_bits"] == 16, "historical Sparse source identity")
    require([row["condition"] for row in prior["conditions"]] == [name for name, _ in Version.V2.conditions],
            "historical V2 condition identity")
    for condition in prior["conditions"]:
        require([row["policy"] for row in condition["outputs"]] == ["A", "B", "C"],
                "historical V2 policy identity")
        require(calibration.is_hex_identity(condition["input_sha256_f32le"], 64)
                and all(calibration.is_hex_identity(row["output_sha256_f32le"], 64)
                        for row in condition["outputs"]), "historical full PCM fingerprints")
        if condition["condition"] != "stress_4x":
            expected = case["historical_controls"][f"{condition['condition']}_pcm_sha256_f32le"]
            require(condition["input_sha256_f32le"] == expected
                    and all(row["output_sha256_f32le"] == expected for row in condition["outputs"]),
                    "historical clean/2x control mismatch")
        else:
            identity = contract["full_render_identity"]
            require(condition["input_sha256_f32le"] == identity["stress_4x_input_sha256_f32le"]
                    and {row["policy"]: row["output_sha256_f32le"] for row in condition["outputs"]}
                    == identity["stress_4x_output_sha256_f32le"], "historical 4x fingerprints")
    normalized_preparation(prior["preparation"])
    return prior


def preflight(*, execute: bool = False) -> Preparation:
    payload = read_contained_regular_file(REPO, Path(PROTOCOL), PROTOCOL, maximum_bytes=8 * 1024 * 1024)
    if execute or PROTOCOL_SHA256 != "execution-not-yet-frozen":
        require(calibration.sha256(payload) == PROTOCOL_SHA256, "artifact protocol execution is not frozen")
    contract = parse_strict_json_object(payload, PROTOCOL)
    require(contract["schema"] == "riotbox.master_bus_limiter_review_artifact.v1"
            and contract["repository_root"] == str(REPO), "artifact protocol schema/workspace")
    require(contract["status"] in {"pre_execution_review", "accepted"}, "artifact protocol status")
    if execute:
        require(contract["status"] == "accepted", "artifact protocol is not accepted")
    require(contract["historical_basis"] == EXPECTED_BASIS, "parent V2 evidence binding")
    basis = contract["historical_basis"]
    parent = calibration.metadata(basis["protocol_path"], basis["protocol_sha256"], repo=REPO)
    case = validate_contract(contract, parent)
    previous = calibration.metadata(basis["report_path"], basis["report_sha256"], repo=REPO)
    historical_case = validate_historical_case(contract, parent, case, previous)
    identities = calibration.load_registered_identities(parent, [CASE_ID], prefix=PROTOCOL)
    graph = calibration.load_case_graph(case)
    return Preparation(contract, parent, case, historical_case, identities, graph)


def verify_full_identity(measured: dict[str, Any], previous: dict[str, Any]) -> None:
    for key in ("protocol_version", "frame_count", "sample_rate_hz", "channels", "controls"):
        require(measured[key] == previous[key], f"full render {key} identity mismatch")
    require(normalized_preparation(measured["preparation"]) == normalized_preparation(previous["preparation"]),
            "preparation differs beyond the single declared capture token")
    require([row["condition"] for row in measured["conditions"]]
            == [row["condition"] for row in previous["conditions"]], "full condition identity mismatch")
    for actual, prior in zip(measured["conditions"], previous["conditions"], strict=True):
        require(actual["input_sha256_f32le"] == prior["input_sha256_f32le"],
                f"{actual['condition']}: full input fingerprint mismatch")
        require([(row["policy"], row["output_sha256_f32le"]) for row in actual["outputs"]]
                == [(row["policy"], row["output_sha256_f32le"]) for row in prior["outputs"]],
                f"{actual['condition']}: full policy fingerprint mismatch")


def report_bytes(report: dict[str, Any]) -> bytes:
    return (json.dumps(report, indent=2, allow_nan=False) + "\n").encode()


def create_review_pack(output: Path, contract: dict[str, Any], evidence: dict[str, Any],
                       report_sha256: str) -> None:
    artifacts = evidence["artifacts"]
    expected = [(item["policy"], str(output / item["filename"])) for item in contract["artifacts"]]
    require([(item["policy"], item["path"]) for item in artifacts] == expected, "publisher artifact assignment")
    require(all(calibration.is_hex_identity(item["sha256"], 64) for item in artifacts), "publisher WAV identities")
    directory = output / contract["review_directory"]
    directory.mkdir(exist_ok=False)
    listening.create_pack(argparse.Namespace(
        ticket="RIOTBOX-1501", output=directory, pr="", command_line=f"python3 scripts/{Path(__file__).name} --execute",
        source_file=None, candidate=[item["path"] for item in artifacts], seed_or_config=PROTOCOL_SHA256,
        technical_status="artifact_identity_and_presentation_preflight_passed",
        automated_musical_fitness_status="not_evaluated_diagnostic_comparison",
        expected=contract["review"]["purpose"],
    ))
    review_path = directory / "review.json"
    review = json.loads(review_path.read_text())
    review.update(
        source_metadata={"case_id": CASE_ID, "source_path": contract["source_path"],
                         "sha256": contract["source_sha256"], "source_audio_attached": False},
        artifact_identities=artifacts,
        artifact_report={"path": str(output / contract["summary_report"]), "sha256": report_sha256},
        diagnostic_scope={"condition": "stress_4x", "frame_window": [0, 96000], "duration_seconds": 2.0,
                          "contributors": contract["contributors"], "silent_owners": contract["silent_owners"],
                          "bass_owner": "unassigned", "playback_authorized": False},
        demo_readiness="unverified", quality_claim=False, pre_listen_assessment="pending_independent_artifact_bound_review",
        demo_worthy_reason="Exact fixed-window limiter differences are available for a separately authorized comparison.",
        not_demo_worthy_reason="Diagnostic artifacts have no human verdict and do not establish product or musical quality.",
    )
    listening.validate_review(review, allow_unverified=True)
    listening.write_json(review_path, review)
    candidates = "\n".join(f"- {item['policy']}: `{item['path']}`; SHA-256 `{item['sha256']}`" for item in artifacts)
    (directory / "prompt.md").write_text(
        "# Fixed one-bar limiter comparison\n\n"
        "No playback is authorized by this pack. Independent artifact-bound assessment, exact-file preflight, "
        "fresh listener readiness and bounded stop/silence verification remain required.\n\n"
        "A, B and C contain the same two-second/one-bar Sparse composite at 120 BPM, "
        "with W-30 source transformation, TR-909 transients and MC-202 punctuation. "
        "Source Monitor and resample tap are silent; bass ownership is unassigned. "
        "Only the inherited limiter policy differs. One shared attenuation preserves their relative levels.\n\n"
        f"{candidates}\n\n"
        "When separately authorized, ask one question at a time:\n\n"
        "1. Can you hear a difference in coloration, transient shape or clarity? ‘No clear difference’ is valid.\n"
        "2. If distinguishable, does any version better preserve this composite, or is there no preference?\n"
        "3. What concrete audible problem, if any, should a later calibration decision address?\n\n"
        "This is not a new hook, isolated lane, bass-pressure, hardness, demo-readiness or production-policy claim. "
        "Human verdict and demo readiness remain unverified; technical metrics cannot choose a winner.\n\n"
        f"Review value: {review['demo_worthy_reason']}\n\n"
        f"Not demo-ready: {review['not_demo_worthy_reason']}\n"
    )


def execute() -> None:
    require(sys.platform.startswith("linux"), "this artifact phase is Linux-only")
    prepared = preflight(execute=True)
    contract, parent, case = prepared.contract, prepared.parent, prepared.case
    output = REPO / contract["output_directory"]
    require(not output.exists() and not output.is_symlink(), "artifact output directory must be new")
    head = calibration.git_identity()
    binary, build = calibration.build_executor(head, expected_rustc=parent["build_rustc_verbose_version"])
    tool_preflight = artifact_io.preflight_tools(contract)
    require(calibration.git_identity() == head, "implementation changed during artifact preflight")
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema": "riotbox.master_bus_limiter_review_artifact_report.v1", "status": "started",
              "protocol_sha256": PROTOCOL_SHA256, "git_head": head, "build": build,
              "historical_basis": contract["historical_basis"], "tool_preflight": tool_preflight,
              "human_verdict": "unverified", "quality_proof": False, "playback_authorized": False}

    def save() -> None:
        (output / contract["summary_report"]).write_bytes(report_bytes(report))

    save()

    def owner(identity: SourceIdentity, payload: bytes, access: dict[str, Any]) -> None:
        require(identity.case_id == CASE_ID, "only the fixed Sparse source may be delivered")
        report["pending_case"] = {"case_id": CASE_ID, "access": access, "status": "admitted_before_executor"}
        save()
        require(calibration.git_identity() == head, "implementation changed during artifact session")
        require(calibration.sha256((REPO / PROTOCOL).read_bytes()) == PROTOCOL_SHA256, "artifact protocol changed")
        require(calibration.sha256(binary.read_bytes()) == build["sha256"], "artifact executor changed")
        capture_output = output / CASE_ID
        capture_output.mkdir(exist_ok=False)

        def retain(fields: dict[str, Any]) -> None:
            report["pending_case"].update(fields)
            save()

        raw = calibration.run_case_executor(binary, Version.V2,
            {"case_id": CASE_ID, "graph": prepared.graph, "source_wav_bytes": list(payload),
             "output_dir": str(capture_output)}, contract["budget"], retain)
        retain(calibration.case_observations(raw))
        measured = measure_case(raw, case["frame_count"], version=Version.V2,
                                historical_controls=case["historical_controls"],
                                minimum_mix_rms=case.get("minimum_mix_rms"), listening_frame_window=(0, 96000))
        measured.update(case_id=CASE_ID, preparation=raw["preparation"], access=access)
        report["case"] = measured
        save()
        verify_full_identity(measured, prepared.historical_case)
        report["historical_full_render_identity"] = "passed_with_declared_capture_token_normalization"
        save()
        try:
            report["artifact_evidence"] = artifact_io.publish_artifacts(raw, output, contract, tool_preflight=tool_preflight)
        except Exception as error:
            if isinstance(getattr(error, "evidence", None), dict):
                report["artifact_evidence"] = error.evidence
                save()
            raise
        del report["pending_case"]
        save()

    registry = parent["protected_registry"]
    try:
        access = run_development_access_session(prepared.identities, [CASE_ID], repo=REPO,
            registry=PinnedStageARegistry(REPO / registry["path"], registry["schema"], registry["sha256"]),
            access_log_path=output / contract["access_log"],
            qualification_owner_id="riotbox-1501-limiter-review-artifacts-v1", qualification_owner=owner)
        report["access_session_id"] = access["access_session_id"]
        report["review_directory"] = str(output / contract["review_directory"])
        report["status"] = "artifact_preflight_complete_human_unverified"
        # Bind the final report bytes without a report/review self-hash cycle.
        final_report = report_bytes(report)
        create_review_pack(output, contract, report["artifact_evidence"], calibration.sha256(final_report))
        (output / contract["summary_report"]).write_bytes(final_report)
    except (Exception, SystemExit) as error:
        report["status"] = "failed_closed"
        report["failure"] = {"type": type(error).__name__, "message": str(error)}
        save()
        raise


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.execute:
        execute()
    else:
        prepared = preflight()
        print(f"artifact metadata preflight passed: {prepared.case['case_id']}; no source audio opened; "
              f"protocol status={prepared.contract['status']}; playback unauthorized")


if __name__ == "__main__":
    main()
