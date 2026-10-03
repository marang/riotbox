"""Operator checks against generated children and metadata, never PipeWire."""

import gzip
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from run_silent_host_probe import (
    HostCommands, UnverifiedGroupCleanup, UnverifiedStreamCleanup, main, remove_owned_sink, run_program,
)
from silent_host_evidence import EvidenceError, load_protocol
from silent_host_routes import find_sink
from test_silent_host_evidence import transcript
from test_silent_host_routes import fixture


PROTOCOL = load_protocol(Path(__file__).resolve().parents[1]
                         / "docs/benchmarks/silent_host_observation_v2.json")


class CloseFailureLog:
    """Close the generated real log, then inject an OS-boundary close failure."""

    def __init__(self, output):
        self.output = output

    def write(self, data):
        return self.output.write(data)

    def flush(self):
        return self.output.flush()

    def close(self):
        self.output.close()
        raise OSError("generated route log close failure")

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        self.close()


class GeneratedHost:
    def __init__(self, stdout, *, lose_route=False):
        self.stdout = stdout
        self.calls = 0
        self.route_calls = 0
        self.lose_route = lose_route

    def snapshot(self):
        self.calls += 1
        data = self.stdout.read_bytes() if self.stdout.exists() else b""
        lines = data.splitlines()
        if not lines:
            return fixture()[:1]
        first = json.loads(lines[0])
        self.route_calls += 1
        if (not Path(f"/proc/{first['pid']}").exists()
                or b'"stopped"' in data
                or (self.lose_route and self.route_calls > 3)):
            return fixture()[:1]
        snapshot = fixture()
        snapshot[1]["info"]["props"]["application.process.id"] = first["pid"]
        return snapshot


def generated_command(*, fail_sample=False, early_exit=False):
    records = transcript()
    records = [records[0], records[1], records[-1]]
    records[-1]["sample_index"] = 1
    records[-1]["elapsed_ms"] = 1000
    records[-1]["health"]["callback_count"] = 5
    if fail_sample:
        records[1]["health"]["stream_error_count"] = 1
    code = f"""
import json, os, time
records = json.loads({json.dumps(records)!r})
for record in records:
    record['pid'] = os.getpid()
print(json.dumps(records[0]), flush=True)
if {early_exit!r}:
    raise SystemExit(0)
time.sleep(1.05)
print(json.dumps(records[1]), flush=True)
print(json.dumps(records[2]), flush=True)
"""
    return [sys.executable, "-c", code]


