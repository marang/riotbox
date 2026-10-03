import io
import json
import subprocess
import sys
import tempfile
import unittest
import wave
from pathlib import Path
from unittest import mock

import json_stdio_sidecar as sidecar
import source_wave
from source_limits import (
    SOURCE_WAV_MAX_DECODED_SAMPLES_V1,
    SOURCE_WAV_MAX_ENCODED_BYTES_V1,
    SourceResource,
    SourceResourceLimitError,
)


def generated_wave(width: int, channels: int, frames: int) -> bytes:
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as writer:
        writer.setparams((channels, width, 8000, 0, "NONE", "not compressed"))
        writer.writeframes(b"\0" * width * channels * frames)
    return buffer.getvalue()


class SourceSampleAdmissionTests(unittest.TestCase):
    def test_stereo_limit_rejects_before_reading_frames_or_converting_to_mono(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.wav"
            path.write_bytes(generated_wave(2, 2, 3))
            with (
                mock.patch.object(source_wave, "SOURCE_WAV_MAX_DECODED_SAMPLES_V1", 4),
                mock.patch.object(wave.Wave_read, "readframes", side_effect=AssertionError("PCM payload read before admission")),
                mock.patch.object(source_wave, "_decode_pcm_samples", side_effect=AssertionError("converted before admission")),
                mock.patch.object(sidecar, "build_source_map_buckets", side_effect=AssertionError("features before admission")),
            ):
                response = sidecar.handle_message({
                    "type": "analyze_source_file", "request_id": "samples",
                    "source_path": str(path), "analysis_seed": 0,
                })
            self.assertEqual(response["type"], "error")
            self.assertEqual(response["code"], "source_resource_limit")
            self.assertEqual(response["request_id"], "samples")
            self.assertFalse(response["retryable"])
            self.assertIn("decoded interleaved samples", response["message"])
            self.assertIn("required 6, limit 4", response["message"])

    def test_all_widths_count_interleaved_samples_and_admit_empty_exact_or_below(self) -> None:
        with mock.patch.object(source_wave, "SOURCE_WAV_MAX_DECODED_SAMPLES_V1", 4):
            for width in [1, 2, 3, 4]:
                for channels in [1, 2]:
                    for frames in [0, 1, 4 // channels]:
                        with self.subTest(width=width, channels=channels, frames=frames):
                            decoded = source_wave.decode_source_wave(generated_wave(width, channels, frames))
                            self.assertEqual(decoded.frame_count, frames)
                            self.assertEqual(len(decoded.samples), frames)
                    over_frames = 4 // channels + 1
                    with self.subTest(width=width, channels=channels, over_frames=over_frames):
                        with self.assertRaises(SourceResourceLimitError) as raised:
                            source_wave.decode_source_wave(generated_wave(width, channels, over_frames))
                        self.assertIs(raised.exception.resource, SourceResource.DECODED_INTERLEAVED_SAMPLES)
                        self.assertEqual(raised.exception.required, over_frames * channels)
                        self.assertEqual(raised.exception.limit, 4)

    def test_fixed_default_limit_rejects_metadata_probe_without_large_allocation(self) -> None:
        self.assertEqual(SOURCE_WAV_MAX_ENCODED_BYTES_V1, 268435456)
        self.assertEqual(SOURCE_WAV_MAX_DECODED_SAMPLES_V1, 67108864)
        required = SOURCE_WAV_MAX_DECODED_SAMPLES_V1 + 1
        reader = mock.MagicMock()
        reader.getcomptype.return_value = "NONE"
        reader.getframerate.return_value = 8000
        reader.getnchannels.return_value = 1
        reader.getsampwidth.return_value = 2
        reader.getnframes.return_value = required
        reader.readframes.side_effect = AssertionError("large PCM read attempted")
        # Only this metadata-boundary probe bypasses layout. Other cases use
        # real small WAVs; no 128 MiB-plus-one-sample payload is materialized.
        with (
            mock.patch.object(source_wave, "_validate_container", return_value=required * 2),
            mock.patch.object(source_wave.wave, "open") as opened,
        ):
            opened.return_value.__enter__.return_value = reader
            with self.assertRaises(SourceResourceLimitError) as raised:
                source_wave.decode_source_wave(b"metadata-only test seam")
        self.assertEqual(raised.exception.required, required)
        self.assertEqual(raised.exception.limit, 67108864)
        reader.readframes.assert_not_called()
        self.assertLess(44 + required * 2, SOURCE_WAV_MAX_ENCODED_BYTES_V1)

    def test_invalid_frame_geometry_precedes_resource_error(self) -> None:
        content = generated_wave(2, 2, 2)
        with (
            mock.patch.object(source_wave, "SOURCE_WAV_MAX_DECODED_SAMPLES_V1", 4),
            mock.patch.object(wave.Wave_read, "getnframes", return_value=3),
        ):
            with self.assertRaisesRegex(ValueError, "declared frame count") as raised:
                source_wave.decode_source_wave(content)
        self.assertNotIsInstance(raised.exception, SourceResourceLimitError)

    def test_small_budget_real_peer_error_ping_and_exact_limit_analysis(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            over = Path(directory) / "over.wav"
            valid = Path(directory) / "exact.wav"
            over.write_bytes(generated_wave(2, 2, 3))
            valid.write_bytes(generated_wave(2, 1, 4))
            requests = [
                {"type": "analyze_source_file", "request_id": "over", "source_path": str(over), "analysis_seed": 0},
                {"type": "ping", "request_id": "after-error"},
                {"type": "analyze_source_file", "request_id": "exact", "source_path": str(valid), "analysis_seed": 0},
            ]
            # A test-only injected budget, not a product flag/environment knob.
            program = """
import sys
sys.path.insert(0, sys.argv[1])
import source_wave
source_wave.SOURCE_WAV_MAX_DECODED_SAMPLES_V1 = 4
import json_stdio_sidecar
json_stdio_sidecar.main()
"""
            result = subprocess.run(
                [sys.executable, "-c", program, str(Path(sidecar.__file__).resolve().parent)],
                input="".join(json.dumps(value) + "\n" for value in requests),
                text=True, capture_output=True, check=True, timeout=10,
            )
            responses = [json.loads(line) for line in result.stdout.splitlines()]
            self.assertEqual(len(responses), 3)
            self.assertEqual(responses[0]["code"], "source_resource_limit")
            self.assertEqual(responses[0]["request_id"], "over")
            self.assertFalse(responses[0]["retryable"])
            self.assertIn("required 6, limit 4", responses[0]["message"])
            self.assertEqual(responses[1]["type"], "pong")
            self.assertEqual(responses[1]["request_id"], "after-error")
            self.assertEqual(responses[2]["type"], "source_graph_built")
            self.assertEqual(responses[2]["request_id"], "exact")
