"""Exercise configured exact loaded fences with generated bindings and mocked calls.

No real approval is represented by these controls. Every phase, authorization,
binary and fixture stand-in lives below a fresh tempfile ROOT; no audio child,
host command, source material or old owner is accessed.
"""

import copy
from contextlib import contextmanager
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import run_silent_host_probe as operator
from loaded_recorder_evidence import LoadedObservationVersion, PROTOCOL_SHA256
from loaded_recorder_fixture import GeneratedFixture
from loaded_recorder_workload import LoadedRecorderWorkload
from silent_host_diagnostics import AlsaDiagnostics
from silent_host_evidence import EvidenceError


ROOT = Path(__file__).resolve().parents[1]
REVISION = "a" * 40
RUN_ID, RUN_ATTEMPT = 123456789, 2
BINARY_NAMES = ("cpal_spike", "riotbox-app", "dense_break_live_path_render",
                "libriotbox_silent_alsa_diag.so")
REJECTIONS = (EvidenceError, OSError, subprocess.SubprocessError)


def digest(data):
    return hashlib.sha256(data).hexdigest()


@dataclass(frozen=True)
class PhaseConfig:
    ticket: str
    phase_path: str
    document_name: str
    contract_name: str
    contract_sha256: str
    loaded_sha256: str
    observation_version: LoadedObservationVersion
    protocol_files: tuple[str, ...]


