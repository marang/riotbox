//! One synchronous request/response exchange, no detached I/O workers.
//! The caller supplies one absolute deadline for both pipe phases.

use std::{
    io::{self, Write},
    process::{ChildStdin, ChildStdout},
    time::Instant,
};

use crate::pipe_platform::{self, PipeDirection};

const READ_CHUNK_BYTES: usize = 8192;

#[derive(Debug)]
pub(crate) enum TransportError {
    Io(io::Error),
    Deadline,
    Eof,
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
                // Retain only trailing bytes, not the largest completed frame's
                // allocation for the rest of this client's lifetime.
                let trailing = self.pending.split_off(end);
                let line = std::mem::replace(&mut self.pending, trailing);
                return decode_line(line);
            }
            scanned = self.pending.len();
            match pipe_platform::read_available(&mut self.stdout, &mut chunk) {
                Ok(0) if self.pending.is_empty() => return Err(TransportError::Eof),
                Ok(0) => {
                    // BufRead::read_line previously delivered a final EOF frame
                    // even without a newline. Preserve that compatibility.
                    return decode_line(std::mem::take(&mut self.pending));
                }
                Ok(count) => self.pending.extend_from_slice(&chunk[..count]),
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
    use super::{READ_CHUNK_BYTES, StdioTransport};
    use std::{
        fs,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };

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
