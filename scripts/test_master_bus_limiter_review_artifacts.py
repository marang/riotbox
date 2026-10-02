"""Generated PCM and private temp files only; never Development audio or playback."""

import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import numpy as np

import master_bus_limiter_review_artifacts as artifacts
import master_bus_limiter_calibration_metrics as metrics


def contract():
    # Repository preregistration metadata only; never open its linked paths.
    path = Path(__file__).resolve().parents[1] / "docs/benchmarks/master_bus_limiter_review_artifact_protocol_v1.json"
    return json.loads(path.read_text())


def generated_raw(amplitude=0.96):
    clock = np.arange(artifacts.FULL_FRAMES, dtype=np.float64) / artifacts.SAMPLE_RATE
    baseline = np.column_stack((amplitude * np.sin(2 * np.pi * 440 * clock),
                                amplitude * 0.8 * np.cos(2 * np.pi * 880 * clock))).astype(np.float32)
    candidate = (baseline * np.float32(0.99)
                 + (0.008 * np.sin(2 * np.pi * 3300 * clock))[:, None]).astype(np.float32)
    return {"case_id": "sparse_kicksnr_120", "protocol_version": "v2",
            "sample_rate_hz": 48000, "channels": 2, "frame_count": artifacts.FULL_FRAMES,
            "conditions": [{"condition": "clean", "outputs": []},
                           {"condition": "stress_2x", "outputs": []},
                           {"condition": "stress_4x", "outputs": [
                               {"policy": policy, "samples": samples.reshape(-1)}
                               for policy, samples in zip(artifacts.POLICIES,
                                   (baseline, candidate, baseline * np.float32(0.97)), strict=True)]}]}


class ArtifactTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = contract()
        cls.proof = artifacts.preflight_tools(cls.contract)

    def publish(self, raw, output, protocol=None):
        return artifacts.publish_artifacts(raw, output, protocol or self.contract,
                                           tool_preflight=self.proof)

    def test_generated_tool_preflight_proves_float_roundtrip_without_source_access(self):
        self.assertEqual(self.proof["source_accesses"], 0)
        self.assertEqual(self.proof["playbacks"], 0)
        self.assertTrue(self.proof["generated_roundtrip"]["decode_bit_exact"])
        self.assertEqual(self.proof["generated_roundtrip"]["ffprobe"]["streams"][0]["duration_ts"], 96000)
        self.assertEqual(set(self.proof["tools"]), {"ffmpeg", "ffprobe"})

    def test_three_real_float_wavs_preserve_common_gain_crop_and_exact_file_evidence(self):
        raw = generated_raw()
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-artifact-test-") as temp:
            output = Path(temp)
            report = self.publish(raw, output)
            self.assertEqual(sorted(path.name for path in output.iterdir()), ["A.wav", "B.wav", "C.wav"])
            self.assertEqual(report["status"], "technical_preflight_complete_no_playback")
            self.assertFalse(report["quality_proof"])
            self.assertEqual(report["human_verdict"], "unverified")
            gain = np.float32(report["presentation_safety"]["uniform_gain"])
            self.assertLess(gain, 1)
            crops, hashes = artifacts._crop_outputs(raw)
            for index, row in enumerate(report["artifacts"]):
                self.assertEqual(row["policy"], artifacts.POLICIES[index])
                self.assertEqual(row["full_raw_pcm_sha256_f32le"], hashes[index])
                self.assertEqual(row["raw_crop_pcm_sha256_f32le"], metrics.pcm_identity(crops[index]))
                expected = (crops[index] * gain).astype(np.float32)
                self.assertEqual(row["presented_pcm_sha256_f32le"], metrics.pcm_identity(expected))
                self.assertEqual(row["sha256"], hashlib.sha256(Path(row["path"]).read_bytes()).hexdigest())
                self.assertTrue(row["decode_bit_exact"])
                self.assertEqual(row["ffprobe"]["streams"][0]["codec_name"], "pcm_f32le")
                self.assertEqual(row["ffprobe"]["streams"][0]["duration_ts"], 96000)
                self.assertEqual(row["level"]["clip_count"], 0)
                self.assertGreater(row["level"]["rms_f64"], 0)
                self.assertLessEqual(row["conservative_true_peak_dbtp"], -1.0)
                self.assertLessEqual(row["ffmpeg_true_peak_dbfs"], -1.0)
                self.assertTrue(np.isfinite(row["integrated_lufs"]))
                self.assertEqual(len(row["per_channel_windows"]), 8)
                self.assertEqual([item["frames"] for item in row["per_channel_windows"]],
                                 [[0, 960]] * 2 + [[960, 5760]] * 2 + [[5760, 12000]] * 2 + [[0, 96000]] * 2)
            self.assertTrue(report["artifacts"][0]["delta_vs_a"]["bit_identical"])
            self.assertFalse(report["artifacts"][1]["delta_vs_a"]["bit_identical"])
            self.assertGreater(report["artifacts"][1]["delta_vs_a"]["absolute_delta_peak"], 0)
            json.dumps(report, allow_nan=False)

    def test_repeated_generated_publication_has_identical_wav_bytes(self):
        raw = generated_raw(0.2)
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-repeat-test-") as temp:
            reports = []
            for name in ("first", "second"):
                output = Path(temp) / name
                output.mkdir()
                reports.append(self.publish(raw, output))
            self.assertEqual([row["sha256"] for row in reports[0]["artifacts"]],
                             [row["sha256"] for row in reports[1]["artifacts"]])
            self.assertEqual(reports[0]["presentation_safety"]["uniform_gain"], 1.0)

    def test_condition_is_selected_by_id_and_tail_outside_crop_is_not_normalized(self):
        raw = generated_raw(0.2)
        raw["conditions"].reverse()
        for row in raw["conditions"][0]["outputs"]:
            row["samples"][artifacts.FRAMES * 2:] = 3.0
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-crop-test-") as temp:
            report = self.publish(raw, Path(temp))
            self.assertEqual(report["presentation_safety"]["uniform_gain"], 1.0)
            self.assertLess(report["artifacts"][0]["level"]["peak_abs"], 0.3)

    def test_mutated_fixed_contracts_fail_before_first_write(self):
        mutations = [
            lambda c: c.update(condition="clean"), lambda c: c.update(frame_window=[1, 96001]),
            lambda c: c["artifacts"][2].update(filename="D.wav"),
            lambda c: c["artifacts"][0].update(policy="B"),
            *[lambda c, key=key, value=value: c["format"].update({key: value})
              for key, value in (("codec", "pcm_s16le"), ("sample_rate_hz", 44100),
                                 ("channels", 1), ("frame_count", 95999), ("duration_seconds", 3.0))],
            *[lambda c, key=key, value=value: c["presentation"].update({key: value})
              for key, value in (("target_true_peak_dbtp", -0.5), ("maximum_true_peak_dbtp", 0),
                                 ("oversample_factor", 2), ("gain_rule", "normalize_each"))],
            lambda c: c["analysis"]["local_frame_windows"][0].__setitem__(2, 961),
        ]
        raw = generated_raw()
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-contract-test-") as temp:
            for mutate in mutations:
                with self.subTest(mutation=mutate):
                    changed = copy.deepcopy(self.contract)
                    mutate(changed)
                    with patch.object(artifacts, "_write_wav") as writer, self.assertRaises(artifacts.ArtifactPublicationError):
                        self.publish(raw, Path(temp), changed)
                    writer.assert_not_called()
            self.assertEqual(list(Path(temp).iterdir()), [])

    def test_all_destinations_preflight_including_dangling_symlink(self):
        raw = generated_raw()
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-existing-test-") as temp:
            output = Path(temp)
            (output / "B.wav").write_bytes(b"existing-not-audio")
            with self.assertRaisesRegex(artifacts.ArtifactPublicationError, "absent"):
                self.publish(raw, output)
            self.assertEqual((output / "B.wav").read_bytes(), b"existing-not-audio")
            self.assertFalse((output / "A.wav").exists())
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-symlink-test-") as temp:
            output = Path(temp)
            (output / "C.wav").symlink_to(output / "missing-generated-only")
            with self.assertRaisesRegex(artifacts.ArtifactPublicationError, "absent"):
                self.publish(raw, output)
            self.assertFalse((output / "A.wav").exists())

    def test_invalid_raw_and_silence_fail_before_any_wav(self):
        for failure in ("case", "duplicate", "policies", "nonfinite", "silent", "alignment"):
            raw = generated_raw()
            rows = raw["conditions"][-1]["outputs"]
            if failure == "case": raw["case_id"] = "dense_beat03_130"
            elif failure == "duplicate": raw["conditions"].append(raw["conditions"][-1])
            elif failure == "policies": rows.reverse()
            elif failure == "nonfinite": rows[0]["samples"][0] = np.nan
            elif failure == "silent": rows[0]["samples"][:] = 0
            else: rows[0]["samples"] = rows[0]["samples"][:-1]
            with self.subTest(failure=failure), tempfile.TemporaryDirectory(prefix="riotbox-limiter-invalid-test-") as temp:
                with self.assertRaises(artifacts.ArtifactPublicationError):
                    self.publish(raw, Path(temp))
                self.assertEqual(list(Path(temp).iterdir()), [])

    def test_late_writer_failure_preserves_first_metrics_and_never_writes_third(self):
        real_writer = artifacts._write_wav
        calls = []
        def fail_second(path, samples):
            calls.append(path.name)
            if path.name == "B.wav":
                path.write_bytes(b"synthetic-partial-write")
                raise ValueError("generated writer failure")
            real_writer(path, samples)
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-failed-test-") as temp:
            with patch.object(artifacts, "_write_wav", side_effect=fail_second), self.assertRaises(artifacts.ArtifactPublicationError) as caught:
                self.publish(generated_raw(), Path(temp))
            evidence = caught.exception.evidence
            self.assertEqual(evidence["failed_stage"], "write_B")
            self.assertEqual(evidence["artifacts"][0]["status"], "technical_preflight_pass")
            self.assertIn("integrated_lufs", evidence["artifacts"][0])
            self.assertEqual((Path(temp) / "B.wav").read_bytes(), b"synthetic-partial-write")
            self.assertEqual(calls, ["A.wav", "B.wav"])
            self.assertFalse((Path(temp) / "C.wav").exists())

    def test_written_byte_mismatch_preserves_hash_and_stops(self):
        real_writer = artifacts._write_wav
        def wrong_pcm(path, samples):
            real_writer(path, samples * np.float32(0.5))
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-mismatch-test-") as temp:
            with patch.object(artifacts, "_write_wav", side_effect=wrong_pcm), self.assertRaises(artifacts.ArtifactPublicationError) as caught:
                self.publish(generated_raw(), Path(temp))
            evidence = caught.exception.evidence
            self.assertEqual(evidence["failed_stage"], "analyze_A")
            self.assertIn("sha256", evidence["artifacts"][0])
            self.assertIn("presented_pcm_sha256_f32le", evidence["artifacts"][0])
            self.assertFalse((Path(temp) / "B.wav").exists())

    def test_unsafe_written_fft_estimate_stops_without_attenuation_retry(self):
        estimator = artifacts.dense.conservative_true_peak_amplitude
        calls = 0
        def fail_exact_file(signal):
            nonlocal calls
            calls += 1
            return 1.1 if calls == 4 else estimator(signal)
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-fft-gate-test-") as temp:
            with patch.object(artifacts.dense, "conservative_true_peak_amplitude", side_effect=fail_exact_file), self.assertRaises(artifacts.ArtifactPublicationError) as caught:
                self.publish(generated_raw(), Path(temp))
            evidence = caught.exception.evidence
            self.assertEqual(evidence["failed_stage"], "analyze_A")
            self.assertGreater(evidence["artifacts"][0]["conservative_true_peak_dbtp"], 0)
            self.assertTrue(evidence["artifacts"][0]["decode_bit_exact"])
            self.assertEqual(calls, 4)
            self.assertEqual(sorted(path.name for path in Path(temp).iterdir()), ["A.wav"])

    def test_unsafe_ffmpeg_true_peak_retains_lufs_and_stops_without_retry(self):
        tool = artifacts._tool
        def unsafe_peak(args, *positional, **keywords):
            if "ebur128=peak=true" in args:
                return b"", b"Summary:\nIntegrated loudness:\n I: -12.0 LUFS\nTrue peak:\n Peak: -0.8 dBFS\n"
            return tool(args, *positional, **keywords)
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-ebur-gate-test-") as temp:
            with patch.object(artifacts, "_tool", side_effect=unsafe_peak), self.assertRaises(artifacts.ArtifactPublicationError) as caught:
                self.publish(generated_raw(), Path(temp))
            evidence = caught.exception.evidence
            self.assertEqual(evidence["failed_stage"], "analyze_A")
            self.assertEqual(evidence["artifacts"][0]["integrated_lufs"], -12.0)
            self.assertEqual(evidence["artifacts"][0]["ffmpeg_true_peak_dbfs"], -0.8)
            self.assertLessEqual(evidence["artifacts"][0]["conservative_true_peak_dbtp"], -1.0)
            self.assertEqual(sorted(path.name for path in Path(temp).iterdir()), ["A.wav"])

    def test_live_tool_caps_and_deadline_fail_closed(self):
        with self.assertRaisesRegex(ValueError, "stdout byte limit"):
            artifacts._tool([sys.executable, "-c", "import os; os.write(1, b'x'*8192)"], stdout_limit=32)
        with self.assertRaisesRegex(ValueError, "stderr byte limit"):
            artifacts._tool([sys.executable, "-c", "import os; os.write(2, b'x'*8192)"], stderr_limit=32)
        with patch.object(artifacts, "TOOL_TIMEOUT_SECONDS", 0.05), self.assertRaisesRegex(ValueError, "timeout"):
            artifacts._tool([sys.executable, "-c", "import time; time.sleep(2)"])

    def test_nonfinite_external_metrics_remain_strict_json_failure_evidence(self):
        tool = artifacts._tool
        def invalid_peak(args, *positional, **keywords):
            if "ebur128=peak=true" in args:
                return b"", b"Summary:\nIntegrated loudness:\n I: -inf LUFS\nTrue peak:\n Peak: nan dBFS\n"
            return tool(args, *positional, **keywords)
        with tempfile.TemporaryDirectory(prefix="riotbox-limiter-nonfinite-test-") as temp:
            with patch.object(artifacts, "_tool", side_effect=invalid_peak), self.assertRaises(artifacts.ArtifactPublicationError) as caught:
                self.publish(generated_raw(), Path(temp))
            evidence = caught.exception.evidence
            self.assertIsNone(evidence["artifacts"][0]["integrated_lufs"])
            self.assertIsNone(evidence["artifacts"][0]["ffmpeg_true_peak_dbfs"])
            json.dumps(evidence, allow_nan=False)
            self.assertFalse((Path(temp) / "B.wav").exists())


if __name__ == "__main__":
    unittest.main()
