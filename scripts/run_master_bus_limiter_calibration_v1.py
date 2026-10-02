#!/usr/bin/env python3
"""One frozen Linux-only Development comparison; metadata preflight by default."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from master_bus_limiter_calibration_metrics import measure_case
from source_holdout_development_access import (
    PinnedStageARegistry,
    SourceIdentity,
    parse_strict_json_object,
    preflight_development_identities,
    read_contained_regular_file,
    require,
    run_development_access_session,
)


REPO = Path(__file__).resolve().parents[1]
PROTOCOL = "docs/benchmarks/master_bus_limiter_calibration_protocol_v1.json"
# RBX-418: frozen after source-free review; never retune from source results.
PROTOCOL_SHA256 = "c9d72c910aea77045a25c30ecc8a5bfec2e66481a9762aa2ba8827aa0f94a42d"
CASE_IDS = ["dense_beat03_130", "tonal_rusharp_120", "sparse_kicksnr_120"]


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def metadata(path: str, digest: str, *, repo: Path = REPO) -> dict[str, Any]:
    payload = read_contained_regular_file(repo, Path(path), path, maximum_bytes=8 * 1024 * 1024)
    require(sha256(payload) == digest, f"metadata pin mismatch: {path}")
    return parse_strict_json_object(payload, path)


def preflight(*, execute: bool = False) -> tuple[dict[str, Any], list[SourceIdentity], dict[str, Any]]:
    protocol_bytes = (REPO / PROTOCOL).read_bytes()
    if execute or PROTOCOL_SHA256 != "execution-not-yet-frozen":
        require(sha256(protocol_bytes) == PROTOCOL_SHA256, "protocol execution is not frozen")
    protocol = parse_strict_json_object(protocol_bytes, PROTOCOL)
    require(protocol["schema"] == "riotbox.master_bus_limiter_calibration.v1", "protocol schema")
    require(str(REPO) == protocol["repository_root"], "registered repository root changed")
    require(protocol["status"] in {"pre_execution_review", "accepted"}, "protocol status")
    if execute:
        require(protocol["status"] == "accepted", "protocol is not accepted")
    require([case["case_id"] for case in protocol["cases"]] == CASE_IDS, "exact case order")
    corpus = metadata(protocol["corpus"]["path"], protocol["corpus"]["sha256"])
    registry_binding = protocol["protected_registry"]
    registry = metadata(registry_binding["path"], registry_binding["sha256"])
    require(registry["schema"] == registry_binding["schema"], "registry schema")
    protected = [entry for entry in registry["entries"] if entry["partition"].startswith("holdout_")]
    require(len(protected) == registry_binding["expected_protected_count"], "protected count")
    identities = [SourceIdentity(entry["case_id"], entry["source_path"], entry["sha256"],
                                 entry["partition"], {}) for entry in protected]
    corpus_by_id = {entry["case_id"]: entry for entry in corpus["entries"]}
    for case in protocol["cases"]:
        registered = corpus_by_id[case["case_id"]]
        require(registered["source_path"] == case["source_path"]
                and registered["source_family"] == case["source_family"], "corpus case binding")
        fmt = {key: protocol["source_format_admission"][key] for key in
               ("sample_rate_hz", "channels", "compression_type", "maximum_duration_seconds")}
        if "sample_width_bits" in case:
            fmt["sample_width_bits"] = case["sample_width_bits"]
        widths = tuple(case["allowed_sample_width_bits"]) if "allowed_sample_width_bits" in case else None
        identities.append(SourceIdentity(case["case_id"], case["source_path"], case["sha256"],
                                         case["partition"], fmt, allowed_sample_width_bits=widths))
    # Pure exclusion of every identity/path/hash before any selected audio access.
    preflight_development_identities(identities, CASE_IDS, prefix=PROTOCOL)
    graphs = {}
    for case in protocol["cases"]:
        directory = case["metadata_directory"]
        graph = metadata(f"{directory}/source-graph.json", case["graph_sha256"])
        historical = metadata(f"{directory}/session.json", case["session_sha256"])
        source = graph["source"]
        require(source["source_id"] == case["source_id"]
                and source["path"] == str(REPO / case["source_path"])
                and source["content_hash"] == f"sha256:{case['sha256']}"
                and graph["provenance"]["source_hash"] == source["content_hash"]
                and source["sample_rate"] == 44100 and source["channel_count"] == 2
                and source["duration_seconds"] == case["graph_duration_seconds"]
                and source["decode_profile"] == "Native", "graph/source metadata binding")
        timing = historical["runtime_state"]["source_timing"]
        require(timing["confirmed_grid"]["source_id"] == case["source_id"]
                and timing["confirmed_grid"]["hypothesis_id"] == case["hypothesis_id"]
                and timing["confirmed_bpm"] == case["confirmed_bpm"]
                and graph["timing"]["primary_hypothesis_id"] == case["hypothesis_id"],
                "historical committed timing identity")
        graphs[case["case_id"]] = graph
    return protocol, identities, graphs


def git_identity() -> str:
    status = subprocess.run(["git", "status", "--porcelain", "--untracked-files=all"], cwd=REPO,
                            capture_output=True, text=True, check=True).stdout
    require(not status.strip(), "commit/review all implementation changes before source execution")
    return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True,
                          text=True, check=True).stdout.strip()


def build_executor(head: str) -> tuple[Path, dict[str, Any]]:
    """Build the reviewed HEAD/feature before any source open; never trust target by existence."""
    compiler = subprocess.run(["rustc", "-vV"], cwd=REPO, capture_output=True,
                              text=True, check=True).stdout
    hosts = [line.removeprefix("host: ") for line in compiler.splitlines() if line.startswith("host: ")]
    require(len(hosts) == 1 and "linux" in hosts[0], "expected one native Linux compiler host")
    command = ["cargo", "build", "--locked", "--manifest-path", str(REPO / "Cargo.toml"),
               "--target-dir", str(REPO / "target"), "--target", hosts[0], "--profile", "dev",
               "-p", "riotbox-app", "--bin", "dense_break_live_path_render",
               "--features", "limiter-calibration", "--message-format=json"]
    built = subprocess.run(command, cwd=REPO, capture_output=True, text=True, check=True)
    artifacts = []
    for line in built.stdout.splitlines():
        item = json.loads(line)
        if (item.get("reason") == "compiler-artifact"
                and item["target"]["name"] == "dense_break_live_path_render"
                and item.get("executable") is not None):
            require("limiter-calibration" in item["features"], "executor feature absent")
            artifacts.append(Path(item["executable"]))
    require(len(artifacts) == 1, "build did not bind exactly one calibration executor")
    binary = artifacts[0]
    require(binary.is_relative_to(REPO / "target"), "executor escaped the selected target directory")
    require(git_identity() == head, "implementation changed while building executor")
    return binary, {"command": command, "rustc_verbose_version": compiler,
                    "executable": str(binary), "sha256": sha256(binary.read_bytes())}


def execute() -> None:
    require(sys.platform.startswith("linux"), "this calibration is Linux-only")
    protocol, identities, graphs = preflight(execute=True)
    head = git_identity()
    binary, build = build_executor(head)
    binary_hash = build["sha256"]
    output = REPO / protocol["output_directory"]
    output.mkdir(parents=True, exist_ok=False)
    report: dict[str, Any] = {"schema": "riotbox.master_bus_limiter_calibration_report.v1",
                              "protocol_sha256": PROTOCOL_SHA256, "git_head": head,
                              "build": build, "binary_sha256": binary_hash,
                              "cases": [], "status": "started",
                              "human_verdict": "unverified", "quality_proof": False}

    def save_report() -> None:
        (output / protocol["summary_report"]).write_text(
            json.dumps(report, indent=2, allow_nan=False) + "\n")

    save_report()
    cases = {case["case_id"]: case for case in protocol["cases"]}

    def owner(identity: SourceIdentity, payload: bytes, access: dict[str, Any]) -> None:
        report["pending_case"] = {"case_id": identity.case_id, "access": access,
                                  "status": "admitted_before_executor"}
        save_report()
        require(git_identity() == head, "implementation changed during bounded session")
        require(sha256((REPO / PROTOCOL).read_bytes()) == PROTOCOL_SHA256, "protocol changed")
        require(sha256(binary.read_bytes()) == binary_hash, "executor changed")
        case = cases[identity.case_id]
        case_output = output / identity.case_id
        case_output.mkdir(exist_ok=False)
        request = json.dumps({"case_id": identity.case_id, "graph": graphs[identity.case_id],
                              "source_wav_bytes": list(payload), "output_dir": str(case_output)},
                             allow_nan=False).encode()
        require(len(request) <= protocol["budget"]["max_stdin_json_bytes"], "request byte budget")
        require(len(payload) <= protocol["budget"]["max_wav_payload_bytes"], "WAV byte budget")
        try:
            process = subprocess.run([str(binary), "--limiter-calibration-v1"], input=request, cwd=REPO,
                                     capture_output=True, timeout=protocol["budget"]["child_timeout_seconds"])
        except subprocess.TimeoutExpired as error:
            report["pending_case"].update(
                status="executor_timeout",
                executor_stderr=(error.stderr or b"")[:protocol["budget"]["max_child_stderr_bytes"]].decode(errors="replace"))
            save_report()
            raise
        stderr = process.stderr[:protocol["budget"]["max_child_stderr_bytes"]].decode(errors="replace")
        report["pending_case"].update(executor_exit_code=process.returncode, executor_stderr=stderr,
                                      stderr_truncated=len(process.stderr) > protocol["budget"]["max_child_stderr_bytes"])
        save_report()
        require(len(process.stdout) <= protocol["budget"]["max_child_stdout_bytes"], "response byte budget")
        require(len(process.stderr) <= protocol["budget"]["max_child_stderr_bytes"], "diagnostic byte budget")
        require(process.returncode == 0, f"{identity.case_id}: executor rejected; see retained pending_case")
        raw = parse_strict_json_object(process.stdout, identity.case_id)
        require(raw["case_id"] == identity.case_id, "response case identity")
        report["pending_case"] = {
            "case_id": identity.case_id, "access": access, "preparation": raw["preparation"],
            "limiter_observations": [
                {"condition": condition["condition"], "outputs": [
                    {"policy": row["policy"], "limiter": row["limiter"]}
                    for row in condition["outputs"]]}
                for condition in raw["conditions"]],
        }
        save_report()
        listening = protocol["preselected_future_listening"]
        selected_window = tuple(listening["frame_window"]) if identity.case_id == listening["case_id"] else None
        measured = measure_case(raw, case["frame_count"], minimum_mix_rms=case.get("minimum_mix_rms"),
                                listening_frame_window=selected_window)
        measured.update({"case_id": identity.case_id, "access": access,
                         "preparation": raw["preparation"]})
        report["cases"].append(measured)
        del report["pending_case"]
        save_report()

    binding = protocol["protected_registry"]
    try:
        access = run_development_access_session(
            identities, CASE_IDS, repo=REPO,
            registry=PinnedStageARegistry(REPO / binding["path"], binding["schema"], binding["sha256"]),
            access_log_path=output / protocol["access_log"],
            qualification_owner_id="riotbox-1501-limiter-calibration-v1", qualification_owner=owner)
        report["access_session_id"] = access["access_session_id"]
        selected = next(case for case in report["cases"]
                        if case["case_id"] == protocol["preselected_future_listening"]["case_id"])
        stress = selected["conditions"][1]["outputs"]
        report["preselected_future_listening"] = {
            "case_id": selected["case_id"], "artifact_generated": False,
            "frame_window": protocol["preselected_future_listening"]["frame_window"],
            "protection_observed_any_policy": any(row["preselected_window"]["modified_samples"] > 0 for row in stress),
            "any_policy_differs_from_a": any(not row["preselected_window"]["bit_identical_to_a"] for row in stress),
            "human_verdict": "unverified",
        }
        report["status"] = "technical_comparison_complete_no_policy_selection"
    except Exception as error:
        report["status"] = "failed_closed"
        report["failure"] = {"type": type(error).__name__, "message": str(error)}
        save_report()
        raise
    save_report()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.execute:
        execute()
    else:
        protocol, _, _ = preflight()
        print(f"metadata preflight passed: {len(protocol['cases'])} cases; no source audio opened; "
              f"protocol status={protocol['status']}")


if __name__ == "__main__":
    main()
