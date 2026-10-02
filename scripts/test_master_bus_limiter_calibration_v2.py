"""Generated PCM/mocked metadata only; never call the real Development preflight."""

import copy
import json
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import numpy as np

import master_bus_limiter_calibration_runner as runner
import source_holdout_development_access as access_guard
from master_bus_limiter_calibration_metrics import Version, measure_case, pcm, pcm_identity
from source_holdout_development_access import SourceIdentity
from test_master_bus_limiter_calibration import generated_report


def historical_controls(raw):
    pre = pcm(raw["pre_samples"], raw["frame_count"] * 2)
    return {f"{name}_pcm_sha256_f32le": pcm_identity(pre * np.float32(gain))
            for name, gain in Version.V1.conditions}


def protocol_contracts():
    # Versioned repository contracts are safe inputs; no referenced files are opened.
    return tuple(json.loads((runner.REPO / runner.PROTOCOLS[version].path).read_text())
                 for version in Version)


def historical_report(protocol):
    """Construct report-shaped origin evidence without reading the actual report."""
    report = {"schema": "riotbox.master_bus_limiter_calibration_report.v1",
              "status": "technical_comparison_complete_no_policy_selection",
              "protocol_sha256": protocol["historical_basis"]["protocol_sha256"],
              "git_head": protocol["historical_basis"]["implementation_commit"],
              "build": {"rustc_verbose_version": protocol["build_rustc_verbose_version"]}, "cases": []}
    for case in protocol["cases"]:
        report["cases"].append({
            "case_id": case["case_id"], "frame_count": case["frame_count"],
            "sample_rate_hz": 48000, "channels": 2,
            "controls": {"repeat_128_bit_exact": True, "partition_257_bit_exact": True,
                         "baseline_api_bit_exact": True},
            "access": {"actual_sha256": case["sha256"],
                       "actual_source_format": {"sample_width_bits": case["sample_width_bits"]}},
            "conditions": [{"condition": name,
                            "input_sha256_f32le": case["historical_controls"][f"{name}_pcm_sha256_f32le"],
                            "outputs": [{"policy": policy,
                                         "output_sha256_f32le": case["historical_controls"][f"{name}_pcm_sha256_f32le"]}
                                        for policy in ("A", "B", "C")]}
                           for name, _ in Version.V1.conditions],
        })
    return report


def generated_children():
    children = []
    for case_id in runner.CASE_IDS:
        raw = generated_report(Version.V2)
        raw.update(case_id=case_id, preparation={"actions": [10, 20], "capture": "generated-only"})
        children.append(raw)
    return children


def execute_generated_session(children, *, failed_child=False):
    """Real access-owner loop, but generated deliveries and no actual audio/registry reads."""
    _, protocol = protocol_contracts()
    with tempfile.TemporaryDirectory(prefix="riotbox-calibration-v2-test-") as temp:
        repo = Path(temp)
        protocol_file = repo / "protocol.json"
        protocol_file.write_text("{}")
        executable = repo / "synthetic-executor"
        executable.write_bytes(b"never-executed-generated-artifact")
        protocol["output_directory"] = "result"
        protocol["preselected_future_listening"]["frame_window"] = [0, 6000]
        for case in protocol["cases"]:
            case["frame_count"] = 12000
            case["historical_controls"] = historical_controls(generated_report(Version.V2))
        identities = [SourceIdentity(case_id, f"generated-{index}.wav", "generated", "development", {})
                      for index, case_id in enumerate(runner.CASE_IDS)]
        opened = []
        transitions = []

        def build(head, *, expected_rustc):
            if expected_rustc != protocol["build_rustc_verbose_version"]:
                raise AssertionError("expected compiler binding was not forwarded")
            transitions.append("build")
            return executable, {"sha256": runner.sha256(executable.read_bytes())}

        def generated_delivery(repo, path, digest, fmt, prefix, *, on_open, **kwargs):
            transitions.append("admitted")
            opened.append(str(path))
            on_open(repo / path)
            return b"generated-buffer-not-a-source-read", {"actual_sha256": "generated"}

        responses = [subprocess.CompletedProcess([], 0, json.dumps(raw, allow_nan=False).encode(), b"")
                     for raw in children]
        if failed_child:
            envelope = {"preparation": children[0]["preparation"],
                        "diagnostics": {name: {"available": True} for name, _ in Version.V2.conditions}}
            responses[0] = subprocess.CompletedProcess([], 1, b"",
                b"RIOTBOX_LIMITER_CALIBRATION_FAILURE " + json.dumps(envelope).encode())
        binding = runner.ProtocolBinding("protocol.json", runner.sha256(protocol_file.read_bytes()))
        failure = None
        with patch.object(runner, "REPO", repo), patch.dict(runner.PROTOCOLS, {Version.V2: binding}), \
                patch.object(runner, "preflight", return_value=(protocol, identities, dict.fromkeys(runner.CASE_IDS, {}))), \
                patch.object(runner, "git_identity", return_value="generated-head"), \
                patch.object(runner, "build_executor", side_effect=build), \
                patch.object(access_guard, "assert_registry_pin_current"), \
                patch.object(access_guard, "validate_contained_source_file", side_effect=generated_delivery), \
                patch.object(runner.subprocess, "run", side_effect=responses) as child:
            try:
                runner.execute(Version.V2)
            except ValueError as error:
                failure = error
        report = json.loads((repo / "result/comparison.json").read_text())
        access = json.loads((repo / "result/development-access.json").read_text())
        return report, access, opened, transitions, child.call_args_list, failure


