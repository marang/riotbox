"""Execute the exact authorized V3 fence using generated adapters only."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError, V3_PROTOCOL_SHA256

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/benchmarks/silent_host_riotbox_1572_once_2026-10-09.md"
PHASE = "artifacts/audio_qa/local-silent-host-riotbox-1572-v3-2026-10-09"
REVISION = "a" * 40


class ProspectiveV3InvocationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.phase = self.root / PHASE
        self.phase.mkdir(parents=True)
        self.owner = self.phase / "attempt-01"
        self.marker = self.phase / "invocation-consumed.json"
        self.binding_path = self.phase / "prospective-binding.json"
        protocol = self.root / "docs/benchmarks/silent_host_observation_v3.json"
        protocol.parent.mkdir(parents=True)
        protocol.write_bytes((ROOT / "docs/benchmarks/silent_host_observation_v3.json").read_bytes())
        binary_dir = self.root / "target/debug"
        binary_dir.mkdir(parents=True)
        hashes = {}
        for name in ("cpal_spike", "silent_host_probe"):
            binary = binary_dir / name
            binary.write_bytes(f"generated non-executable {name}".encode())
            hashes[name] = operator.digest(binary)
        self.binding = {
            "schema": "riotbox.silent_host_prospective_binding.v1",
            "owner_ticket": "RIOTBOX-1572",
            "owner_path": str(self.owner),
            "reviewed_git_revision": REVISION,
            "protocol_sha256": V3_PROTOCOL_SHA256,
            "binary_sha256": hashes,
        }
        self.write_binding()
        contract = CONTRACT.read_text()
        self.assertEqual(contract.count("```python\n"), 1)
        self.command = contract.split("```python\n", 1)[1].split("\n```", 1)[0]
        self.git_revision = REVISION
        self.git_dirty = ""
        root_patch = patch.object(operator, "ROOT", self.root)
        root_patch.start()
        self.addCleanup(root_patch.stop)
        execute_patch = patch.object(operator, "execute_attempt")
        self.execute = execute_patch.start()
        self.addCleanup(execute_patch.stop)
        git_patch = patch.object(operator.subprocess, "check_output", side_effect=self.git)
        git_patch.start()
        self.addCleanup(git_patch.stop)

    def write_binding(self):
        self.binding_path.write_text(json.dumps(self.binding))

    def git(self, command, *, cwd, text, timeout):
        self.assertEqual(cwd, self.root)
        self.assertTrue(text)
        self.assertEqual(timeout, 3)
        if command == ["git", "rev-parse", "HEAD"]:
            return self.git_revision + "\n"
        self.assertEqual(command, ["git", "status", "--porcelain"])
        return self.git_dirty

    def invoke(self):
        exec(compile(self.command, str(CONTRACT), "exec"), {})

    def assert_rejected_without_host_or_consumption(self):
        with self.assertRaises((EvidenceError, FileNotFoundError)):
            self.invoke()
        self.execute.assert_not_called()
        self.assertFalse(self.marker.exists())

    def test_exact_binding_invokes_once_and_marker_binds_captured_bytes(self):
        expected_hash = hashlib.sha256(self.binding_path.read_bytes()).hexdigest()
        self.invoke()
        self.execute.assert_called_once()
        self.assertEqual(self.execute.call_args.kwargs, {"owner": self.owner})
        consumed = json.loads(self.marker.read_bytes())
        self.assertEqual(consumed["binding_sha256"], expected_hash)
        self.assertEqual(consumed["reviewed_git_revision"], REVISION)
        self.assertEqual(consumed["owner_ticket"], "RIOTBOX-1572")
        with self.assertRaises(FileExistsError):
            self.invoke()
        self.execute.assert_called_once()

    def test_early_failure_consumes_authorization_without_attempt_directory(self):
        self.execute.side_effect = EvidenceError("generated early admission failure")
        with self.assertRaisesRegex(EvidenceError, "early admission"):
            self.invoke()
        self.assertTrue(self.marker.exists())
        self.assertFalse(self.owner.exists())
        with self.assertRaises(FileExistsError):
            self.invoke()
        self.execute.assert_called_once()

    def test_wrong_or_dirty_revision_rejects_before_invocation(self):
        self.git_revision = "b" * 40
        self.assert_rejected_without_host_or_consumption()
        self.git_revision = REVISION
        self.git_dirty = " M tracked-file\n"
        self.assert_rejected_without_host_or_consumption()

    def test_changed_binary_or_protocol_rejects_before_invocation(self):
        binary = self.root / "target/debug/cpal_spike"
        original = binary.read_bytes()
        binary.write_bytes(b"changed binary")
        self.assert_rejected_without_host_or_consumption()
        binary.write_bytes(original)
        (self.root / "docs/benchmarks/silent_host_observation_v3.json").write_bytes(b"{}")
        self.assert_rejected_without_host_or_consumption()

    def test_invalid_binding_or_existing_owner_rejects(self):
        for field, value in (("schema", "other"),
                             ("owner_ticket", "RIOTBOX-other"),
                             ("owner_path", str(self.phase / "alternate-owner")),
                             ("protocol_sha256", "0" * 64),
                             ("binary_sha256", {})):
            with self.subTest(field=field):
                original = self.binding[field]
                self.binding[field] = value
                self.write_binding()
                self.assert_rejected_without_host_or_consumption()
                self.binding[field] = original
        self.write_binding()
        self.owner.mkdir()
        self.assert_rejected_without_host_or_consumption()

    def test_missing_binding_does_not_authorize_operator(self):
        self.binding_path.unlink()
        self.assert_rejected_without_host_or_consumption()


if __name__ == "__main__":
    unittest.main()
