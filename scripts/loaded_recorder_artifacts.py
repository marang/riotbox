"""Read back the existing recorder's exact published artifacts, without audio I/O.

This is a bounded QA reader for the producer's frozen 56-byte float-WAV layout,
not a product decoder or a replacement for Core's Session/readiness validation.
Only paths allocated by the closed loaded workload are opened.
"""

import math
import os
from pathlib import Path
import stat
import struct

from loaded_recorder_evidence import load_json_bytes, require, sha256
from silent_host_evidence import EvidenceError, integer

JSON_LIMIT = 16 * 1024 * 1024
PROOF_LIMIT = 262144
TIMING_FIELDS = (
    "confirmed_bpm_micros", "bar_grid_anchor_position_microbeats",
    "beat_span_per_frame_nanobeats", "requested_start_position_microbeats",
    "captured_start_position_microbeats", "captured_end_position_microbeats",
    "start_alignment_error_frame_micros", "duration_error_frame_micros",
    "beats_per_bar", "duration_beats",
)
FAULT_FIELDS = (
    "callback_gap_over_threshold_count", "callback_scratch_overflow_count",
    "stream_error_count", "transport_mismatch_count", "tempo_mismatch_count",
    "timing_window_mismatch_count", "clip_count",
)
PLAN_FIELDS = (
    "action_id", "session_id", "session_pre_capture_sha256", "duration_beats",
    "sample_rate_hz", "channel_count", "frame_count", "source_graph_ref",
    "timing_grid_ref", "source_capture_refs", "lineage_capture_refs",
)
GATES = (
    "live_recording_runtime_master_capture", "live_recording_wav_readback",
    "live_recording_bar_window_alignment",
)


class ArtifactEvidenceError(EvidenceError):
    """Retain already computed technical evidence even when a later gate fails."""

    def __init__(self, message, evidence):
        super().__init__(message)
        self.artifact_evidence = evidence


def _field(value, key):
    if not isinstance(value, dict) or key not in value:
        raise EvidenceError(f"missing loaded recording field: {key}")
    return value[key]


def _number(value, name):
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        raise EvidenceError(f"invalid loaded recording number: {name}")
    return value


def _read_file(path, limit):
    """Bound a known regular file and reject aliases, devices and replaced reads."""
    path = Path(path)
    if not path.is_absolute() or any(part in (".", "..") for part in path.parts):
        raise EvidenceError("loaded recording path must be absolute and normalized")
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK
    try:
        current = Path(path.anchor)
        for part in path.parts[1:-1]:
            current /= part
            if not stat.S_ISDIR(current.lstat().st_mode):
                raise EvidenceError("loaded recording parent is not an ordinary directory")
        with os.fdopen(os.open(path, flags), "rb") as source:
            before = os.fstat(source.fileno())
            if (not stat.S_ISREG(before.st_mode) or before.st_nlink != 1
                    or before.st_uid != os.geteuid() or before.st_size > limit):
                raise EvidenceError("loaded recording file is unowned, aliased or exceeds budget")
            data = source.read(limit + 1)
            after = os.fstat(source.fileno())
    except OSError as error:
        raise EvidenceError(f"cannot read exact loaded recording file: {path.name}") from error
    identity = lambda info: (info.st_dev, info.st_ino, info.st_size,
                             info.st_mtime_ns, info.st_ctime_ns)
    if len(data) > limit or identity(before) != identity(after) or len(data) != before.st_size:
        raise EvidenceError("loaded recording file changed while reading or exceeds budget")
    return data


def _round(value):
    # Rust f64::round for the nonnegative metrics used by this producer.
    return math.floor(value + 0.5)


def _f32(value):
    return struct.unpack("<f", struct.pack("<f", value))[0]


