"""Generated transcript checks; no devices, subprocesses or audio files."""

import copy
import json
import unittest
from pathlib import Path

from silent_host_evidence import (
    EvidenceError, load_protocol, records_from_bytes, validate_preflight, validate_records,
)


PROTOCOL = load_protocol(Path(__file__).resolve().parents[1]
                         / "docs/benchmarks/silent_host_observation_v2.json")


def transcript():
    records = []
    for index in range(62):
        stopped = index == 61
        records.append({
            "schema": "riotbox.silent_host_sample.v1", "pid": 1234,
            "event": "started" if index == 0 else "stopped" if stopped else "sample",
            "sample_index": min(index, 60), "elapsed_ms": min(index, 60) * 1000,
            "result": "observing" if index == 0 else "ok",
            "health": {"lifecycle": "stopped" if stopped else "running",
                       "host_name": "generated", "device_name": "generated",
                       "sample_format": "F32", "sample_rate": 48000,
                       "channel_count": 2, "callback_count": min(index, 60) * 5,
                       "max_callback_gap_micros": 99999999,
                       "callback_scratch_overflow_count": 0, "stream_error_count": 0,
                       "last_stream_error": None}})
    return records


class TranscriptTests(unittest.TestCase):
    def test_full_fixed_transcript_and_incremental_prefix_are_separate(self):
        records = transcript()
        for count in range(61):
            self.assertFalse(validate_records(records[:count], 1234, PROTOCOL))
        self.assertTrue(validate_records(records[:61], 1234, PROTOCOL))
        self.assertTrue(validate_records(records, 1234, PROTOCOL, complete=True))
        with self.assertRaisesRegex(EvidenceError, "stopped evidence"):
            validate_records(records[:61], 1234, PROTOCOL, complete=True)

    def test_false_success_or_corrupt_identity_progress_and_faults_reject(self):
        mutations = [
            lambda r: r[20].update({"pid": 9876}),
            lambda r: r[20].update({"sample_index": 19}),
            lambda r: r[20].update({"result": "failed", "reason": "fixture fault"}),
            lambda r: r[20].update({"elapsed_ms": 19000}),
            lambda r: r[20]["health"].update({"callback_count": 95}),
            lambda r: r[20]["health"].update({"callback_count": 1}),
            lambda r: r[20]["health"].update({"stream_error_count": 1}),
            lambda r: r[20]["health"].update({"stream_error_count": False}),
            lambda r: r[20]["health"].update({"last_stream_error": "retained"}),
            lambda r: r[20]["health"].update({"callback_scratch_overflow_count": 1}),
            lambda r: r[20]["health"].update({"lifecycle": "stopped"}),
            lambda r: r[20]["health"].update({"channel_count": 1}),
            lambda r: r[20]["health"].update({"sample_rate": 44100}),
            lambda r: r[-1]["health"].update({"lifecycle": "running"}),
            lambda r: r.append(copy.deepcopy(r[-1])),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(mutation=index), self.assertRaises(EvidenceError):
                records = transcript()
                mutate(records)
                validate_records(records, 1234, PROTOCOL, complete=True)

    def test_partial_json_is_pending_only_before_process_exit(self):
        line = json.dumps(transcript()[0]).encode() + b"\n"
        self.assertEqual(records_from_bytes(line + b"{"), transcript()[:1])
        with self.assertRaises(EvidenceError):
            records_from_bytes(line + b"{", complete=True)
        for data in (b"{}\n" * 63, b"{\n", b"[]\n", b"x" * 262145):
            with self.subTest(size=len(data)), self.assertRaises(EvidenceError):
                records_from_bytes(data, complete=True)

    def test_late_terminal_batch_cannot_reset_the_teardown_budget(self):
        for delay in (5000, 6000):
            records = transcript()
            records[-1]["elapsed_ms"] += delay
            with self.subTest(delay=delay), self.assertRaisesRegex(EvidenceError, "teardown"):
                validate_records(records, 1234, PROTOCOL, complete=True)
        records = transcript()
        records[-1]["elapsed_ms"] += 4999
        self.assertTrue(validate_records(records, 1234, PROTOCOL, complete=True))

    def test_preflight_requires_typed_ok_and_actual_error_free_callbacks(self):
        data = (b"callback_count: 9\nstream_error_count: 0\n"
                b"callback_scratch_overflow_count: 0\nstream_result: Ok\n")
        self.assertEqual(validate_preflight(data)["callback_count"], "9")
        for bad in (data.replace(b"count: 9", b"count: 0"),
                    data.replace(b"stream_error_count: 0", b"stream_error_count: 1"),
                    data.replace(b"stream_result: Ok", b"stream_result: Failed"),
                    data + b"stream_result: Ok\n", data.replace(b"stream_error_count: 0\n", b"")):
            with self.subTest(data=bad), self.assertRaises(EvidenceError):
                validate_preflight(bad)


if __name__ == "__main__":
    unittest.main()
