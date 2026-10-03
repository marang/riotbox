"""Original-source encoded admission; capture/export no-follow rules are separate."""

import errno
import os
import stat


# RBX-431 extends the existing Rust V1 encoded budget, not its PCM/RSS guarantees.
SOURCE_WAV_MAX_ENCODED_BYTES_V1 = 256 * 1024 * 1024
_READ_CHUNK_BYTES = 64 * 1024


class SourceResourceLimitError(ValueError):
    def __init__(self, required: int, limit: int) -> None:
        super().__init__(
            f"source WAV encoded bytes exceed admission limit: required {required}, limit {limit}"
        )


def _open_nonblocking(path: str, flags: int) -> int:
    # On Unix, avoid waiting for a FIFO producer before descriptor admission.
    # Symlinks to regular source files intentionally remain supported.
    return os.open(path, flags | getattr(os, "O_NONBLOCK", 0))


def read_source_wav_bytes(path: str) -> bytes:
    """Read one regular descriptor within V1, without reopening for identity."""
    # Unbuffered reads prevent BufferedReader from prefetching past the probe.
    with open(path, "rb", buffering=0, opener=_open_nonblocking) as handle:
        metadata = os.fstat(handle.fileno())
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError("source WAV is not a regular file")
        limit = SOURCE_WAV_MAX_ENCODED_BYTES_V1
        if metadata.st_size > limit:
            raise SourceResourceLimitError(metadata.st_size, limit)

        content = bytearray()
        while True:
            # At the limit, request only the one-byte probe. Do not retain it
            # in the admitted content or return a truncated successful source.
            try:
                chunk = handle.read(min(_READ_CHUNK_BYTES, limit - len(content) + 1))
            except InterruptedError:
                continue
            if chunk is None:
                raise BlockingIOError(errno.EAGAIN, "source WAV read would block before EOF")
            if not chunk:
                return bytes(content)
            required = len(content) + len(chunk)
            if required > limit:
                raise SourceResourceLimitError(required, limit)
            # One accumulator avoids retaining an object per short read.
            content.extend(chunk)
