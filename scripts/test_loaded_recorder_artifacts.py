"""Fresh synthetic publication controls only: no source, runtime or host access."""

import copy
import hashlib
import json
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest

from loaded_recorder_artifacts import (
    ArtifactEvidenceError, TIMING_FIELDS, _check_proof, _read_file, _wav_evidence, validate_final,
)
from silent_host_evidence import EvidenceError


def _bytes(value):
    return (json.dumps(value, indent=2, allow_nan=False) + "\n").encode()


def _hash(data):
    return hashlib.sha256(data).hexdigest()


def _round(value):
    return int(value + 0.5)


def _wav(rate, channels, frames, sample=0.125):
    payload = struct.pack("<f", sample) * (frames * channels)
    return struct.pack("<4sI8sIHHIIHH4sII4sI", b"RIFF", 48 + len(payload), b"WAVEfmt ",
                       16, 3, channels, rate, rate * channels * 4, channels * 4, 32,
                       b"fact", 4, frames, b"data", len(payload)) + payload


class FrozenFixture:
    def __init__(self, root, baseline):
        self.baseline_data = baseline
        self.session_id = baseline["session_id"]
        self.source_graph_ref = baseline["source_graph_refs"][0]
        self.timing_grid_ref = baseline["runtime_state"]["source_timing"]["confirmed_grid"]
        self.capture_ids = {capture["capture_id"] for capture in baseline["captures"]}
        self.path = root / "session.json"
        self.path.write_bytes(_bytes(baseline))
        self.digest = _hash(self.path.read_bytes())
        self.checks = 0

    def verify_unchanged(self):
        self.checks += 1
        if _hash(self.path.read_bytes()) != self.digest:
            raise EvidenceError("generated fixture baseline changed")


