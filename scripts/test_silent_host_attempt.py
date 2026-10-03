"""Generated attempt-boundary effects only; host commands and launch are fakes."""

from contextlib import ExitStack
import json
import os
from pathlib import Path
import signal
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError
from silent_host_signals import OperatorInterrupts
from test_silent_host_routes import fixture


class AttemptTests(unittest.TestCase):
    def run_attempt(self, error):
        class Host:
            def __init__(self, *args):
                self.created = False
                self.unloaded = False

            def text(self, command):
                if command[:3] == ["git", "status", "--porcelain"]:
                    return ""
                if command[:2] == ["git", "rev-parse"]:
                    return "generated-revision"
                if command[0] == "loginctl":
                    return f"Active=yes\nRemote=no\nUser={os.getuid()}"
                if command == ["pactl", "--format=text", "list", "short", "modules"]:
                    return ("42\tmodule-null-sink\tsink_name=riotbox_silent_host_v2_fixture "
                            "channels=2 channel_map=front-left,front-right\t\n"
                            if self.created and not self.unloaded else "")
                if command[:2] == ["pactl", "load-module"]:
                    self.created = True
                    return "42"
                if command[:2] == ["pactl", "unload-module"]:
                    self.unloaded = True
                    return ""
                raise AssertionError(command)

            def json(self, command):
                if command[-1] == "info":
                    return {"is_local": "yes", "server_name": "PipeWire generated", "host_name": "fixture"}
                raise AssertionError(command)

            def snapshot(self):
                data = fixture()[:1]
                data[0]["info"]["props"]["node.name"] = "riotbox_silent_host_v2_fixture"
                return data if self.created and not self.unloaded else []

            def default_state(self):
                return {"default_sink_name": "untouched", "mute": False, "volume": {"fixture": 100}}

        host = Host()
        with tempfile.TemporaryDirectory(prefix="riotbox-attempt-fixture-") as directory:
            root = Path(directory)
            protocol = root / "docs/benchmarks/silent_host_observation_v2.json"
            protocol.parent.mkdir(parents=True)
            protocol.write_bytes((operator.ROOT / "docs/benchmarks/silent_host_observation_v2.json").read_bytes())
            owner = root / "generated-attempt"
            for name in ("cpal_spike", "silent_host_probe"):
                binary = root / "target/debug" / name
                binary.parent.mkdir(parents=True, exist_ok=True)
                binary.write_bytes(b"generated, never executed")
            with ExitStack() as stack:
                stack.enter_context(patch.object(operator, "ROOT", root))
                stack.enter_context(patch.object(operator, "HostCommands", return_value=host))
                stack.enter_context(patch.object(operator.shutil, "which", return_value="generated"))
                stack.enter_context(patch.object(operator.uuid, "uuid4", return_value=SimpleNamespace(hex="fixture")))
                stack.enter_context(patch.object(operator.subprocess, "run", return_value=SimpleNamespace(returncode=1, stdout="none")))
                launch = stack.enter_context(patch.object(operator, "run_program", side_effect=error))
                stack.enter_context(patch.dict(os.environ, {"XDG_SESSION_ID": "generated", "XDG_RUNTIME_DIR": "/run/user/1000"}))
                with OperatorInterrupts() as interrupts:
                    with self.assertRaises(type(error)):
                        operator.execute_attempt(interrupts, owner=owner)
                launch.assert_called_once()
                result = json.loads((owner / "result.json").read_text())
                return host, result

    def test_unverified_group_or_stream_removal_retains_containment_and_failure_attribution(self):
        for error in (operator.UnverifiedGroupCleanup("leader 1234, child 4321"),
                      operator.UnverifiedStreamCleanup("pid 4321, node 30")):
            with self.subTest(error=type(error).__name__):
                host, result = self.run_attempt(error)
                self.assertFalse(host.unloaded)
                self.assertFalse(result["cleanup_verified"])
                self.assertEqual(result["result"], "failed")
                self.assertIn(str(error), result["error"])
                self.assertEqual(result["module_id"], 42)

    def test_ordinary_observation_failure_cleans_up_without_retry(self):
        host, result = self.run_attempt(EvidenceError("generated observation failure"))
        self.assertTrue(host.unloaded)
        self.assertTrue(result["cleanup_verified"])
        self.assertEqual(result["result"], "failed")

    def test_first_signal_during_result_publication_cannot_leave_success(self):
        original = json.dump
        calls = []

        def interrupting_dump(*args, **kwargs):
            result = original(*args, **kwargs)
            calls.append(True)
            if len(calls) == 1:
                os.kill(os.getpid(), signal.SIGTERM)
            return result

        with tempfile.TemporaryDirectory(prefix="riotbox-result-fixture-") as directory:
            path = Path(directory) / "result.json"
            with self.assertRaises(KeyboardInterrupt):
                with OperatorInterrupts() as interrupts:
                    with interrupts.cleanup(), patch.object(operator.json, "dump", interrupting_dump):
                        operator.write_result(path, {"result": "pass"}, interrupts)
                    interrupts.check()
            self.assertEqual(json.loads(path.read_text())["result"], "failed")

    def test_first_signal_during_result_close_cannot_leave_success(self):
        original = Path.open

        class InterruptOnClose:
            def __init__(self, output):
                self.output = output

            def __enter__(self):
                return self.output

            def __exit__(self, *args):
                self.output.__exit__(*args)
                os.kill(os.getpid(), signal.SIGTERM)

        def open_with_interrupted_close(path, mode="r", *args, **kwargs):
            output = original(path, mode, *args, **kwargs)
            return InterruptOnClose(output) if mode == "x" else output

        with tempfile.TemporaryDirectory(prefix="riotbox-result-close-") as directory:
            path = Path(directory) / "result.json"
            with self.assertRaises(KeyboardInterrupt):
                with OperatorInterrupts() as interrupts:
                    with interrupts.cleanup(), patch.object(Path, "open", open_with_interrupted_close):
                        operator.write_result(path, {"result": "pass"}, interrupts)
                    interrupts.check()
            self.assertEqual(json.loads(path.read_text())["result"], "failed")


if __name__ == "__main__":
    unittest.main()
