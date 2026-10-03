"""Linux-only, source-free process supervision for the frozen host observation.

GNU timeout is an independent watchdog. Do not replace waitid(WNOWAIT) with
Popen.poll(): the unreaped leader reserves the owned process-group identifier.
"""

from contextlib import contextmanager
import math
import os
from pathlib import Path
import signal
import subprocess
import threading
import time


class ManagedProcessError(RuntimeError):
    """The owned process could not complete the supervised lifecycle."""


class ManagedProcess:
    def __init__(
        self, command: list[str], stdout_path, stderr_path, env,
        timeout_seconds, kill_grace_seconds=2,
    ):
        if (not isinstance(command, list) or not command
                or not all(isinstance(part, str) for part in command) or not command[0]):
            raise ValueError("command must be a nonempty list of strings")
        for value in (timeout_seconds, kill_grace_seconds):
            if (isinstance(value, bool) or not isinstance(value, (int, float))
                    or not math.isfinite(value) or value <= 0):
                raise ValueError("process deadline and kill grace must be positive and finite")
        self._command = list(command)
        self._stdout_path = Path(stdout_path)
        self._stderr_path = Path(stderr_path)
        self._env = dict(env)
        self._timeout_seconds = timeout_seconds
        self._kill_grace_seconds = kill_grace_seconds
        self._process = None
        self._child_pid = None
        self._finished = False
        self._returncode = None
        self._completion_error = None
        self.cleanup_verified = False

    def __enter__(self):
        if self._process is not None or self._finished:
            raise ManagedProcessError("a managed process cannot be restarted")
        try:
            # Keep failed-attempt logs, but never overwrite an earlier owner.
            with _defer_interrupts():
                with self._stdout_path.open("xb") as stdout:
                    with self._stderr_path.open("xb") as stderr:
                        self._process = subprocess.Popen(
                            [
                                "timeout", "--signal=TERM",
                                f"--kill-after={self._kill_grace_seconds}s",
                                f"{self._timeout_seconds}s", *self._command,
                            ],
                            stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                            env=self._env, start_new_session=True,
                        )
            return self
        except BaseException as error:
            self._cleanup_preserving(error)
            raise

    @property
    def leader_pid(self):
        if self._process is None:
            raise ManagedProcessError("process has not been started")
        return self._process.pid

    @property
    def child_pid(self):
        if self._child_pid is not None:
            return self._child_pid
        leader = self.leader_pid
        children = Path(f"/proc/{leader}/task/{leader}/children").read_text().split()
        if not children:
            if self.exited():
                raise ManagedProcessError("watchdog exited before its command could be identified")
            return None
        if len(children) != 1:
            raise ManagedProcessError("watchdog has more than one direct child")
        child = int(children[0])
        try:
            state, parent, group, session = _process_identity(child)
        except FileNotFoundError:
            return None
        if parent != leader or group != leader or session != leader:
            raise ManagedProcessError("watchdog child is outside its owned process group")
        if state == "Z":
            return None
        self._child_pid = child
        return child

    def exited(self):
        if self._finished:
            return True
        return self._observe_exit() is not None

    def finish(self):
        if self._finished:
            if self._completion_error is not None:
                raise self._completion_error
            return self._returncode
        if not self.exited():
            error = ManagedProcessError("finish called before the watchdog exited")
            self._completion_error = error
            self._cleanup_preserving(error)
            raise error
        unexpected = _live_group_members(self.leader_pid)
        self._cleanup()
        if unexpected:
            self._completion_error = ManagedProcessError(
                f"watchdog exited with live owned descendants: {unexpected}; group terminated"
            )
            raise self._completion_error
        return self._returncode

    def __exit__(self, _exception_type, exception, _traceback):
        if exception is not None:
            self._cleanup_preserving(exception)
            return False
        if not self._finished:
            error = ManagedProcessError("managed process left without finish()")
            self._cleanup_preserving(error)
            raise error
        if self._completion_error is not None:
            raise self._completion_error
        if self._returncode != 0:
            raise ManagedProcessError(f"watchdog/command exited with status {self._returncode}")
        return False

    def _observe_exit(self):
        try:
            return os.waitid(
                os.P_PID, self.leader_pid, os.WEXITED | os.WNOHANG | os.WNOWAIT,
            )
        except ChildProcessError as error:
            raise ManagedProcessError(
                "owned watchdog was already reaped; refusing an unsafe group signal"
            ) from error

    def _cleanup_preserving(self, error):
        try:
            self._cleanup()
        except BaseException as cleanup_error:
            error.add_note(
                f"owned process cleanup failed for leader {self.leader_pid} "
                f"(command child {self._child_pid}): {cleanup_error}"
            )

    def _cleanup(self):
        if self._process is None or self._finished:
            return
        try:
            with _defer_interrupts():
                self._cleanup_group()
        except BaseException as error:
            self._completion_error = error
            raise

    def _cleanup_group(self):
        self._observe_exit()  # Prove that the unreaped PID still belongs to us.
        leader = self.leader_pid
        if os.getpgid(leader) != leader:
            raise ManagedProcessError("watchdog no longer leads its owned process group")
        started = time.monotonic()
        deadline = started + self._kill_grace_seconds
        cleanup_deadline = started + 2 * self._kill_grace_seconds
        _signal_group(leader, signal.SIGTERM)
        while _live_group_members(leader) and time.monotonic() < deadline:
            time.sleep(0.01)
        if _live_group_members(leader):
            _signal_group(leader, signal.SIGKILL)
        while _live_group_members(leader) and time.monotonic() < cleanup_deadline:
            time.sleep(0.01)
        survivors = _live_group_members(leader)
        if survivors:
            raise ManagedProcessError(f"owned processes survived group cleanup: {survivors}")
        # All group signals and member checks precede the one consuming wait.
        self._returncode = self._process.wait(timeout=max(0, cleanup_deadline - time.monotonic()))
        self._finished = True
        # Dead orphan descendants can await init reaping. They are not running,
        # but are still residual processes: never claim complete disappearance.
        remaining = _live_group_members(leader, include_dead=True)
        while remaining and time.monotonic() < cleanup_deadline:
            time.sleep(0.01)
            remaining = _live_group_members(leader, include_dead=True)
        if remaining:
            raise ManagedProcessError(f"owned descendants remain after leader reap: {remaining}")
        self.cleanup_verified = True


