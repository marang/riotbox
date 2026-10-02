"""Artifact orchestration tests: synthetic metadata/PCM delivery, no original access."""

import contextlib
import copy
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import call, patch

import master_bus_limiter_calibration_runner as calibration
from master_bus_limiter_calibration_metrics import Version
import run_master_bus_limiter_review_artifacts_v1 as phase
import source_holdout_development_access as access_guard
from source_holdout_development_access import SourceIdentity


def contracts(*, repository_root=None):
    # Only tracked contracts, never the ignored metadata/audio paths they contain.
    contract = json.loads((phase.REPO / phase.PROTOCOL).read_text())
    parent = json.loads((phase.REPO / phase.EXPECTED_BASIS["protocol_path"]).read_text())
    contract["repository_root"] = str(repository_root if repository_root is not None else phase.REPO)
    return contract, parent


def preparation(token="Ab12Cd"):
    actions = [{"id": index + 1, "command": f"Synthetic{index}", "result": {"summary": "unchanged"}}
               for index in range(11)]
    actions[4].update(command="PromoteCaptureToPad", result={"summary":
        f"capture 1 bar from source into W-30 path | audio artifact written captures/capture-{token}.wav | promoted to pad bank-a/pad-01"})
    return {"committed_actions": actions, "commit_records": [{"action_id": 5, "committed_at": 50}],
            "capture_audio_identity": {"sha256": "sha256:" + "c" * 64},
            "capture_window": {"start_frame": 0, "end_frame": 88200}, "source_timing": {"confirmed_bpm": 120.0}}


def measured_case():
    conditions = []
    for name, _ in Version.V2.conditions:
        shared = calibration.sha256(f"generated-{name}".encode())
        conditions.append({"condition": name, "input_sha256_f32le": shared, "outputs": [
            {"policy": policy, "output_sha256_f32le": shared if name != "stress_4x"
             else calibration.sha256(f"generated-{name}-{policy}".encode()),
             "limiter": {"limited_sample_count": 2 if name == "stress_4x" else 0}}
            for policy in ["A", "B", "C"]]})
    return {"case_id": phase.CASE_ID, "protocol_version": "v2", "frame_count": 192000,
            "sample_rate_hz": 48000, "channels": 2,
            "controls": {"repeat_128_bit_exact": True, "partition_257_bit_exact": True,
                         "baseline_api_bit_exact": True},
            "conditions": conditions, "preparation": preparation(), "human_verdict": "unverified"}


def synthetic_phase(*, repository_root=None):
    contract, parent = contracts(repository_root=repository_root)
    case = copy.deepcopy(next(row for row in parent["cases"] if row["case_id"] == phase.CASE_ID))
    prior = measured_case()
    prior["preparation"] = preparation("Ef34Gh")
    prior["access"] = {"source_path": case["source_path"], "actual_sha256": case["sha256"],
                       "actual_source_format": {"sample_width_bits": 16}}
    for row in prior["conditions"][:2]:
        case["historical_controls"][f"{row['condition']}_pcm_sha256_f32le"] = row["input_sha256_f32le"]
    last = prior["conditions"][2]
    contract["full_render_identity"].update(stress_4x_input_sha256_f32le=last["input_sha256_f32le"],
        stress_4x_output_sha256_f32le={row["policy"]: row["output_sha256_f32le"] for row in last["outputs"]})
    parent["cases"] = [case if row["case_id"] == phase.CASE_ID else row for row in parent["cases"]]
    report = {"schema": "riotbox.master_bus_limiter_calibration_report.v2",
              "status": "technical_comparison_complete_no_policy_selection",
              "protocol_sha256": phase.EXPECTED_BASIS["protocol_sha256"],
              "git_head": phase.EXPECTED_BASIS["implementation_commit"],
              "build": {"rustc_verbose_version": parent["build_rustc_verbose_version"]},
              "cases": [{"case_id": name} if name != phase.CASE_ID else prior for name in calibration.CASE_IDS]}
    identities = [SourceIdentity(name, f"generated-{index}.wav", "generated", "development", {})
                  for index, name in enumerate(calibration.CASE_IDS)]
    return phase.Preparation(contract, parent, case, prior, identities, {}), report


