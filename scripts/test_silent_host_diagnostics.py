"""Generated versioned ALSA stub only: no CPAL, device, service or audio file."""

import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import tempfile
import time
import unittest

from silent_host_diagnostics import (AlsaDiagnostics, HEADER, PCM, SIZE, decode_measurements,
                                     load_protocol_v4)
from silent_host_evidence import EvidenceError
from silent_host_process import ManagedProcess, ManagedProcessError

ROOT = Path(__file__).resolve().parents[1]
NATIVE = ROOT / "scripts/native"


@unittest.skipUnless(platform.system() == "Linux" and platform.machine() == "x86_64", "Linux diagnostic ABI")
class AlsaDiagnosticsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix="riotbox-alsa-generated-")
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.root = Path(cls.temporary.name)
        cls.library = cls.root / "diagnostics.so"
        cls.child = cls.root / "generated-child"
        flags = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror", "-O2"]
        for command in (
            [*flags, "-fPIC", "-shared", str(NATIVE / "silent_host_alsa_diagnostics.c"), "-ldl", "-o", str(cls.library)],
            [*flags, "-fPIC", "-shared", str(NATIVE / "fixtures/silent_host_alsa_fake.c"),
             f"-Wl,--version-script={NATIVE / 'fixtures/silent_host_alsa.map'}", "-o", str(cls.root / "libfixture.so")],
            [*flags, str(NATIVE / "fixtures/silent_host_alsa_child.c"), "-pthread", f"-L{cls.root}",
             "-lfixture", f"-Wl,-rpath,{cls.root}", "-o", str(cls.child)],
        ):
            result = subprocess.run(command, capture_output=True, text=True, timeout=30)
            if result.returncode:
                raise RuntimeError(result.stderr[:6000])
        cls.library_hash = hashlib.sha256(cls.library.read_bytes()).hexdigest()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="riotbox-alsa-owner-")
        self.addCleanup(self.temp.cleanup)
        self.prefix = Path(self.temp.name) / "generated"
        self.adapter = AlsaDiagnostics(ROOT, self.library, self.library_hash)
        self.environment = {key: value for key, value in os.environ.items()
                            if not key.startswith("LD_") and key != "RIOTBOX_ALSA_DIAGNOSTICS"}

    def run_child(self, mode="clean"):
        command = self.adapter.wrap([str(self.child), mode], self.prefix, self.environment)
        # Same production supervision; fake library is the only ALSA provider.
        process = ManagedProcess(command, self.prefix.with_suffix(".out"), self.prefix.with_suffix(".err"),
                                 self.environment, 4)
        status = None
        try:
            with process:
                pid = None
                deadline = time.monotonic() + 5
                while not process.exited() and time.monotonic() < deadline:
                    pid = process.child_pid or pid
                    time.sleep(0.005)
                self.assertIsNotNone(pid)
                status = process.finish()
        except ManagedProcessError as error:
            if status is None or status == 0 or str(error) != f"watchdog/command exited with status {status}":
                raise
        self.assertTrue(process.cleanup_verified)
        return pid, status

    def test_real_abi_forwarding_geometry_pid_and_watchdog_isolation(self):
        pid, status = self.run_child()
        self.assertEqual(status, 0)
        report = self.adapter.finish(self.prefix, pid, complete=True)
        active = report["active_pcm"]
        self.assertEqual((active["buffer_frames"], active["period_frames"], active["sample_rate"], active["channels"]),
                         (4096, 512, 44100, 2))
        self.assertEqual(active["calls"]["avail"]["count"], 6000)
        self.assertEqual(active["calls"]["write"]["count"], 6000)
        self.assertEqual((active["prepare_calls"], active["recover_calls"]), (1, 1))
        self.assertEqual(report["pid"], pid)
        self.assertNotEqual(pid, os.getpid())
        self.assertNotIn("LD_PRELOAD", self.environment)
        self.adapter.verify_geometry(report, {"default_output_config":
                                     "F32, channels=2, sample_rate=44100, buffer_size=unknown"}, preflight=True)
        with self.assertRaisesRegex(EvidenceError, "disagree"):
            self.adapter.verify_geometry(report, {"first": {"health": {
                                         "channel_count": 2, "sample_rate": 48000}}}, preflight=False)
        with self.assertRaises(FileExistsError):
            self.adapter.wrap([str(self.child)], self.prefix, self.environment)

    def test_epipe_site_is_distinct_and_return_errno_remain_transparent(self):
        for mode, operation in (("avail_error", "avail"), ("write_error", "write")):
            with self.subTest(mode=mode):
                self.prefix = Path(self.temp.name) / mode
                pid, status = self.run_child(mode)
                self.assertEqual(status, 0)
                calls = self.adapter.finish(self.prefix, pid, complete=False)["active_pcm"]["calls"]
                self.assertEqual(calls[operation]["epipe_count"], 1)
                self.assertEqual(calls[operation]["first_negative"], -32)
                self.assertGreater(calls[operation]["first_epipe_ns"], 0)
                self.assertEqual(calls["write" if operation == "avail" else "avail"]["epipe_count"], 0)
                with self.assertRaisesRegex(EvidenceError, "contradicts"):
                    decode_measurements(self.prefix.with_suffix(".alsa.bin").read_bytes(), pid, complete=True)

    def test_reused_handle_has_distinct_generations_not_merged(self):
        pid, status = self.run_child("reuse")
        self.assertEqual(status, 0)
        with self.assertRaisesRegex(EvidenceError, "exactly one"):
            self.adapter.finish(self.prefix, pid, complete=True)
        data = self.prefix.with_suffix(".alsa.bin").read_bytes()
        self.assertEqual(HEADER.unpack_from(data)[3], 2)
        first = PCM.unpack_from(data, HEADER.size)
        second = PCM.unpack_from(data, HEADER.size + PCM.size)
        self.assertEqual(first[0], second[0])
        self.assertEqual((first[1], second[1], first[8], second[8]), (1, 1, 6000, 6000))

    def test_concurrent_atomic_capture_never_races_or_merges_counts(self):
        pid, status = self.run_child("concurrent")
        self.assertEqual(status, 0)
        report = self.adapter.finish(self.prefix, pid, complete=True)
        self.assertEqual(report["active_pcm"]["calls"]["avail"]["count"], 12000)
        self.assertEqual(report["active_pcm"]["calls"]["write"]["count"], 12000)

    def test_capacity_and_capture_direction_stop_instead_of_bypass(self):
        for mode in ("capacity", "capture"):
            with self.subTest(mode=mode):
                self.prefix = Path(self.temp.name) / mode
                _pid, status = self.run_child(mode)
                self.assertEqual(status, 126)

    def test_geometry_and_wrong_pid_fail_and_preserve_raw_report(self):
        pid, status = self.run_child("geometry_error")
        self.assertEqual(status, 0)
        with self.assertRaisesRegex(EvidenceError, "geometry"):
            self.adapter.finish(self.prefix, pid, complete=True)
        self.assertIn("validation_error", json.loads(self.prefix.with_suffix(".alsa.json").read_bytes()))
        with self.assertRaises(EvidenceError):
            decode_measurements(self.prefix.with_suffix(".alsa.bin").read_bytes(), pid + 1, complete=True)

    def test_forked_process_cannot_open_or_mutate_parent_measurements(self):
        pid, status = self.run_child("fork_open")
        self.assertEqual(status, 126)
        data = self.prefix.with_suffix(".alsa.bin").read_bytes()
        self.assertEqual(HEADER.unpack_from(data)[2:4], (pid, 1))

    def test_missing_foreign_or_malformed_mapping_cannot_run_uninstrumented(self):
        command = self.adapter.wrap([str(self.child)], self.prefix, self.environment)
        self.prefix.with_suffix(".alsa.bin").write_bytes(b"bad")
        result = subprocess.run(command, env=self.environment, capture_output=True, timeout=4)
        self.assertEqual(result.returncode, 125)
        self.prefix.with_suffix(".alsa.bin").unlink()
        result = subprocess.run(command, env=self.environment, capture_output=True, timeout=4)
        self.assertEqual(result.returncode, 125)

    def test_protocol_compatibility_and_library_binding_reject_changes(self):
        self.assertEqual(load_protocol_v4(ROOT)["run_seconds"], 60)
        with self.assertRaisesRegex(EvidenceError, "library changed"):
            AlsaDiagnostics(ROOT, self.library, "0" * 64)
        self.environment["LD_PRELOAD"] = "unrelated"
        with self.assertRaisesRegex(EvidenceError, "watchdog"):
            self.adapter.wrap([str(self.child)], self.prefix, self.environment)

    def test_error_in_a_nonwriting_pcm_cannot_be_discarded_by_active_selection(self):
        first = [1, 1, 1, 0, 4096, 512, 2, 44100] + [0] * 21
        first[8:17] = [1, 1, (1 << 64) - 32, 1, 100, 90, 0, 10, 0]
        second = [2, 1, 1, 0, 4096, 512, 2, 44100] + [0] * 21
        second[8:17] = [1, 0, 0, 0, 0, 120, 0, 10, 0]
        second[17:26] = [1, 0, 0, 0, 0, 130, 0, 10, 512]
        data = (HEADER.pack(int.from_bytes(b"RBXALSA1", "little"), 1, 1234, 2, 0, 1, 0, 1)
                + PCM.pack(*first) + PCM.pack(*second) + bytes(62 * PCM.size))
        with self.assertRaisesRegex(EvidenceError, "ALSA error"):
            decode_measurements(data, 1234, complete=True)
        partial = decode_measurements(data, 1234, complete=False)
        self.assertEqual(partial["pcms"][0]["calls"]["avail"]["negative_count"], 1)

    def test_missing_symbols_and_foreign_mapping_cannot_bypass_capture(self):
        command = self.adapter.wrap(["/usr/bin/true"], self.prefix, self.environment)
        result = subprocess.run(command, env=self.environment, capture_output=True, timeout=4)
        self.assertEqual(result.returncode, 125)
        data = bytearray(SIZE)
        data[16:24] = (os.getpid()).to_bytes(8, "little")
        self.prefix.with_suffix(".alsa.bin").write_bytes(data)
        command[-1] = str(self.child)
        result = subprocess.run(command, env=self.environment, capture_output=True, timeout=4)
        self.assertEqual(result.returncode, 125)

    def test_terminated_child_retains_partial_mapping_without_destructor(self):
        for signum in (signal.SIGTERM, signal.SIGKILL):
            self.prefix = Path(self.temp.name) / f"terminated-{signum}"
            command = self.adapter.wrap([str(self.child), "hang"], self.prefix, self.environment)
            process = ManagedProcess(command, self.prefix.with_suffix(".out"), self.prefix.with_suffix(".err"),
                                     self.environment, 4)
            pid = None
            with self.assertRaises(ManagedProcessError):
                with process:
                    deadline = time.monotonic() + 3
                    while time.monotonic() < deadline:
                        pid = process.child_pid or pid
                        if pid and "ready" in self.prefix.with_suffix(".out").read_text():
                            break
                        time.sleep(0.005)
                    self.assertIsNotNone(pid)
                    self.assertIn("ready", self.prefix.with_suffix(".out").read_text())
                    os.kill(pid, signum)
                    while not process.exited() and time.monotonic() < deadline:
                        time.sleep(0.005)
                    self.assertNotEqual(process.finish(), 0)
            self.assertTrue(process.cleanup_verified)
            self.assertFalse(Path(f"/proc/{pid}").exists())
            report = self.adapter.finish(self.prefix, pid, complete=False)
            self.assertFalse(report["complete_lifetime"])
            self.assertEqual(report["active_pcm"]["calls"]["write"]["count"], 6000)
            with self.assertRaisesRegex(EvidenceError, "close"):
                decode_measurements(self.prefix.with_suffix(".alsa.bin").read_bytes(), pid, complete=True)

    def test_generated_overhead_is_reported_not_a_performance_gate(self):
        baseline = subprocess.run([str(self.child)], env=self.environment, capture_output=True,
                                  text=True, check=True, timeout=4)
        pid, status = self.run_child()
        self.assertEqual(status, 0)
        self.adapter.finish(self.prefix, pid, complete=True)
        observed = int(self.prefix.with_suffix(".out").read_text())
        plain = int(baseline.stdout)
        self.assertGreater(plain, 0)
        self.assertGreater(observed, 0)
        print(f"generated 6000-cycle observation only: baseline_ns={plain}, instrumented_ns={observed}")


if __name__ == "__main__":
    unittest.main()
