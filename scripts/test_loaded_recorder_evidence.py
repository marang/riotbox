"""Generated observer/contract controls; no host or source access."""

import copy
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest

from loaded_recorder_evidence import (
    EvidenceError, load_json_bytes, load_protocol, read_observer, validate_marker,
    verify_loaded_geometry,
)


def evidence_fixture(directory):
    case = SimpleNamespace(bars=2, session_path=directory / "run-session.json",
                           observer_path=directory / "run.observer.ndjson")
    fixture = SimpleNamespace(
        session_id="generated-only", source_graph_ref={"source_id": "source-1",
            "graph_version": "V1", "graph_hash": "a" * 64, "storage_mode": "External"},
        timing_grid_ref=None, capture_ids=("capture-1",),
        baseline_data={"action_log": {"actions": [{"id": 3}]},
            "runtime_state": {"lane_state": {"w30": {"last_capture": "capture-1"}}},
            "captures": [{"capture_id": "capture-1", "lineage_capture_refs": []}]},
    )
    marker = {"event": "live_master_recording_window_stopped",
        "schema": "riotbox.user_session_observer.v1",
        "observation_schema": "riotbox.live_master_recording_window_stopped.v1",
        "pid": 1234, "clock_domain": "linux_clock_monotonic", "opt_in": True,
        "capture_context": "non_interactive_real_audio_cli", "realtime_callback_io": False,
        "raw_audio_recording": False, "live_master_recording": True,
        "window_complete": True, "runtime_stopped": True, "fault_count": 0,
        "session_id": "generated-only", "duration_bars": 2, "duration_beats": 8,
        "channel_count": 2, "window_complete_monotonic_ns": 10,
        "runtime_stopped_monotonic_ns": 11, "frame_count": 160364, "sample_rate_hz": 44100,
        "action_id": 4, "session_pre_capture_sha256": "b" * 64,
        "requested_start_position_beats": 4.0,
        "source_graph_ref": {key: fixture.source_graph_ref[key]
                             for key in ("source_id", "graph_version", "graph_hash")},
        "timing_grid_ref": None, "source_capture_refs": ["capture-1"],
        "lineage_capture_refs": []}
    terminal = {key: marker[key] for key in ("schema", "opt_in", "capture_context",
        "realtime_callback_io", "raw_audio_recording", "live_master_recording")}
    terminal.update(event="live_master_recording_execute", summary={
        "ready": True, "runtime_stopped": True, "session_path": str(case.session_path)})
    return fixture, case, marker, terminal


class LoadedEvidenceTests(unittest.TestCase):
    def test_pinned_contract_keeps_containment_budgets_and_closed_geometry(self):
        protocol = load_protocol()
        self.assertEqual(protocol["window_bars"], [2, 8, 16])
        self.assertFalse(protocol["host_execution_allowed"])
        self.assertNotIn("run_seconds", protocol)

    def test_json_rejects_duplicate_nonfinite_scalar_and_overflow(self):
        for data in (b'{"a":1,"a":2}', b'{"a":NaN}', b'{"a":1e999}', b'[]', b'{'):
            with self.subTest(data=data), self.assertRaises(EvidenceError):
                load_json_bytes(data)

    def test_marker_is_not_final_success_and_prefix_may_be_incomplete(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture, case, marker, terminal = evidence_fixture(Path(directory))
            self.assertIsNone(read_observer(case.observer_path, 1234, case, fixture))
            case.observer_path.write_text(json.dumps(marker) + "\n" + '{"event":')
            self.assertEqual(read_observer(case.observer_path, 1234, case, fixture), marker)
            with self.assertRaises(EvidenceError):
                read_observer(case.observer_path, 1234, case, fixture, complete=True)
            case.observer_path.write_text(json.dumps(marker) + "\n" + json.dumps(terminal) + "\n")
            self.assertEqual(read_observer(case.observer_path, 1234, case, fixture, complete=True), marker)

    def test_wrong_identity_clock_geometry_fault_or_verdict_never_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture, case, marker, _ = evidence_fixture(Path(directory))
            for key, value in (("pid", 999), ("pid", True), ("window_complete_monotonic_ns", None),
                ("window_complete_monotonic_ns", 12), ("window_complete_monotonic_ns", False),
                ("clock_domain", "wall"), ("fault_count", 1), ("runtime_stopped", 1),
                ("action_id", 3), ("sample_rate_hz", 96000), ("channel_count", 1),
                ("session_id", "foreign"), ("session_pre_capture_sha256", "bad"),
                ("requested_start_position_beats", -1.0), ("source_capture_refs", ["foreign"]),
                ("lineage_capture_refs", ["capture-1"]), ("ready", True)):
                changed = copy.deepcopy(marker)
                changed[key] = value
                with self.subTest(key=key, value=value), self.assertRaises(EvidenceError):
                    validate_marker(changed, 1234, case, fixture)

    def test_confirmed_source_bar_phase_is_not_assumed_to_start_at_zero(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture, case, marker, _ = evidence_fixture(Path(directory))
            # Actual fresh generated V1 preparation requests beat 23 for its
            # confirmed beat-3 bar phase. Final proof/Core own that alignment.
            marker["requested_start_position_beats"] = 23.0
            self.assertEqual(validate_marker(marker, 1234, case, fixture), marker)

    def test_missing_duplicate_failed_malformed_and_partial_final_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture, case, marker, terminal = evidence_fixture(Path(directory))
            failed = dict(terminal, event="live_master_recording_execute_failed")
            for records in ([], [marker], [marker, marker], [marker, terminal, terminal],
                            [marker, failed], [terminal]):
                case.observer_path.write_text("".join(json.dumps(item) + "\n" for item in records))
                with self.subTest(records=len(records)), self.assertRaises(EvidenceError):
                    read_observer(case.observer_path, 1234, case, fixture, complete=True)

    def test_marker_symlink_and_oversized_observer_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            fixture, case, marker, _ = evidence_fixture(Path(directory))
            target = Path(directory) / "target"
            target.write_text(json.dumps(marker) + "\n")
            case.observer_path.symlink_to(target)
            with self.assertRaises(EvidenceError):
                read_observer(case.observer_path, 1234, case, fixture)
            case.observer_path.unlink()
            case.observer_path.write_bytes(b"x" * 262145)
            with self.assertRaises(EvidenceError):
                read_observer(case.observer_path, 1234, case, fixture)

    def test_loaded_geometry_is_not_silent_interval_evidence(self):
        report = {"active_pcm": {"channels": 2, "sample_rate": 44100}}
        summary = {"channel_count": 2, "sample_rate_hz": 44100}
        verify_loaded_geometry(object(), report, summary)
        for changed in ({"channel_count": 1, "sample_rate_hz": 44100},
                        {"channel_count": 2, "sample_rate_hz": 48000}):
            with self.assertRaises(EvidenceError):
                verify_loaded_geometry(object(), report, changed)


if __name__ == "__main__":
    unittest.main()
