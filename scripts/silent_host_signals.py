"""Scoped operator interrupts: first signal aborts, cleanup stays bounded."""

from contextlib import contextmanager
import signal


class OperatorInterrupts:
    """Convert termination into stack unwinding, without interrupting cleanup.

    This is a main-thread CLI scope, not a process-global policy installed on
    import. SIGKILL and host loss remain outside recoverable supervision.
    """

    def __init__(self):
        self.received = None
        self._cleaning_up = False
        self._previous = {}

    def _receive(self, signum, _frame):
        if self.received is None:
            self.received = signum
            if not self._cleaning_up:
                self.check()

    def check(self):
        if self.received is not None:
            raise KeyboardInterrupt(f"operator interrupted by {signal.Signals(self.received).name}")

    def __enter__(self):
        signals = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
        old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, signals)
        try:
            self._previous = {signum: signal.getsignal(signum) for signum in signals}
            for signum in signals:
                signal.signal(signum, self._receive)
            signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
        except BaseException:
            # Pending signals can raise while the mask is restored, before the
            # with-statement owns this scope and can invoke __exit__ for us.
            signal.pthread_sigmask(signal.SIG_BLOCK, signals)
            self._restore_handlers(old_mask)
            raise
        return self

    def __exit__(self, exception_type, _exception, _traceback):
        old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, self._previous)
        self._restore_handlers(old_mask)
        if exception_type is None:
            self.check()

    def _restore_handlers(self, old_mask):
        try:
            for signum, handler in self._previous.items():
                signal.signal(signum, handler)
        finally:
            signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)

    @contextmanager
    def cleanup(self):
        previous = self._cleaning_up
        self._cleaning_up = True
        try:
            yield
        finally:
            self._cleaning_up = previous
