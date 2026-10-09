"""Reuse generated one-shot binding controls for the exact new V4 fence."""

import hashlib
from pathlib import Path
from unittest.mock import patch

from silent_host_diagnostics import AlsaDiagnostics, V4_PROTOCOL_SHA256
from silent_host_evidence import EvidenceError
import test_silent_host_prospective_1572 as predecessor

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/benchmarks/silent_host_observation_v4.md"
PHASE = "artifacts/audio_qa/local-silent-host-riotbox-1574-v4-2026-10-09"


class ProspectiveV4InvocationTests(predecessor.ProspectiveV3InvocationTests):
    def setUp(self):
        # Only the reusable GENERATED test setup is adapted; old contracts/bytes
        # and actual ignored owners are neither opened nor changed.
        for name, value in (("CONTRACT", CONTRACT), ("PHASE", PHASE)):
            replacement = patch.object(predecessor, name, value)
            replacement.start()
            self.addCleanup(replacement.stop)
        super().setUp()
        (self.root / "docs/benchmarks/silent_host_observation_v4.json").write_bytes(
            (ROOT / "docs/benchmarks/silent_host_observation_v4.json").read_bytes())
        library = self.root / "target/debug/libriotbox_silent_alsa_diag.so"
        library.write_bytes(b"generated non-executable diagnostic library")
        self.binding.update(schema="riotbox.silent_host_prospective_binding.v2",
                            owner_ticket="RIOTBOX-1574", protocol_sha256=V4_PROTOCOL_SHA256)
        self.binding["binary_sha256"][library.name] = hashlib.sha256(library.read_bytes()).hexdigest()
        self.write_binding()

    def test_exact_binding_invokes_once_and_marker_binds_captured_bytes(self):
        import json
        expected = hashlib.sha256(self.binding_path.read_bytes()).hexdigest()
        self.invoke()
        self.execute.assert_called_once()
        arguments = self.execute.call_args.kwargs
        self.assertEqual(arguments["owner"], self.owner)
        self.assertIsInstance(arguments["diagnostics"], AlsaDiagnostics)
        self.assertEqual(arguments["diagnostics"].expected_hash,
                         self.binding["binary_sha256"]["libriotbox_silent_alsa_diag.so"])
        marker = json.loads(self.marker.read_bytes())
        self.assertEqual(marker["owner_ticket"], "RIOTBOX-1574")
        self.assertEqual(marker["binding_sha256"], expected)
        with self.assertRaises(FileExistsError):
            self.invoke()
        self.execute.assert_called_once()

    def test_changed_diagnostic_library_rejects_before_host_or_consumption(self):
        (self.root / "target/debug/libriotbox_silent_alsa_diag.so").write_bytes(b"changed")
        self.assert_rejected_without_host_or_consumption()

    def test_wrong_v4_protocol_and_missing_library_binding_reject(self):
        path = self.root / "docs/benchmarks/silent_host_observation_v4.json"
        original = path.read_bytes()
        path.write_bytes(b"{}")
        self.assert_rejected_without_host_or_consumption()
        path.write_bytes(original)
        del self.binding["binary_sha256"]["libriotbox_silent_alsa_diag.so"]
        self.write_binding()
        self.assert_rejected_without_host_or_consumption()
