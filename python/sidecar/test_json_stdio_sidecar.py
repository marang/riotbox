import hashlib
import io
import re
import tempfile
import unittest
import wave
from pathlib import Path
from unittest import mock

import json_stdio_sidecar as sidecar


class SidecarClockContractTests(unittest.TestCase):
    def test_graph_generation_uses_injected_clock_once(self) -> None:
        calls = []

        def fixed_clock() -> str:
            calls.append("called")
            return "2030-01-02T03:04:05.678Z"

        response = sidecar.handle_message(
            {
                "type": "build_source_graph_stub",
                "request_id": "req-clock",
                "source": {
                    "source_id": "src-clock",
                    "path": "fixture.wav",
                    "content_hash": "sha256:clock",
                    "duration_seconds": 4.0,
                    "sample_rate": 48000,
                    "channel_count": 2,
                    "decode_profile": "NormalizedStereo",
                },
                "analysis_seed": 7,
            },
            clock=fixed_clock,
        )

        self.assertEqual(calls, ["called"])
        self.assertEqual(
            response["graph"]["provenance"]["generated_at"],
            "2030-01-02T03:04:05.678Z",
        )

    def test_production_clock_emits_utc_rfc3339_milliseconds(self) -> None:
        self.assertRegex(
            sidecar.utc_generated_at(),
            re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$"),
        )


class SidecarIngestIdentityTests(unittest.TestCase):
    def test_source_replacement_keeps_hash_metadata_and_features_on_original_bytes(self) -> None:
        def generated_wave(sample_rate: int, channels: int, sample: int) -> bytes:
            buffer = io.BytesIO()
            with wave.open(buffer, "wb") as wav_file:
                wav_file.setnchannels(channels)
                wav_file.setsampwidth(2)
                wav_file.setframerate(sample_rate)
                wav_file.writeframes(sample.to_bytes(2, "little", signed=True) * channels * 8000)
            return buffer.getvalue()

        original = generated_wave(sample_rate=8000, channels=1, sample=32767)
        replacement = generated_wave(sample_rate=16000, channels=2, sample=0)
        original_hash = f"sha256:{hashlib.sha256(original).hexdigest()}"

        with tempfile.TemporaryDirectory() as directory:
            source_path = Path(directory) / "source.wav"
            replacement_path = Path(directory) / "replacement.wav"
            source_path.write_bytes(original)
            replacement_path.write_bytes(replacement)
            real_open = open
            replaced = False

            def open_with_replacement(path, mode="r", *args, **kwargs):
                handle = real_open(path, mode, *args, **kwargs)
                if Path(path) == source_path and mode == "rb" and not replaced:
                    real_read = handle.read

                    def read_then_replace(*read_args, **read_kwargs):
                        nonlocal replaced
                        content = real_read(*read_args, **read_kwargs)
                        if handle.tell() == len(original) and not replaced:
                            replacement_path.replace(source_path)
                            replaced = True
                        return content

                    handle.read = read_then_replace
                return handle

            with mock.patch("builtins.open", side_effect=open_with_replacement):
                response = sidecar.handle_message(
                    {
                        "type": "analyze_source_file",
                        "request_id": "req-replacement",
                        "source_path": str(source_path),
                        "analysis_seed": 7,
                    },
                    clock=lambda: "2030-01-02T03:04:05.678Z",
                )

            self.assertTrue(replaced, "fixture must replace the path after its complete read")
            self.assertEqual(source_path.read_bytes(), replacement)
            self.assertEqual(response["type"], "source_graph_built")
            graph = response["graph"]
            self.assertEqual(graph["source"]["content_hash"], original_hash)
            self.assertEqual(graph["provenance"]["source_hash"], original_hash)
            self.assertEqual(graph["source"]["source_id"], f"src-{original_hash[7:19]}")
            self.assertEqual(graph["source"]["path"], str(source_path.resolve()))
            self.assertEqual(graph["source"]["sample_rate"], 8000)
            self.assertEqual(graph["source"]["channel_count"], 1)
            self.assertEqual(graph["source"]["duration_seconds"], 1.0)
            self.assertTrue(graph["source_map"]["buckets"])
            for bucket in graph["source_map"]["buckets"]:
                self.assertEqual(bucket["energy_class"], "Peak")
            self.assertTrue(graph["phrase_audio_features"])
            for features in graph["phrase_audio_features"]:
                self.assertEqual(features["low_band_rms"], 1.0)
                self.assertEqual(features["low_mid_ratio"], 1.0)


if __name__ == "__main__":
    unittest.main()
