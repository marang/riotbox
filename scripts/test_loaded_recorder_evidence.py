"""Generated observer/contract controls; no host or source access."""

import copy
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

import loaded_recorder_evidence as subject

from loaded_recorder_evidence import (
    EvidenceError, LoadedObservationVersion, PROTOCOL_SHA256, V2_PROTOCOL_SHA256,
    load_json_bytes, load_protocol, load_protocol_v2, read_observer, sha256,
    validate_marker, verify_loaded_geometry,
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

    def test_v1_and_v2_have_distinct_pinned_identity_and_complete_inheritance(self):
        self.assertEqual([(item.name, item.value) for item in LoadedObservationVersion],
                         [("V1", "v1"), ("V2", "v2")])
        root = Path(__file__).resolve().parents[1]
        self.assertEqual(PROTOCOL_SHA256,
                         "687bea31dfa903a299fc14b9e01567bf34b3cd4536220397f2cbb6f780e943ca")
        self.assertEqual(V2_PROTOCOL_SHA256,
                         "4cf9514cf33057d67e268f4ee6d234e083f4703d25a2f3d8537de2b3c99564ed")
        for version, expected_hash in ((1, PROTOCOL_SHA256), (2, V2_PROTOCOL_SHA256)):
            data = (root / f"docs/benchmarks/loaded_recorder_observation_v{version}.json").read_bytes()
            self.assertEqual(sha256(data), expected_hash)
        previous, current = load_protocol(), load_protocol_v2()
        expected = {**previous, "schema": "riotbox.loaded_recorder_observation.v2",
                    "schema_version": 2, "predecessor_protocol_sha256": PROTOCOL_SHA256,
                    "node_id_reuse_policy": "validated_distinct_client_is_node_absence",
                    "evidence_scope": "source_free_loaded_recorder_client_reuse_preparation"}
        self.assertEqual(current, expected)
        self.assertFalse(current["host_execution_allowed"])

    def generated_protocol_root(self, directory):
        root = Path(directory)
        destination = root / "docs/benchmarks"
        destination.mkdir(parents=True)
        repository = Path(__file__).resolve().parents[1] / "docs/benchmarks"
        for name in ("loaded_recorder_observation_v1.json", "loaded_recorder_observation_v2.json",
                     "generated_live_master_load_fixture_v1.json",
                     "silent_host_observation_v3.json", "silent_host_observation_v4.json"):
            (destination / name).write_bytes((repository / name).read_bytes())
        return root

    def test_mutated_or_missing_v2_and_predecessor_contracts_fail_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.generated_protocol_root(directory)
            for path in (root / "docs/benchmarks").iterdir():
                with self.subTest(contract=path.name):
                    original = path.read_bytes()
                    path.write_bytes(original + b"\n")
                    with self.assertRaises(EvidenceError):
                        load_protocol_v2(root)
                    path.unlink()
                    with self.assertRaises(FileNotFoundError):
                        load_protocol_v2(root)
                    path.write_bytes(original)

    def test_v2_inheritance_checks_every_field_and_exact_key_set_beyond_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = self.generated_protocol_root(directory)
            path = root / "docs/benchmarks/loaded_recorder_observation_v2.json"
            original = load_json_bytes(path.read_bytes())
            mutations = [(key, {**original, key: "changed-generated-value"}) for key in original]
            mutations += [("missing " + key, {name: value for name, value in original.items()
                                              if name != key}) for key in original]
            mutations.append(("extra key", {**original, "unreviewed_policy": False}))
            mutations.append(("integer is not bool", {**original, "schema_version": True}))
            for label, changed in mutations:
                with self.subTest(field=label):
                    data = json.dumps(changed).encode()
                    path.write_bytes(data)
                    # Deliberately repin only the test's generated bytes to prove
                    # semantic inheritance is independent of the byte-hash gate.
                    with patch.object(subject, "V2_PROTOCOL_SHA256", sha256(data)), \
                            self.assertRaisesRegex(EvidenceError, "complete predecessor inheritance"):
                        load_protocol_v2(root)

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
