"""Generated existing observation seam plus fake diagnostic adapter; no host."""

from pathlib import Path
import os
import tempfile
import unittest
from unittest.mock import Mock, patch

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError
from test_silent_host_operator import GeneratedHost, PROTOCOL, generated_command
from test_silent_host_routes import fixture


class DiagnosticOperatorTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix="riotbox-diagnostic-operator-")
        self.addCleanup(temp.cleanup)
        self.prefix = Path(temp.name) / "generated"
        self.sink = operator.find_sink(fixture(), "owned-test")
        self.host = GeneratedHost(self.prefix.with_suffix(".stdout.ndjson"))
        self.protocol = dict(PROTOCOL, run_seconds=1, route_poll_interval_ms=10,
                             startup_route_deadline_seconds=1, process_timeout_seconds=3,
                             teardown_deadline_seconds=1)
        self.adapter = Mock()
        self.adapter.wrap.side_effect = lambda command, *_args: command

    def observe(self, **options):
        return operator.run_program(generated_command(**options), self.prefix, self.host, self.sink,
                                    self.protocol, os.environ.copy(), diagnostics=self.adapter)

    def test_read_only_after_verified_child_exit_and_attach_to_actual_pid(self):
        def finish(_prefix, pid, *, complete):
            self.assertFalse(Path(f"/proc/{pid}").exists())
            self.assertTrue(complete)
            return {"generated": True, "pid": pid}
        self.adapter.finish.side_effect = finish
        result = self.observe()
        self.assertEqual(result["alsa_measurements"]["pid"], result["pid"])
        self.adapter.wrap.assert_called_once()

    def test_diagnostic_failure_keeps_original_driver_failure_and_notes(self):
        def finish(_prefix, pid, *, complete):
            self.assertFalse(Path(f"/proc/{pid}").exists())
            self.assertFalse(complete)
            raise EvidenceError("generated incomplete measurement")
        self.adapter.finish.side_effect = finish
        with self.assertRaisesRegex(EvidenceError, "stream_error_count") as caught:
            self.observe(fail_sample=True)
        self.assertIn("generated incomplete measurement", " ".join(caught.exception.__notes__))

    def test_diagnostics_cannot_read_while_group_cleanup_is_uncertain(self):
        with patch.object(operator, "ManagedProcess") as managed:
            managed.return_value.cleanup_verified = False
            managed.return_value.__enter__.side_effect = RuntimeError("generated group failure")
            with self.assertRaises(operator.UnverifiedGroupCleanup):
                self.observe()
        self.adapter.finish.assert_not_called()

    def test_missing_measurement_cannot_grant_success(self):
        self.adapter.finish.side_effect = EvidenceError("generated missing coverage")
        with self.assertRaisesRegex(EvidenceError, "missing coverage"):
            self.observe()