def run_generated_phase(*, measured_mutation=None, child_failure=False, publication_failure=False,
                        existing_output=False):
    prepared, _ = synthetic_phase()
    with tempfile.TemporaryDirectory(prefix="riotbox-review-artifact-phase-test-") as temp:
        repo = Path(temp)
        prepared.contract["repository_root"] = str(repo)
        prepared.contract["output_directory"] = "result"
        protocol_file = repo / "protocol.json"
        protocol_file.write_text("{}")
        binary = repo / "generated-executor"
        binary.write_bytes(b"not-an-executable")
        output = repo / "result"
        if existing_output:
            output.mkdir()
        raw = measured_case()
        for row in raw["conditions"]:
            for policy in row["outputs"]:
                policy["samples"] = [0.1, -0.1]  # Mock transport payload, not a full render claim.
        measured = measured_case()
        if measured_mutation is not None:
            measured_mutation(measured)
        raw["preparation"] = measured["preparation"]
        child = subprocess.CompletedProcess([], 0, json.dumps(raw).encode(), b"")
        if child_failure:
            child = subprocess.CompletedProcess([], 1, b"", b'RIOTBOX_LIMITER_CALIBRATION_FAILURE {"stage":"historical_controls","preparation":{"retained":true}}')
        events, opened = [], []

        def build(head, *, expected_rustc):
            events.append("build")
            if expected_rustc != prepared.parent["build_rustc_verbose_version"]:
                raise AssertionError("historical compiler was not bound")
            return binary, {"sha256": calibration.sha256(binary.read_bytes())}

        def tools(contract):
            events.append("tool_preflight")
            return {"generated_roundtrip": "passed"}

        def deliver(repo, path, digest, fmt, prefix, *, on_open, **kwargs):
            events.append("admitted")
            opened.append(str(path))
            on_open(repo / path)
            return b"generated-payload", {"actual_sha256": "generated"}

        def measure(*args, **kwargs):
            events.append("measure")
            if kwargs["version"] is not Version.V2 or kwargs["listening_frame_window"] != (0, 96000):
                raise AssertionError("measurement version/window changed")
            return copy.deepcopy(measured)

        def publish(raw, destination, contract, *, tool_preflight):
            events.append("publish")
            saved = json.loads((destination / contract["summary_report"]).read_text())
            if "case" not in saved or "historical_full_render_identity" not in saved:
                raise AssertionError("publication preceded retained identity evidence")
            artifacts = []
            for policy in ("A", "B", "C"):
                path = destination / f"{policy}.wav"
                data = f"generated mocked WAV {policy}".encode()
                path.write_bytes(data)
                artifacts.append({"policy": policy, "path": str(path), "sha256": calibration.sha256(data)})
                if publication_failure:
                    error = ValueError("generated safety rejection")
                    error.evidence = {"stage": "final_file_safety", "artifacts": artifacts}
                    raise error
            return {"artifacts": artifacts, "shared_gain": 0.5}

        failure = None
        with patch.object(phase, "REPO", repo), patch.object(calibration, "REPO", repo), \
                patch.object(phase, "PROTOCOL", "protocol.json"), \
                patch.object(phase, "PROTOCOL_SHA256", calibration.sha256(protocol_file.read_bytes())), \
                patch.object(phase, "preflight", return_value=prepared), \
                patch.object(calibration, "git_identity", return_value="generated-head"), \
                patch.object(calibration, "build_executor", side_effect=build), \
                patch.object(phase.artifact_io, "preflight_tools", side_effect=tools), \
                patch.object(phase.artifact_io, "publish_artifacts", side_effect=publish) as publisher, \
                patch.object(phase, "measure_case", side_effect=measure), \
                patch.object(access_guard, "assert_registry_pin_current"), \
                patch.object(access_guard, "validate_contained_source_file", side_effect=deliver), \
                patch.object(calibration.subprocess, "run", return_value=child), \
                contextlib.redirect_stdout(io.StringIO()):
            try:
                phase.execute()
            except ValueError as error:
                failure = error
        report_path = output / "artifact-report.json"
        report = json.loads(report_path.read_text()) if report_path.exists() else None
        review_path = output / "listening-review/review.json"
        review = json.loads(review_path.read_text()) if review_path.exists() else None
        metrics_path = output / "listening-review/metrics.json"
        metrics = json.loads(metrics_path.read_text()) if metrics_path.exists() else None
        access_path = output / "development-access.json"
        access = json.loads(access_path.read_text()) if access_path.exists() else None
        wavs = [name for name in ("A.wav", "B.wav", "C.wav") if (output / name).exists()]
        digest = calibration.sha256(report_path.read_bytes()) if report_path.exists() else None
        return {"report": report, "review": review, "metrics": metrics, "access": access,
                "wavs": wavs, "report_sha256": digest, "events": events, "opened": opened,
                "failure": failure, "publisher_calls": publisher.call_count}


