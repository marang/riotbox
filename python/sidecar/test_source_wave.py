import io
import json
import struct
import subprocess
import sys
import tempfile
import unittest
import wave
from pathlib import Path
from unittest import mock

import json_stdio_sidecar as sidecar
from source_wave import decode_source_wave


def chunk(identifier: bytes, payload: bytes, pad: bool = True) -> bytes:
    padding = b"\0" if pad and len(payload) % 2 else b""
    return identifier + struct.pack("<I", len(payload)) + payload + padding


def riff(body: bytes) -> bytes:
    return b"RIFF" + struct.pack("<I", len(body) + 4) + b"WAVE" + body


def format_chunk(channels: int = 1, width: int = 2, rate: int = 8000) -> bytes:
    frame_width = channels * width
    return chunk(b"fmt ", struct.pack("<HHIIHH", 1, channels, rate, rate * frame_width, frame_width, width * 8))


def request(path: Path, request_id: str = "container") -> dict:
    return {
        "type": "analyze_source_file", "request_id": request_id,
        "source_path": str(path), "analysis_seed": 0,
    }


class SourceWaveTests(unittest.TestCase):
    def test_malformed_layout_and_geometry_fail_before_features(self) -> None:
        fmt = format_chunk()
        data = chunk(b"data", b"\0\x10" * 4)
        valid = riff(fmt + data)
        cases = {
            "empty": b"",
            "short-riff": b"RIFF",
            "truncated-fmt": riff(b"fmt " + struct.pack("<I", 16) + b"\x01\x00"),
            "complete-short-fmt": riff(chunk(b"fmt ", b"\x01\x00") + data),
            "overlong-junk": riff(b"JUNK" + struct.pack("<I", 0xFFFFFFF0) + b"ab" + fmt + data),
            "missing-pcm": riff(fmt + b"data" + struct.pack("<I", 4)),
            "short-pcm": riff(fmt + b"data" + struct.pack("<I", 4) + b"\0\x10"),
            "partial-mono-frame": riff(fmt + chunk(b"data", b"abc", pad=False)),
            "partial-stereo-frame": riff(format_chunk(channels=2) + chunk(b"data", b"ab")),
            "zero-rate": riff(format_chunk(rate=0) + data),
            "zero-channels": riff(format_chunk(channels=0) + data),
            "hidden-pcm": valid[:4] + struct.pack("<I", 36) + valid[8:],
            "overstated-riff": valid[:4] + struct.pack("<I", 0xFFFFFFFF) + valid[8:],
            "duplicate-fmt": riff(fmt + format_chunk(channels=2) + data),
            "duplicate-data": riff(fmt + data + chunk(b"data", b"\0\0" * 4)),
            "trailing-debris": riff(fmt + data + b"abc"),
            "missing-data": riff(fmt),
            "missing-fmt": riff(data),
            "data-before-fmt": riff(data + fmt),
            "bad-post-data-metadata": riff(fmt + data + b"JUNK" + struct.pack("<I", 4) + b"ab"),
            "unpadded-terminal-metadata": riff(fmt + data + chunk(b"JUNK", b"x", pad=False)),
            "missing-interior-padding": riff(chunk(b"JUNK", b"x", pad=False) + fmt + data),
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated-invalid.wav"
            for name, content in cases.items():
                with self.subTest(case=name):
                    path.write_bytes(content)
                    with mock.patch.object(sidecar, "build_source_map_buckets", side_effect=AssertionError("features reached")):
                        response = sidecar.handle_message(request(path))
                    self.assertEqual(response["type"], "error")
                    self.assertEqual(response["code"], "source_unsupported")
                    self.assertEqual(response["request_id"], "container")
                    self.assertFalse(response["retryable"])
                    self.assertTrue(response["message"])

    def test_valid_integer_widths_channels_metadata_and_terminal_padding_preserve_samples(self) -> None:
        for width in [1, 2, 3, 4]:
            for channels in [1, 2]:
                scale = (1 << (width * 8 - 1)) - 1
                values = [-scale - 1, 0, scale]
                frames = b""
                expected = []
                for value in values:
                    frame = [value] + [0] * (channels - 1)
                    frames += b"".join(
                        bytes([sample + 128]) if width == 1 else sample.to_bytes(width, "little", signed=True)
                        for sample in frame
                    )
                    expected.append((float(value) / scale) / channels)
                fmt = format_chunk(channels, width)
                buffer = io.BytesIO()
                with wave.open(buffer, "wb") as writer:
                    writer.setparams((channels, width, 8000, 0, "NONE", "not compressed"))
                    writer.writeframes(frames)
                layouts = {
                    "stdlib-writer": buffer.getvalue(),
                    "unpadded-terminal-data": riff(fmt + chunk(b"data", frames, pad=False)),
                    "padded-terminal-data": riff(fmt + chunk(b"data", frames)),
                    "metadata": riff(chunk(b"JUNK", b"abc") + fmt + chunk(b"data", frames) + chunk(b"LIST", b"INFO")),
                }
                for name, content in layouts.items():
                    with self.subTest(width=width, channels=channels, layout=name):
                        decoded = decode_source_wave(content)
                        self.assertEqual(decoded.sample_rate, 8000)
                        self.assertEqual(decoded.channel_count, channels)
                        self.assertEqual(decoded.frame_count, 3)
                        self.assertEqual(decoded.samples, expected)

    def test_genuine_empty_data_remains_supported(self) -> None:
        for width in [1, 2, 3, 4]:
            for channels in [1, 2]:
                with self.subTest(width=width, channels=channels):
                    decoded = decode_source_wave(riff(format_chunk(channels, width) + chunk(b"data", b"")))
                    self.assertEqual(decoded.frame_count, 0)
                    self.assertEqual(decoded.samples, [])

    def test_every_truncated_prefix_and_short_decoder_read_is_rejected(self) -> None:
        content = riff(format_chunk() + chunk(b"data", b"\0\x10" * 4))
        for length in range(len(content)):
            with self.subTest(prefix_length=length), self.assertRaises(ValueError):
                decode_source_wave(content[:length])
        with mock.patch.object(wave.Wave_read, "readframes", return_value=b"\0\x10"):
            with self.assertRaisesRegex(ValueError, "decoded byte count"):
                decode_source_wave(content)
        with mock.patch.object(wave.Wave_read, "getnframes", return_value=3):
            with self.assertRaisesRegex(ValueError, "declared frame count"):
                decode_source_wave(content)

    def test_downstream_analysis_errors_are_not_hidden_as_parser_errors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.wav"
            path.write_bytes(riff(format_chunk() + chunk(b"data", b"\0\x10" * 4)))
            for error_type in [RuntimeError, EOFError, AssertionError]:
                with self.subTest(error=error_type.__name__):
                    with mock.patch.object(sidecar, "build_source_map_buckets", side_effect=error_type("analysis fault")):
                        with self.assertRaisesRegex(error_type, "analysis fault"):
                            sidecar.handle_message(request(path))
    def test_truncated_fmt_is_request_error_and_real_peer_remains_usable(self) -> None:
        malformed = b"RIFF" + struct.pack("<I", 14) + b"WAVEfmt " + struct.pack("<I", 16) + b"\x01\x00"
        with tempfile.TemporaryDirectory() as directory:
            invalid_path = Path(directory) / "truncated.wav"
            valid_path = Path(directory) / "generated.wav"
            invalid_path.write_bytes(malformed)
            with wave.open(str(valid_path), "wb") as writer:
                writer.setparams((1, 2, 8000, 0, "NONE", "not compressed"))
                writer.writeframes(b"\x00\x10" * 8000)
            requests = [
                request(invalid_path, "bad"),
                {"type": "ping", "request_id": "after-bad"},
                request(valid_path, "good"),
            ]
            result = subprocess.run(
                [sys.executable, str(Path(sidecar.__file__).resolve())],
                input="".join(json.dumps(value) + "\n" for value in requests),
                text=True, capture_output=True, timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            responses = [json.loads(line) for line in result.stdout.splitlines()]
            self.assertEqual(len(responses), 3)
            self.assertEqual(responses[0]["type"], "error")
            self.assertEqual(responses[0]["code"], "source_unsupported")
            self.assertEqual(responses[0]["request_id"], "bad")
            self.assertFalse(responses[0]["retryable"])
            self.assertTrue(responses[0]["message"])
            self.assertEqual(responses[1]["type"], "pong")
            self.assertEqual(responses[1]["request_id"], "after-bad")
            self.assertEqual(responses[2]["type"], "source_graph_built")
            self.assertEqual(responses[2]["request_id"], "good")


if __name__ == "__main__":
    unittest.main()
