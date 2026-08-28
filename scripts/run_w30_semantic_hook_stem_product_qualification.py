#!/usr/bin/env python3
"""Run RIOTBOX-1482's frozen four-source Development qualification."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import struct
import subprocess
import wave
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


CONTRACT = Path("docs/benchmarks/w30_semantic_hook_stem_product_qualification_v4.json")
EXPECTED_SCHEMA = "riotbox.w30_semantic_hook_stem_product_qualification.v4"
EXPECTED_CONTRACT_SHA256 = "459567e3990707f19d58f3cf551c4099e5ebbdfb814909517377e56739ccf996"
FINAL_ACCESS_LOG = Path("artifacts/development/riotbox-1482/access-log-product-v4-final.json")
RECONCILIATION_CONTRACT = Path(
    "docs/benchmarks/w30_semantic_hook_stem_product_v4_reconciliation_v1.json"
)
EXPECTED_RECONCILIATION_CONTRACT_SHA256 = (
    "8e71af16471e26d766cee34d96d1730da5154d17224b69aa9781ed1bf1755966"
)
RECONCILIATION_REPORT = Path(
    "artifacts/development/riotbox-1482/product-v4-reconciliation-final.json"
)
EXPECTED_OWNER_ACTIONS = [
    "SourceTimingConfirmGrid",
    "PresetActivate",
    "CaptureSetLength",
    "CaptureBarGroup",
    "PromoteCaptureToPad",
    "W30TriggerPad",
]
EXPECTED_QA_GATES = {
    "stem_package_artifact_set_evidence",
    "stem_package_per_stem_hash_stability",
    "stem_package_per_stem_non_silence",
    "stem_package_per_stem_lineage",
    "stem_package_per_stem_fallback_comparison",
}


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path: Path, payload: dict[str, Any]) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def create_exclusive_json(path: Path, payload: dict[str, Any]) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags, 0o600)
    try:
        data = (json.dumps(payload, indent=2) + "\n").encode()
        while data:
            data = data[os.write(descriptor, data) :]
    finally:
        os.close(descriptor)


def read_original_once(path: Path) -> tuple[bytes, str]:
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(path, flags)
    try:
        metadata = os.fstat(descriptor)
        require(stat.S_ISREG(metadata.st_mode), f"not a regular file: {path}")
        chunks: list[bytes] = []
        digest = hashlib.sha256()
        while chunk := os.read(descriptor, 1024 * 1024):
            digest.update(chunk)
            chunks.append(chunk)
        return b"".join(chunks), digest.hexdigest()
    finally:
        os.close(descriptor)


def write_private_copy(path: Path, data: bytes) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags, 0o600)
    try:
        remaining = memoryview(data)
        while remaining:
            remaining = remaining[os.write(descriptor, remaining) :]
    finally:
        os.close(descriptor)


def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(payload, dict), f"expected JSON object: {path}")
    return payload


def validate_frozen_inputs(repo: Path, contract: dict[str, Any]) -> None:
    registry = contract["development_registry"]
    require(
        file_sha256(repo / registry["path"]) == registry["raw_sha256"],
        "Development registry SHA-256 changed",
    )
    keep = contract["development_keep"]
    require(
        file_sha256(repo / keep["contract_path"]) == keep["contract_sha256"],
        "Development keep contract SHA-256 changed",
    )
    review = repo / "artifacts/audio_qa/local/listening-reviews/RIOTBOX-1482/review.json"
    require(
        file_sha256(review) == keep["structured_review_sha256"],
        "Development keep review SHA-256 changed",
    )
    review_payload = load_json(review)
    require(review_payload.get("human_verdict") == "keep", "Development verdict is not keep")
    require(contract.get("partition") == "development", "contract is not Development-only")
    access = contract["access"]
    require(access["maximum_unique_development_files"] == 4, "source access limit changed")
    require(access["maximum_opens_per_development_file"] == 1, "source open limit changed")
    require(access["directory_discovery_allowed"] is False, "directory discovery was enabled")
    require(access["holdout_audio_allowed"] is False, "Holdout access was enabled")
    require(
        access["commercial_reference_audio_allowed"] is False,
        "commercial-reference access was enabled",
    )
    owner = contract["exact_product_owner"]
    require(owner["binary"] == "w30_live_path_render", "exact owner binary changed")
    require(
        owner["exact_committed_actions_before_export"] == EXPECTED_OWNER_ACTIONS,
        "exact owner action sequence changed",
    )
    require(owner["additional_pre_export_actions_allowed"] is False, "extra owner actions allowed")
    require(contract["render_contract"]["transport_start_beat"] == 8.0, "render start changed")
    require(
        contract["stopping_rule"]["this_is_the_final_riotbox_1482_product_qualification_attempt"]
        is True,
        "final-attempt boundary changed",
    )


def analyze_wav(path: Path, expected_frames: int) -> dict[str, Any]:
    with wave.open(str(path), "rb") as wav:
        channels = wav.getnchannels()
        sample_width = wav.getsampwidth()
        sample_rate = wav.getframerate()
        frame_count = wav.getnframes()
        compression = wav.getcomptype()
        frames = wav.readframes(frame_count)
    require(channels == 2, "semantic stem is not stereo")
    require(sample_width == 2, "semantic stem is not PCM16")
    require(sample_rate == 48_000, "semantic stem is not 48 kHz")
    require(compression == "NONE", "semantic stem is compressed")
    require(frame_count == expected_frames, "semantic stem frame count mismatch")
    samples = struct.unpack(f"<{len(frames) // 2}h", frames)
    active_samples = sum(sample != 0 for sample in samples)
    clipped_samples = sum(sample in (-32768, 32767) for sample in samples)
    require(active_samples > 0, "semantic stem is silent")
    require(clipped_samples == 0, "semantic stem contains full-scale clipped PCM")
    return {
        "sample_rate_hz": sample_rate,
        "channel_count": channels,
        "sample_width_bits": sample_width * 8,
        "frame_count": frame_count,
        "active_samples": active_samples,
        "full_scale_sample_count": clipped_samples,
        "sha256": file_sha256(path),
    }


def validate_exact_owner_session(
    session: dict[str, Any],
    source_graph: dict[str, Any],
    source: dict[str, Any],
    summary: dict[str, Any],
    receipt: dict[str, Any],
) -> None:
    committed = [
        action for action in session["action_log"]["actions"] if action["status"] == "Committed"
    ]
    require(len(committed) == 7, "V4 Session must contain six owner actions plus export")
    owner = committed[:6]
    require([action["command"] for action in owner] == EXPECTED_OWNER_ACTIONS, "owner order changed")
    require(
        [action["id"] for action in owner] == summary["exact_owner"]["committed_action_ids"],
        "owner action identities changed",
    )
    require(
        [action["quantization"] for action in owner]
        == ["Immediate", "Immediate", "Immediate", "NextBar", "NextBar", "NextBeat"],
        "owner quantization changed",
    )
    source_id = source_graph["source"]["source_id"]
    timing_params = owner[0]["params"]["SourceTimingGrid"]
    hypothesis_id = timing_params["hypothesis_id"]
    require(
        timing_params["source_id"] == source_id
        and timing_params["confirmed_bpm"] == summary["committed_bpm"],
        "timing confirmation params changed",
    )
    require(owner[0]["target"]["object_id"] == hypothesis_id, "timing Action target changed")
    confirmed_grid = session["runtime_state"]["source_timing"]["confirmed_grid"]
    require(
        confirmed_grid["source_id"] == source_id
        and confirmed_grid["hypothesis_id"] == hypothesis_id
        and confirmed_grid["confirmed_by_action"] == owner[0]["id"],
        "Session timing confirmation differs from the Action",
    )
    expected_hypothesis = (
        "probe-bpm-primary"
        if source.get("downbeat_seconds") is None
        else "manual-source-grid-v1-42f00000-00000000"
    )
    require(hypothesis_id == expected_hypothesis, "timing hypothesis does not match source contract")
    require(
        owner[1]["params"]["Preset"] == {"preset_id": "feral_break_alpha_v2"},
        "preset params changed",
    )
    require(
        owner[2]["params"]["CaptureLength"] == {"intent": "one_bar"},
        "capture-length params changed",
    )
    require(owner[3]["params"]["Capture"] == {"bars": None}, "capture params changed")
    capture_id = receipt_stem(receipt)["source_capture_refs"][0]
    require(
        owner[4]["params"]["Promotion"]
        == {"capture_id": capture_id, "destination": "w30:bank-a/pad-01"},
        "promotion params changed",
    )
    require(
        owner[5]["params"]["Mutation"] == {"intensity": 0.84, "target_id": capture_id},
        "trigger params changed",
    )
    require(committed[6]["id"] == receipt["created_by_action"], "export action identity changed")


def receipt_stem(receipt: dict[str, Any]) -> dict[str, Any]:
    return next(
        artifact for artifact in receipt["artifact_set"] if artifact["role"] == "w30_hook_loop"
    )


def validate_case(
    repo: Path,
    contract: dict[str, Any],
    source: dict[str, Any],
    case_output: Path,
) -> dict[str, Any]:
    summary_path = case_output / "w30-hook-product-export-summary.json"
    session_path = case_output / "session.json"
    summary = load_json(summary_path)
    require(
        summary.get("schema") == "riotbox.w30_hook_product_export_summary.v4",
        "unexpected product summary schema",
    )
    require(summary.get("status") == "pass", "product summary did not pass")
    require(
        summary.get("source_sha256") == f"sha256:{source['sha256']}",
        "product Source Graph SHA-256 mismatch",
    )
    product_bpm = float(summary["product_bpm"])
    require(
        abs(product_bpm - float(source["expected_product_bpm"]))
        <= float(contract["technical_gates"]["maximum_absolute_product_bpm_delta"]),
        "product BPM admission failed",
    )
    committed_bpm = float(summary["committed_bpm"])
    require(
        abs(committed_bpm - float(source["confirmation_bpm"])) <= 0.0001,
        "committed Session BPM differs from the confirmed source BPM",
    )

    receipt = summary["receipt"]
    require(receipt["export_scope"] == "stem_package", "wrong receipt scope")
    require(receipt["pack_id"] == "stem-package-w30-hook-loop", "wrong pack id")
    require(
        receipt["export_boundary"] == "stem_package_w30_hook_loop_v4",
        "wrong receipt boundary",
    )
    require(receipt["readiness_status"] == "reproducible", "receipt is not reproducible")
    require(receipt["unsupported_scopes"] == [], "receipt has unsupported scopes")
    roles = [artifact["role"] for artifact in receipt["artifact_set"]]
    require(
        sorted(roles) == ["export_manifest", "product_export_proof", "w30_hook_loop"],
        "receipt artifact set is not exact",
    )
    gates = receipt["qa_gates"]
    require({gate["gate_id"] for gate in gates} == EXPECTED_QA_GATES, "QA gate set changed")
    require(all(gate["status"] == "passed" for gate in gates), "receipt QA gate failed")

    stem = receipt_stem(receipt)
    require(stem["source_graph_ref"] is not None, "stem omitted Source Graph lineage")
    require(stem["timing_grid_ref"] is not None, "stem omitted timing-grid lineage")
    require(len(stem["source_capture_refs"]) == 1, "stem capture lineage is not exact")
    require(
        stem["fallback_comparison"]["rms_difference_micros"] > 0,
        "stem does not differ from missing-source silence",
    )
    stem_path = Path(stem["location"]["path"])
    require(stem_path.is_relative_to(case_output), "stem escaped the case output")
    expected_frames = round(8.0 * 60.0 * 48_000 / committed_bpm)
    wav_report = analyze_wav(stem_path, expected_frames)
    require(wav_report["sha256"] == stem["sha256"], "written stem SHA-256 mismatch")
    require(
        summary["semantic_stem"]["sha256"] == stem["sha256"],
        "summary and receipt stem identities differ",
    )
    owner = summary["exact_owner"]
    require(owner["product_path"] == "ordinary_promoted_w30_control_v1", "wrong product owner")
    require(owner["binary"] == "w30_live_path_render", "wrong owner binary")
    require(owner["committed_actions"] == EXPECTED_OWNER_ACTIONS, "wrong owner action sequence")
    owner_render = summary["owner_render"]
    require(owner_render["start_beat"] == 8.0, "wrong owner render start")
    require(owner_render["duration_beats"] == 8.0, "wrong owner render duration")
    require(owner_render["frame_count"] == expected_frames, "owner frame count mismatch")
    require(owner_render["active_samples"] > 0, "owner render is silent")
    require(owner_render["pre_limiter_clip_count"] == 0, "owner render clipped pre-limiter")
    require(owner_render["limited_sample_count"] == 0, "owner render used limiter")
    require(owner_render["post_limiter_clip_count"] == 0, "owner render clipped post-limiter")
    require(
        owner_render["callback_partition_128_vs_257_sample_exact"] is True,
        "owner callback partition changed output",
    )
    require(owner_render["written_product_wav_byte_exact"] is True, "owner/product mismatch")
    owner_control_path = Path(summary["owner_control_path"])
    require(owner_control_path.is_relative_to(case_output), "owner control escaped case output")
    require(file_sha256(owner_control_path) == wav_report["sha256"], "owner/product WAV mismatch")
    if source["case_id"] == "dense_beat03_130":
        require(
            wav_report["sha256"]
            == contract["technical_gates"]["dense_case_expected_sha256"],
            "dense product WAV differs from the kept Development loop",
        )

    lifecycle = [
        event
        for event in summary["observer_snapshot"]["export"]["lifecycle"]
        if event.get("action_id") == receipt["created_by_action"]
    ]
    require(
        [event["stage"] for event in lifecycle] == ["requested", "started", "completed"],
        "observer export lifecycle is not exact",
    )
    require(
        all(event.get("receipt", {}).get("receipt_id") == receipt["receipt_id"] for event in lifecycle),
        "observer lifecycle receipt identity mismatch",
    )

    session = load_json(session_path)
    source_graph = load_json(case_output / "source-graph.json")
    validate_exact_owner_session(session, source_graph, source, summary, receipt)
    session_receipt = next(
        (item for item in session["export_receipts"] if item["receipt_id"] == receipt["receipt_id"]),
        None,
    )
    require(session_receipt == receipt, "Session receipt differs from product receipt")
    action = next(
        (
            item
            for item in session["action_log"]["actions"]
            if item["id"] == receipt["created_by_action"]
        ),
        None,
    )
    require(action is not None and action["command"] == "ExportStemPackage", "action is missing")
    require(action["status"] == "Committed", "export action is not committed")
    params = action["params"]["StemPackageExport"]
    require(params["boundary"] == "w30_hook_loop_v4", "action boundary mismatch")
    require(params["export_scope"] == "stem_package", "action scope mismatch")
    require(params["export_role"] == "package_manifest", "action role kind mismatch")
    require(params["include_manifest"] is True, "manifest was disabled")
    require(params["destination_kind"] == "local_artifact_directory", "destination changed")
    require(params["handoff_proof_path"] is None, "unexpected handoff proof")
    require(params["lineage_policy"] == "require_any_core_lineage", "lineage policy changed")
    require(params["fallback_comparison_policy"] == "required", "fallback policy changed")
    committed_before_export = [
        candidate["command"]
        for candidate in session["action_log"]["actions"]
        if candidate["status"] == "Committed" and candidate["id"] != receipt["created_by_action"]
    ]
    require(committed_before_export == EXPECTED_OWNER_ACTIONS, "Session owner action sequence mismatch")
    capture_id = stem["source_capture_refs"][0]
    latest_damage = next(
        (
            candidate
            for candidate in reversed(session["action_log"]["actions"])
            if candidate["command"] == "W30ApplyDamageProfile"
            and candidate["status"] == "Committed"
            and candidate["params"].get("Mutation", {}).get("target_id") == capture_id
        ),
        None,
    )
    require(
        latest_damage is None
        or latest_damage["params"]["Mutation"].get("intensity", 0.0) <= 0.0,
        "qualification Session contains active W-30 damage",
    )
    require(params["claimed_stem_roles"] == ["w30_hook_loop"], "action role mismatch")

    manifest_path = Path(receipt["manifest_path"])
    proof_path = Path(receipt["proof_path"])
    require(file_sha256(manifest_path) == receipt["export_hash"], "manifest identity mismatch")
    manifest = load_json(manifest_path)
    proof = load_json(proof_path)
    require(manifest["claimed_stem_roles"] == ["w30_hook_loop"], "manifest role mismatch")
    require(proof["claimed_stem_roles"] == ["w30_hook_loop"], "proof role mismatch")
    require(proof["manifest_sha256"] == receipt["export_hash"], "proof manifest hash mismatch")
    return {
        "product_bpm": product_bpm,
        "committed_bpm": committed_bpm,
        "receipt_id": receipt["receipt_id"],
        "action_id": receipt["created_by_action"],
        "stem_path": (
            stem_path.relative_to(repo).as_posix()
            if stem_path.is_relative_to(repo)
            else stem_path.as_posix()
        ),
        "stem": wav_report,
        "summary_sha256": file_sha256(summary_path),
        "session_sha256": file_sha256(session_path),
        "manifest_sha256": file_sha256(manifest_path),
        "proof_sha256": file_sha256(proof_path),
        "observer_stages": [event["stage"] for event in lifecycle],
    }


def prepare_review(repo: Path, contract: dict[str, Any], session: str) -> int:
    root = repo / "artifacts/development/riotbox-1482"
    access_log_path = repo / FINAL_ACCESS_LOG
    access_log = load_json(access_log_path)
    reconciliation_path = repo / RECONCILIATION_REPORT
    reconciliation = load_json(reconciliation_path)
    require(reconciliation.get("status") == "passed_all_four_unchanged_cases", "technical reconciliation has not passed")
    require(
        reconciliation.get("failed_access_log_sha256") == file_sha256(access_log_path),
        "reconciliation is not bound to the failed V4 access log",
    )
    representative_id = contract["human_review"]["representative_case"]
    source = next(item for item in contract["source_set"] if item["case_id"] == representative_id)
    case_output = root / f"product-qualification-{session}" / representative_id
    private_source = case_output / "registered-source-owner-copy.wav"
    stem = case_output / "w30-hook-product-export/stem_package/stems/w30_hook_loop.wav"
    review_dir = root / f"product-review-{session}"
    require(not review_dir.exists(), f"review directory already exists: {review_dir}")
    review_dir.mkdir()
    review_path = review_dir / "01_tonal_source_then_w30_hook_loop.wav"
    duration = 8.0 * 60.0 / float(source["confirmation_bpm"])
    filter_graph = (
        f"[0:a]atrim=start=0:end={duration:.9f},asetpts=PTS-STARTPTS,"
        "aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,asplit=2[s1][s2];"
        "[s1][s2]concat=n=2:v=0:a=1[src];"
        "anullsrc=r=48000:cl=stereo:d=1[sil];"
        "[1:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,"
        "asplit=3[h1][h2][h3];[h1][h2][h3]concat=n=3:v=0:a=1[hook];"
        "[src][sil][hook]concat=n=3:v=0:a=1[out]"
    )
    completed = subprocess.run(
        [
            "ffmpeg", "-v", "error", "-nostdin", "-i", str(private_source), "-i", str(stem),
            "-filter_complex", filter_graph, "-map", "[out]", "-c:a", "pcm_s16le", str(review_path),
        ],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    require(completed.returncode == 0, f"review artifact build failed: {completed.stdout[-1000:]}")
    with wave.open(str(review_path), "rb") as wav:
        review_metadata = {
            "sample_rate_hz": wav.getframerate(),
            "channel_count": wav.getnchannels(),
            "sample_width_bits": wav.getsampwidth() * 8,
            "frame_count": wav.getnframes(),
            "duration_seconds": wav.getnframes() / wav.getframerate(),
        }
    review_record = {
        "schema": "riotbox.w30_semantic_hook_stem_product_review_artifact.v4",
        "ticket": "RIOTBOX-1482",
        "session": session,
        "created_at_utc": utc_now(),
        "technical_access_log_sha256": file_sha256(access_log_path),
        "technical_reconciliation_sha256": file_sha256(reconciliation_path),
        "case_id": representative_id,
        "source_original_reopened": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "artifact_path": review_path.relative_to(repo).as_posix(),
        "artifact_sha256": file_sha256(review_path),
        "audible_order": contract["human_review"]["order"],
        "questions": contract["human_review"]["questions"],
        "technical_preflight": review_metadata,
        "status": "ready_for_listening_review_gate",
    }
    review_record_path = review_dir / "review-artifact.json"
    create_exclusive_json(review_record_path, review_record)
    print(review_record_path.relative_to(repo))
    return 0


def reconcile_existing_outputs(repo: Path, contract: dict[str, Any], session: str) -> int:
    reconciliation_path = repo / RECONCILIATION_CONTRACT
    reconciliation_bytes = reconciliation_path.read_bytes()
    require(
        hashlib.sha256(reconciliation_bytes).hexdigest()
        == EXPECTED_RECONCILIATION_CONTRACT_SHA256,
        "frozen reconciliation contract changed",
    )
    reconciliation_contract = json.loads(reconciliation_bytes)
    require(
        reconciliation_contract.get("schema")
        == "riotbox.w30_semantic_hook_stem_product_v4_reconciliation.v1",
        "unexpected reconciliation schema",
    )
    require(
        reconciliation_contract["product_qualification"]["session"] == session,
        "reconciliation session changed",
    )
    access_log_path = repo / FINAL_ACCESS_LOG
    access_log_sha256 = file_sha256(access_log_path)
    require(
        access_log_sha256
        == reconciliation_contract["product_qualification"]["failed_access_log_sha256"],
        "failed V4 access-log identity changed",
    )
    access_log = load_json(access_log_path)
    require(access_log.get("status") == "failed_closed", "expected preserved failed V4 log")
    require(
        access_log.get("failure")
        == reconciliation_contract["product_qualification"]["recorded_failure"],
        "failed V4 reason changed",
    )
    source_records = {record["case_id"]: record for record in access_log["cases"]}
    expected_outputs = {
        record["case_id"]: record
        for record in reconciliation_contract["exact_existing_outputs"]
    }
    output_root = repo / "artifacts/development/riotbox-1482" / f"product-qualification-{session}"
    results: list[dict[str, Any]] = []
    for source in contract["source_set"]:
        case_id = source["case_id"]
        source_record = source_records[case_id]
        require(source_record["original_open_count"] == 1, f"{case_id}: source open count changed")
        require(source_record["observed_sha256"] == source["sha256"], f"{case_id}: source identity changed")
        case_output = output_root / case_id
        summary_path = case_output / "w30-hook-product-export-summary.json"
        expected_output = expected_outputs[case_id]
        require(
            file_sha256(summary_path) == expected_output["summary_sha256"],
            f"{case_id}: product summary changed",
        )
        result = validate_case(repo, contract, source, case_output)
        required_stem_sha256 = expected_output.get("required_w30_hook_loop_sha256")
        if required_stem_sha256 is not None:
            require(
                result["stem"]["sha256"] == required_stem_sha256,
                f"{case_id}: semantic stem changed",
            )
        results.append({"case_id": case_id, "qualification": result})

    report_path = repo / RECONCILIATION_REPORT
    require(not report_path.exists(), f"reconciliation report already exists: {report_path}")
    report = {
        "schema": "riotbox.w30_semantic_hook_stem_product_v4_reconciliation_report.v1",
        "ticket": "RIOTBOX-1482",
        "session": session,
        "created_at_utc": utc_now(),
        "contract_path": RECONCILIATION_CONTRACT.as_posix(),
        "contract_sha256": EXPECTED_RECONCILIATION_CONTRACT_SHA256,
        "failed_access_log_path": FINAL_ACCESS_LOG.as_posix(),
        "failed_access_log_sha256": access_log_sha256,
        "original_source_audio_reopened": False,
        "source_directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "audio_rerendered": False,
        "cases": results,
        "status": "passed_all_four_unchanged_cases",
    }
    create_exclusive_json(report_path, report)
    print(report_path.relative_to(repo))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--session", required=True)
    parser.add_argument("--prepare-review-from-passed-session", action="store_true")
    parser.add_argument("--reconcile-existing-final-session", action="store_true")
    args = parser.parse_args()
    require(args.session.replace("-", "").isalnum(), "session must be a safe token")

    repo = Path(__file__).resolve().parent.parent
    contract_path = repo / CONTRACT
    contract_bytes = contract_path.read_bytes()
    require(
        hashlib.sha256(contract_bytes).hexdigest() == EXPECTED_CONTRACT_SHA256,
        "frozen contract changed; create a new version and Decision",
    )
    contract = json.loads(contract_bytes)
    require(contract.get("schema") == EXPECTED_SCHEMA, "unexpected qualification schema")
    sources = contract.get("source_set")
    require(isinstance(sources, list) and len(sources) == 4, "contract must contain four sources")
    validate_frozen_inputs(repo, contract)
    require(
        not (
            args.prepare_review_from_passed_session
            and args.reconcile_existing_final_session
        ),
        "select only one post-qualification operation",
    )
    if args.reconcile_existing_final_session:
        return reconcile_existing_outputs(repo, contract, args.session)
    if args.prepare_review_from_passed_session:
        return prepare_review(repo, contract, args.session)

    root = repo / "artifacts/development/riotbox-1482"
    output_root = root / f"product-qualification-{args.session}"
    access_log_path = repo / FINAL_ACCESS_LOG
    root.mkdir(parents=True, exist_ok=True)
    require(not access_log_path.exists(), f"access log already exists: {access_log_path}")
    require(not output_root.exists(), f"output directory already exists: {output_root}")
    output_root.mkdir()
    access_log: dict[str, Any] = {
        "schema": "riotbox.w30_semantic_hook_stem_product_access_log.v4",
        "ticket": "RIOTBOX-1482",
        "session": args.session,
        "started_at_utc": utc_now(),
        "contract_path": CONTRACT.as_posix(),
        "contract_sha256": EXPECTED_CONTRACT_SHA256,
        "scope": "Development-only exact registered paths",
        "directory_discovery_performed": False,
        "holdout_audio_opened": False,
        "commercial_reference_audio_opened": False,
        "maximum_unique_development_files": 4,
        "maximum_original_opens_per_file": 1,
        "cases": [],
        "status": "started_before_first_source_open",
    }
    create_exclusive_json(access_log_path, access_log)

    try:
        for source in sources:
            case_id = source["case_id"]
            case_output = output_root / case_id
            case_output.mkdir()
            record: dict[str, Any] = {
                "case_id": case_id,
                "family": source["family"],
                "registered_path": source["path"],
                "expected_sha256": source["sha256"],
                "original_open_count": 0,
                "status": "opening_exact_registered_source_once",
            }
            access_log["cases"].append(record)
            write_json(access_log_path, access_log)

            source_bytes, source_hash = read_original_once(repo / source["path"])
            record["original_open_count"] = 1
            record["observed_sha256"] = source_hash
            require(source_hash == source["sha256"], f"{case_id}: source SHA-256 mismatch")
            private_source = case_output / "registered-source-owner-copy.wav"
            write_private_copy(private_source, source_bytes)
            record["private_owner_copy_sha256"] = file_sha256(private_source)
            record["status"] = "rendering_product_path_from_private_owner_copy"
            write_json(access_log_path, access_log)

            command = [
                "cargo", "run", "--quiet", "-p", "riotbox-app", "--bin",
                "w30_live_path_render", "--", "--source", str(private_source),
                "--bpm", str(source["confirmation_bpm"]), "--output", str(case_output),
                "--qualify-semantic-hook-product-v4",
            ]
            if source.get("downbeat_seconds") is not None:
                command.extend(["--downbeat-seconds", str(source["downbeat_seconds"])])
            completed = subprocess.run(
                command,
                cwd=repo,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                check=False,
            )
            record["renderer_exit_code"] = completed.returncode
            record["renderer_output_tail"] = completed.stdout.splitlines()[-8:]
            require(completed.returncode == 0, f"{case_id}: product renderer failed")
            record["qualification"] = validate_case(repo, contract, source, case_output)
            record["status"] = "passed"
            record["completed_at_utc"] = utc_now()
            write_json(access_log_path, access_log)

        access_log["status"] = "passed_all_four_cases"
        access_log["completed_at_utc"] = utc_now()
        write_json(access_log_path, access_log)
        print(access_log_path.relative_to(repo))
        return 0
    except Exception as error:
        access_log["status"] = "failed_closed"
        access_log["failure"] = str(error)
        access_log["completed_at_utc"] = utc_now()
        write_json(access_log_path, access_log)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