class ArtifactPhaseIdentityTests(unittest.TestCase):
    def test_execution_rejects_unfrozen_wrong_pin_and_nonaccepted_status_before_metadata(self):
        for name, status in (("placeholder", "accepted"), ("wrong_pin", "accepted"),
                             ("not_accepted", "pre_execution_review")):
            contract, _ = contracts()
            contract["status"] = status
            payload = json.dumps(contract).encode()
            pin = {"placeholder": "execution-not-yet-frozen", "wrong_pin": "0" * 64,
                   "not_accepted": calibration.sha256(payload)}[name]
            with self.subTest(case=name), \
                    patch.object(phase, "read_contained_regular_file", return_value=payload), \
                    patch.object(phase, "PROTOCOL_SHA256", pin), \
                    patch.object(calibration, "metadata") as metadata:
                with self.assertRaisesRegex(ValueError, "not accepted|not frozen"):
                    phase.preflight(execute=True)
                metadata.assert_not_called()

    def test_current_fixed_contract_and_generated_previous_report(self):
        prepared, report = synthetic_phase()
        case = phase.validate_contract(prepared.contract, prepared.parent)
        self.assertEqual(case["case_id"], phase.CASE_ID)
        self.assertEqual(phase.validate_historical_case(prepared.contract, prepared.parent, case, report),
                         prepared.historical_case)

    def test_only_one_capture_token_is_normalized_without_mutating_raw_evidence(self):
        original = preparation()
        self.assertEqual(phase.normalized_preparation(original), phase.normalized_preparation(preparation("Z9y8X7")))
        self.assertIn("capture-Ab12Cd.wav", original["committed_actions"][4]["result"]["summary"])
        for token in ("abcde", "abcdefg", "abc_def", "../bad", "ABC-12"):
            with self.subTest(token=token), self.assertRaises(ValueError):
                phase.normalized_preparation(preparation(token))

    def test_every_other_preparation_field_and_all_twelve_pcm_identities_remain_exact(self):
        original = measured_case()
        previous = copy.deepcopy(original)
        previous["preparation"] = preparation("Ef34Gh")
        phase.verify_full_identity(original, previous)
        mutations = [
            lambda item: item["preparation"]["committed_actions"][4].update(id=6),
            lambda item: item["preparation"]["committed_actions"][4].update(command="Other"),
            lambda item: item["preparation"]["committed_actions"][0]["result"].update(summary="changed"),
            lambda item: item["preparation"]["capture_audio_identity"].update(sha256="sha256:" + "d" * 64),
            lambda item: item["preparation"]["capture_window"].update(end_frame=88201),
            lambda item: item["preparation"].update(new_field=True),
            lambda item: item["controls"].update(repeat_128_bit_exact=False),
        ]
        for condition in range(3):
            mutations.append(lambda item, index=condition: item["conditions"][index].update(input_sha256_f32le="0" * 64))
            for policy in range(3):
                mutations.append(lambda item, ci=condition, pi=policy:
                                 item["conditions"][ci]["outputs"][pi].update(output_sha256_f32le="0" * 64))
        for mutation in mutations:
            changed = copy.deepcopy(original)
            mutation(changed)
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                phase.verify_full_identity(changed, previous)

    def test_preflight_in_distinct_checkout_loads_only_sparse_and_exact_parent_metadata(self):
        with tempfile.TemporaryDirectory(prefix="riotbox-review-foreign-checkout-") as temp:
            repo = Path(temp)
            self.assertNotEqual(repo, phase.REPO)
            # Load tracked templates before patching REPO; all preflight reads are mocked below.
            prepared, previous = synthetic_phase(repository_root=repo)
            contract = prepared.contract
            contract["status"] = "accepted"
            payload = json.dumps(contract).encode()
            with patch.object(phase, "REPO", repo), patch.object(calibration, "REPO", repo), \
                    patch.object(phase, "read_contained_regular_file", return_value=payload) as read, \
                    patch.object(phase, "PROTOCOL_SHA256", calibration.sha256(payload)), \
                    patch.object(calibration, "metadata", side_effect=[prepared.parent, previous]) as metadata, \
                    patch.object(calibration, "load_registered_identities", return_value=prepared.identities) as identities, \
                    patch.object(calibration, "load_case_graph", return_value={}) as graph:
                result = phase.preflight(execute=True)
            self.assertEqual(result.case["case_id"], phase.CASE_ID)
            self.assertEqual(result.contract["repository_root"], str(repo))
            read.assert_called_once_with(repo, Path(phase.PROTOCOL), phase.PROTOCOL, maximum_bytes=8 * 1024 * 1024)
            graph.assert_called_once_with(prepared.case)
            identities.assert_called_once_with(prepared.parent, [phase.CASE_ID], prefix=phase.PROTOCOL)
            self.assertEqual(metadata.call_args_list, [
                call(phase.EXPECTED_BASIS["protocol_path"], phase.EXPECTED_BASIS["protocol_sha256"], repo=repo),
                call(phase.EXPECTED_BASIS["report_path"], phase.EXPECTED_BASIS["report_sha256"], repo=repo),
            ])

    def test_bad_historical_control_or_4x_origin_blocks_graph_and_source_admission(self):
        for condition in (0, 1, 2):
            prepared, previous = synthetic_phase()
            previous["cases"][2]["conditions"][condition]["outputs"][1]["output_sha256_f32le"] = "0" * 64
            payload = json.dumps(prepared.contract).encode()
            with self.subTest(condition=condition), \
                    patch.object(phase, "read_contained_regular_file", return_value=payload), \
                    patch.object(phase, "PROTOCOL_SHA256", calibration.sha256(payload)), \
                    patch.object(calibration, "metadata", side_effect=[prepared.parent, previous]) as metadata, \
                    patch.object(calibration, "load_registered_identities") as identities, \
                    patch.object(calibration, "load_case_graph") as graph:
                expected = "historical clean/2x control mismatch" if condition < 2 else "historical 4x fingerprints"
                with self.assertRaisesRegex(ValueError, expected):
                    phase.preflight()
                self.assertEqual(metadata.call_args_list, [
                    call(phase.EXPECTED_BASIS["protocol_path"], phase.EXPECTED_BASIS["protocol_sha256"], repo=phase.REPO),
                    call(phase.EXPECTED_BASIS["report_path"], phase.EXPECTED_BASIS["report_sha256"], repo=phase.REPO),
                ])
                identities.assert_not_called()
                graph.assert_not_called()


