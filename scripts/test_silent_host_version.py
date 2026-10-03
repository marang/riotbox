"""Frozen V2 compatibility and disabled execution; no host calls permitted."""

import hashlib
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError, load_protocol

ROOT = Path(__file__).resolve().parents[1]
V1_HASH = "ba26bc50db85a6b745738917100689a7ca5ac3c2aa95cc7c699db2ef404358d3"
V2_HASH = "9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea"


class VersionTests(unittest.TestCase):
    def test_v2_loader_binds_new_contract_without_changing_v1_budgets(self):
        legacy = (ROOT / "docs/benchmarks/silent_host_observation_v1.json").read_bytes()
        self.assertEqual(hashlib.sha256(legacy).hexdigest(), V1_HASH)
        self.assertEqual(hashlib.sha256(
            (ROOT / "docs/benchmarks/silent_host_observation_v2.json").read_bytes()).hexdigest(), V2_HASH)
        v2 = load_protocol(ROOT / "docs/benchmarks/silent_host_observation_v2.json")
        self.assertEqual(v2["schema"], "riotbox.silent_host_observation.v2")
        self.assertEqual(v2["execution_policy"], "source_free_validation_only")
        self.assertEqual(v2["predecessor_protocol_sha256"], V1_HASH)
        for key, expected in json.loads(legacy).items():
            if key not in {"schema", "schema_version"}:
                self.assertEqual(v2[key], expected, key)
        with self.assertRaises(EvidenceError):
            load_protocol(ROOT / "docs/benchmarks/silent_host_observation_v1.json")

    def test_old_opt_in_cannot_start_v2_or_create_attempt_evidence(self):
        with patch.object(operator, "execute_attempt") as execute, \
                patch.object(operator, "HostCommands") as host, \
                patch.object(operator.Path, "mkdir") as mkdir, \
                patch.object(operator.subprocess, "run") as command, \
                patch.object(operator.sys, "stderr", io.StringIO()) as error:
            status = operator.main(["--execute-reviewed-attempt"])
            self.assertEqual(status, 1)
            self.assertIn("source-free", error.getvalue())
            execute.assert_not_called()
            host.assert_not_called()
            mkdir.assert_not_called()
            command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
