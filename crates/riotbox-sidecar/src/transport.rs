//! One synchronous request/response exchange, no detached I/O workers.
//! The caller supplies one absolute deadline for both pipe phases.

use std::{
    collections::TryReserveError,
    io::{self, Write},
    process::{ChildStdin, ChildStdout},
    time::Instant,
};

use crate::pipe_platform::{self, PipeDirection};

const READ_CHUNK_BYTES: usize = 8192;

/// Rust stdio response admission V1: UTF-8 wire bytes, including CR/LF when present.
/// This bounds framing storage, not parsed JSON or the peer's process memory.
pub const SIDECAR_MAX_RESPONSE_BYTES_V1: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub(crate) enum TransportError {
    Io(io::Error),
    Deadline,
    Eof,
    ResponseTooLarge,
    ResponseAllocationFailed(TryReserveError),
}

impl From<io::Error> for TransportError {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::TimedOut {
            Self::Deadline
        } else {
            Self::Io(error)
        }
    }
}

pub(crate) struct StdioTransport {
    stdin: ChildStdin,
    stdout: ChildStdout,
    pending: Vec<u8>,
}

impl StdioTransport {
    pub(crate) fn new(stdin: ChildStdin, stdout: ChildStdout) -> io::Result<Self> {
        pipe_platform::configure(&stdin, &stdout)?;
        Ok(Self {
            stdin,
            stdout,
            pending: Vec::new(),
        })
    }

