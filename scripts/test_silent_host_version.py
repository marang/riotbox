"""Frozen V1/V2 compatibility and source-free V3; no host calls permitted."""

import hashlib
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import run_silent_host_probe as operator
from silent_host_evidence import EvidenceError, load_protocol, load_protocol_v3

ROOT = Path(__file__).resolve().parents[1]
V1_HASH = "ba26bc50db85a6b745738917100689a7ca5ac3c2aa95cc7c699db2ef404358d3"
V2_HASH = "9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea"
V3_HASH = "a0934097be7b2f160ff698b3d69c1dd7180e86defcdc0974c5e23606297ef85b"


class VersionTests(unittest.TestCase):
    def test_v3_keeps_all_predecessor_operational_values_and_cannot_load_v2(self):
        path = ROOT / "docs/benchmarks/silent_host_observation_v3.json"
        self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), V3_HASH)
        v3 = load_protocol_v3(path)
        v2 = load_protocol(ROOT / "docs/benchmarks/silent_host_observation_v2.json")
        self.assertEqual(v3["schema"], "riotbox.silent_host_observation.v3")
        self.assertEqual(v3["execution_policy"], "source_free_validation_only")
        self.assertEqual(v3["predecessor_protocol_sha256"], V2_HASH)
        changed = {"schema", "schema_version", "predecessor_protocol_sha256", "module_identity"}
        self.assertEqual(v3.keys(), v2.keys())
        for key in v2.keys() - changed:
            self.assertEqual(v3[key], v2[key], key)
        with self.assertRaises(EvidenceError):
            load_protocol(path)
        with self.assertRaises(EvidenceError):
            operator.load_protocol(ROOT / "docs/benchmarks/silent_host_observation_v2.json")

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