def _wav_evidence(data, rate, channels, frames):
    """Read the exact App writer layout and independently measure its samples."""
    if len(data) < 56:
        raise EvidenceError("loaded recording WAV header is truncated")
    expected = (b"RIFF", len(data) - 8, b"WAVEfmt ", 16, 3, channels, rate,
                rate * channels * 4, channels * 4, 32, b"fact", 4, frames,
                b"data", frames * channels * 4)
    header = struct.unpack("<4sI8sIHHIIHH4sII4sI", data[:56])
    if header != expected or len(data) != 56 + frames * channels * 4:
        raise EvidenceError("loaded recording WAV differs from frozen float32 geometry")
    peak, active, clips, nonfinite = 0.0, 0, 0, 0
    sum_squares = 0.0
    activity_threshold = _f32(0.0001)  # riotbox_audio::runtime::signal_metrics
    for (sample,) in struct.iter_unpack("<f", memoryview(data)[56:]):
        if not math.isfinite(sample):
            nonfinite += 1
            sum_squares = None
            continue
        magnitude = abs(sample)
        peak = max(peak, magnitude)
        active += magnitude > activity_threshold
        clips += magnitude >= 1.0
        # Match signal_metrics' sequential f32 square/sum, not a new tolerance
        # around a cross-language double-precision RMS calculation.
        if sum_squares is not None:
            try:
                sum_squares = _f32(sum_squares + _f32(sample * sample))
            except OverflowError:
                # Invalid extreme float payloads can overflow the product's f32
                # energy reduction. Preserve other metrics, never invent zero RMS.
                sum_squares = None
    rms = (None if sum_squares is None else
           _f32(math.sqrt(_f32(sum_squares / _f32(frames * channels)))))
    evidence = {
        "wav_sha256": sha256(data), "sample_payload_sha256": sha256(data[56:]),
        "sample_rate_hz": rate, "channel_count": channels, "frame_count": frames,
        "sample_count": frames * channels, "nonfinite_sample_count": nonfinite,
        "active_sample_count": active, "clip_count": clips,
        "peak_amplitude_micros": _round(peak * 1_000_000),
        "rms_amplitude_micros": None if rms is None else _round(rms * 1_000_000),
    }
    if nonfinite or not active or peak <= _f32(1.0e-5) or clips or peak >= 1.0:
        raise ArtifactEvidenceError("loaded recording WAV is non-finite, silent or clipped", evidence)
    return evidence


def _check_summary(summary, marker, case):
    for key, expected in {
        "mode": "live_master_recording_execute", "status": "ready", "ready": True,
        "writes_files": True, "mutates_session": True, "runtime_stopped": True,
        "observer_events": True, "observer_status": "recorded", "readiness_blockers": [],
        "duration_bars": case.bars, "duration_beats": case.bars * 4, "beats_per_bar": 4,
        "session_path": str(case.session_path), "destination_path": str(case.destination_path),
        "proof_path": str(case.proof_path),
    }.items():
        require(_field(summary, key), expected, "final " + key)
    if "failure_reason" in summary or "observer_failure_reason" in summary:
        raise EvidenceError("loaded recording retained a failure reason")
    for key in ("duration_bars", "duration_beats", "requested_start_position_beats",
                "sample_rate_hz", "channel_count", "frame_count"):
        actual, expected = _field(summary, key), _field(marker, key)
        if key == "requested_start_position_beats":
            require(_number(actual, key), _number(expected, key), key)
        else:
            require(actual, expected, key)
    version = 4 if case.bars == 2 else 3
    boundary = f"runtime_master_bar_window_v{version}"
    require(_field(summary, "boundary"), boundary, "action boundary")
    require(_field(summary, "receipt_boundary"), "live_recording." + boundary, "receipt boundary")
    return version, boundary, {2: "two_bars", 8: "eight_bars", 16: "sixteen_bars"}[case.bars]