@unittest.skipUnless(sys.platform == "linux", "Linux-only owned-process observation")
class OperatorTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="riotbox-generated-host-")
        self.addCleanup(self.directory.cleanup)
        self.prefix = Path(self.directory.name) / "run"
        self.protocol = dict(PROTOCOL, run_seconds=1, route_poll_interval_ms=10,
                             startup_route_deadline_seconds=1,
                             process_timeout_seconds=3, teardown_deadline_seconds=1)
        self.sink = find_sink(fixture(), "owned-test")
        self.host = GeneratedHost(self.prefix.with_suffix(".stdout.ndjson"))

    def test_complete_generated_child_and_route_evidence_pass(self):
        result = run_program(generated_command(), self.prefix, self.host,
                             self.sink, self.protocol, os.environ.copy())
        self.assertEqual(result["result"]["sample_count"], 1)
        self.assertGreater(result["route_observations"], 0)
        self.assertGreaterEqual(result["elapsed_seconds"], 1)
        self.assertFalse(Path(f"/proc/{result['pid']}").exists())
        with gzip.open(self.prefix.with_suffix(".routes.ndjson.gz"), "rt", encoding="utf-8") as routes:
            records = [json.loads(line) for line in routes]
        self.assertEqual(records[-1]["phase"], "teardown")

    def test_route_log_close_error_without_primary_failure_still_fails(self):
        original_open = gzip.open
        with patch("run_silent_host_probe.gzip.open", side_effect=lambda *args, **kwargs:
                   CloseFailureLog(original_open(*args, **kwargs))):
            with self.assertRaisesRegex(OSError, "generated route log close failure"):
                run_program(generated_command(), self.prefix, self.host,
                            self.sink, self.protocol, os.environ.copy())
        first = json.loads(self.prefix.with_suffix(".stdout.ndjson").read_text().splitlines()[0])
        self.assertFalse(Path(f"/proc/{first['pid']}").exists())

    def test_reported_error_never_passes_even_when_process_exits_zero(self):
        with self.assertRaisesRegex(EvidenceError, "stream_error_count"):
            run_program(generated_command(fail_sample=True), self.prefix, self.host,
                        self.sink, self.protocol, os.environ.copy())
        first = json.loads(self.prefix.with_suffix(".stdout.ndjson").read_text().splitlines()[0])
        self.assertFalse(Path(f"/proc/{first['pid']}").exists())

    def test_route_loss_aborts_owned_child_and_retains_logs(self):
        self.host.lose_route = True
        with self.assertRaisesRegex(EvidenceError, "route disappeared"):
            run_program(generated_command(), self.prefix, self.host,
                        self.sink, self.protocol, os.environ.copy())
        first = json.loads(self.prefix.with_suffix(".stdout.ndjson").read_text().splitlines()[0])
        self.assertFalse(Path(f"/proc/{first['pid']}").exists())
        self.assertTrue(self.prefix.with_suffix(".routes.ndjson.gz").is_file())

    def test_early_zero_exit_cannot_claim_continuity(self):
        with self.assertRaises(EvidenceError):
            run_program(generated_command(early_exit=True), self.prefix, self.host,
                        self.sink, self.protocol, os.environ.copy())

    def test_unverified_group_cleanup_remains_a_typed_containment_failure(self):
        class BrokenCleanup:
            cleanup_verified = False

            def __init__(self, *args):
                pass

            def __enter__(self):
                error = RuntimeError("generated cleanup failure")
                error.add_note("residual leader 1234, child 4321")
                raise error

            def __exit__(self, *args):
                raise AssertionError("entry never completed")

        with patch("run_silent_host_probe.ManagedProcess", BrokenCleanup):
            with self.assertRaises(UnverifiedGroupCleanup) as captured:
                run_program(generated_command(), self.prefix, self.host,
                            self.sink, self.protocol, os.environ.copy())
            self.assertIn("residual leader 1234, child 4321", captured.exception.__notes__)

    def test_group_containment_failure_survives_route_log_close_error(self):
        original_open = gzip.open
        failure = RuntimeError("generated group cleanup failure")
        failure.add_note("residual leader 1234, child 4321")
        with patch("run_silent_host_probe.ManagedProcess") as managed:
            managed.return_value.cleanup_verified = False
            managed.return_value.__enter__.side_effect = failure
            with patch("run_silent_host_probe.gzip.open", side_effect=lambda *args, **kwargs:
                       CloseFailureLog(original_open(*args, **kwargs))):
                with self.assertRaises(UnverifiedGroupCleanup) as captured:
                    run_program(generated_command(), self.prefix, self.host,
                                self.sink, self.protocol, os.environ.copy())
        notes = " ".join(captured.exception.__notes__)
        self.assertIn("residual leader 1234, child 4321", notes)
        self.assertIn("OSError: generated route log close failure", notes)

    def test_metadata_deadline_failure_is_not_an_empty_successful_snapshot(self):
        with patch("run_silent_host_probe.subprocess.run",
                   side_effect=subprocess.TimeoutExpired("generated", 0.01)):
            with self.assertRaises(subprocess.TimeoutExpired):
                HostCommands(0.01).snapshot()

    def test_metadata_text_preserves_empty_short_module_argument_columns(self):
        output = subprocess.CompletedProcess([], 0, "7\tmodule-always-sink\t\t\n", "")
        with patch("run_silent_host_probe.subprocess.run", return_value=output):
            self.assertEqual(HostCommands(3).text(["generated-metadata"]),
                             "7\tmodule-always-sink\t\t")

    def test_active_route_serial_change_is_rejected_even_with_identical_ids(self):
        class ReplacedNodeHost(GeneratedHost):
            def snapshot(self):
                data = super().snapshot()
                if len(data) > 1 and self.route_calls > 1:
                    data[2]["info"]["props"]["object.serial"] = 4000
                return data

        host = ReplacedNodeHost(self.prefix.with_suffix(".stdout.ndjson"))
        with self.assertRaisesRegex(EvidenceError, "admitted process route changed"):
            run_program(generated_command(), self.prefix, host,
                        self.sink, self.protocol, os.environ.copy())
        first = json.loads(self.prefix.with_suffix(".stdout.ndjson").read_text().splitlines()[0])
        self.assertFalse(Path(f"/proc/{first['pid']}").exists())

    def test_generated_process_teardown_keeps_original_lifetime_when_id_is_recycled(self):
        for kind in ("Client", "Node"):
            with self.subTest(replacement=kind):
                prefix = self.prefix.with_name(f"reused-{kind}")

                class RecycledIdHost(GeneratedHost):
                    def snapshot(self):
                        data = super().snapshot()
                        if len(data) == 1 and self.route_calls > 1:
                            data.append({
                                "id": 30, "type": f"PipeWire:Interface:{kind}",
                                "info": {"props": {"object.serial": 4000,
                                                   "application.process.id": 9876,
                                                   "media.class": "Audio/Sink"}},
                            })
                        return data

                result = run_program(generated_command(), prefix,
                                     RecycledIdHost(prefix.with_suffix(".stdout.ndjson")),
                                     self.sink, self.protocol, os.environ.copy())
                self.assertEqual(result["route"]["node"], {"node_id": 30, "serial": 3000})
                self.assertEqual(result["result"]["sample_count"], 1)
                self.assertFalse(Path(f"/proc/{result['pid']}").exists())

    def test_failed_stream_removal_retains_typed_attribution_after_successful_process_exit(self):
        with patch("run_silent_host_probe.require_removed", side_effect=EvidenceError("node remains")):
            with self.assertRaises(UnverifiedStreamCleanup) as captured:
                run_program(generated_command(), self.prefix, self.host,
                            self.sink, self.protocol, os.environ.copy())
        first = json.loads(self.prefix.with_suffix(".stdout.ndjson").read_text().splitlines()[0])
        self.assertIn(f"pid {first['pid']}, node 30", str(captured.exception))
        self.assertIn("node remains", " ".join(captured.exception.__notes__))
        self.assertFalse(Path(f"/proc/{first['pid']}").exists())

    def test_stream_containment_failure_survives_route_log_close_error(self):
        original_open = gzip.open
        with patch("run_silent_host_probe.require_removed", side_effect=EvidenceError("node remains")):
            with patch("run_silent_host_probe.gzip.open", side_effect=lambda *args, **kwargs:
                       CloseFailureLog(original_open(*args, **kwargs))):
                with self.assertRaises(UnverifiedStreamCleanup) as captured:
                    run_program(generated_command(), self.prefix, self.host,
                                self.sink, self.protocol, os.environ.copy())
        first = json.loads(self.prefix.with_suffix(".stdout.ndjson").read_text().splitlines()[0])
        self.assertIn(f"pid {first['pid']}, node 30", str(captured.exception))
        notes = " ".join(captured.exception.__notes__)
        self.assertIn("node remains", notes)
        self.assertIn("OSError: generated route log close failure", notes)
        self.assertFalse(Path(f"/proc/{first['pid']}").exists())

    def test_no_opt_in_never_contacts_a_host(self):
        with patch("run_silent_host_probe.execute_attempt") as execute:
            with self.assertRaises(SystemExit):
                main([])
            execute.assert_not_called()


class OwnedSinkCleanupTests(unittest.TestCase):
    def test_only_exact_owned_module_is_removed(self):
        class Modules:
            def __init__(self):
                self.own_present = True
                self.commands = []

            def text(self, command):
                if command == ["pactl", "--format=text", "list", "short", "modules"]:
                    rows = "7\tmodule-null-sink\tsink_name=unrelated\t\n"
                    if self.own_present:
                        rows += ("42\tmodule-null-sink\tsink_name=owned-test channels=2 "
                                 "channel_map=front-left,front-right\t\n")
                    return rows
                if command != ["pactl", "unload-module", "42"]:
                    raise AssertionError(command)
                self.commands.append(command)
                self.own_present = False
                return ""

            def snapshot(self):
                return fixture()[:1] if self.own_present else []

        host = Modules()
        with self.assertRaises(EvidenceError):
            remove_owned_sink(host, "owned-test", 43, PROTOCOL)
        self.assertEqual(host.commands, [])
        remove_owned_sink(host, "owned-test", 42, PROTOCOL)
        self.assertEqual(host.commands, [["pactl", "unload-module", "42"]])


if __name__ == "__main__":
    unittest.main()