class CalibrationV2MetricsTests(unittest.TestCase):
    def test_v1_default_preserved_and_v2_is_exactly_three_fixed_conditions(self):
        self.assertEqual(measure_case(generated_report(), 12000),
                         measure_case(generated_report(), 12000, version=Version.V1))
        raw = generated_report(Version.V2)
        result = measure_case(raw, 12000, version=Version.V2,
                              historical_controls=historical_controls(raw))
        self.assertEqual([row["condition"] for row in result["conditions"]],
                         ["clean", "stress_2x", "stress_4x"])
        self.assertEqual(result["conditions"][2]["input_sha256_f32le"],
                         pcm_identity(pcm(raw["pre_samples"], 24000) * np.float32(4.0)))
        self.assertEqual(result["human_verdict"], "unverified")
        self.assertTrue(all(row["delta_vs_a"]["bit_identical"]
                            for condition in result["conditions"] for row in condition["outputs"]))

    def test_v2_rejects_wrong_version_condition_set_or_historical_pcm(self):
        original = generated_report(Version.V2)
        fingerprints = historical_controls(original)
        mutations = [
            lambda raw: raw.update(protocol_version="v1"),
            lambda raw: raw["conditions"].reverse(),
            lambda raw: raw["conditions"].pop(),
            lambda raw: raw["conditions"].append(copy.deepcopy(raw["conditions"][-1])),
            lambda raw: raw["conditions"][-1].update(condition="stress_8x"),
            lambda raw: raw["pre_samples"].__setitem__(0, 0.11),
            lambda raw: raw["conditions"][1]["outputs"][1]["samples"].__setitem__(0, 0.19),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                raw = copy.deepcopy(original)
                mutate(raw)
                with self.assertRaises(ValueError):
                    measure_case(raw, 12000, version=Version.V2, historical_controls=fingerprints)
        with self.assertRaisesRegex(ValueError, "historical input"):
            measure_case(original, 12000, version=Version.V2,
                         historical_controls={key: "0" * 64 for key in fingerprints})
        with self.assertRaisesRegex(ValueError, "unknown calibration version"):
            measure_case(original, 12000, version="v2")

    def test_v2_selected_window_does_not_borrow_later_4x_changes(self):
        raw = generated_report(Version.V2)
        candidate = raw["conditions"][2]["outputs"][2]
        candidate["samples"][-1] = -0.7
        candidate["limiter"]["limited_sample_count"] = 1
        result = measure_case(raw, 12000, version=Version.V2,
                              historical_controls=historical_controls(raw),
                              listening_frame_window=(0, 6000))
        selected = result["conditions"][2]["outputs"][2]
        self.assertFalse(selected["delta_vs_a"]["bit_identical"])
        self.assertTrue(selected["preselected_window"]["bit_identical_to_a"])
        self.assertEqual(selected["preselected_window"]["modified_samples"], 0)


class CalibrationV2ContractTests(unittest.TestCase):
    def test_current_contracts_and_generated_historical_metadata_agree(self):
        v1, v2 = protocol_contracts()
        self.assertEqual(runner.sha256((runner.REPO / runner.PROTOCOLS[Version.V1].path).read_bytes()),
                         runner.PROTOCOLS[Version.V1].sha256)
        runner.validate_v2_contract(v2, v1)
        runner.validate_historical_report(v2, historical_report(v2))

    def test_fixed_gain_budget_width_window_and_parent_contract_mutations_rejected(self):
        v1, original = protocol_contracts()
        mutations = [
            lambda item: item["render"]["conditions"][-1].update(gain=8.0),
            lambda item: item["render"].update(stress_4x_gain_f32_bits="41000000"),
            lambda item: item["budget"].update(original_file_opens=4),
            lambda item: item["budget"].update(runtime_mix_renders=12),
            lambda item: item["budget"].update(distinct_policy_outputs=36),
            lambda item: item["budget"].update(max_child_stdout_bytes=67108864),
            lambda item: item["cases"][1].update(sample_width_bits=24),
            lambda item: item["cases"][1].update(allowed_sample_width_bits=[16, 24]),
            lambda item: item["cases"][0].update(start_beat=9),
            lambda item: item["cases"][0]["historical_controls"].update(clean_pcm_sha256_f32le="new"),
            lambda item: item["preselected_future_listening"].update(condition="stress_2x"),
            lambda item: item["preselected_future_listening"].update(frame_window=[96000, 192000]),
            lambda item: item["historical_basis"].update(protocol_sha256="0" * 64),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                changed = copy.deepcopy(original)
                mutate(changed)
                with self.assertRaises(ValueError):
                    runner.validate_v2_contract(changed, v1)

    def test_historical_origin_width_and_every_policy_fingerprint_are_checked(self):
        _, protocol = protocol_contracts()
        original = historical_report(protocol)
        mutations = [
            lambda report: report.update(status="failed_closed"),
            lambda report: report.update(git_head="0" * 40),
            lambda report: report.update(protocol_sha256="0" * 64),
            lambda report: report["build"].update(rustc_verbose_version="another compiler"),
            lambda report: report["cases"].reverse(),
            lambda report: report["cases"][1]["access"]["actual_source_format"].update(sample_width_bits=24),
            lambda report: report["cases"][0]["conditions"][0].update(input_sha256_f32le="0" * 64),
            lambda report: report["cases"][0]["conditions"][1]["outputs"][2].update(output_sha256_f32le="0" * 64),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                changed = copy.deepcopy(original)
                mutate(changed)
                with self.assertRaises(ValueError):
                    runner.validate_historical_report(protocol, changed)

    def test_compiler_mismatch_stops_before_build(self):
        compiler = subprocess.CompletedProcess([], 0, "rustc wrong\nhost: x86_64-unknown-linux-gnu\n", "")
        with patch.object(runner.subprocess, "run", return_value=compiler) as command:
            with self.assertRaisesRegex(ValueError, "native compiler differs"):
                runner.build_executor("head", expected_rustc="rustc frozen")
        self.assertEqual(command.call_count, 1)
        self.assertEqual(command.call_args.args[0], ["rustc", "-vV"])

    def test_bad_historical_report_stops_before_graph_metadata_build_or_access(self):
        v1, protocol = protocol_contracts()
        prior = historical_report(protocol)
        prior["cases"][0]["conditions"][1]["outputs"][2]["output_sha256_f32le"] = "0" * 64
        with tempfile.TemporaryDirectory(prefix="riotbox-calibration-preflight-test-") as temp:
            repo = Path(temp)
            protocol["repository_root"] = str(repo)
            protocol["status"] = "accepted"
            payload = json.dumps(protocol).encode()
            (repo / "protocol.json").write_bytes(payload)
            binding = runner.ProtocolBinding("protocol.json", runner.sha256(payload))
            with patch.object(runner, "REPO", repo), patch.dict(runner.PROTOCOLS, {Version.V2: binding}), \
                    patch.object(runner, "metadata", side_effect=[v1, prior]) as metadata, \
                    patch.object(runner, "build_executor") as build, \
                    patch.object(runner, "run_development_access_session") as access:
                with self.assertRaisesRegex(ValueError, "historical policy output"):
                    runner.execute(Version.V2)
                self.assertEqual(metadata.call_count, 2)
                self.assertEqual(metadata.call_args.args[:2],
                                 (protocol["historical_basis"]["report_path"], protocol["historical_basis"]["report_sha256"]))
                build.assert_not_called()
                access.assert_not_called()
            self.assertFalse((repo / protocol["output_directory"]).exists())

    def test_metadata_digest_and_unfrozen_v2_reject_before_metadata_admission(self):
        with patch.object(runner, "read_contained_regular_file", return_value=b"{}") as read:
            with self.assertRaisesRegex(ValueError, "metadata pin mismatch"):
                runner.metadata("generated.json", "0" * 64, repo=Path("/synthetic-not-opened"))
            self.assertEqual(read.call_args.kwargs["maximum_bytes"], 8 * 1024 * 1024)
        with tempfile.TemporaryDirectory(prefix="riotbox-calibration-unfrozen-test-") as temp:
            repo = Path(temp)
            (repo / "protocol.json").write_text("{}")
            binding = runner.ProtocolBinding("protocol.json", "execution-not-yet-frozen")
            with patch.object(runner, "REPO", repo), patch.dict(runner.PROTOCOLS, {Version.V2: binding}), \
                    patch.object(runner, "metadata") as metadata:
                with self.assertRaisesRegex(ValueError, "not frozen"):
                    runner.preflight(Version.V2, execute=True)
                metadata.assert_not_called()

    def test_thin_entrypoints_select_closed_version_without_preflight(self):
        for version in Version:
            with self.subTest(version=version), patch.object(runner, "main") as main:
                runpy.run_path(str(runner.REPO / "scripts" / f"run_master_bus_limiter_calibration_{version.value}.py"),
                               run_name="__main__")
                main.assert_called_once_with(version)


class CalibrationV2ExecutionTests(unittest.TestCase):
    def test_success_keeps_27_outputs_and_selects_4x_not_second_condition(self):
        children = generated_children()
        changed = children[2]["conditions"][2]["outputs"][2]
        changed["samples"][0] = 0.3
        changed["limiter"]["limited_sample_count"] = 1
        report, access, opened, transitions, calls, failure = execute_generated_session(children)
        self.assertIsNone(failure)
        self.assertEqual(report["schema"], "riotbox.master_bus_limiter_calibration_report.v2")
        self.assertEqual(report["status"], "technical_comparison_complete_no_policy_selection")
        self.assertEqual(len(opened), 3)
        self.assertEqual(transitions, ["build", "admitted", "admitted", "admitted"])
        self.assertEqual(access["qualification_owner"]["owner_id"], "riotbox-1501-limiter-calibration-v2")
        self.assertTrue(all(call.args[0][-1] == "--limiter-calibration-v2" for call in calls))
        self.assertEqual(sum(len(condition["outputs"]) for case in report["cases"]
                             for condition in case["conditions"]), 27)
        self.assertTrue(report["preselected_future_listening"]["protection_observed_any_policy"])
        self.assertTrue(report["preselected_future_listening"]["any_policy_differs_from_a"])
        self.assertFalse(report["preselected_future_listening"]["artifact_generated"])

    def test_python_gate_failure_retains_all_conditions_and_stops_real_access_owner(self):
        children = generated_children()
        children[0]["conditions"][2]["outputs"][1]["limiter"]["limited_sample_count"] = 1
        report, access, opened, transitions, calls, failure = execute_generated_session(children)
        self.assertIn("actual write-count mismatch", str(failure))
        self.assertEqual(report["status"], "failed_closed")
        self.assertEqual(report["cases"], [])
        self.assertEqual([row["condition"] for row in report["pending_case"]["limiter_observations"]],
                         ["clean", "stress_2x", "stress_4x"])
        self.assertEqual(report["pending_case"]["preparation"], children[0]["preparation"])
        self.assertEqual(opened, ["generated-0.wav"])
        self.assertEqual(transitions, ["build", "admitted"])
        self.assertEqual(len(calls), 1)
        self.assertEqual(access["access_status"], "aborted")

    def test_child_failure_retains_all_available_diagnostics_and_stops_remaining_cases(self):
        children = generated_children()
        report, access, opened, _, calls, failure = execute_generated_session(children, failed_child=True)
        self.assertIn("executor rejected", str(failure))
        self.assertEqual(report["status"], "failed_closed")
        for marker in ("preparation", "clean", "stress_2x", "stress_4x"):
            self.assertIn(marker, report["pending_case"]["executor_stderr"])
        self.assertFalse(report["pending_case"]["stderr_truncated"])
        self.assertEqual(opened, ["generated-0.wav"])
        self.assertEqual(len(calls), 1)
        self.assertEqual(access["access_status"], "aborted")


if __name__ == "__main__":
    unittest.main()