def _check_proof(proof, summary, marker, fixture, version, duration, wav):
    for key in PLAN_FIELDS:
        require(_field(proof, key), _field(marker, key), "milestone/proof " + key)
    require(proof["session_id"], fixture.session_id, "proof Session identity")
    graph = {key: fixture.source_graph_ref[key]
             for key in ("source_id", "graph_version", "graph_hash")}
    require(proof["source_graph_ref"], graph, "fixture Source Graph")
    require(proof["timing_grid_ref"], fixture.timing_grid_ref, "fixture timing grid")
    captures = fixture.baseline_data["captures"]
    active_capture = fixture.baseline_data["runtime_state"]["lane_state"]["w30"]["last_capture"]
    require(proof["source_capture_refs"], [active_capture], "fixed active capture")
    lineage = sorted({ref for capture in captures if capture["capture_id"] == active_capture
                      for ref in capture["lineage_capture_refs"]})
    require(proof["lineage_capture_refs"], lineage, "actual capture lineage")
    for key in ("wav_sha256", "sample_payload_sha256", "active_sample_count",
                "peak_amplitude_micros", "rms_amplitude_micros", "clip_count"):
        require(_field(proof, key), wav[key], "measured " + key)
    for key, expected in {
        "schema": f"riotbox.live_recording_runtime_master_bar_window.v{version}",
        "duration": duration, "duration_beats": summary["duration_beats"],
        "beats_per_bar": 4, "confirmed_bpm_micros": 132_000_000,
        "wav_sample_format": "ieee_float32",
    }.items():
        require(_field(proof, key), expected, "proof " + key)
    for key in FAULT_FIELDS:
        require(_field(proof, key), 0, "proof " + key)
    for key in ("callback_count", "armed_callback_count"):
        if integer(_field(proof, key), key) == 0:
            raise EvidenceError("loaded recording has no observed callbacks")
    gap = _field(proof, "max_callback_gap_micros")
    if gap is not None:
        integer(gap, "max callback gap")
    for key in ("host", "device", "device_sample_format"):
        if not isinstance(_field(proof, key), str) or not proof[key].strip():
            raise EvidenceError("loaded recording output identity is blank")
    for key in ("host", "device"):
        require(_field(summary, key), proof[key], "output " + key)
    scene = fixture.baseline_data["runtime_state"]["scene_state"]["active_scene"]
    require(_field(proof, "scene_id"), scene, "active scene")
    for key in TIMING_FIELDS:
        integer(_field(proof, key), key)
    rate, frames = wav["sample_rate_hz"], wav["frame_count"]
    span = 132.0 / 60.0 / rate
    require(proof["beat_span_per_frame_nanobeats"], _round(span * 1e9), "frame beat span")
    anchor = integer(_field(summary, "bar_grid_anchor_beat_cursor"), "bar anchor")
    require(proof["bar_grid_anchor_position_microbeats"], anchor * 1_000_000, "bar anchor")
    for prefix in ("requested_start", "captured_start", "captured_end"):
        position = _number(_field(summary, prefix + "_position_beats"), prefix)
        require(proof[prefix + "_position_microbeats"], _round(position * 1e6), prefix)
    requested = proof["requested_start_position_microbeats"]
    start = proof["captured_start_position_microbeats"]
    end = proof["captured_end_position_microbeats"]
    beat_span = proof["beat_span_per_frame_nanobeats"]
    if (requested < anchor * 1_000_000 or (requested - anchor * 1_000_000) % 4_000_000
            or start < requested or end <= start
            or proof["start_alignment_error_frame_micros"] > 1_000_000
            or proof["duration_error_frame_micros"] > 500_001
            or (start - requested) * 1000 > beat_span + 1000
            or abs((end - start) * 1000 - summary["duration_beats"] * 1_000_000_000)
            > (beat_span + 1) // 2 + 1000):
        raise EvidenceError("loaded recording timing window violates Core's existing bounds")
    require(frames, _round(rate * 60.0 * summary["duration_beats"] / 132.0), "window frames")


def _check_session(session, fixture, marker, case, boundary, duration, proof):
    baseline = fixture.baseline_data
    require(_field(session, "session_id"), fixture.session_id, "persisted Session identity")
    # These are the only top-level owners the existing recorder may update.
    for key, value in baseline.items():
        if key not in ("runtime_state", "action_log", "export_receipts", "updated_at"):
            require(_field(session, key), value, "unchanged Session " + key)
    require(set(session), set(baseline), "Session fields")
    before_runtime, after_runtime = baseline["runtime_state"], _field(session, "runtime_state")
    require(set(after_runtime), set(before_runtime), "runtime fields")
    for key, value in before_runtime.items():
        if key != "transport":
            require(_field(after_runtime, key), value, "unchanged runtime " + key)
    before_transport, transport = before_runtime["transport"], _field(after_runtime, "transport")
    require(_field(before_transport, "is_playing"), True, "fixed fixture playing state")
    require(set(transport), set(before_transport), "transport fields")
    for key, value in before_transport.items():
        if key != "position_beats":
            require(_field(transport, key), value, "unchanged transport " + key)
    if (_number(transport["position_beats"], "persisted position")
            < _number(before_transport["position_beats"], "baseline position")):
        raise EvidenceError("loaded recording transport position went backwards")
    before_log, log = baseline["action_log"], _field(session, "action_log")
    require(set(log), set(before_log), "action log fields")
    require(_field(log, "replay_policy"), before_log["replay_policy"], "replay policy")
    appended = []
    for key in ("actions", "commit_records"):
        old, new = before_log[key], _field(log, key)
        if not isinstance(new, list) or len(new) != len(old) + 1:
            raise EvidenceError("loaded recording must append exactly one action and commit")
        require(new[:-1], old, "unchanged " + key)
        appended.append(new[-1])
    action, commit = appended
    action_id = integer(_field(marker, "action_id"), "recording action")
    if action_id <= max((entry["id"] for entry in before_log["actions"]), default=0):
        raise EvidenceError("loaded recording action identity is not fresh")
    for key, value in {"id": action_id, "command": "ExportLiveRecording", "actor": "User",
                       "status": "Committed", "quantization": "Immediate"}.items():
        require(_field(action, key), value, "new action " + key)
    require(_field(_field(action, "result"), "accepted"), True, "accepted recording action")
    require(_field(commit, "action_id"), action_id, "recording commit identity")
    committed = integer(_field(action, "committed_at"), "action commit timestamp")
    require(_field(commit, "committed_at"), committed, "commit timestamp")
    if integer(_field(action, "requested_at"), "action request timestamp") > committed:
        raise EvidenceError("loaded recording action committed before request")
    target = _field(action, "target")
    require(_field(target, "scope"), "Session", "recording target scope")
    require(_field(target, "scene_id"), proof["scene_id"], "recording target scene")
    committed_boundary = _field(commit, "boundary")
    require(_field(committed_boundary, "kind"), "Immediate", "recording commit boundary")
    require(_field(committed_boundary, "scene_id"), proof["scene_id"], "commit scene")
    for key in ("beat_index", "bar_index", "phrase_index"):
        integer(_field(committed_boundary, key), key)
    sequence = 1 + max((entry["commit_sequence"] for entry in before_log["commit_records"]
                        if entry["boundary"] == committed_boundary), default=0)
    require(_field(commit, "commit_sequence"), sequence, "commit sequence")
    require(_field(commit, "mc202_source_phrase_plan"), None, "recording phrase side effect")
    params = _field(action, "params")
    require(set(params), {"LiveRecordingExport"}, "action params variant")
    expected = {
        "export_scope": "live_recording", "export_role": "live_recording_capture",
        "boundary": boundary, "duration": duration, "include_manifest": True,
        "destination_kind": "local_file_path", "destination_path": str(case.destination_path),
        "receipt_id": None,
    }
    require(params["LiveRecordingExport"], expected, "recording params")
    old_receipts, receipts = baseline.get("export_receipts", []), _field(session, "export_receipts")
    if not isinstance(receipts, list) or len(receipts) != len(old_receipts) + 1:
        raise EvidenceError("loaded recording must append exactly one receipt")
    require(receipts[:-1], old_receipts, "unchanged receipts")
    receipt = receipts[-1]
    receipt_id = f"export-receipt-a-{action_id:04}"
    require(_field(receipt, "receipt_id"), receipt_id, "receipt ID")
    require(proof["receipt_id"], receipt_id, "proof receipt ID")
    if any(entry["receipt_id"] == receipt_id for entry in old_receipts):
        raise EvidenceError("loaded recording receipt identity is not fresh")
    require(_field(receipt, "created_by_action"), action_id, "receipt action")
    require(_field(receipt, "created_at"), committed, "receipt timestamp")
    return receipt


def _check_receipt(receipt, summary, proof, case, version, boundary, duration, wav, proof_hash):
    for key, value in {
        "export_scope": "live_recording", "export_role": "live_recording_capture",
        "pack_id": f"live-recording-runtime-master-bar-window-v{version}",
        "export_boundary": "live_recording_" + boundary, "live_recording_duration": duration,
        "artifact_path": str(case.destination_path), "proof_path": str(case.proof_path),
        "manifest_path": str(case.proof_path), "export_hash": wav["wav_sha256"],
        "normalized_manifest_hash": proof_hash, "readiness_status": "reproducible",
        "unsupported_scopes": [], "arrangement_placement_refs": [], "daw_tempo_map_ref": None,
    }.items():
        require(_field(receipt, key), value, "receipt " + key)
    final_receipt = _field(summary, "receipt")
    for key in ("receipt_id", "pack_id", "export_scope", "export_role", "qa_gates"):
        require(_field(final_receipt, key), _field(receipt, key), "final receipt " + key)
    require(_field(final_receipt, "sha256"), wav["wav_sha256"], "final WAV hash")
    require(_field(final_receipt, "proof_sha256"), proof_hash, "final raw proof hash")
    require(_field(final_receipt, "host_audio_readiness"), {"status": "ready", "blockers": []},
            "Core host audio readiness")
    gates = _field(receipt, "qa_gates")
    if (not isinstance(gates, list) or len(gates) != len(GATES)
            or any(not isinstance(gate, dict) for gate in gates)):
        raise EvidenceError("loaded recording requires exactly the existing three Core gates")
    for gate_id in GATES:
        matching = [gate for gate in gates if gate.get("gate_id") == gate_id]
        if len(matching) != 1:
            raise EvidenceError("loaded recording Core gate identity is missing or duplicated")
        require(_field(matching[0], "status"), "passed", gate_id)
    artifacts = _field(receipt, "artifact_set")
    if not isinstance(artifacts, list) or len(artifacts) != 2:
        raise EvidenceError("loaded recording artifact set must contain WAV and proof only")
    audio, manifest = artifacts
    for artifact, role, media, path, digest in (
        (audio, "live_recording_capture", "audio_wav", case.destination_path, wav["wav_sha256"]),
        (manifest, "product_export_proof", "json", case.proof_path, proof_hash),
    ):
        for key, value in {"role": role, "media_type": media, "sha256": digest,
                           "location": {"kind": "local_path", "path": str(path)}}.items():
            require(_field(artifact, key), value, "artifact " + key)
    for key in ("source_graph_ref", "timing_grid_ref", "source_capture_refs", "lineage_capture_refs"):
        require(_field(audio, key), proof[key], "audio artifact " + key)
    duration_ms = (wav["frame_count"] * 1000 + wav["sample_rate_hz"] // 2) // wav["sample_rate_hz"]
    for key, value in {"sample_rate_hz": wav["sample_rate_hz"], "channel_count": 2,
                       "duration_ms": duration_ms}.items():
        require(_field(audio, key), value, "audio " + key)
    for key, value in {"peak_amplitude_micros": wav["peak_amplitude_micros"],
                       "rms_amplitude_micros": wav["rms_amplitude_micros"],
                       "total_frame_count": wav["frame_count"]}.items():
        require(_field(_field(audio, "audio_metrics"), key), value, "audio metrics " + key)
    host_refs = _field(receipt, "live_recording_host_audio_refs")
    if not isinstance(host_refs, list) or len(host_refs) != 1:
        raise EvidenceError("loaded recording requires one host audio identity")
    gap = proof["max_callback_gap_micros"]
    require(host_refs[0], {
        "host": proof["host"], "device": proof["device"], "recording_duration_ms": duration_ms,
        "callback_gap_summary": {"max_gap_ms": None if gap is None else (gap + 999) // 1000,
                                 "over_threshold_count": 0},
        "stream_error_summary": {"error_count": 0, "last_error": None},
        "timing_window": {key: proof[key] for key in TIMING_FIELDS},
    }, "host audio/proof identity")


def validate_final(summary, marker, case, fixture):
    """Check published bytes and Core-owned identities; never launch or play audio."""
    evidence = {}
    try:
        fixture.verify_unchanged()
        if type(case.bars) is not int or case.bars not in (2, 8, 16):
            raise EvidenceError("unsupported loaded recording bar count")
        require(str(case.proof_path), str(case.destination_path) + ".riotbox.json",
                "producer proof filename")
        version, boundary, duration = _check_summary(summary, marker, case)
        rate = integer(_field(marker, "sample_rate_hz"), "sample rate")
        require(_field(marker, "channel_count"), 2, "stereo output")
        if rate not in (44100, 48000):
            raise EvidenceError("loaded recording rate is outside tested generated geometry")
        frames = _round(rate * 60.0 * case.bars * 4 / 132.0)
        require(_field(marker, "frame_count"), frames, "exact recording frame count")
        proof_data = _read_file(case.proof_path, PROOF_LIMIT)
        proof = load_json_bytes(proof_data)
        session_data = _read_file(case.session_path, JSON_LIMIT)
        session = load_json_bytes(session_data)
        wav_data = _read_file(case.destination_path, 56 + frames * 2 * 4)
        evidence.update(_wav_evidence(wav_data, rate, 2, frames))
        evidence["proof_sha256"] = sha256(proof_data)
        evidence["persisted_session_sha256"] = sha256(session_data)
        _check_proof(proof, summary, marker, fixture, version, duration, evidence)
        receipt = _check_session(session, fixture, marker, case, boundary, duration, proof)
        _check_receipt(receipt, summary, proof, case, version, boundary, duration,
                       evidence, evidence["proof_sha256"])
        fixture.verify_unchanged()
        evidence.update({"action_id": marker["action_id"], "receipt_id": receipt["receipt_id"],
                         "baseline_unchanged": True, "quality_proof": False,
                         "classification": "generated_control_preparation_only"})
        return evidence
    except ArtifactEvidenceError:
        raise
    except (EvidenceError, KeyError, TypeError, ValueError, OverflowError) as error:
        raise ArtifactEvidenceError(str(error), evidence) from error
