#!/usr/bin/env python3
"""One frozen Linux-only Development comparison; metadata preflight by default."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

from master_bus_limiter_calibration_metrics import Version, measure_case
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
CASE_IDS = ["dense_beat03_130", "tonal_rusharp_120", "sparse_kicksnr_120"]


@dataclass(frozen=True)
class ProtocolBinding:
    path: str
    sha256: str


PROTOCOLS = {
    # RBX-418: v1 remains immutable after its consumed source session.
    Version.V1: ProtocolBinding("docs/benchmarks/master_bus_limiter_calibration_protocol_v1.json",
                                "c9d72c910aea77045a25c30ecc8a5bfec2e66481a9762aa2ba8827aa0f94a42d"),
    Version.V2: ProtocolBinding("docs/benchmarks/master_bus_limiter_calibration_protocol_v2.json",
                                "e30a439dbc0d61245cb56ad099c4110481342a0bad0e4a3426125cfcaaeae9af"),
}


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def metadata(path: str, digest: str, *, repo: Path = REPO) -> dict[str, Any]:
    payload = read_contained_regular_file(repo, Path(path), path, maximum_bytes=8 * 1024 * 1024)
    require(sha256(payload) == digest, f"metadata pin mismatch: {path}")
    return parse_strict_json_object(payload, path)


def is_hex_identity(value: Any, length: int) -> bool:
    return isinstance(value, str) and len(value) == length and set(value) <= set("0123456789abcdef")


def validate_v2_contract(protocol: dict[str, Any], historical: dict[str, Any]) -> None:
    """Pure recipe/budget checks before reading even historical source metadata."""
    basis = protocol["historical_basis"]
    require(basis["protocol_path"] == PROTOCOLS[Version.V1].path
            and basis["protocol_sha256"] == PROTOCOLS[Version.V1].sha256,
            "historical protocol binding")
    require(basis["report_path"] == "artifacts/development/riotbox-1501/calibration-v1/comparison.json"
            and is_hex_identity(basis["report_sha256"], 64)
            and is_hex_identity(basis["implementation_commit"], 40), "historical report binding")
    compiler = protocol["build_rustc_verbose_version"]
    require(isinstance(compiler, str) and compiler.startswith("rustc ")
            and sum(line.startswith("host: ") and "linux" in line for line in compiler.splitlines()) == 1,
            "historical native compiler binding")
    require(protocol["policies"] == historical["policies"]
            and protocol["measurement"] == historical["measurement"], "frozen policies/windows changed")
    expected_render = dict(historical["render"])
    expected_render.update(conditions=[{"id": name, "gain": gain} for name, gain in Version.V2.conditions],
                           stress_4x_gain_f32_bits="40800000", comparison=protocol["render"]["comparison"])
    require(protocol["render"] == expected_render, "v2 render condition/recipe contract")
    expected_budget = dict(historical["budget"])
    expected_budget.update(distinct_policy_outputs=27, max_child_stdout_bytes=128 * 1024 * 1024)
    require(protocol["budget"] == expected_budget, "v2 fixed access/render/output budget")
    require(protocol["output_directory"] == "artifacts/development/riotbox-1501/calibration-v2"
            and protocol["summary_report"] == historical["summary_report"]
            and protocol["access_log"] == historical["access_log"], "v2 output identity")
    require(protocol["corpus"] == historical["corpus"]
            and protocol["protected_registry"] == historical["protected_registry"], "source registry binding")
    for key in ("sample_rate_hz", "channels", "compression_type", "maximum_duration_seconds"):
        require(protocol["source_format_admission"][key] == historical["source_format_admission"][key],
                "source format admission changed")
    require([case["case_id"] for case in protocol["cases"]] == CASE_IDS, "exact case order")
    for current, previous, width in zip(protocol["cases"], historical["cases"], (24, 16, 16), strict=True):
        expected = dict(previous)
        expected.pop("allowed_sample_width_bits", None)
        expected["sample_width_bits"] = width
        controls = current.get("historical_controls")
        require(isinstance(controls, dict)
                and set(controls) == {"clean_pcm_sha256_f32le", "stress_2x_pcm_sha256_f32le"}
                and all(is_hex_identity(value, 64) for value in controls.values()),
                "historical control fingerprint contract")
        expected["historical_controls"] = controls
        require(current == expected, "v2 case identity/preparation/native-width contract")
    listening = protocol["preselected_future_listening"]
    require(listening["case_id"] == "sparse_kicksnr_120"
            and listening["condition"] == "stress_4x"
            and listening["frame_window"] == [0, 96000], "v2 fixed future condition/window")
    require(protocol["platform"] == "linux" and protocol["quality_proof"] is False
            and protocol["production_policy_changes"] is False
            and protocol["human_verdict"] == "unverified", "v2 diagnostic-only boundary")


def validate_historical_report(protocol: dict[str, Any], report: dict[str, Any]) -> None:
    """Bind the consumed v1 metadata, never hydrate or reopen its audio/captures."""
    basis = protocol["historical_basis"]
    require(report["schema"] == "riotbox.master_bus_limiter_calibration_report.v1"
            and report["status"] == "technical_comparison_complete_no_policy_selection"
            and report["protocol_sha256"] == basis["protocol_sha256"]
            and report["git_head"] == basis["implementation_commit"]
            and report["build"]["rustc_verbose_version"] == protocol["build_rustc_verbose_version"],
            "historical result/build identity")
    require([case["case_id"] for case in report["cases"]] == CASE_IDS, "historical exact case order")
    for case, prior in zip(protocol["cases"], report["cases"], strict=True):
        require(prior["frame_count"] == case["frame_count"]
                and prior["sample_rate_hz"] == 48000 and prior["channels"] == 2
                and prior["controls"] == {"repeat_128_bit_exact": True, "partition_257_bit_exact": True,
                                          "baseline_api_bit_exact": True}, "historical render controls")
        access = prior["access"]
        require(access["actual_sha256"] == case["sha256"]
                and access["actual_source_format"]["sample_width_bits"] == case["sample_width_bits"],
                "historical source identity/native width")
        require([row["condition"] for row in prior["conditions"]] == ["clean", "stress_2x"],
                "historical condition order")
        for condition in prior["conditions"]:
            expected = case["historical_controls"][f"{condition['condition']}_pcm_sha256_f32le"]
            require(condition["input_sha256_f32le"] == expected, "historical input fingerprint")
            require([row["policy"] for row in condition["outputs"]] == ["A", "B", "C"],
                    "historical policy order")
            require(all(row["output_sha256_f32le"] == expected for row in condition["outputs"]),
                    "historical policy output fingerprint")


def preflight(version: Version = Version.V1, *, execute: bool = False) -> tuple[dict[str, Any], list[SourceIdentity], dict[str, Any]]:
    require(isinstance(version, Version), "unknown calibration version")
    binding = PROTOCOLS[version]
    protocol_bytes = (REPO / binding.path).read_bytes()
    if execute or binding.sha256 != "execution-not-yet-frozen":
        require(sha256(protocol_bytes) == binding.sha256, "protocol execution is not frozen")
    protocol = parse_strict_json_object(protocol_bytes, binding.path)
    require(protocol["schema"] == f"riotbox.master_bus_limiter_calibration.{version.value}", "protocol schema")
    require(str(REPO) == protocol["repository_root"], "registered repository root changed")
    require(protocol["status"] in {"pre_execution_review", "accepted"}, "protocol status")
    if execute:
        require(protocol["status"] == "accepted", "protocol is not accepted")
    require([case["case_id"] for case in protocol["cases"]] == CASE_IDS, "exact case order")
    if version is Version.V2:
        previous = PROTOCOLS[Version.V1]
        historical = metadata(previous.path, previous.sha256, repo=REPO)
        validate_v2_contract(protocol, historical)
        basis = protocol["historical_basis"]
        historical_report = metadata(basis["report_path"], basis["report_sha256"], repo=REPO)
        validate_historical_report(protocol, historical_report)
    identities = load_registered_identities(protocol, CASE_IDS, prefix=binding.path)
    graphs = {case["case_id"]: load_case_graph(case) for case in protocol["cases"]}
    return protocol, identities, graphs


def load_registered_identities(protocol: dict[str, Any], case_ids: list[str], *, prefix: str) -> list[SourceIdentity]:
    """Read only registry/corpus metadata and exclude protected identities before source access."""
    corpus = metadata(protocol["corpus"]["path"], protocol["corpus"]["sha256"], repo=REPO)
    registry_binding = protocol["protected_registry"]
    registry = metadata(registry_binding["path"], registry_binding["sha256"], repo=REPO)
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
    preflight_development_identities(identities, case_ids, prefix=prefix)
    return identities


def load_case_graph(case: dict[str, Any]) -> dict[str, Any]:
    """Read exactly one registered Graph/Session metadata pair, never hydrate audio."""
    directory = case["metadata_directory"]
    graph = metadata(f"{directory}/source-graph.json", case["graph_sha256"], repo=REPO)
    historical = metadata(f"{directory}/session.json", case["session_sha256"], repo=REPO)
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
    return graph


def git_identity() -> str:
    status = subprocess.run(["git", "status", "--porcelain", "--untracked-files=all"], cwd=REPO,
                            capture_output=True, text=True, check=True).stdout
    require(not status.strip(), "commit/review all implementation changes before source execution")
    return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True,
                          text=True, check=True).stdout.strip()


def build_executor(head: str, *, expected_rustc: str | None = None) -> tuple[Path, dict[str, Any]]:
    """Build the reviewed HEAD/feature before any source open; never trust target by existence."""
    compiler = subprocess.run(["rustc", "-vV"], cwd=REPO, capture_output=True,
                              text=True, check=True).stdout
    if expected_rustc is not None:
        require(compiler == expected_rustc, "native compiler differs from frozen historical build")
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


def run_case_executor(binary: Path, version: Version, request: dict[str, Any], budget: dict[str, Any],
                      retain: Callable[[dict[str, Any]], None]) -> dict[str, Any]:
    """One bounded child; persist diagnostics before rejecting any consumed source."""
    require(isinstance(version, Version), "unknown calibration version")
    encoded = json.dumps(request, allow_nan=False).encode()
    require(len(encoded) <= budget["max_stdin_json_bytes"], "request byte budget")
    require(len(request["source_wav_bytes"]) <= budget["max_wav_payload_bytes"], "WAV byte budget")
    try:
        process = subprocess.run([str(binary), f"--limiter-calibration-{version.value}"], input=encoded, cwd=REPO,
                                 capture_output=True, timeout=budget["child_timeout_seconds"])
    except subprocess.TimeoutExpired as error:
        retain({"status": "executor_timeout",
                "executor_stderr": (error.stderr or b"")[:budget["max_child_stderr_bytes"]].decode(errors="replace")})
        raise
    retain({"executor_exit_code": process.returncode,
            "executor_stderr": process.stderr[:budget["max_child_stderr_bytes"]].decode(errors="replace"),
            "stderr_truncated": len(process.stderr) > budget["max_child_stderr_bytes"]})
    require(len(process.stdout) <= budget["max_child_stdout_bytes"], "response byte budget")
    require(len(process.stderr) <= budget["max_child_stderr_bytes"], "diagnostic byte budget")
    require(process.returncode == 0, f"{request['case_id']}: executor rejected; see retained pending_case")
    raw = parse_strict_json_object(process.stdout, request["case_id"])
    require(raw["case_id"] == request["case_id"], "response case identity")
    return raw


def case_observations(raw: dict[str, Any]) -> dict[str, Any]:
    """Bounded preparation/reports retained independently of subsequent identity/metric gates."""
    return {"preparation": raw["preparation"], "limiter_observations": [
        {"condition": condition["condition"], "outputs": [
            {"policy": row["policy"], "limiter": row["limiter"]} for row in condition["outputs"]]}
        for condition in raw["conditions"]]}


def execute(version: Version = Version.V1) -> None:
    require(sys.platform.startswith("linux"), "this calibration is Linux-only")
    require(isinstance(version, Version), "unknown calibration version")
    binding = PROTOCOLS[version]
    protocol, identities, graphs = preflight(version, execute=True)
    head = git_identity()
    expected_rustc = protocol["build_rustc_verbose_version"] if version is Version.V2 else None
    binary, build = build_executor(head, expected_rustc=expected_rustc)
    binary_hash = build["sha256"]
    output = REPO / protocol["output_directory"]
    output.mkdir(parents=True, exist_ok=False)
    report: dict[str, Any] = {"schema": f"riotbox.master_bus_limiter_calibration_report.{version.value}",
                              "protocol_sha256": binding.sha256, "git_head": head,
                              "build": build, "binary_sha256": binary_hash,
                              "cases": [], "status": "started",
                              "human_verdict": "unverified", "quality_proof": False}
    if version is Version.V2:
        report["historical_basis"] = protocol["historical_basis"]

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
        require(sha256((REPO / binding.path).read_bytes()) == binding.sha256, "protocol changed")
        require(sha256(binary.read_bytes()) == binary_hash, "executor changed")
        case = cases[identity.case_id]
        case_output = output / identity.case_id
        case_output.mkdir(exist_ok=False)
        request = {"case_id": identity.case_id, "graph": graphs[identity.case_id],
                   "source_wav_bytes": list(payload), "output_dir": str(case_output)}

        def retain(fields: dict[str, Any]) -> None:
            report["pending_case"].update(fields)
            save_report()

        raw = run_case_executor(binary, version, request, protocol["budget"], retain)
        report["pending_case"] = {"case_id": identity.case_id, "access": access, **case_observations(raw)}
        save_report()
        listening = protocol["preselected_future_listening"]
        selected_window = tuple(listening["frame_window"]) if identity.case_id == listening["case_id"] else None
        measured = measure_case(raw, case["frame_count"], version=version,
                                historical_controls=case.get("historical_controls") if version is Version.V2 else None,
                                minimum_mix_rms=case.get("minimum_mix_rms"),
                                listening_frame_window=selected_window)
        measured.update({"case_id": identity.case_id, "access": access,
                         "preparation": raw["preparation"]})
        report["cases"].append(measured)
        del report["pending_case"]
        save_report()

    registry_binding = protocol["protected_registry"]
    try:
        access = run_development_access_session(
            identities, CASE_IDS, repo=REPO,
            registry=PinnedStageARegistry(REPO / registry_binding["path"], registry_binding["schema"], registry_binding["sha256"]),
            access_log_path=output / protocol["access_log"],
            qualification_owner_id=f"riotbox-1501-limiter-calibration-{version.value}", qualification_owner=owner)
        report["access_session_id"] = access["access_session_id"]
        selected = next(case for case in report["cases"]
                        if case["case_id"] == protocol["preselected_future_listening"]["case_id"])
        stress = next(condition["outputs"] for condition in selected["conditions"]
                      if condition["condition"] == protocol["preselected_future_listening"]["condition"])
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


def main(version: Version = Version.V1) -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.execute:
        execute(version)
    else:
        protocol, _, _ = preflight(version)
        print(f"metadata preflight passed: {len(protocol['cases'])} cases; no source audio opened; "
              f"protocol status={protocol['status']}")
