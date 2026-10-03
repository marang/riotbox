"""Real signals to generated operators; no audio service or CPAL is used."""

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from silent_host_signals import OperatorInterrupts


@unittest.skipUnless(sys.platform == "linux", "Linux operator signal policy")
class OperatorSignalTests(unittest.TestCase):
    def test_pending_signal_during_entry_restores_handlers_and_original_mask(self):
        numbers = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
        original = {number: signal.getsignal(number) for number in numbers}
        original_mask = signal.pthread_sigmask(signal.SIG_BLOCK, ())
        install = signal.signal
        for pending in numbers:
            with self.subTest(signal=pending):
                injected = False

                def pending_during_install(number, handler):
                    nonlocal injected
                    previous = install(number, handler)
                    if number == signal.SIGHUP and not injected:
                        injected = True
                        os.kill(os.getpid(), pending)
                    return previous

                try:
                    with patch("signal.signal", side_effect=pending_during_install):
                        with self.assertRaisesRegex(KeyboardInterrupt, signal.Signals(pending).name):
                            with OperatorInterrupts():
                                self.fail("interrupted entry must not enter the body")
                    self.assertTrue(injected)
                    for number, handler in original.items():
                        self.assertEqual(signal.getsignal(number), handler)
                    self.assertEqual(signal.pthread_sigmask(signal.SIG_BLOCK, ()), original_mask)
                finally:
                    # Keep this OS-boundary fault isolated even when the tested
                    # rollback is broken: later tests must retain their handlers.
                    signal.pthread_sigmask(signal.SIG_BLOCK, numbers)
                    try:
                        for number, handler in original.items():
                            install(number, handler)
                    finally:
                        signal.pthread_sigmask(signal.SIG_SETMASK, original_mask)

    def test_first_signal_raises_but_repeated_signals_do_not_interrupt_cleanup(self):
        previous = signal.getsignal(signal.SIGTERM)
        with self.assertRaisesRegex(KeyboardInterrupt, "SIGTERM"):
            with OperatorInterrupts() as interrupts:
                with self.assertRaisesRegex(KeyboardInterrupt, "SIGTERM"):
                    os.kill(os.getpid(), signal.SIGTERM)
                os.kill(os.getpid(), signal.SIGTERM)
                os.kill(os.getpid(), signal.SIGHUP)
                self.assertEqual(interrupts.received, signal.SIGTERM)
        self.assertEqual(signal.getsignal(signal.SIGTERM), previous)

    def test_first_signal_during_cleanup_is_recorded_without_abandoning_cleanup(self):
        with self.assertRaisesRegex(KeyboardInterrupt, "SIGHUP"):
            with OperatorInterrupts() as interrupts:
                with interrupts.cleanup():
                    interrupts.check()
                    os.kill(os.getpid(), signal.SIGHUP)
                    self.assertEqual(interrupts.received, signal.SIGHUP)
                # Normal exit must deliver even an interruption arriving after
                # the caller's last explicit check during cleanup/publication.

    def test_term_hup_and_int_during_active_observation_reap_child_and_enter_outer_cleanup(self):
        code = """
import json, os, sys
from pathlib import Path
import run_silent_host_probe as operator
from silent_host_signals import OperatorInterrupts
from test_silent_host_operator import GeneratedHost, PROTOCOL, generated_command
from test_silent_host_routes import fixture
from silent_host_routes import find_sink
owner = Path(sys.argv[1])
prefix = owner / 'run'
class Host(GeneratedHost):
    def snapshot(self):
        result = super().snapshot()
        if len(result) > 1:
            (owner / 'ready').touch()
        return result
def execute(interrupts):
    try:
        operator.run_program(generated_command(), prefix,
            Host(prefix.with_suffix('.stdout.ndjson')),
            find_sink(fixture(), 'owned-test'),
            dict(PROTOCOL, run_seconds=1, route_poll_interval_ms=10,
                 startup_route_deadline_seconds=1, process_timeout_seconds=3,
                 teardown_deadline_seconds=1), os.environ.copy())
    finally:
        with interrupts.cleanup():
            (owner / 'outer-cleanup').touch()
try:
    with OperatorInterrupts() as interrupts:
        execute(interrupts)
except (Exception, KeyboardInterrupt) as error:
    print(str(error), file=sys.stderr)
    raise SystemExit(1)
"""
        environment = dict(os.environ, PYTHONPATH=str(Path(__file__).resolve().parent))
        for signum in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT):
            with self.subTest(signal=signum), tempfile.TemporaryDirectory(prefix="riotbox-signal-fixture-") as directory:
                owner = Path(directory)
                with subprocess.Popen([sys.executable, "-c", code, directory], env=environment,
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                      start_new_session=True) as process:
                    try:
                        deadline = time.monotonic() + 5
                        while not (owner / "ready").exists() and process.poll() is None:
                            if time.monotonic() >= deadline:
                                self.fail("generated operator never reached active observation")
                            time.sleep(0.005)
                        self.assertIsNone(process.poll())
                        process.send_signal(signum)
                        stdout, stderr = process.communicate(timeout=6)
                        self.assertEqual(process.returncode, 1, (stdout, stderr))
                        self.assertIn(signal.Signals(signum).name.encode(), stderr)
                        self.assertTrue((owner / "outer-cleanup").exists())
                        record = json.loads((owner / "run.stdout.ndjson").read_text().splitlines()[0])
                        self.assertFalse(Path(f"/proc/{record['pid']}").exists())
                    finally:
                        if process.poll() is None:
                            process.kill()
                            process.wait(timeout=3)


if __name__ == "__main__":
    unittest.main()
