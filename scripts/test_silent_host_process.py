import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

from silent_host_process import ManagedProcess, ManagedProcessError


class ManagedProcessTests(unittest.TestCase):
    def test_interrupt_during_handler_restoration_preserves_all_operator_handlers(self):
        signals = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
        original = {number: signal.getsignal(number) for number in signals}
        real_signal = signal.signal
        injected = False

        def interrupt_on_first_restore(number, handler):
            nonlocal injected
            previous = real_signal(number, handler)
            if number == signal.SIGINT and handler is original[number] and not injected:
                injected = True
                os.kill(os.getpid(), signal.SIGINT)
            return previous

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = ManagedProcess(
                [sys.executable, "-c", "import time; time.sleep(30)"],
                root / "out", root / "err", os.environ.copy(), 3,
                kill_grace_seconds=0.2,
            )
            try:
                with mock.patch("signal.signal", side_effect=interrupt_on_first_restore):
                    with self.assertRaises(KeyboardInterrupt):
                        with process:
                            self.fail("interrupted launch entered the body")
                self.assertTrue(injected)
                self.assertTrue(process.cleanup_verified)
                for number in signals:
                    self.assertIs(signal.getsignal(number), original[number])
            finally:
                for number, handler in original.items():
                    real_signal(number, handler)

    def test_invalid_constructor_values_never_create_logs_or_start_processes(self):
        for command, timeout, grace in [
            ([], 3, 0.2), ("true", 3, 0.2), ([""], 3, 0.2),
            (["true"], 0, 0.2), (["true"], float("nan"), 0.2),
            (["true"], 3, True),
        ]:
            with self.subTest(command=command, timeout=timeout, grace=grace):
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    with mock.patch("subprocess.Popen") as spawn:
                        with self.assertRaises(ValueError):
                            ManagedProcess(command, root / "out", root / "err", os.environ.copy(), timeout, grace)
                        spawn.assert_not_called()
                    self.assertEqual(list(root.iterdir()), [])

    def test_missing_command_fails_with_preserved_stderr(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            missing = str(root / "missing-generated-command")
            with self.assertRaisesRegex(ManagedProcessError, "status 127"):
                with ManagedProcess(
                    [missing], root / "out", root / "err", os.environ.copy(), 3,
                    kill_grace_seconds=0.2,
                ) as process:
                    self.wait_for_exit(process)
                    self.assertEqual(process.finish(), 127)
                    self.assertTrue(process.cleanup_verified)
            self.assertIn(missing, (root / "err").read_text())

    def test_missing_watchdog_fails_start_and_closes_new_log_handles(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = ManagedProcess(
                [sys.executable, "-c", "raise AssertionError('must not launch')"],
                root / "out", root / "err",
                {**os.environ, "PATH": str(root / "no-executables")}, 3,
                kill_grace_seconds=0.2,
            )
            opened = []
            real_open = Path.open

            def record_open(path, *args, **kwargs):
                result = real_open(path, *args, **kwargs)
                opened.append(result)
                return result

            with mock.patch.object(Path, "open", record_open):
                with self.assertRaises(FileNotFoundError):
                    with process:
                        self.fail("missing watchdog entered the body")
            self.assertEqual(len(opened), 2)
            self.assertTrue(all(stream.closed for stream in opened))
            self.assertFalse(process.cleanup_verified)
            with self.assertRaisesRegex(ManagedProcessError, "not been started"):
                _ = process.leader_pid
            self.assertEqual((root / "out").read_bytes(), b"")
            self.assertEqual((root / "err").read_bytes(), b"")

    def test_existing_log_paths_are_never_overwritten_or_used_to_launch(self):
        for existing in ["out", "err"]:
            with self.subTest(existing=existing), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / existing).write_text("preserve generated evidence")
                with mock.patch("subprocess.Popen") as spawn:
                    with self.assertRaises(FileExistsError):
                        with ManagedProcess(
                            [sys.executable, "-c", "raise AssertionError('must not launch')"],
                            root / "out", root / "err", os.environ.copy(), 3,
                        ):
                            self.fail("existing evidence entered the body")
                    spawn.assert_not_called()
                self.assertEqual((root / existing).read_text(), "preserve generated evidence")

    def test_cleanup_error_annotates_original_failure_and_keeps_cleanup_unverified(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = ManagedProcess(
                [sys.executable, "-c", "import time; time.sleep(30)"],
                root / "out", root / "err", os.environ.copy(), 0.4,
                kill_grace_seconds=0.2,
            )
            failure = RuntimeError("generated body failure")
            try:
                with mock.patch("os.killpg", side_effect=PermissionError("generated signal denial")):
                    with self.assertRaises(RuntimeError) as raised:
                        with process:
                            self.wait_for_child(process)
                            raise failure
                self.assertIs(raised.exception, failure)
                self.assertFalse(process.cleanup_verified)
                notes = "\n".join(failure.__notes__)
                self.assertIn("generated signal denial", notes)
                self.assertIn(f"leader {process.leader_pid}", notes)
            finally:
                # The OS-boundary injection has ended; let the independent
                # watchdog expire and consume the same still-owned leader.
                self.wait_for_exit(process)
                process.finish()

    def test_nonzero_early_exit_is_returned_but_never_accepted_by_context(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ManagedProcessError, "status 7"):
                with ManagedProcess(
                    [sys.executable, "-c", "raise SystemExit(7)"],
                    root / "out", root / "err", os.environ.copy(), 3,
                    kill_grace_seconds=0.2,
                ) as process:
                    self.wait_for_exit(process)
                    self.assertEqual(process.finish(), 7)
                    self.assertTrue(process.cleanup_verified)

    def test_early_finish_stops_child_and_latches_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ManagedProcessError, "before the watchdog exited"):
                with ManagedProcess(
                    [sys.executable, "-c", "import time; time.sleep(30)"],
                    root / "out", root / "err", os.environ.copy(), 3,
                    kill_grace_seconds=0.2,
                ) as process:
                    child = self.wait_for_child(process)
                    with self.assertRaisesRegex(ManagedProcessError, "before the watchdog exited"):
                        process.finish()
                    self.assertTrue(process.cleanup_verified)
                    self.assert_not_running(child)
                    # Swallowing finish's error must not turn context exit green.

    def test_body_exceptions_and_base_exceptions_keep_identity_after_cleanup(self):
        class GeneratedBaseException(BaseException):
            pass

        for failure in [RuntimeError("generated"), KeyboardInterrupt(), SystemExit(7), GeneratedBaseException()]:
            with self.subTest(exception=type(failure).__name__), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                with self.assertRaises(type(failure)) as raised:
                    with ManagedProcess(
                        [sys.executable, "-c", "import time; time.sleep(30)"],
                        root / "out", root / "err", os.environ.copy(), 3,
                        kill_grace_seconds=0.2,
                    ) as process:
                        child = self.wait_for_child(process)
                        raise failure
                self.assertIs(raised.exception, failure)
                self.assertTrue(process.cleanup_verified)
                self.assert_not_running(child)
                with self.assertRaises(ChildProcessError):
                    os.waitid(os.P_PID, process.leader_pid, os.WEXITED | os.WNOHANG)

    def test_watchdog_forces_kill_when_generated_child_ignores_term(self):
        program = "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); print('ready', flush=True); time.sleep(30)"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ManagedProcessError, "status -9"):
                with ManagedProcess(
                    [sys.executable, "-c", program],
                    root / "out", root / "err", os.environ.copy(), 0.6,
                    kill_grace_seconds=0.2,
                ) as process:
                    child = self.wait_for_child(process)
                    self.wait_for_text(root / "out", "ready\n")
                    # GNU timeout, not a cooperative coordinator, escalates.
                    time.sleep(1)
                    self.assertTrue(process.exited())
                    self.assert_not_running(child)
                    self.assertEqual(process.finish(), -signal.SIGKILL)
                    self.assertTrue(process.cleanup_verified)

    def test_watchdog_expires_without_coordinator_polling(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ManagedProcessError, "status 124"):
                with ManagedProcess(
                    [sys.executable, "-c", "import time; time.sleep(30)"],
                    root / "out", root / "err", os.environ.copy(), 0.3,
                    kill_grace_seconds=0.2,
                ) as process:
                    child = self.wait_for_child(process)
                    # No polling or finish/cleanup calls can enforce this bound.
                    time.sleep(0.7)
                    self.assertTrue(process.exited())
                    self.assert_not_running(child)
                    self.assertEqual(process.finish(), 124)
                    self.assertTrue(process.cleanup_verified)

    def test_interrupt_during_launch_cannot_leave_an_unregistered_watchdog(self):
        launched = []
        real_popen = subprocess.Popen

        def interrupt_after_spawn(*args, **kwargs):
            process = real_popen(*args, **kwargs)
            launched.append(process)
            os.kill(os.getpid(), signal.SIGINT)
            return process

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            try:
                with mock.patch("subprocess.Popen", side_effect=interrupt_after_spawn):
                    with self.assertRaises(KeyboardInterrupt):
                        with ManagedProcess(
                            [sys.executable, "-c", "import time; time.sleep(30)"],
                            root / "out", root / "err", os.environ.copy(), 3,
                            kill_grace_seconds=0.2,
                        ):
                            self.fail("interrupted launch entered the body")
                self.assertEqual(len(launched), 1)
                self.assert_not_running(launched[0].pid)
                self.assertIsNotNone(launched[0].returncode)
            finally:
                # Failure cleanup for this generated OS-boundary fault only.
                # returncode=None means this test still owns an unreaped PID.
                for process in launched:
                    if process.returncode is None:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait(timeout=2)

    def test_successful_wrapper_cannot_hide_a_live_descendant(self):
        program = """
import os, time
child = os.fork()
if child == 0:
    time.sleep(30)
else:
    print(child, flush=True)
    time.sleep(0.2)
"""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ManagedProcessError, "descendant"):
                with ManagedProcess(
                    [sys.executable, "-c", program], root / "out", root / "err",
                    os.environ.copy(), 3, kill_grace_seconds=0.2,
                ) as process:
                    self.wait_for_child(process)
                    self.wait_for_exit(process)
                    process.finish()
            descendant = int((root / "out").read_text())
            self.assert_not_running(descendant)

    def test_natural_completion_retains_owned_leader_until_finish(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with ManagedProcess(
                [sys.executable, "-c", "import time; print('generated', flush=True); time.sleep(0.2)"],
                root / "stdout.log",
                root / "stderr.log",
                os.environ.copy(),
                timeout_seconds=3,
                kill_grace_seconds=0.2,
            ) as process:
                child = self.wait_for_child(process)
                self.assertNotEqual(child, process.leader_pid)
                self.assertEqual(os.getpgid(child), process.leader_pid)
                self.wait_for_exit(process)
                # Repeated observation must not consume the owned group leader.
                self.assertTrue(process.exited())
                self.assertFalse(process.cleanup_verified)
                observed = os.waitid(os.P_PID, process.leader_pid, os.WEXITED | os.WNOWAIT)
                self.assertEqual(observed.si_status, 0)
                self.assertEqual(process.finish(), 0)
                self.assertTrue(process.cleanup_verified)
                with self.assertRaises(ChildProcessError):
                    os.waitid(os.P_PID, process.leader_pid, os.WEXITED | os.WNOHANG)
            self.assertEqual((root / "stdout.log").read_text(), "generated\n")
            self.assertEqual((root / "stderr.log").read_text(), "")

    def wait_for_child(self, process):
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            child = process.child_pid
            if child is not None:
                return child
            time.sleep(0.005)
        self.fail("generated child did not appear")

    def wait_for_exit(self, process):
        deadline = time.monotonic() + 4
        while time.monotonic() < deadline:
            if process.exited():
                return
            time.sleep(0.005)
        self.fail("generated process exceeded its test deadline")

    def wait_for_text(self, path, expected):
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if path.read_text() == expected:
                return
            time.sleep(0.005)
        self.fail("generated process did not report its ready state")

    def assert_not_running(self, pid):
        stat = Path(f"/proc/{pid}/stat")
        if stat.exists():
            self.assertIn(stat.read_text().rsplit(")", 1)[1].split()[0], {"Z", "X"})


if __name__ == "__main__":
    unittest.main()