class ArtifactPhaseExecutionTests(unittest.TestCase):
    def test_success_three_mock_artifacts_metadata_only_review_and_stable_report_hash(self):
        result = run_generated_phase()
        self.assertIsNone(result["failure"])
        self.assertEqual(result["events"], ["build", "tool_preflight", "admitted", "measure", "publish"])
        self.assertEqual(result["opened"], ["generated-2.wav"])
        self.assertEqual(result["wavs"], ["A.wav", "B.wav", "C.wav"])
        self.assertEqual(result["report"]["status"], "artifact_preflight_complete_human_unverified")
        review = result["review"]
        self.assertEqual(review["artifact_report"]["sha256"], result["report_sha256"])
        self.assertEqual(len(review["artifacts"]["candidate_audio"]), 3)
        self.assertIsNone(review["source_file"])
        self.assertIsNone(result["metrics"]["source_file"])
        self.assertEqual(review["human_verdict"], "unverified")
        self.assertEqual(review["hook_after_two_bars"], "unverified")
        self.assertFalse(review["diagnostic_scope"]["playback_authorized"])
        self.assertFalse(review["quality_claim"])

    def test_regenerated_4x_or_preparation_mismatch_retains_case_and_never_publishes(self):
        for mutate in (
            lambda item: item["conditions"][2]["outputs"][0].update(output_sha256_f32le="0" * 64),
            lambda item: item["preparation"]["capture_window"].update(end_frame=88201),
        ):
            result = run_generated_phase(measured_mutation=mutate)
            self.assertIsInstance(result["failure"], ValueError)
            self.assertEqual(result["publisher_calls"], 0)
            self.assertEqual(result["wavs"], [])
            self.assertEqual(result["opened"], ["generated-2.wav"])
            self.assertEqual(result["access"]["access_status"], "aborted")
            self.assertEqual(result["report"]["status"], "failed_closed")
            self.assertIn("preparation", result["report"]["case"])
            self.assertEqual(len(result["report"]["pending_case"]["limiter_observations"]), 3)

    def test_child_failure_keeps_diagnostics_and_no_publication(self):
        result = run_generated_phase(child_failure=True)
        self.assertIn("executor rejected", str(result["failure"]))
        self.assertIn("historical_controls", result["report"]["pending_case"]["executor_stderr"])
        self.assertEqual(result["publisher_calls"], 0)
        self.assertEqual(result["opened"], ["generated-2.wav"])
        self.assertEqual(result["access"]["access_status"], "aborted")

    def test_publisher_failure_keeps_partial_wav_measurements_and_raw_preparation(self):
        result = run_generated_phase(publication_failure=True)
        self.assertIn("safety rejection", str(result["failure"]))
        self.assertEqual(result["wavs"], ["A.wav"])
        self.assertEqual(result["report"]["artifact_evidence"]["stage"], "final_file_safety")
        self.assertIn("preparation", result["report"]["case"])
        self.assertEqual(result["report"]["status"], "failed_closed")
        self.assertIsNone(result["review"])

    def test_existing_output_stops_before_build_tools_or_source_delivery(self):
        result = run_generated_phase(existing_output=True)
        self.assertIn("must be new", str(result["failure"]))
        self.assertEqual(result["events"], [])
        self.assertEqual(result["opened"], [])
        self.assertIsNone(result["report"])


if __name__ == "__main__":
    unittest.main()