def _process_identity(pid):
    fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    return fields[0], int(fields[1]), int(fields[2]), int(fields[3])


def _live_group_members(group, include_dead=False):
    members = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdecimal():
            continue
        try:
            state, _parent, process_group, _session = _process_identity(int(entry.name))
        except (FileNotFoundError, ProcessLookupError):
            continue
        if process_group == group and (include_dead or state not in {"Z", "X"}):
            members.append(int(entry.name))
    return members


def _signal_group(group, signum):
    try:
        os.killpg(group, signum)
    except ProcessLookupError:
        pass


@contextmanager
def _defer_interrupts():
    """Register ownership or finish cleanup before dispatching termination signals.

    No signal mask leaks into the child: temporary Python handlers reset on exec.
    Python dispatches handlers only on the main thread, so worker calls need no
    handler replacement. The surrounding operator owns its normal signal policy.
    """
    if threading.current_thread() is not threading.main_thread():
        yield
        return
    received = []
    previous = {}

    def remember(signum, frame):
        if not received:
            received.append((signum, frame))

    try:
        for signum in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            previous[signum] = signal.signal(signum, remember)
        yield
    finally:
        # This tiny mask begins only after child creation/cleanup, so the child
        # cannot inherit it. A second interrupt cannot strand a temporary handler
        # halfway through restoration; pending signals dispatch after all restore.
        old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, previous)
        try:
            for signum, handler in previous.items():
                signal.signal(signum, handler)
        finally:
            signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
    if received:
        signum, frame = received[0]
        handler = previous[signum]
        if callable(handler):
            handler(signum, frame)
        elif handler != signal.SIG_IGN:
            raise KeyboardInterrupt(f"supervision interrupted by signal {signum}")
