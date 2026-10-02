"""Generated PCM only; no product sources, devices or artifact-directory discovery."""

import copy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import numpy as np

from master_bus_limiter_calibration_metrics import (
    POLICIES, delta, level, measure_case, pcm, pcm_identity, spectral_fractions,
)
import run_master_bus_limiter_calibration_v1 as runner
from source_holdout_development_access import SourceIdentity


def generated_report():
    pre = np.tile(np.array([0.1, -0.1, 0.2, -0.2], dtype=np.float32), 6000)
    conditions = []
    for name, gain in (("clean", 1.0), ("stress_2x", 2.0)):
        conditions.append({"condition": name, "outputs": [
            {"policy": policy, "samples": (pre * np.float32(gain)).tolist(),
             "limiter": {"threshold_bits": int(np.float32(threshold).view(np.uint32)),
                         "ceiling_bits": int(np.float32(ceiling).view(np.uint32)),
                         "limited_sample_count": 0,
                         "post": {"active_samples": pre.size, "rms": 0.15 * gain}}}
            for policy, threshold, ceiling in POLICIES]})
    return {"sample_rate_hz": 48000, "channels": 2, "frame_count": 12000,
            "controls": {"repeat_128_bit_exact": True, "partition_257_bit_exact": True,
                         "baseline_api_bit_exact": True},
            "pre_samples": pre.tolist(), "conditions": conditions}