    pub(crate) fn exchange(
        &mut self,
        line: &str,
        deadline: Instant,
    ) -> Result<String, TransportError> {
        let mut remaining = line.as_bytes();
        while !remaining.is_empty() {
            check_deadline(deadline)?;
            match self.stdin.write(remaining) {
                // Windows nonblocking byte pipes can accept zero bytes when full.
                Ok(0) => self.wait(PipeDirection::Write, deadline)?,
                Ok(written) => remaining = &remaining[written..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait(PipeDirection::Write, deadline)?
                }
                Err(error) => return Err(error.into()),
            }
        }
        loop {
            check_deadline(deadline)?;
            match self.stdin.flush() {
                Ok(()) => break,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait(PipeDirection::Write, deadline)?
                }
                Err(error) => return Err(error.into()),
            }
        }
        self.read_line(deadline)
    }

    fn read_line(&mut self, deadline: Instant) -> Result<String, TransportError> {
        let mut chunk = [0; READ_CHUNK_BYTES];
        let mut scanned = 0;
        loop {
            check_deadline(deadline)?;
            // Scan only newly received bytes, not the entire growing frame.
            if let Some(offset) = self.pending[scanned..]
                .iter()
                .position(|byte| *byte == b'\n')
            {
                let end = scanned + offset + 1;
                return take_response_line(&mut self.pending, end);
            }
            scanned = self.pending.len();
            // At the limit, read only one overrun byte to distinguish a valid
            // final EOF frame from an oversized frame. Never append that byte.
            // Restricting reads also leaves the next frame in the pipe when the
            // current one ends exactly at the limit.
            let read_len =
                (SIDECAR_MAX_RESPONSE_BYTES_V1 - self.pending.len()).clamp(1, READ_CHUNK_BYTES);
            match pipe_platform::read_available(&mut self.stdout, &mut chunk[..read_len]) {
                Ok(0) if self.pending.is_empty() => return Err(TransportError::Eof),
                Ok(0) => {
                    // BufRead::read_line previously delivered a final EOF frame
                    // even without a newline. Preserve that compatibility.
                    return decode_line(std::mem::take(&mut self.pending));
                }
                Ok(count) => append_response_bytes(&mut self.pending, &chunk[..count])?,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    self.wait(PipeDirection::Read, deadline)?
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn wait(&self, direction: PipeDirection, deadline: Instant) -> Result<(), TransportError> {
        pipe_platform::wait_ready(&self.stdin, &self.stdout, direction, deadline)
            .map_err(Into::into)
    }
}

fn append_response_bytes(pending: &mut Vec<u8>, chunk: &[u8]) -> Result<(), TransportError> {
    if chunk.len() > SIDECAR_MAX_RESPONSE_BYTES_V1 - pending.len() {
        return Err(TransportError::ResponseTooLarge);
    }
    let needed = pending.len() + chunk.len();
    if needed > pending.capacity() {
        // Keep geometric growth linear overall without requesting capacity
        // beyond the fixed admission limit.
        let capacity = pending
            .capacity()
            .saturating_mul(2)
            .max(needed)
            .min(SIDECAR_MAX_RESPONSE_BYTES_V1);
        pending
            .try_reserve_exact(capacity - pending.len())
            .map_err(TransportError::ResponseAllocationFailed)?;
    }
    pending.extend_from_slice(chunk);
    Ok(())
}

fn take_response_line(pending: &mut Vec<u8>, end: usize) -> Result<String, TransportError> {
    // Retain only the bounded trailing read chunk, not the largest completed
    // frame's allocation. Unlike split_off, allocation failure is recoverable.
    let mut trailing = Vec::new();
    trailing
        .try_reserve_exact(pending.len() - end)
        .map_err(TransportError::ResponseAllocationFailed)?;
    trailing.extend_from_slice(&pending[end..]);
    pending.truncate(end);
    decode_line(std::mem::replace(pending, trailing))
}

fn check_deadline(deadline: Instant) -> Result<(), TransportError> {
    if Instant::now() >= deadline {
        Err(TransportError::Deadline)
    } else {
        Ok(())
    }
}

fn decode_line(bytes: Vec<u8>) -> Result<String, TransportError> {
    String::from_utf8(bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error).into())
}

#[cfg(test)]
mod tests {
    use super::{
        READ_CHUNK_BYTES, SIDECAR_MAX_RESPONSE_BYTES_V1, StdioTransport, TransportError,
        append_response_bytes, take_response_line,
    };
    use std::{
        fs,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

    #[test]
    fn response_buffer_never_appends_or_reserves_an_overrun_byte() {
        assert_eq!(SIDECAR_MAX_RESPONSE_BYTES_V1, 8 * 1024 * 1024);
        let mut pending = Vec::new();
        let chunk = [b' '; READ_CHUNK_BYTES];
        while pending.len() < SIDECAR_MAX_RESPONSE_BYTES_V1 {
            append_response_bytes(&mut pending, &chunk).unwrap();
            assert!(pending.capacity() <= SIDECAR_MAX_RESPONSE_BYTES_V1);
        }
        let capacity = pending.capacity();
        assert!(matches!(
            append_response_bytes(&mut pending, b"\n"),
            Err(TransportError::ResponseTooLarge)
        ));
        assert_eq!(pending.len(), SIDECAR_MAX_RESPONSE_BYTES_V1);
        assert_eq!(pending.capacity(), capacity);
    }

    #[test]
    fn splitting_a_frame_preserves_trailing_bytes_and_releases_large_capacity() {
        let mut pending = Vec::with_capacity(SIDECAR_MAX_RESPONSE_BYTES_V1);
        pending.extend_from_slice(b"first\nsecond\npartial");
        assert_eq!(take_response_line(&mut pending, 6).unwrap(), "first\n");
        assert_eq!(pending, b"second\npartial");
        assert!(pending.capacity() < READ_CHUNK_BYTES);
        assert_eq!(take_response_line(&mut pending, 7).unwrap(), "second\n");
        assert_eq!(pending, b"partial");
    }

    #[test]
    fn completed_large_frame_does_not_remain_allocated_in_the_client_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("synthetic_frame_peer.py");
        fs::write(&script, "import sys\nsys.stdin.readline()\nsys.stdout.buffer.write(b'x' * (1024 * 1024) + b'\\nnext\\n')\nsys.stdout.buffer.flush()\n").unwrap();
        let mut child = Command::new("python3")
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut transport =
            StdioTransport::new(child.stdin.take().unwrap(), child.stdout.take().unwrap()).unwrap();
        let result = transport.exchange("{}\n", Instant::now() + Duration::from_secs(5));
        let _ = child.kill();
        let _ = child.wait();
        let line = result.unwrap();
        assert_eq!(line.len(), 1024 * 1024 + 1);
        assert!(
            transport.pending.capacity() < READ_CHUNK_BYTES,
            "completed frame allocation was retained: {}",
            transport.pending.capacity()
        );
    }
}
