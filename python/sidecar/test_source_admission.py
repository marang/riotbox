import hashlib
import json
import os
import subprocess
import sys
import tempfile
import tracemalloc
import unittest
import wave
from pathlib import Path
from unittest import mock

import json_stdio_sidecar as sidecar
import source_bytes
from source_limits import SourceResourceLimitError


def analyze_request(path: Path) -> dict:
    return {
        "type": "analyze_source_file",
        "request_id": "admission",
        "source_path": str(path),
        "analysis_seed": 0,
    }


def generated_wave(path: Path) -> bytes:
    with wave.open(str(path), "wb") as handle:
        handle.setnchannels(1)
        handle.setsampwidth(2)
        handle.setframerate(8000)
        handle.writeframes(b"\x00\x10" * 8000)
    return path.read_bytes()


class SourceAdmissionTests(unittest.TestCase):
    def test_sparse_oversize_rejects_before_payload_hash_or_decode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated-oversize.wav"
            with path.open("wb") as handle:
                handle.truncate(268435456 + 1)
            real_open = open

            def guarded_open(*args, **kwargs):
                handle = real_open(*args, **kwargs)
                handle.read = mock.Mock(side_effect=AssertionError("oversize payload was read"))
                return handle

            with (
                mock.patch("builtins.open", side_effect=guarded_open),
                mock.patch.object(sidecar.hashlib, "sha256", side_effect=AssertionError("hashed")),
                mock.patch.object(sidecar, "decode_source_wave", side_effect=AssertionError("decoded")),
            ):
                response = sidecar.handle_message(
                    {
                        "type": "analyze_source_file",
                        "request_id": "oversize",
                        "source_path": str(path),
                        "analysis_seed": 0,
                    }
                )
            self.assertEqual(response["type"], "error")
            self.assertEqual(response["request_id"], "oversize")
            self.assertEqual(response["code"], "source_resource_limit")
            self.assertFalse(response["retryable"])
            self.assertIn("268435456", response["message"])

    def test_fixed_v1_budget_and_small_exact_empty_or_oversize_files(self) -> None:
        self.assertEqual(source_bytes.SOURCE_WAV_MAX_ENCODED_BYTES_V1, 268435456)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "encoded.bin"
            with (
                mock.patch.object(source_bytes, "SOURCE_WAV_MAX_ENCODED_BYTES_V1", 4),
                mock.patch.object(source_bytes, "_READ_CHUNK_BYTES", 3),
            ):
                for payload in [b"", b"1", b"1234"]:
                    path.write_bytes(payload)
                    self.assertEqual(source_bytes.read_source_wav_bytes(str(path)), payload)
                path.write_bytes(b"12345")
                with self.assertRaises(SourceResourceLimitError):
                    source_bytes.read_source_wav_bytes(str(path))

    def test_understated_size_short_reads_and_interruptions_stop_at_one_overrun_byte(self) -> None:
        real_open = open
        real_fstat = os.fstat

        def understated_stat(fd):
            fields = list(real_fstat(fd))
            fields[6] = 0
            return os.stat_result(fields)

        for interrupt_at in [None, 0, 4]:
            with self.subTest(interrupt_at=interrupt_at), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "growing.bin"
                path.write_bytes(b"123456789")
                positions = []
                descriptor_positions = []
                opened = []
                interrupted = False

                def short_open(*args, **kwargs):
                    handle = real_open(*args, **kwargs)
                    opened.append(handle)
                    real_read = handle.read

                    def short_read(size):
                        nonlocal interrupted
                        self.assertGreater(size, 0)
                        self.assertLessEqual(size, 5)
                        if handle.tell() == interrupt_at and not interrupted:
                            interrupted = True
                            raise InterruptedError("synthetic interrupted read")
                        chunk = real_read(min(size, 1))
                        positions.append(handle.tell())
                        descriptor_positions.append(os.lseek(handle.fileno(), 0, os.SEEK_CUR))
                        return chunk

                    handle.read = short_read
                    return handle

                with (
                    mock.patch("builtins.open", side_effect=short_open),
                    mock.patch.object(os, "fstat", side_effect=understated_stat),
                    mock.patch.object(source_bytes, "SOURCE_WAV_MAX_ENCODED_BYTES_V1", 4),
                ):
                    with self.assertRaisesRegex(SourceResourceLimitError, "required 5, limit 4"):
                        source_bytes.read_source_wav_bytes(str(path))
                self.assertEqual(positions[-1], 5)
                self.assertEqual(descriptor_positions[-1], 5)
                self.assertTrue(opened[0].closed)
                self.assertEqual(interrupted, interrupt_at is not None)

    def test_read_error_is_visible_and_closes_descriptor(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.wav"
            generated_wave(path)
            real_open = open
            opened = []

            def failing_open(*args, **kwargs):
                handle = real_open(*args, **kwargs)
                opened.append(handle)
                handle.read = mock.Mock(side_effect=OSError("synthetic read failure"))
                return handle

            with mock.patch("builtins.open", side_effect=failing_open):
                response = sidecar.handle_message(analyze_request(path))
            self.assertEqual(response["code"], "source_unreadable")
            self.assertIn("synthetic read failure", response["message"])
            self.assertTrue(opened[0].closed)

    def test_would_block_is_not_accepted_as_eof_or_partial_success(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.wav"
            content = generated_wave(path)
            real_open = open
            for prefix in [b"", content[:10], content]:
                with self.subTest(prefix_length=len(prefix)):
                    opened = []

                    def stalled_open(*args, **kwargs):
                        handle = real_open(*args, **kwargs)
                        opened.append(handle)
                        handle.read = mock.Mock(side_effect=([prefix] if prefix else []) + [None])
                        return handle

                    with (
                        mock.patch("builtins.open", side_effect=stalled_open),
                        mock.patch.object(sidecar.hashlib, "sha256", side_effect=AssertionError("hashed")),
                        mock.patch.object(sidecar, "decode_source_wave", side_effect=AssertionError("decoded")),
                    ):
                        response = sidecar.handle_message(analyze_request(path))
                    self.assertEqual(response["code"], "source_unreadable")
                    self.assertIn("would block", response["message"])
                    self.assertTrue(opened[0].closed)

    def test_short_reads_do_not_amplify_retained_encoded_storage_per_chunk(self) -> None:
        payload = b"x" * (128 * 1024)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fragmented.bin"
            path.write_bytes(payload)
            tracing_before = tracemalloc.is_tracing()
            # Own the measurement in a fresh process, including when the suite
            # itself was started with PYTHONTRACEMALLOC. Never reset/stop the
            # caller's tracing or inherit its unrelated peak.
            probe = """
import json, sys, tracemalloc
from unittest import mock
sys.path.insert(0, sys.argv[1])
import source_bytes
real_open = open
def short_open(*args, **kwargs):
    handle = real_open(*args, **kwargs)
    real_read = handle.read
    handle.read = lambda size: real_read(min(size, 1))
    return handle
with mock.patch('builtins.open', side_effect=short_open):
    tracemalloc.stop()
    tracemalloc.start()
    result = source_bytes.read_source_wav_bytes(sys.argv[2])
    _, peak = tracemalloc.get_traced_memory()
    tracemalloc.stop()
print(json.dumps({'matches': result == b'x' * (128 * 1024), 'peak': peak}))
"""
            completed = subprocess.run(
                [sys.executable, "-E", "-c", probe, str(Path(__file__).parent), str(path)],
                text=True, capture_output=True, check=True, timeout=10,
            )
            measured = json.loads(completed.stdout)
            self.assertTrue(measured["matches"])
            self.assertEqual(tracemalloc.is_tracing(), tracing_before)
            # A generous synthetic regression guard, not a product/RSS budget.
            # Retaining each tiny read until join measured over 120x here.
            self.assertLess(measured["peak"], len(payload) * 8)

    def test_directory_rejects_without_hash_or_decode(self) -> None:
        with tempfile.TemporaryDirectory() as directory, (
            mock.patch.object(sidecar.hashlib, "sha256", side_effect=AssertionError("hashed"))
        ):
            response = sidecar.handle_message(analyze_request(Path(directory)))
            self.assertEqual(response["code"], "source_unreadable")

    @unittest.skipUnless(os.name == "posix", "original-source Unix symlink policy")
    def test_regular_source_symlink_preserves_canonical_path_hash_and_decode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "generated.wav"
            original = generated_wave(path)
            link = Path(directory) / "source-link.wav"
            link.symlink_to(path)
            response = sidecar.handle_message(analyze_request(link))
            self.assertEqual(response["type"], "source_graph_built")
            source = response["graph"]["source"]
            self.assertEqual(source["path"], str(path.resolve()))
            self.assertEqual(source["content_hash"], "sha256:" + hashlib.sha256(original).hexdigest())
            self.assertEqual(source["sample_rate"], 8000)
            self.assertEqual(source["channel_count"], 1)

    @unittest.skipUnless(hasattr(os, "mkfifo") and hasattr(os, "O_NONBLOCK"), "Unix FIFO admission")
    def test_fifo_and_fifo_symlink_reject_without_waiting_and_peer_still_pings(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fifo = Path(directory) / "generated.fifo"
            link = Path(directory) / "fifo-link.wav"
            os.mkfifo(fifo)
            link.symlink_to(fifo)
            for path in [fifo, link]:
                with self.subTest(path=path.name):
                    requests = [analyze_request(path), {"type": "ping", "request_id": "after-error"}]
                    result = subprocess.run(
                        [sys.executable, str(Path(sidecar.__file__).resolve())],
                        input="".join(json.dumps(request) + "\n" for request in requests),
                        text=True, capture_output=True, check=True, timeout=10,
                    )
                    responses = [json.loads(line) for line in result.stdout.splitlines()]
                    self.assertEqual(responses[0]["code"], "source_unsupported")
                    self.assertIn("not a regular file", responses[0]["message"])
                    self.assertEqual(responses[1]["type"], "pong")
                    self.assertEqual(responses[1]["request_id"], "after-error")