class CalibrationMetricsTests(unittest.TestCase):
    def test_generated_equal_clean_and_unexercised_stress_is_not_a_human_pass(self):
        result = measure_case(generated_report(), 12000)
        self.assertEqual(result["human_verdict"], "unverified")
        self.assertEqual(len(result["conditions"]), 2)
        for condition in result["conditions"]:
            self.assertEqual(len(condition["outputs"]), 3)
            self.assertTrue(all(row["delta_vs_a"]["bit_identical"] for row in condition["outputs"]))

    def test_degenerate_metrics_are_undefined_not_epsilon_normalized(self):
        zero = np.zeros(32, dtype=np.float32)
        self.assertIsNone(level(zero)["crest_factor"])
        self.assertIsNone(spectral_fractions(zero))
        self.assertIsNone(delta(zero, zero)["relative_delta_rms"])
        self.assertIsNone(delta(zero, zero)["pearson_correlation"])

    def test_inherited_activity_and_sparse_rms_gates(self):
        raw = generated_report()
        measure_case(raw, 12000, minimum_mix_rms=0.01)
        raw["conditions"][0]["outputs"][0]["limiter"]["post"]["rms"] = 0.001
        with self.assertRaisesRegex(ValueError, "inherited RMS"):
            measure_case(raw, 12000, minimum_mix_rms=0.01)
        raw = generated_report()
        raw["conditions"][0]["outputs"][0]["limiter"]["post"]["active_samples"] = 0
        with self.assertRaisesRegex(ValueError, "inactive"):
            measure_case(raw, 12000)

    def test_future_window_does_not_borrow_later_policy_difference(self):
        raw = generated_report()
        candidate = raw["conditions"][1]["outputs"][2]
        candidate["samples"][-1] = -0.3
        candidate["limiter"]["limited_sample_count"] = 1
        report = measure_case(raw, 12000, listening_frame_window=(0, 6000))
        measured = report["conditions"][1]["outputs"][2]
        self.assertFalse(measured["delta_vs_a"]["bit_identical"])
        self.assertTrue(measured["preselected_window"]["bit_identical_to_a"])
        self.assertEqual(measured["preselected_window"]["modified_samples"], 0)

    def test_known_spectrum_and_delta(self):
        tone = np.sin(np.arange(4800) * (2.0 * np.pi * 100.0 / 48000.0)).astype(np.float32)
        fractions = spectral_fractions(tone)
        self.assertGreater(fractions[0], 0.999)
        self.assertAlmostEqual(sum(fractions), 1.0)
        comparison = delta(tone, tone * np.float32(0.5))
        self.assertAlmostEqual(comparison["relative_delta_rms"], 0.5)
        self.assertAlmostEqual(comparison["pearson_correlation"], 1.0)

    def test_no_padding_nonfinite_or_channel_reinterpretation(self):
        for values, count in (([0.1], 2), ([0.1], 1), ([float("nan"), 0.0], 2),
                              ([[0.1, 0.2]], 2), ([float("inf"), 0.0], 2)):
            with self.assertRaises(ValueError):
                pcm(values, count)

    def test_signed_zero_identity_is_bitwise(self):
        positive = np.array([0.0, 0.0], dtype=np.float32)
        negative = np.array([-0.0, 0.0], dtype=np.float32)
        self.assertNotEqual(pcm_identity(positive), pcm_identity(negative))
        self.assertFalse(delta(positive, negative)["bit_identical"])

    def test_gate_mutations_fail_closed(self):
        original = generated_report()
        mutations = [
            lambda raw: raw["controls"].update(partition_257_bit_exact=False),
            lambda raw: raw.update(channels=1),
            lambda raw: raw.update(frame_count=12001),
            lambda raw: raw["conditions"].reverse(),
            lambda raw: raw["conditions"][0]["outputs"].reverse(),
            lambda raw: raw["conditions"][0]["outputs"][0]["limiter"].update(threshold_bits=0),
            lambda raw: raw["conditions"][0]["outputs"][0]["limiter"].update(limited_sample_count=1),
            lambda raw: raw["conditions"][0]["outputs"][0]["samples"].pop(),
            lambda raw: raw["conditions"][0]["outputs"][0]["samples"].__setitem__(0, 1.1),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                raw = copy.deepcopy(original)
                mutate(raw)
                with self.assertRaises(ValueError):
                    measure_case(raw, 12000)


class CalibrationExecutionTests(unittest.TestCase):
    def test_build_requires_feature_bound_artifact_and_unchanged_head(self):
        with tempfile.TemporaryDirectory(prefix="riotbox-calibration-build-test-") as temp:
            repo = Path(temp)
            executable = repo / "target" / "debug" / "dense_break_live_path_render"
            executable.parent.mkdir(parents=True)
            executable.write_bytes(b"generated-non-executable-test-artifact")
            artifact = {"reason": "compiler-artifact", "target": {"name": executable.name},
                        "executable": str(executable), "features": ["limiter-calibration"]}
            compiler = subprocess.CompletedProcess([], 0, "host: x86_64-unknown-linux-gnu\n", "")
            built = subprocess.CompletedProcess([], 0, json.dumps(artifact), "")
            with patch.object(runner, "REPO", repo), patch.object(runner, "git_identity", return_value="head"), \
                    patch.object(runner.subprocess, "run", side_effect=[compiler, built]) as command:
                binary, evidence = runner.build_executor("head")
                self.assertEqual(binary, executable)
                self.assertIn("--locked", command.call_args.args[0])
                self.assertEqual(evidence["sha256"], runner.sha256(executable.read_bytes()))
            artifact["features"] = []
            built.stdout = json.dumps(artifact)
            with patch.object(runner, "REPO", repo), \
                    patch.object(runner.subprocess, "run", side_effect=[compiler, built]):
                with self.assertRaisesRegex(ValueError, "feature absent"):
                    runner.build_executor("head")

    def test_failed_consumed_case_retains_diagnostics_and_build_precedes_access(self):
        with tempfile.TemporaryDirectory(prefix="riotbox-calibration-owner-test-") as temp:
            repo = Path(temp)
            executable = repo / "synthetic-executor"
            executable.write_bytes(b"test-only-never-executed")
            protocol_file = repo / "protocol.json"
            protocol_file.write_text("{}")
            protocol = {"output_directory": "result", "summary_report": "comparison.json",
                        "access_log": "access.json", "cases": [{"case_id": "generated", "frame_count": 12000}],
                        "protected_registry": {"path": "registry.json", "schema": "fixture", "sha256": "fixture"},
                        "budget": {"max_stdin_json_bytes": 10000, "max_wav_payload_bytes": 1000,
                                   "max_child_stdout_bytes": 10000, "max_child_stderr_bytes": 65536,
                                   "child_timeout_seconds": 1}}
            transitions = []

            def build(head):
                transitions.append("build")
                return executable, {"sha256": runner.sha256(executable.read_bytes())}

            def admitted(*args, **kwargs):
                transitions.append("admitted")
                kwargs["qualification_owner"](
                    SourceIdentity("generated", "not-opened.wav", "fixture", "development", {}),
                    b"generated-buffer", {"actual_sha256": "fixture"})
                self.fail("failed owner must not resume access")

            child = subprocess.CompletedProcess([], 1, b"", b'RIOTBOX_LIMITER_CALIBRATION_FAILURE {"writes":2}')
            with patch.object(runner, "REPO", repo), patch.object(runner, "PROTOCOL", "protocol.json"), \
                    patch.object(runner, "PROTOCOL_SHA256", runner.sha256(protocol_file.read_bytes())), \
                    patch.object(runner, "preflight", return_value=(protocol, [], {"generated": {}})), \
                    patch.object(runner, "git_identity", return_value="head"), \
                    patch.object(runner, "build_executor", side_effect=build), \
                    patch.object(runner, "run_development_access_session", side_effect=admitted), \
                    patch.object(runner.subprocess, "run", return_value=child):
                with self.assertRaisesRegex(ValueError, "executor rejected"):
                    runner.execute()
            report = json.loads((repo / "result/comparison.json").read_text())
            self.assertEqual(transitions, ["build", "admitted"])
            self.assertEqual(report["status"], "failed_closed")
            self.assertEqual(report["pending_case"]["case_id"], "generated")
            self.assertIn('"writes":2', report["pending_case"]["executor_stderr"])
            self.assertEqual(report["cases"], [])


if __name__ == "__main__":
    unittest.main()
