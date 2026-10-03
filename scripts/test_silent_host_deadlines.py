"""Deterministic operator clock regressions; no process or device is launched."""

import json
import io
import tempfile
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from run_silent_host_probe import UnverifiedStreamCleanup, remove_owned_sink, require_removed, run_program
from silent_host_evidence import EvidenceError
from silent_host_routes import NodeIdentity, find_sink
from test_silent_host_evidence import PROTOCOL, transcript
from test_silent_host_routes import fixture


class DeadlineTests(unittest.TestCase):
    def run_generated(self, *, late_start=False, terminal_exit=66.0,
                      overlap_final_metadata=False):
        clock = SimpleNamespace(now=0.0)
        state = SimpleNamespace(closed=False)
        records = transcript()
        records[-1]["elapsed_ms"] = int(terminal_exit * 1000)
        snapshot = fixture()
        snapshot[1]["info"]["props"]["application.process.id"] = 1234

        def sleep(seconds):
            if not late_start and clock.now < 60:
                clock.now = 59.5 if overlap_final_metadata else 60.0
            else:
                clock.now += seconds

        class Process:
            child_pid = 1234
            cleanup_verified = False

            def __init__(self, command, stdout, stderr, *args):
                self.stdout, self.stderr = stdout, stderr

            def __enter__(self):
                data = (b"callback_count: 9\nstream_error_count: 0\n"
                        b"callback_scratch_overflow_count: 0\nstream_result: Ok\n"
                        if late_start else b"\n".join(json.dumps(r).encode() for r in records) + b"\n")
                self.stdout.write_bytes(data)
                self.stderr.write_bytes(b"")
                return self

            def exited(self):
                return clock.now >= (6.0 if late_start else terminal_exit)

            def finish(self):
                self.cleanup_verified = state.closed = True
                return 0

            def __exit__(self, *args):
                self.cleanup_verified = state.closed = True

        class Host:
            def snapshot(self):
                if state.closed:
                    if overlap_final_metadata:
                        clock.now += 2.0
                    return fixture()[:1]
                if late_start:
                    clock.now = 6.0  # Two legal metadata waits can cross the total deadline.
                    return snapshot
                if overlap_final_metadata and clock.now == 59.5:
                    clock.now = 62.0  # A legal query overlaps the final sample.
                return snapshot if clock.now < 60 else fixture()[:1]

        def read_records(path, *, complete=False):
            if clock.now < 60:
                return records[:1]
            return records if clock.now >= terminal_exit else records[:61]

        with tempfile.TemporaryDirectory(prefix="riotbox-deadline-fixture-") as directory:
            with patch("run_silent_host_probe.ManagedProcess", Process), \
                    patch("run_silent_host_probe.time", SimpleNamespace(monotonic=lambda: clock.now, sleep=sleep)), \
                    patch("run_silent_host_probe.read_transcript", read_records):
                return run_program(["generated-only"], Path(directory) / "run", Host(),
                                   find_sink(fixture(), "owned-test"), PROTOCOL, {},
                                   preflight=late_start)

    def test_first_valid_route_after_total_startup_deadline_is_rejected(self):
        with self.assertRaisesRegex(EvidenceError, "startup"):
            self.run_generated(late_start=True)

    def test_six_second_terminal_drain_cannot_use_the_process_watchdog_budget(self):
        with self.assertRaises(UnverifiedStreamCleanup) as captured:
            self.run_generated(terminal_exit=66.0)
        self.assertIn("terminal teardown deadline", " ".join(captured.exception.__notes__))

    def test_complete_natural_stop_inside_terminal_budget_is_admitted(self):
        result = self.run_generated(terminal_exit=61.0)
        self.assertEqual(result["result"]["sample_count"], 60)

    def test_late_final_sample_detection_does_not_extend_removal_budget(self):
        with self.assertRaises(UnverifiedStreamCleanup) as captured:
            self.run_generated(terminal_exit=64.0, overlap_final_metadata=True)
        self.assertIn("terminal teardown deadline", " ".join(captured.exception.__notes__))

    def test_overlapped_final_sample_can_still_complete_inside_original_budget(self):
        result = self.run_generated(terminal_exit=62.5, overlap_final_metadata=True)
        self.assertEqual(result["result"]["sample_count"], 60)

    def test_sink_absence_observed_after_total_deadline_does_not_pass(self):
        clock = SimpleNamespace(now=0.0)

        class Host:
            def text(self, command):
                assert command == ["pactl", "--format=text", "list", "short", "modules"]
                clock.now += 2.9
                return ""

            def snapshot(self):
                clock.now += 2.9
                return []

        with patch("silent_host_modules.time", SimpleNamespace(monotonic=lambda: clock.now)):
            with self.assertRaisesRegex(EvidenceError, "deadline|budget"):
                remove_owned_sink(Host(), "owned-test", None, PROTOCOL)

    def test_stream_absence_after_deadline_does_not_pass(self):
        clock = SimpleNamespace(now=0.0)

        class Host:
            def snapshot(self):
                clock.now = 5.0
                return fixture()[:1]

        with patch("run_silent_host_probe.time", SimpleNamespace(monotonic=lambda: clock.now)):
            with self.assertRaisesRegex(EvidenceError, "teardown deadline"):
                require_removed(Host(), find_sink(fixture(), "owned-test"), 1234,
                                PROTOCOL, io.StringIO(), deadline=5.0, node=NodeIdentity(30, 3000))

    def test_orphaned_target_node_is_not_absence_even_after_client_and_links_disappear(self):
        clock = SimpleNamespace(now=0.0)
        orphan = [item for item in fixture() if item["id"] not in {20, 60, 61}]

        def sleep(seconds):
            clock.now += seconds

        host = SimpleNamespace(snapshot=lambda: orphan)
        with patch("run_silent_host_probe.time", SimpleNamespace(monotonic=lambda: clock.now, sleep=sleep)):
            with self.assertRaisesRegex(EvidenceError, "teardown deadline"):
                require_removed(host, find_sink(fixture(), "owned-test"), 1234,
                                PROTOCOL, io.StringIO(), node=NodeIdentity(30, 3000))


if __name__ == "__main__":
    unittest.main()
