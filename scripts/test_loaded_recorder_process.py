"""Generated supervision checks: fake processes/time/metadata, never host/audio."""
from dataclasses import dataclass
import gzip
import json
from pathlib import Path
import tempfile
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

import loaded_recorder_process as subject
import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError
from silent_host_routes import find_sink
from test_silent_host_routes import fixture as route_fixture


NS = 1_000_000_000


class Clock:
    def __init__(self):
        self.value = 100 * NS

    def read(self, domain):
        if domain != subject.time.CLOCK_MONOTONIC:
            raise AssertionError("explicit CLOCK_MONOTONIC is required")
        return self.value

    def sleep(self, seconds):
        self.value += round(seconds * NS)


class Fixture:
    def __init__(self, fail_after=False):
        self.calls = 0
        self.fail_after = fail_after

    def verify_unchanged(self):
        self.calls += 1
        if self.fail_after and self.calls > 1:
            raise EvidenceError("generated fixture changed")


@dataclass
class Scenario:
    complete: float = 100.05
    stopped: float = 100.06
    visible: float | None = 100.12
    exit_at: float | None = 100.3
    query_seconds: float = 0.01
    absence_call: int = 2
    bad_marker: str | None = None
    summary: bytes = b'{"ready":true,"runtime_stopped":true,"observer_status":"recorded"}'
    status: int = 0
    foreign_call: int | None = None
    changed_route: bool = False
    replaced_absent: bool = False
    changed_default: bool = False
    group_cleanup: bool = True
    stream_cleanup: bool = True
    geometry_error: bool = False


class FakeProcess:
    def __init__(self, test, command, stdout, stderr, environment, watchdog, grace):
        self.test = test
        self.cleanup_verified = False
        self.active = False
        self.command = command
        self.watchdog, self.grace = watchdog, grace
        self.stdout, self.stderr = stdout, stderr
        test.process = self

    def __enter__(self):
        self.active = True
        self.stdout.write_bytes(self.test.scenario.summary)
        self.stderr.write_bytes(b"")
        self.test.case.observer_path.write_bytes(b"")
        return self

    @property
    def child_pid(self):
        return 42

    def exited(self):
        end = self.test.scenario.exit_at
        return end is not None and self.test.clock.value >= round(end * NS)

    def finish(self):
        self.cleanup_verified = self.test.scenario.group_cleanup
        self.active = False
        return self.test.scenario.status

    def __exit__(self, *_args):
        self.cleanup_verified = self.test.scenario.group_cleanup
        self.active = False


class Host:
    def __init__(self, test):
        self.test = test
        self.calls = 0

    def default_state(self):
        changed = self.test.scenario.changed_default and self.calls >= 2
        return {"default_sink_name": "changed" if changed else "untouched", "mute": False,
                "volume": {"left": 1, "right": 1}}

    def snapshot(self):
        self.test.clock.sleep(self.test.scenario.query_seconds)
        if not self.test.process.active:
            if not self.test.scenario.stream_cleanup:
                raise EvidenceError("generated residual stream")
            return route_fixture()[:1]
        self.calls += 1
        data = route_fixture()
        data[1]["info"]["props"]["application.process.id"] = 42
        if self.calls == self.test.scenario.foreign_call:
            data[-1]["info"]["output-node-id"] = data[0]["id"]
            return data
        if self.test.scenario.changed_route and self.calls >= 2:
            data[2]["info"]["props"]["object.serial"] += 1
        if self.calls >= self.test.scenario.absence_call:
            if self.test.scenario.replaced_absent:
                data[1]["info"]["props"]["application.process.id"] = 999
                data[2]["info"]["props"]["object.serial"] += 1
                return data[:3]
            return data[:1]
        return data


class Diagnostics:
    def __init__(self, test):
        self.test = test
        self.command = None
        self.finished = []

    def wrap(self, command, prefix, environment):
        self.command = list(command)
        return command

    def finish(self, prefix, pid, *, complete):
        self.finished.append((pid, complete))
        result = {"pid": pid, "complete": complete, "generated_only": True}
        prefix.with_suffix(".alsa.json").write_text(json.dumps(result))
        return result


