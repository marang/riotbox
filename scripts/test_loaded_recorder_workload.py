"""Closed workload/Core-report seam with generated metadata, never audio launch."""

from contextlib import ExitStack
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from loaded_recorder_evidence import EvidenceError, load_protocol, sha256
from loaded_recorder_workload import LoadedRecorderWorkload
from run_silent_host_probe import execute_attempt
import run_silent_host_probe as operator
from silent_host_signals import OperatorInterrupts
from test_silent_host_routes import fixture as route_fixture


class LoadedWorkloadTests(unittest.TestCase):
    def test_arbitrary_workload_rejects_before_host_commands(self):
        with patch("run_silent_host_probe.HostCommands") as commands:
            with self.assertRaisesRegex(EvidenceError, "closed loaded"):
                execute_attempt(None, owner=Path("/unused"), workload=object())
            commands.assert_not_called()

    def test_binary_binding_and_required_diagnostics_precede_host(self):
        workload = LoadedRecorderWorkload(object(), "a" * 64)
        with patch("run_silent_host_probe.HostCommands") as commands:
            with self.assertRaisesRegex(EvidenceError, "admitted generated fixture"):
                execute_attempt(None, owner=Path("/unused"), workload=workload)
            commands.assert_not_called()

    def run_case(self, *, changed=None, binary_changed=False):
        with tempfile.TemporaryDirectory() as directory:
            prefix = Path(directory) / "run-1"
            binary = Path(directory) / "riotbox-app"
            binary.write_bytes(b"generated-never-executed")
            fixture = SimpleNamespace(verify_unchanged=lambda: None,
                                      verify_case_before_launch=lambda case: None)
            case = SimpleNamespace(session_path=Path(directory) / "session.json")
            workload = LoadedRecorderWorkload(fixture, sha256(binary.read_bytes()))
            readiness = {"mode": "live_recording_readiness_report", "ready": True,
                "status": "ready", "writes_files": False, "runtime_master_recording_ready": True,
                "session_path": str(case.session_path), "readiness_blockers": [],
                "receipt": {"receipt_id": "receipt-4", "created_by_action": 4}}
            if changed:
                readiness.update(changed)
            commands = []

            def text(command):
                commands.append(command)
                if binary_changed:
                    binary.write_bytes(b"changed-after-Core-validation")
                return json.dumps(readiness)

            result = {"result": {"receipt": {"receipt_id": "receipt-4"}},
                      "marker": {"action_id": 4}}
            with patch("loaded_recorder_process.run_recording", return_value=result) as run:
                report = workload.run_case(binary, case, prefix, SimpleNamespace(text=text),
                                           object(), {}, {}, object())
            run.assert_called_once()
            self.assertEqual(commands, [[str(binary), "--live-recording-readiness-report",
                                        "--session", str(case.session_path)]])
            self.assertEqual(report["core_readiness_sha256"],
                             sha256(prefix.with_suffix(".readiness.json").read_bytes()))

    def test_existing_Core_metadata_validator_is_required(self):
        self.run_case()
        for changed in ({"ready": False}, {"runtime_master_recording_ready": False},
                        {"readiness_blockers": ["blocked"]},
                        {"receipt": {"receipt_id": "foreign", "created_by_action": 4}},
                        {"receipt": {"receipt_id": "receipt-4", "created_by_action": 3}}):
            with self.subTest(changed=changed), self.assertRaises(EvidenceError):
                self.run_case(changed=changed)

    def test_changed_binary_after_Core_report_never_passes(self):
        with self.assertRaisesRegex(EvidenceError, "binary"):
            self.run_case(binary_changed=True)

    def test_run_copy_is_admitted_again_before_process_hydration(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "riotbox-app"
            binary.write_bytes(b"generated-never-executed")
            check = Mock(side_effect=EvidenceError("changed run Session"))
            workload = LoadedRecorderWorkload(SimpleNamespace(verify_case_before_launch=check),
                                              sha256(binary.read_bytes()))
            case = object()
            with patch("loaded_recorder_process.run_recording") as process:
                with self.assertRaisesRegex(EvidenceError, "changed run Session"):
                    workload.run_case(binary, case, Path(directory) / "run", None, None, {}, {}, object())
                check.assert_called_once_with(case)
                process.assert_not_called()

    def run_attempt(self, *, fail_first=False):
        host = Mock()
        def text(command):
            if command[:2] == ["git", "status"]:
                return ""
            if command[:2] == ["git", "rev-parse"]:
                return "generated-revision"
            if command[0] == "loginctl":
                return f"Active=yes\nRemote=no\nUser={os.getuid()}"
            if command[:2] == ["pactl", "load-module"]:
                return "42"
            raise AssertionError(command)
        host.text.side_effect = text
        host.json.return_value = {"is_local": "yes", "server_name": "PipeWire generated",
                                  "host_name": "generated"}
        host.default_state.return_value = {"default_sink_name": "unchanged", "mute": False,
                                           "volume": {"generated": 100}}
        snapshot = route_fixture()[:1]
        snapshot[0]["info"]["props"]["node.name"] = "riotbox_silent_host_v3_fixture"
        host.snapshot.return_value = snapshot
        protocol = load_protocol()
        workload = LoadedRecorderWorkload(object(), "a" * 64)
        diagnostics = SimpleNamespace(protocol={}, expected_hash="b" * 64)
        with tempfile.TemporaryDirectory() as directory, ExitStack() as stack:
            owner = Path(directory) / "attempt"
            stack.enter_context(patch.object(LoadedRecorderWorkload, "admit", return_value=protocol))
            stack.enter_context(patch.object(LoadedRecorderWorkload, "admission", return_value={
                "schema": "riotbox.loaded_recorder_attempt.v1", "quality_proof": False}))
            prepare = stack.enter_context(patch.object(LoadedRecorderWorkload, "prepare_cases",
                return_value=[SimpleNamespace(bars=bars, session_path=owner / f"run-{bars}.json",
                    destination_path=owner / f"master-{bars}.wav", proof_path=owner / f"proof-{bars}.json",
                    observer_path=owner / f"observer-{bars}.ndjson") for bars in (2, 8, 16)]))
            run = stack.enter_context(patch.object(LoadedRecorderWorkload, "run_case",
                side_effect=EvidenceError("generated first case failure") if fail_first else
                [{"pid": pid} for pid in (43, 44, 45)]))
            stack.enter_context(patch.object(operator, "HostCommands", return_value=host))
            stack.enter_context(patch.object(operator.shutil, "which", return_value="generated"))
            stack.enter_context(patch.object(operator, "local_host_environment", return_value={}))
            stack.enter_context(patch.object(operator, "prepare_child_environment", return_value={
                "ALSA_CONFIG_PATH": str(owner / "generated.conf"), "PIPEWIRE_REMOTE": "pipewire-0"}))
            stack.enter_context(patch.object(operator, "digest", return_value="c" * 64))
            stack.enter_context(patch.object(operator.uuid, "uuid4", return_value=SimpleNamespace(hex="fixture")))
            stack.enter_context(patch.object(operator.subprocess, "run",
                return_value=SimpleNamespace(returncode=1, stdout="none")))
            stack.enter_context(patch.object(operator, "owned_modules", return_value=[]))
            stack.enter_context(patch.object(operator, "require_sink_absent"))
            remove = stack.enter_context(patch.object(operator, "remove_owned_sink"))
            preflight = stack.enter_context(patch.object(operator, "run_program", return_value={"pid": 42}))
            stack.enter_context(patch.dict(os.environ, {"XDG_SESSION_ID": "generated"}))
            with OperatorInterrupts() as interrupts:
                if fail_first:
                    with self.assertRaisesRegex(EvidenceError, "first case failure"):
                        execute_attempt(interrupts, owner=owner, diagnostics=diagnostics, workload=workload)
                else:
                    execute_attempt(interrupts, owner=owner, diagnostics=diagnostics, workload=workload)
            prepare.assert_called_once_with(owner)
            preflight.assert_called_once()
            self.assertTrue(preflight.call_args.kwargs["preflight"])
            self.assertEqual([call.args[1].bars for call in run.call_args_list],
                             [2] if fail_first else [2, 8, 16])
            remove.assert_called_once()
            result = json.loads((owner / "result.json").read_bytes())
            self.assertTrue(result["cleanup_verified"])
            self.assertFalse(result["quality_proof"])
            self.assertEqual(result["schema"], "riotbox.loaded_recorder_attempt.v1")
            self.assertEqual(result["result"], "failed" if fail_first else "pass")
            self.assertEqual(len(result["cases"]), 3)
            self.assertEqual(json.loads((owner / "cases.json").read_bytes()), result["cases"])
            self.assertEqual(result["active_run_prefix"], str(owner / ("run-1" if fail_first else "run-3")))

    def test_existing_attempt_reuses_preflight_and_exact_three_loaded_cases(self):
        self.run_attempt()

    def test_existing_attempt_stops_at_first_loaded_failure_without_retry(self):
        self.run_attempt(fail_first=True)


if __name__ == "__main__":
    unittest.main()