class ProspectiveFenceTests(unittest.TestCase):
    CONFIG: PhaseConfig

    def setUp(self):
        config = self.CONFIG
        temporary = tempfile.TemporaryDirectory(prefix=f"{config.ticket.lower()}-generated-fence-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name) / "repo"
        self.foreign = Path(temporary.name) / "generated-outside-repo"
        self.foreign.mkdir()
        self.phase = self.root / config.phase_path
        self.phase.mkdir(parents=True)
        self.owner = self.phase / "attempt-01"
        self.marker = self.phase / "invocation-consumed.json"
        self.binding_path = self.phase / "prospective-binding.json"
        self.authorization_path = self.phase / "fresh-authorization.json"
        self.fixture_path = self.phase / "fixture-v1/preparation.json"
        self.fixture_path.parent.mkdir()
        self.fixture_path.write_bytes(b"generated report stand-in; admission is separately mocked\n")
        protocols = self.root / "docs/benchmarks"
        protocols.mkdir(parents=True)
        for name in config.protocol_files:
            (protocols / name).write_bytes((ROOT / "docs/benchmarks" / name).read_bytes())
        self.assertEqual(operator.digest(protocols / config.contract_name), config.contract_sha256)
        binary_directory = self.root / "target/debug"
        binary_directory.mkdir(parents=True)
        hashes = {}
        for name in BINARY_NAMES:
            path = binary_directory / name
            path.write_bytes(f"generated non-executable {name}\n".encode())
            self.assertEqual(path.stat().st_mode & 0o111, 0)
            hashes[name] = operator.digest(path)
        self.producer_hash = hashes["dense_break_live_path_render"]
        self.authorization = {
            "schema": "riotbox.loaded_recorder_host_authorization.v1",
            "owner_ticket": config.ticket, "phase_path": str(self.phase),
            "phase_protocol_sha256": config.contract_sha256, "fresh_user_approval": True,
            "approval_note": "Generated test approval control only; no real user authorization.",
        }
        self.write_authorization()
        self.binding = {
            "schema": "riotbox.loaded_recorder_prospective_binding.v1",
            "owner_ticket": config.ticket, "owner_path": str(self.owner),
            "fixture_report_path": str(self.fixture_path), "phase_protocol_sha256": config.contract_sha256,
            "loaded_protocol_sha256": config.loaded_sha256, "reviewed_git_revision": REVISION,
            "native_ci_run_id": RUN_ID, "native_ci_run_attempt": RUN_ATTEMPT,
            "binary_sha256": hashes, "fixture_report_sha256": operator.digest(self.fixture_path),
            "authorization_sha256": operator.digest(self.authorization_path),
        }
        self.write_binding()
        self.ci = {"id": RUN_ID, "run_attempt": RUN_ATTEMPT, "head_sha": REVISION,
                   "status": "completed", "conclusion": "success", "name": "Rust CI",
                   "repository": {"full_name": "marang/riotbox"}}
        self.head, self.dirty = REVISION, ""
        self.commands = []
        self.ci_error = None
        self.ci_response = None
        self.admission_error = None
        self.fixture = object()  # Explicit generated admission stand-in, never hydrated.
        document_path = ROOT / "docs/benchmarks" / config.document_name
        document = document_path.read_text()
        self.assertEqual(document.count("```python\n"), 1)
        self.fence = compile(document.split("```python\n", 1)[1].split("\n```", 1)[0],
                             str(document_path), "exec")
        for replacement in (
            patch.object(operator, "ROOT", self.root),
            patch.object(operator.subprocess, "check_output", side_effect=self.command),
            patch.object(operator.subprocess, "Popen",
                         side_effect=AssertionError("generated fence must not spawn a process")),
            patch.object(operator, "HostCommands",
                         side_effect=AssertionError("generated fence must not query the host")),
        ):
            replacement.start()
            self.addCleanup(replacement.stop)
        execute = patch.object(operator, "execute_attempt")
        self.execute = execute.start()
        self.addCleanup(execute.stop)
        admit = patch.object(GeneratedFixture, "admit", side_effect=self.admit)
        self.admit_call = admit.start()
        self.addCleanup(admit.stop)

    def write_authorization(self):
        self.authorization_path.write_text(json.dumps(self.authorization))

    def write_binding(self):
        self.binding_path.write_text(json.dumps(self.binding))

    def command(self, command, *, cwd, text, timeout):
        self.commands.append(command)
        self.assertEqual((cwd, text, timeout), (self.root, True, 3))
        if command == ["git", "rev-parse", "HEAD"]:
            return self.head + "\n"
        if command == ["git", "status", "--porcelain"]:
            return self.dirty
        if command == ["gh", "api", f"repos/marang/riotbox/actions/runs/{RUN_ID}"]:
            if self.ci_error is not None:
                raise self.ci_error
            return json.dumps(self.ci) if self.ci_response is None else self.ci_response
        raise AssertionError(f"unexpected generated command; no command executed: {command}")

    def admit(self, path, *, expected_report_sha256, expected_producer_sha256):
        self.assertEqual(path, self.fixture_path)
        if expected_report_sha256 != operator.digest(self.fixture_path):
            raise EvidenceError("generated report byte binding mismatch")
        if expected_producer_sha256 != self.producer_hash:
            raise EvidenceError("generated producer binding mismatch")
        if self.admission_error is not None:
            raise self.admission_error
        return self.fixture

    def invoke(self):
        exec(self.fence, {})

    def rejected(self):
        with self.assertRaises(REJECTIONS):
            self.invoke()
        self.execute.assert_not_called()
        self.assertFalse(self.marker.exists())
        self.assertFalse(self.owner.exists())

    def json_inputs(self):
        return [self.binding_path, self.authorization_path,
                *(self.root / "docs/benchmarks" / name for name in self.CONFIG.protocol_files)]

    def known_inputs(self):
        return [*self.json_inputs(),
                *(self.root / "target/debug" / name for name in BINARY_NAMES)]

    @contextmanager
    def forbidden_content_reads(self, *paths):
        """Fail before opening a generated alias, rather than risking a FIFO wait.

        Both lexical aliases and their generated outside-ROOT targets are guarded.
        lstat/type inspection is permitted; opening their contents is not.
        """
        forbidden = (*paths, self.foreign)
        original_open, original_read_bytes = os.open, Path.read_bytes

        def check(path):
            if isinstance(path, (str, bytes, os.PathLike)):
                candidate = Path(os.fsdecode(path))
                self.assertFalse(any(candidate.is_relative_to(parent) for parent in forbidden),
                                 f"rejected input contents were opened: {candidate}")

        def guarded_open(path, *args, **kwargs):
            check(path)
            return original_open(path, *args, **kwargs)

        def guarded_read_bytes(path):
            check(path)
            return original_read_bytes(path)

        with patch("os.open", side_effect=guarded_open), \
                patch.object(Path, "read_bytes", guarded_read_bytes):
            yield

    def rejected_before_fixture(self):
        self.rejected()
        self.admit_call.assert_not_called()

    def test_exact_fence_invokes_once_with_exact_workload_diagnostics_and_consumption(self):
        binding_bytes = self.binding_path.read_bytes()
        authorization_bytes = self.authorization_path.read_bytes()
        ci_bytes = json.dumps(self.ci).encode()
        self.invoke()
        self.execute.assert_called_once()
        arguments = self.execute.call_args.kwargs
        self.assertEqual(set(arguments), {"owner", "diagnostics", "workload"})
        self.assertEqual(arguments["owner"], self.owner)
        self.assertIs(type(arguments["diagnostics"]), AlsaDiagnostics)
        self.assertEqual(arguments["diagnostics"].library,
                         self.root / "target/debug/libriotbox_silent_alsa_diag.so")
        self.assertEqual(arguments["diagnostics"].expected_hash,
                         self.binding["binary_sha256"]["libriotbox_silent_alsa_diag.so"])
        self.assertIs(type(arguments["workload"]), LoadedRecorderWorkload)
        self.assertIs(arguments["workload"].observation_version, self.CONFIG.observation_version)
        self.assertIs(arguments["workload"].fixture, self.fixture)
        self.assertEqual(arguments["workload"].expected_binary_sha256,
                         self.binding["binary_sha256"]["riotbox-app"])
        self.admit_call.assert_called_once_with(
            self.fixture_path, expected_report_sha256=operator.digest(self.fixture_path),
            expected_producer_sha256=self.producer_hash)
        consumed = json.loads(self.marker.read_bytes())
        self.assertEqual(consumed, {
            "schema": "riotbox.loaded_recorder_invocation_consumed.v1",
            "owner_ticket": self.CONFIG.ticket, "binding_sha256": digest(binding_bytes),
            "authorization_sha256": digest(authorization_bytes),
            "reviewed_git_revision": REVISION, "native_ci_run_id": RUN_ID,
            "native_ci_run_attempt": RUN_ATTEMPT, "native_ci_response_sha256": digest(ci_bytes),
        })
        before = self.marker.read_bytes()
        with self.assertRaises(EvidenceError):
            self.invoke()
        self.assertEqual(self.marker.read_bytes(), before)
        self.execute.assert_called_once()
        self.admit_call.assert_called_once()

    def test_early_operator_failure_consumes_before_an_attempt_owner_exists(self):
        def fail(*_args, **kwargs):
            self.assertTrue(self.marker.is_file())
            self.assertFalse(kwargs["owner"].exists())
            raise EvidenceError("generated early operator failure")
        self.execute.side_effect = fail
        with self.assertRaisesRegex(EvidenceError, "early operator"):
            self.invoke()
        self.assertTrue(self.marker.exists())
        self.assertFalse(self.owner.exists())
        with self.assertRaises(EvidenceError):
            self.invoke()
        self.execute.assert_called_once()

    def test_missing_binding_or_authorization_never_consumes(self):
        binding = self.binding_path.read_bytes()
        self.binding_path.unlink()
        self.rejected()
        self.binding_path.write_bytes(binding)
        self.authorization_path.unlink()
        self.rejected()

    def test_malformed_duplicate_or_nonobject_binding_and_authorization_reject(self):
        for path in (self.binding_path, self.authorization_path):
            original = path.read_bytes()
            for payload in (b"not JSON", b'[]', b'{"schema":1,"schema":2}'):
                with self.subTest(path=path.name, payload=payload):
                    path.write_bytes(payload)
                    if path == self.authorization_path:
                        self.binding["authorization_sha256"] = operator.digest(path)
                        self.write_binding()
                    self.rejected()
            path.write_bytes(original)
        self.binding["authorization_sha256"] = operator.digest(self.authorization_path)
        self.write_binding()

    def test_wrong_or_dirty_revision_and_invalid_revision_spelling_reject(self):
        self.head = "b" * 40
        self.rejected()
        self.head, self.dirty = REVISION, " M generated-tracked-file\n"
        self.rejected()
        self.dirty = ""
        for value in (None, True, "a" * 39, "A" * 40, "not-a-revision"):
            with self.subTest(revision=value):
                self.binding["reviewed_git_revision"] = value
                self.write_binding()
                self.rejected()

    def test_old_ticket_phase_owner_fixture_or_protocol_binding_rejects(self):
        original = copy.deepcopy(self.binding)
        for key, value in (
            ("schema", "riotbox.silent_host_prospective_binding.v2"),
            ("owner_ticket", self.CONFIG.ticket + "-other"),
            ("owner_path", str(self.phase / "attempt-02")),
            ("owner_path", str(self.root / "old-phase/attempt-01")),
            ("fixture_report_path", str(self.phase / "other-fixture/preparation.json")),
            ("phase_protocol_sha256", "0" * 64),
            ("loaded_protocol_sha256", "0" * 64),
        ):
            with self.subTest(key=key, value=value):
                self.binding = copy.deepcopy(original)
                self.binding[key] = value
                self.write_binding()
                self.rejected()

    def test_refused_stale_wrong_phase_or_unexplained_authorization_rejects(self):
        original = copy.deepcopy(self.authorization)
        for key, value in (
            ("schema", "other"), ("owner_ticket", self.CONFIG.ticket + "-other"),
            ("phase_path", str(self.root / "old-phase")), ("phase_protocol_sha256", "0" * 64),
            ("fresh_user_approval", False), ("fresh_user_approval", 1),
            ("approval_note", ""), ("approval_note", "  \n"), ("approval_note", None),
        ):
            with self.subTest(key=key, value=value):
                self.authorization = copy.deepcopy(original)
                self.authorization[key] = value
                self.write_authorization()
                self.binding["authorization_sha256"] = operator.digest(self.authorization_path)
                self.write_binding()
                self.rejected()

    def test_authorization_byte_mutation_or_wrong_digest_cannot_reuse_approval(self):
        self.authorization["approval_note"] = "changed generated approval record"
        self.write_authorization()  # Deliberately do not rebind the changed bytes.
        self.rejected()
        self.binding["authorization_sha256"] = "0" * 64
        self.write_binding()
        self.rejected()

    def test_changed_missing_extra_or_old_binary_binding_rejects(self):
        original = copy.deepcopy(self.binding["binary_sha256"])
        for name in BINARY_NAMES:
            with self.subTest(binary=name):
                path = self.root / "target/debug" / name
                data = path.read_bytes()
                path.write_bytes(data + b"changed")
                self.rejected()
                path.unlink()
                self.rejected()
                path.write_bytes(data)
        for hashes in ({}, {**original, "silent_host_probe": "0" * 64},
                       {key: value for key, value in original.items() if key != "riotbox-app"},
                       {**original, "riotbox-app": "0" * 64}):
            with self.subTest(binding=hashes):
                self.binding["binary_sha256"] = hashes
                self.write_binding()
                self.rejected()

    def test_changed_phase_loaded_fixture_or_inherited_protocol_rejects(self):
        for name in self.CONFIG.protocol_files:
            with self.subTest(protocol=name):
                path = self.root / "docs/benchmarks" / name
                original = path.read_bytes()
                path.write_bytes(b"{}")
                self.rejected()
                path.write_bytes(original)

    def test_report_producer_or_fully_valid_fixture_admission_failure_precedes_consumption(self):
        original_report = self.fixture_path.read_bytes()
        self.fixture_path.write_bytes(original_report + b"changed")
        self.rejected()
        self.fixture_path.write_bytes(original_report)
        self.producer_hash = "0" * 64
        self.rejected()
        self.producer_hash = self.binding["binary_sha256"]["dense_break_live_path_render"]
        self.admission_error = EvidenceError("generated fixture admission failure")
        self.rejected()
        self.assertIn(["gh", "api", f"repos/marang/riotbox/actions/runs/{RUN_ID}"], self.commands)

    def test_invalid_native_ci_head_status_result_identity_workflow_or_repository_rejects(self):
        original = copy.deepcopy(self.ci)
        for key, value in (
            ("head_sha", "b" * 40), ("status", "in_progress"), ("conclusion", "failure"),
            ("conclusion", None), ("id", RUN_ID + 1), ("id", True),
            ("run_attempt", RUN_ATTEMPT + 1), ("run_attempt", True), ("name", "Other CI"),
            ("repository", {"full_name": "other/riotbox"}), ("repository", {}),
        ):
            with self.subTest(key=key, value=value):
                self.ci = copy.deepcopy(original)
                self.ci[key] = value
                self.rejected()

    def test_missing_invalid_or_zero_native_ci_binding_rejects_before_api(self):
        for key in ("native_ci_run_id", "native_ci_run_attempt"):
            for value in (None, False, 0, -1, "invalid"):
                with self.subTest(key=key, value=value):
                    old = self.binding[key]
                    self.binding[key] = value
                    self.write_binding()
                    self.rejected()
                    self.binding[key] = old

    def test_native_ci_api_failure_or_malformed_response_fails_closed(self):
        for error in (subprocess.CalledProcessError(1, ["gh", "api"]),
                      subprocess.TimeoutExpired(["gh", "api"], 3)):
            self.ci_error = error
            self.rejected()
        self.ci_error = None
        for response in ("{}", "not-json", '{"id":1,"id":2}', "x" * 262145):
            with self.subTest(response=response[:30]):
                self.ci_response = response
                self.rejected()

    def test_existing_owner_file_directory_or_dangling_link_rejects(self):
        self.owner.write_bytes(b"generated occupied owner")
        with self.assertRaises(EvidenceError):
            self.invoke()
        self.owner.unlink()
        self.owner.mkdir()
        with self.assertRaises(EvidenceError):
            self.invoke()
        self.owner.rmdir()
        self.owner.symlink_to(self.phase / "missing-generated-target")
        with self.assertRaises(EvidenceError):
            self.invoke()
        self.execute.assert_not_called()
        self.assertFalse(self.marker.exists())

    def test_same_and_wrong_byte_symlink_inputs_reject_before_content_open(self):
        for number, path in enumerate(self.known_inputs()):
            original = path.read_bytes()
            for same_bytes in (True, False):
                with self.subTest(input=path.relative_to(self.root), same_bytes=same_bytes):
                    target = self.foreign / f"leaf-{number}-{same_bytes}"
                    target.write_bytes(original if same_bytes else b"wrong generated bytes")
                    path.unlink()
                    path.symlink_to(target)
                    try:
                        with self.forbidden_content_reads(path):
                            self.rejected_before_fixture()
                    finally:
                        path.unlink()
                        path.write_bytes(original)

    def test_symlink_phase_docs_and_target_ancestry_rejects_before_content_open(self):
        for number, parent in enumerate((self.phase, self.root / "docs",
                                         self.root / "docs/benchmarks", self.root / "target",
                                         self.root / "target/debug")):
            with self.subTest(parent=parent.relative_to(self.root)):
                target = self.foreign / f"parent-{number}"
                parent.rename(target)
                parent.symlink_to(target, target_is_directory=True)
                try:
                    with self.forbidden_content_reads(parent):
                        self.rejected_before_fixture()
                finally:
                    parent.unlink()
                    target.rename(parent)

    def test_non_directory_phase_docs_and_target_parents_reject_before_content_open(self):
        for number, parent in enumerate((self.phase, self.root / "docs", self.root / "target")):
            with self.subTest(parent=parent.relative_to(self.root)):
                target = self.foreign / f"file-parent-{number}"
                parent.rename(target)
                parent.write_bytes(b"generated non-directory parent")
                try:
                    with self.forbidden_content_reads(parent):
                        self.rejected_before_fixture()
                finally:
                    parent.unlink()
                    target.rename(parent)

    def test_fifo_and_directory_inputs_reject_before_open_without_waiting(self):
        for path in self.known_inputs():
            original = path.read_bytes()
            for kind in ("fifo", "directory"):
                with self.subTest(input=path.relative_to(self.root), kind=kind):
                    path.unlink()
                    if kind == "fifo":
                        os.mkfifo(path)
                    else:
                        path.mkdir()
                    try:
                        with self.forbidden_content_reads(path):
                            self.rejected_before_fixture()
                    finally:
                        if kind == "directory":
                            path.rmdir()
                        else:
                            path.unlink()
                        path.write_bytes(original)

    def test_missing_json_inputs_reject_before_fixture_or_consumption(self):
        for path in self.json_inputs():
            with self.subTest(input=path.relative_to(self.root)):
                original = path.read_bytes()
                path.unlink()
                try:
                    self.rejected_before_fixture()
                finally:
                    path.write_bytes(original)

    def test_oversized_json_inputs_reject_before_fixture_or_consumption(self):
        for path in self.json_inputs():
            with self.subTest(input=path.relative_to(self.root)):
                original = path.read_bytes()
                path.write_bytes(b" " * 262145)
                try:
                    self.rejected_before_fixture()
                finally:
                    path.write_bytes(original)