def strict_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise EvidenceError("duplicate JSON key")
            result[key] = value
        return result
    try:
        value = json.loads(data, object_pairs_hook=pairs,
                           parse_constant=lambda _value: (_ for _ in ()).throw(EvidenceError("nonfinite JSON")))
    except (ValueError, UnicodeError) as error:
        raise EvidenceError("invalid final JSON") from error
    if not isinstance(value, dict):
        raise EvidenceError("final JSON must be an object")
    return value


class ProcessTests(unittest.TestCase):
    def setUp(self):
        owner = tempfile.TemporaryDirectory(prefix="riotbox-generated-loaded-process-")
        self.addCleanup(owner.cleanup)
        self.root = Path(owner.name)
        self.case = SimpleNamespace(bars=2, session_path=self.root / "run-session.json",
                                    destination_path=self.root / "master.wav",
                                    observer_path=self.root / "observer.ndjson")
        self.case.session_path.write_bytes(b"generated Session control")
        self.prefix = self.root / "run-2bars"
        self.clock = Clock()
        self.scenario = Scenario()
        self.fixture = Fixture()
        self.host = Host(self)
        self.sink = find_sink(route_fixture(), "owned-test")
        self.diagnostics = Diagnostics(self)
        self.protocol = {"startup_route_deadline_seconds": 5, "process_timeout_seconds": 75,
                         "teardown_deadline_seconds": 5, "kill_grace_seconds": 2,
                         "route_poll_interval_ms": 100}
        self.process = None
        self.complete_reads = []
        # These fake evidence seams check delegation/rejection propagation, not
        # the root-owned proof/observer validator's separate semantic coverage.
        evidence = ModuleType("loaded_recorder_evidence")
        evidence.read_observer = self.read_observer
        evidence.load_json_bytes = strict_json
        evidence.validate_final = self.validate_final
        evidence.verify_loaded_geometry = self.verify_geometry
        self.addCleanup(patch.stopall)
        patch.dict("sys.modules", {"loaded_recorder_evidence": evidence}).start()
        patch.object(subject, "ManagedProcess", side_effect=lambda *args: FakeProcess(self, *args)).start()
        patch.object(subject.time, "clock_gettime_ns", side_effect=self.clock.read).start()
        patch.object(subject.time, "monotonic", side_effect=lambda: self.clock.value / NS).start()
        patch.object(subject.time, "sleep", side_effect=self.clock.sleep).start()

    def read_observer(self, path, pid, case, fixture, *, complete=False):
        self.complete_reads.append(complete)
        self.assertEqual((path, pid, case, fixture),
                         (self.case.observer_path, 42, self.case, self.fixture))
        visible = self.scenario.visible
        if visible is None or self.clock.value < round(visible * NS):
            if complete:
                raise EvidenceError("missing final marker")
            return None
        if self.scenario.bad_marker is not None:
            raise EvidenceError("generated " + self.scenario.bad_marker + " marker rejected")
        marker = {"pid": 42, "window_complete_monotonic_ns": round(self.scenario.complete * NS),
                  "runtime_stopped_monotonic_ns": round(self.scenario.stopped * NS)}
        self.case.observer_path.write_text(json.dumps(marker) + "\n")
        return marker

    def validate_final(self, summary, marker, case, fixture):
        self.assertIsNotNone(marker)
        self.assertTrue(self.complete_reads[-1])
        if (summary.get("ready") is not True or summary.get("runtime_stopped") is not True
                or summary.get("observer_status") != "recorded"):
            raise EvidenceError("blocked or unrecorded final result")
        return {"generated_artifact_binding": True}

    def verify_geometry(self, diagnostics, measurement, summary):
        self.assertIs(diagnostics, self.diagnostics)
        self.assertEqual(measurement["pid"], 42)
        if self.scenario.geometry_error:
            raise EvidenceError("generated diagnostic geometry mismatch")

    def run_case(self):
        return subject.run_recording(self.root / "riotbox-app", self.case, self.fixture,
                                     self.prefix, self.host, self.sink, self.protocol, {},
                                     diagnostics=self.diagnostics)

    def test_exact_cli_success_complete_observer_and_explicit_query_clock(self):
        result = self.run_case()
        self.assertEqual(self.diagnostics.command, [str(self.root / "riotbox-app"),
            "--live-master-recording-execute", "--session", str(self.case.session_path),
            "--live-recording-destination", str(self.case.destination_path),
            "--live-recording-bars", "2", "--observer", str(self.case.observer_path)])
        self.assertEqual((self.process.watchdog, self.process.grace), (75, 2))
        self.assertTrue(result["result"]["ready"])
        self.assertTrue(result["artifact_evidence"]["generated_artifact_binding"])
        self.assertEqual(self.fixture.calls, 2)
        self.assertEqual(self.diagnostics.finished, [(42, True)])
        self.assertGreater(result["route_observations"], 0)
        for key in ("stdout_sha256", "observer_sha256", "routes_sha256"):
            self.assertEqual(len(result[key]), 64)
        with gzip.open(self.prefix.with_suffix(".routes.ndjson.gz"), "rt") as data:
            rows = [json.loads(line) for line in data]
        for row in rows:
            if row["phase"] in ("observe", "teardown_query"):
                self.assertEqual(row["clock_domain"], "linux_clock_monotonic")
                self.assertLessEqual(row["query_start_monotonic_ns"], row["query_end_monotonic_ns"])

    def test_delayed_marker_can_resolve_only_safe_bounded_absence(self):
        self.scenario.visible = 102.0
        self.scenario.exit_at = 102.2
        result = self.run_case()
        self.assertTrue(result["result"]["ready"])
        self.assertGreaterEqual(self.clock.value, 102 * NS)

    def test_premature_absence_is_rejected_after_delayed_marker(self):
        self.scenario.complete, self.scenario.stopped = 100.5, 100.6
        self.scenario.visible, self.scenario.exit_at = 100.7, 101.0
        with self.assertRaisesRegex(EvidenceError, "before window completion"):
            self.run_case()
        self.assertEqual(self.fixture.calls, 2)
        self.assertEqual(self.diagnostics.finished, [(42, False)])
        self.assertTrue(self.prefix.with_suffix(".alsa.json").exists())

    def test_overlapping_first_absent_query_is_ambiguous_not_a_pass(self):
        self.scenario.complete, self.scenario.stopped = 100.115, 100.116
        with self.assertRaisesRegex(EvidenceError, "overlaps window completion"):
            self.run_case()

    def test_missing_marker_and_finalization_hang_do_not_reset_first_absence_budget(self):
        for marker in (None, 100.12):
            with self.subTest(marker=marker):
                # Separate owned test files for each independent fake attempt.
                self.prefix = self.root / ("missing" if marker is None else "hung")
                self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
                self.clock.value = 100 * NS
                self.host.calls = 0
                self.scenario.visible, self.scenario.exit_at = marker, None
                with self.assertRaises(operator.UnverifiedCleanup) as caught:
                    self.run_case()
                details = str(caught.exception) + " ".join(getattr(caught.exception, "__notes__", []))
                self.assertIn("deadline", details)
                self.assertLess(self.clock.value, 106 * NS)

    def test_duplicate_wrong_pid_wrong_plan_and_malformed_markers_propagate(self):
        for kind in ("duplicate", "wrong PID", "wrong plan", "malformed"):
            with self.subTest(kind=kind):
                self.prefix = self.root / kind.replace(" ", "-")
                self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
                self.clock.value, self.host.calls = 100 * NS, 0
                self.scenario.bad_marker = kind
                with self.assertRaisesRegex(EvidenceError, "marker rejected"):
                    self.run_case()

    def test_stale_and_future_clock_markers_never_authorize_drain(self):
        self.scenario.complete = 99.0
        with self.assertRaisesRegex(EvidenceError, "stale or future"):
            self.run_case()
        self.prefix = self.root / "future"
        self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
        self.clock.value, self.host.calls = 100 * NS, 0
        self.scenario.complete, self.scenario.stopped = 100.05, 120.0
        with self.assertRaisesRegex(EvidenceError, "stale or future"):
            self.run_case()

    def test_foreign_route_and_default_changes_are_never_pending(self):
        self.scenario.visible = 102.0
        self.scenario.foreign_call = 2
        with self.assertRaisesRegex(ValueError, "outgoing link"):
            self.run_case()
        self.assertLess(self.clock.value, 101 * NS)
        self.prefix = self.root / "default-changed"
        self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
        self.clock.value, self.host.calls = 100 * NS, 0
        self.scenario.foreign_call = None
        self.scenario.changed_default = True
        with self.assertRaises(operator.UnverifiedStreamCleanup) as caught:
            self.run_case()
        self.assertIn("default output", " ".join(caught.exception.__notes__))
        self.assertLess(self.clock.value, 101 * NS)

    def test_active_route_identity_replacement_is_rejected(self):
        self.scenario.absence_call = 100
        self.scenario.changed_route = True
        with self.assertRaisesRegex(EvidenceError, "node identity changed"):
            self.run_case()

    def test_nonzero_blocked_invalid_duplicate_or_oversized_final_json_fails(self):
        for index, (status, data) in enumerate([
            (3, Scenario.summary), (0, b'{"ready":false}'), (0, b"malformed"),
            (0, b'{"ready":true,"ready":false}'), (0, b"x" * 262145),
        ]):
            with self.subTest(index=index):
                self.prefix = self.root / f"bad-final-{index}"
                self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
                self.clock.value, self.host.calls = 100 * NS, 0
                self.scenario.status, self.scenario.summary = status, data
                with self.assertRaises(EvidenceError):
                    self.run_case()

    def test_failed_artifact_metrics_are_published_before_rejection(self):
        error = EvidenceError("generated post-measurement proof mismatch")
        error.artifact_evidence = {"frame_count": 12, "clip_count": 0, "wav_sha256": "a" * 64}
        with patch("loaded_recorder_evidence.validate_final", side_effect=error):
            with self.assertRaisesRegex(EvidenceError, "proof mismatch"):
                self.run_case()
        retained = json.loads(self.prefix.with_suffix(".artifacts.json").read_bytes())
        self.assertEqual(retained["result"], "failed")
        self.assertEqual(retained["evidence"], error.artifact_evidence)
        self.assertTrue(self.process.cleanup_verified)

    def test_postflight_analysis_follows_verified_cleanup_not_the_live_drain_budget(self):
        removed = []
        original = operator.require_removed

        def remove(*args, **kwargs):
            original(*args, **kwargs)
            removed.append(True)

        def analyze(summary, marker, case, fixture):
            self.assertTrue(removed)
            self.assertTrue(self.process.cleanup_verified)
            self.clock.sleep(10)  # Only independent analysis, never live runtime.
            return {"generated_analysis_after_cleanup": True}

        with patch.object(operator, "require_removed", side_effect=remove), \
                patch("loaded_recorder_evidence.validate_final", side_effect=analyze):
            result = self.run_case()
        self.assertTrue(result["artifact_evidence"]["generated_analysis_after_cleanup"])

    def test_unverified_streams_cannot_supply_postflight_artifacts(self):
        self.scenario.stream_cleanup = False
        with patch("loaded_recorder_evidence.validate_final") as analyze:
            with self.assertRaises(operator.UnverifiedStreamCleanup):
                self.run_case()
            analyze.assert_not_called()

    def test_occupied_artifact_evidence_is_preserved_and_cannot_pass(self):
        report = self.prefix.with_suffix(".artifacts.json")
        report.write_bytes(b"unowned sentinel")
        with self.assertRaises(FileExistsError):
            self.run_case()
        self.assertEqual(report.read_bytes(), b"unowned sentinel")

    def test_group_or_stream_cleanup_failure_stays_typed_and_fixture_is_checked(self):
        self.scenario.group_cleanup = False
        with self.assertRaises(operator.UnverifiedGroupCleanup):
            self.run_case()
        self.assertEqual(self.fixture.calls, 2)
        self.assertEqual(self.diagnostics.finished, [])
        self.prefix = self.root / "residual-stream"
        self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
        self.clock.value, self.host.calls = 100 * NS, 0
        self.scenario.group_cleanup, self.scenario.stream_cleanup = True, False
        self.fixture = Fixture(fail_after=True)
        with self.assertRaises(operator.UnverifiedStreamCleanup) as caught:
            self.run_case()
        self.assertIn("fixture preservation failed", " ".join(caught.exception.__notes__))
        self.assertEqual(self.diagnostics.finished, [(42, False)])

    def test_diagnostic_geometry_and_postprocess_fixture_mutation_fail_closed(self):
        self.scenario.geometry_error = True
        with self.assertRaisesRegex(EvidenceError, "diagnostic geometry mismatch"):
            self.run_case()
        self.prefix = self.root / "mutated-fixture"
        self.case.observer_path = self.prefix.with_suffix(".observer.ndjson")
        self.clock.value, self.host.calls = 100 * NS, 0
        self.scenario.geometry_error = False
        self.fixture = Fixture(fail_after=True)
        with self.assertRaisesRegex(EvidenceError, "fixture changed"):
            self.run_case()

    def test_no_diagnostics_or_unknown_duration_cannot_launch(self):
        with self.assertRaisesRegex(EvidenceError, "requires.*diagnostics"):
            subject.run_recording("unused", self.case, self.fixture, self.prefix, self.host,
                                  self.sink, self.protocol, {}, diagnostics=None)
        self.assertIsNone(self.process)
        self.case.bars = "2 --source arbitrary"
        with self.assertRaisesRegex(EvidenceError, "unsupported.*duration"):
            self.run_case()
        self.assertIsNone(self.process)

    def test_replaced_node_in_an_absent_route_is_not_a_deferred_state(self):
        self.scenario.replaced_absent = True
        self.scenario.visible = 102.0
        with self.assertRaisesRegex(EvidenceError, "node identity changed"):
            self.run_case()
        self.assertLess(self.clock.value, 101 * NS)

    def test_slow_first_absent_query_cannot_extend_the_pending_budget(self):
        original = self.host.snapshot
        def slow_snapshot():
            if self.host.calls == 1:
                self.clock.sleep(6)
            return original()
        self.host.snapshot = slow_snapshot
        self.scenario.visible = None
        self.scenario.exit_at = None
        with self.assertRaises(operator.UnverifiedCleanup) as caught:
            self.run_case()
        details = str(caught.exception) + " ".join(caught.exception.__notes__)
        self.assertIn("terminal teardown deadline", details)
        self.assertLess(self.host.calls, 3)

    def test_route_log_close_failure_preserves_typed_group_failure(self):
        original_open = gzip.open
        class FailingClose:
            def __init__(self, output):
                self.output = output
            def write(self, data):
                return self.output.write(data)
            def flush(self):
                return self.output.flush()
            def close(self):
                self.output.close()
                raise OSError("generated route close failure")
        self.scenario.group_cleanup = False
        with patch.object(operator.gzip, "open", side_effect=lambda *args, **kwargs:
                          FailingClose(original_open(*args, **kwargs))):
            with self.assertRaises(operator.UnverifiedGroupCleanup) as caught:
                self.run_case()
        self.assertIn("route close failure", " ".join(caught.exception.__notes__))
        self.assertEqual(self.fixture.calls, 2)


if __name__ == "__main__":
    unittest.main()