def publication(root, *, bars=2, rate=44100):
    """Model the existing App wire shape, not an alternative producer."""
    duration = {2: "two_bars", 8: "eight_bars", 16: "sixteen_bars"}[bars]
    version = 4 if bars == 2 else 3
    boundary = f"runtime_master_bar_window_v{version}"
    graph = {"source_id": "generated-source", "graph_version": "V1", "graph_hash": "a" * 64,
             "storage_mode": "External", "external_path": "source-graph.json",
             "embedded_graph": None, "provenance": {"analysis_seed": 23}}
    timing = {"source_id": "generated-source", "hypothesis_id": "generated-grid",
              "confirmed_by_action": 1, "confirmed_at": 100}
    baseline = {
        "session_version": "V1", "session_id": "generated-session", "created_at": "generated",
        "updated_at": "generated", "app_version": "test", "source_refs": [],
        "source_graph_refs": [graph], "captures": [
            {"capture_id": "cap-01", "lineage_capture_refs": [],
             "storage_path": "captures/cap-01.wav", "audio_identity": {"sha256": "b" * 64}},
        ], "snapshots": [], "ghost_state": {}, "notes": None, "export_receipts": [],
        "runtime_state": {
            "transport": {"is_playing": True, "position_beats": 16.0, "current_scene": "scene-1"},
            "scene_state": {"active_scene": "scene-1"},
            "source_timing": {"confirmed_bpm": 132.0, "confirmed_grid": timing},
            "lane_state": {"w30": {"last_capture": "cap-01"}}, "mixer_state": {"master_level": 0.8},
        },
        "action_log": {"replay_policy": "DeterministicPreferred", "actions": [
            {"id": 1, "command": "SourceTimingConfirmGrid", "status": "Committed"},
        ], "commit_records": [
            {"action_id": 1, "committed_at": 100, "commit_sequence": 1,
             "boundary": {"kind": "Immediate", "beat_index": 0, "bar_index": 1,
                          "phrase_index": 1, "scene_id": "scene-1"}},
        ]},
    }
    fixture = FrozenFixture(root, baseline)
    case = SimpleNamespace(bars=bars, session_path=root / f"run-{bars}-session.json",
                           destination_path=root / f"run-{bars}.wav",
                           proof_path=root / f"run-{bars}.wav.riotbox.json",
                           observer_path=root / f"run-{bars}.observer.ndjson")
    frames = _round(rate * 60.0 * bars * 4 / 132.0)
    span = 132.0 / 60.0 / rate
    wav_data = _wav(rate, 2, frames)
    case.destination_path.write_bytes(wav_data)
    plan = {
        "action_id": 2, "session_id": fixture.session_id, "session_pre_capture_sha256": "c" * 64,
        "duration_beats": bars * 4, "frame_count": frames, "sample_rate_hz": rate,
        "channel_count": 2, "source_graph_ref": {key: graph[key] for key in
                                                ("source_id", "graph_version", "graph_hash")},
        "timing_grid_ref": timing, "source_capture_refs": ["cap-01"], "lineage_capture_refs": [],
    }
    marker = {
        **copy.deepcopy(plan), "event": "live_master_recording_window_stopped",
        "schema": "riotbox.user_session_observer.v1",
        "observation_schema": "riotbox.live_master_recording_window_stopped.v1",
        "pid": 1234, "clock_domain": "linux_clock_monotonic",
        "window_complete_monotonic_ns": 100, "runtime_stopped_monotonic_ns": 101,
        "window_complete": True, "runtime_stopped": True, "fault_count": 0,
        "opt_in": True, "capture_context": "non_interactive_real_audio_cli",
        "realtime_callback_io": False, "raw_audio_recording": False, "live_master_recording": True,
        "duration_bars": bars, "requested_start_position_beats": 20.0,
    }
    end = 20.0 + frames * span
    proof = {
        **copy.deepcopy(plan), "schema": f"riotbox.live_recording_runtime_master_bar_window.v{version}",
        "receipt_id": "export-receipt-a-0002", "scene_id": "scene-1", "duration": duration,
        "confirmed_bpm_micros": 132_000_000, "beats_per_bar": 4,
        "bar_grid_anchor_position_microbeats": 0,
        "beat_span_per_frame_nanobeats": _round(span * 1e9),
        "requested_start_position_microbeats": 20_000_000,
        "captured_start_position_microbeats": 20_000_000,
        "captured_end_position_microbeats": _round(end * 1e6),
        "start_alignment_error_frame_micros": 0,
        "duration_error_frame_micros": _round(abs(frames * span - bars * 4) / span * 1e6),
        "host": "generated-host", "device": "generated-owned-sink", "device_sample_format": "F32",
        "wav_sample_format": "ieee_float32", "callback_count": 1000,
        "max_callback_gap_micros": 3000, "callback_gap_over_threshold_count": 0,
        "callback_scratch_overflow_count": 0, "stream_error_count": 0,
        "transport_mismatch_count": 0, "tempo_mismatch_count": 0,
        "timing_window_mismatch_count": 0, "armed_callback_count": 1,
        "active_sample_count": frames * 2, "peak_amplitude_micros": 125000,
        "rms_amplitude_micros": 125000, "clip_count": 0,
        "wav_sha256": _hash(wav_data), "sample_payload_sha256": _hash(wav_data[56:]),
    }
    proof_data = _bytes(proof)
    case.proof_path.write_bytes(proof_data)
    duration_ms = (frames * 1000 + rate // 2) // rate
    gates = [{"gate_id": gate, "status": "passed", "artifact_roles": [], "summary": "generated"}
             for gate in ("live_recording_runtime_master_capture", "live_recording_wav_readback",
                          "live_recording_bar_window_alignment")]
    receipt = {
        "receipt_id": "export-receipt-a-0002", "created_by_action": 2, "created_at": 200,
        "export_scope": "live_recording", "export_role": "live_recording_capture",
        "pack_id": f"live-recording-runtime-master-bar-window-v{version}",
        "export_boundary": "live_recording_" + boundary, "live_recording_duration": duration,
        "artifact_path": str(case.destination_path), "proof_path": str(case.proof_path),
        "manifest_path": str(case.proof_path), "export_hash": _hash(wav_data),
        "normalized_manifest_hash": _hash(proof_data), "readiness_status": "reproducible",
        "unsupported_scopes": [], "arrangement_placement_refs": [], "daw_tempo_map_ref": None,
        "qa_gates": gates, "artifact_set": [
            {"role": "live_recording_capture", "media_type": "audio_wav",
             "location": {"kind": "local_path", "path": str(case.destination_path)},
             "sha256": _hash(wav_data), **{key: copy.deepcopy(plan[key]) for key in
                  ("source_graph_ref", "timing_grid_ref", "source_capture_refs", "lineage_capture_refs")},
             "audio_metrics": {"peak_amplitude_micros": 125000, "rms_amplitude_micros": 125000,
                               "total_frame_count": frames},
             "sample_rate_hz": rate, "channel_count": 2, "duration_ms": duration_ms},
            {"role": "product_export_proof", "media_type": "json",
             "location": {"kind": "local_path", "path": str(case.proof_path)},
             "sha256": _hash(proof_data)},
        ], "live_recording_host_audio_refs": [{
            "host": proof["host"], "device": proof["device"], "recording_duration_ms": duration_ms,
            "callback_gap_summary": {"max_gap_ms": 3, "over_threshold_count": 0},
            "stream_error_summary": {"error_count": 0, "last_error": None},
            "timing_window": {key: proof[key] for key in TIMING_FIELDS},
        }],
    }
    session = copy.deepcopy(baseline)
    session["runtime_state"]["transport"]["position_beats"] = end
    session["action_log"]["actions"].append({
        "id": 2, "actor": "User", "command": "ExportLiveRecording", "status": "Committed",
        "quantization": "Immediate", "requested_at": 150, "committed_at": 200,
        "result": {"accepted": True, "summary": "generated recording"},
        "target": {"scope": "Session", "scene_id": "scene-1"},
        "params": {"LiveRecordingExport": {
            "export_scope": "live_recording", "export_role": "live_recording_capture",
            "boundary": boundary, "duration": duration, "include_manifest": True,
            "destination_kind": "local_file_path", "destination_path": str(case.destination_path),
            "receipt_id": None,
        }},
    })
    session["action_log"]["commit_records"].append({
        "action_id": 2, "committed_at": 200, "commit_sequence": 1,
        "boundary": {"kind": "Immediate", "beat_index": int(end), "bar_index": int(end // 4) + 1,
                     "phrase_index": int(end // 16) + 1, "scene_id": "scene-1"},
        "mc202_source_phrase_plan": None,
    })
    session["export_receipts"].append(receipt)
    case.session_path.write_bytes(_bytes(session))
    summary = {
        "mode": "live_master_recording_execute", "status": "ready", "ready": True,
        "writes_files": True, "mutates_session": True, "runtime_stopped": True,
        "observer_events": True, "observer_status": "recorded", "readiness_blockers": [],
        "session_path": str(case.session_path), "destination_path": str(case.destination_path),
        "proof_path": str(case.proof_path), "boundary": boundary,
        "receipt_boundary": "live_recording." + boundary,
        "duration_bars": bars, "duration_beats": bars * 4, "beats_per_bar": 4,
        "bar_grid_anchor_beat_cursor": 0, "requested_start_position_beats": 20.0,
        "captured_start_position_beats": 20.0, "captured_end_position_beats": end,
        "sample_rate_hz": rate, "channel_count": 2, "frame_count": frames,
        "host": proof["host"], "device": proof["device"],
        "receipt": {**{key: copy.deepcopy(receipt[key]) for key in
                        ("receipt_id", "pack_id", "export_scope", "export_role", "qa_gates")},
                    "sha256": _hash(wav_data), "proof_sha256": _hash(proof_data),
                    "host_audio_readiness": {"status": "ready", "blockers": []}},
    }
    return summary, marker, case, fixture


class ArtifactTests(unittest.TestCase):
    def fixture(self, **kwargs):
        temp = tempfile.TemporaryDirectory(prefix="riotbox-1578-artifact-unit-")
        self.addCleanup(temp.cleanup)
        return publication(Path(temp.name), **kwargs)

    def test_actual_2_8_16_bar_geometry_at_both_generated_rates(self):
        for bars in (2, 8, 16):
            for rate in (44100, 48000):
                with self.subTest(bars=bars, rate=rate):
                    summary, marker, case, fixture = self.fixture(bars=bars, rate=rate)
                    evidence = validate_final(summary, marker, case, fixture)
                    self.assertEqual(evidence["frame_count"], marker["frame_count"])
                    self.assertEqual(evidence["proof_sha256"], _hash(case.proof_path.read_bytes()))
                    self.assertEqual(evidence["receipt_id"], "export-receipt-a-0002")
                    self.assertTrue(evidence["baseline_unchanged"])
                    self.assertFalse(evidence["quality_proof"])
                    self.assertEqual(fixture.checks, 2)

    def test_final_proof_uses_confirmed_nonzero_bar_phase_not_absolute_zero(self):
        summary, marker, case, fixture = self.fixture()
        measured = validate_final(summary, marker, case, fixture)
        proof = json.loads(case.proof_path.read_bytes())
        proof["bar_grid_anchor_position_microbeats"] = 3_000_000
        summary["bar_grid_anchor_beat_cursor"] = 3
        for prefix in ("requested_start", "captured_start", "captured_end"):
            proof[prefix + "_position_microbeats"] += 3_000_000
            summary[prefix + "_position_beats"] += 3.0
        marker["requested_start_position_beats"] = 23.0
        _check_proof(proof, summary, marker, fixture, 4, "two_bars", measured)
        # Shift the complete window by one further beat: lengths still agree,
        # but start 24 is not a bar in the confirmed beat-3 phase.
        for prefix in ("requested_start", "captured_start", "captured_end"):
            proof[prefix + "_position_microbeats"] += 1_000_000
            summary[prefix + "_position_beats"] += 1.0
        marker["requested_start_position_beats"] = 24.0
        with self.assertRaisesRegex(EvidenceError, "timing window"):
            _check_proof(proof, summary, marker, fixture, 4, "two_bars", measured)

    def test_final_ready_observer_identity_and_blocked_states_are_required(self):
        summary, marker, case, fixture = self.fixture()
        changes = (("ready", 1), ("status", "blocked"), ("runtime_stopped", False),
                   ("observer_events", False), ("observer_status", "write_failed"),
                   ("observer_status", "not_requested"), ("readiness_blockers", ["fault"]),
                   ("session_path", "/unowned/session.json"), ("destination_path", "/unowned.wav"),
                   ("proof_path", "/unowned.json"), ("duration_bars", 8),
                   ("boundary", "runtime_master_bar_window_v2"), ("frame_count", True),
                   ("failure_reason", "retained"), ("observer_failure_reason", "retained"))
        for key, value in changes:
            with self.subTest(key=key, value=value), self.assertRaises(EvidenceError):
                changed = copy.deepcopy(summary)
                changed[key] = value
                validate_final(changed, marker, case, fixture)

    def test_exact_baseline_and_file_ownership_reject_changed_or_missing_files(self):
        summary, marker, case, fixture = self.fixture()
        fixture.path.write_bytes(b"{}")
        with self.assertRaisesRegex(EvidenceError, "baseline changed"):
            validate_final(summary, marker, case, fixture)
        fixture.path.write_bytes(_bytes(fixture.baseline_data))
        case.proof_path.unlink()
        with self.assertRaisesRegex(EvidenceError, "cannot read exact"):
            validate_final(summary, marker, case, fixture)

    def test_symlinks_hardlinks_fifo_and_file_budget_are_not_audio_inputs(self):
        summary, marker, case, fixture = self.fixture()
        for target in (case.session_path, case.proof_path, case.destination_path):
            data = target.read_bytes()
            target.unlink()
            target.symlink_to(fixture.path)
            with self.subTest(path=target), self.assertRaises(EvidenceError):
                validate_final(summary, marker, case, fixture)
            target.unlink()
            target.write_bytes(data)
        alias = case.session_path.parent / "aliased.json"
        alias.hardlink_to(case.session_path)
        with self.assertRaisesRegex(EvidenceError, "aliased"):
            _read_file(case.session_path, 16 * 1024 * 1024)
        alias.unlink()
        with self.assertRaisesRegex(EvidenceError, "budget"):
            _read_file(case.session_path, 1)
        import os
        pipe = case.session_path.parent / "generated.fifo"
        os.mkfifo(pipe)
        with self.assertRaisesRegex(EvidenceError, "unowned, aliased"):
            _read_file(pipe, 100)

    def test_raw_proof_bytes_are_hashed_not_python_reserialized(self):
        summary, marker, case, fixture = self.fixture()
        proof = json.loads(case.proof_path.read_bytes())
        case.proof_path.write_bytes(json.dumps(proof, separators=(",", ":")).encode())
        with self.assertRaisesRegex(ArtifactEvidenceError, "normalized_manifest_hash") as caught:
            validate_final(summary, marker, case, fixture)
        self.assertIn("wav_sha256", caught.exception.artifact_evidence)

    def test_more_armed_than_capturing_callbacks_is_not_a_new_failure_gate(self):
        summary, marker, case, fixture = self.fixture()
        proof = json.loads(case.proof_path.read_bytes())
        # Waiting for the next bar legitimately contributes armed-only callbacks.
        proof["armed_callback_count"] = proof["callback_count"] + 100
        proof_data = _bytes(proof)
        case.proof_path.write_bytes(proof_data)
        session = json.loads(case.session_path.read_bytes())
        receipt = session["export_receipts"][-1]
        receipt["normalized_manifest_hash"] = _hash(proof_data)
        receipt["artifact_set"][1]["sha256"] = _hash(proof_data)
        case.session_path.write_bytes(_bytes(session))
        summary["receipt"]["proof_sha256"] = _hash(proof_data)
        validate_final(summary, marker, case, fixture)

    def test_proof_identity_faults_and_metrics_cannot_be_forged(self):
        summary, marker, case, fixture = self.fixture()
        original = case.proof_path.read_bytes()
        changes = (("session_id", "other"), ("session_pre_capture_sha256", "d" * 64),
                   ("action_id", 3), ("source_capture_refs", []), ("lineage_capture_refs", ["cap-01"]),
                   ("duration", "eight_bars"), ("schema", "old"), ("confirmed_bpm_micros", 120000000),
                   ("active_sample_count", 1), ("peak_amplitude_micros", 999),
                   ("rms_amplitude_micros", 999), ("callback_count", 0),
                   ("armed_callback_count", 0), ("transport_mismatch_count", 1),
                   ("tempo_mismatch_count", 1), ("timing_window_mismatch_count", 1),
                   ("callback_scratch_overflow_count", 1), ("stream_error_count", 1),
                   ("callback_gap_over_threshold_count", 1), ("clip_count", 1),
                   ("duration_error_frame_micros", 500002), ("beat_span_per_frame_nanobeats", 1),
                   ("captured_end_position_microbeats", 20_000_000), ("wav_sha256", "d" * 64),
                   ("sample_payload_sha256", "d" * 64), ("device", ""))
        for key, value in changes:
            with self.subTest(key=key), self.assertRaises(EvidenceError):
                proof = json.loads(original)
                proof[key] = value
                case.proof_path.write_bytes(_bytes(proof))
                validate_final(summary, marker, case, fixture)

    def test_action_commit_receipt_and_prior_session_state_are_correlated(self):
        summary, marker, case, fixture = self.fixture()
        original = case.session_path.read_bytes()
        mutations = [
            lambda s: s["action_log"]["actions"][0].update({"status": "Rejected"}),
            lambda s: s["action_log"]["actions"].append(s["action_log"]["actions"][-1]),
            lambda s: s["action_log"]["actions"][-1].update({"command": "TransportPlay"}),
            lambda s: s["action_log"]["actions"][-1].update({"status": "Queued"}),
            lambda s: s["action_log"]["actions"][-1]["result"].update({"accepted": False}),
            lambda s: s["action_log"]["actions"][-1]["params"]["LiveRecordingExport"].update({"duration": "eight_bars"}),
            lambda s: s["action_log"]["commit_records"].pop(),
            lambda s: s["action_log"]["commit_records"][-1].update({"commit_sequence": 2}),
            lambda s: s["action_log"]["commit_records"][-1].update({"action_id": 1}),
            lambda s: s["action_log"]["commit_records"][-1].update({"committed_at": 199}),
            lambda s: s["captures"][0].update({"storage_path": "other.wav"}),
            lambda s: s["source_graph_refs"][0].update({"external_path": "other.json"}),
            lambda s: s["runtime_state"]["mixer_state"].update({"master_level": 0.9}),
            lambda s: s["runtime_state"]["transport"].update({"is_playing": False}),
            lambda s: s["runtime_state"]["transport"].update({"position_beats": 0.0}),
            lambda s: s["export_receipts"][-1].update({"created_by_action": 1}),
            lambda s: s["export_receipts"][-1].update({"export_hash": "0" * 64}),
            lambda s: s["export_receipts"][-1]["qa_gates"][0].update({"status": "deferred"}),
            lambda s: s["export_receipts"][-1]["qa_gates"].append(s["export_receipts"][-1]["qa_gates"][0]),
            lambda s: s["export_receipts"][-1]["artifact_set"][0].update({"channel_count": 1}),
            lambda s: s["export_receipts"][-1]["artifact_set"][0]["audio_metrics"].update({"total_frame_count": 1}),
            lambda s: s["export_receipts"][-1]["live_recording_host_audio_refs"][0]["stream_error_summary"].update({"error_count": 1}),
            lambda s: s["export_receipts"][-1]["live_recording_host_audio_refs"][0]["callback_gap_summary"].update({"over_threshold_count": 1}),
            lambda s: s["export_receipts"][-1].update({"unsupported_scopes": ["live_recording"]}),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(mutation=index), self.assertRaises(EvidenceError):
                session = json.loads(original)
                mutate(session)
                case.session_path.write_bytes(_bytes(session))
                validate_final(summary, marker, case, fixture)

    def test_wav_format_lengths_finiteness_activity_and_clipping_are_independent(self):
        good = _wav(48000, 2, 4)
        evidence = _wav_evidence(good, 48000, 2, 4)
        self.assertEqual(evidence["active_sample_count"], 8)
        for offset in (4, 16, 20, 22, 24, 28, 32, 34, 36, 40, 44, 48, 52):
            changed = bytearray(good)
            changed[offset] ^= 1
            with self.subTest(offset=offset), self.assertRaises(EvidenceError):
                _wav_evidence(bytes(changed), 48000, 2, 4)
        for changed in (good[:40], good[:-1], good + b"x"):
            with self.assertRaises(EvidenceError):
                _wav_evidence(changed, 48000, 2, 4)
        for sample in (0.0, 0.00001, 1.0, -1.0, float("nan"), float("inf")):
            with self.subTest(sample=sample), self.assertRaises(ArtifactEvidenceError) as caught:
                _wav_evidence(_wav(48000, 2, 4, sample), 48000, 2, 4)
            self.assertIn("active_sample_count", caught.exception.artifact_evidence)

    def test_rejected_clipping_retains_actual_rms_and_overflow_is_unavailable(self):
        for sample in (1.0, -1.0):
            with self.subTest(sample=sample), self.assertRaises(ArtifactEvidenceError) as caught:
                _wav_evidence(_wav(48000, 2, 4, sample), 48000, 2, 4)
            evidence = caught.exception.artifact_evidence
            self.assertEqual(evidence["rms_amplitude_micros"], 1_000_000)
            self.assertEqual(evidence["clip_count"], 8)
        for sample in (3.4e38, float("nan"), float("inf")):
            with self.subTest(sample=sample), self.assertRaises(ArtifactEvidenceError) as caught:
                _wav_evidence(_wav(48000, 2, 4, sample), 48000, 2, 4)
            evidence = caught.exception.artifact_evidence
            self.assertIsNone(evidence["rms_amplitude_micros"])
            json.dumps(evidence, allow_nan=False)


if __name__ == "__main__":
    unittest.main()
