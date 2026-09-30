//! Platform pipe operations, with no blocking read/write or background workers.

use std::{
    io,
    process::{ChildStdin, ChildStdout},
    time::Instant,
};

#[cfg(windows)]
const WINDOWS_READY_RETRY: std::time::Duration = std::time::Duration::from_millis(1);

#[derive(Clone, Copy)]
pub(crate) enum PipeDirection {
    Read,
    Write,
}

pub(crate) fn configure(stdin: &ChildStdin, stdout: &ChildStdout) -> io::Result<()> {
    #[cfg(unix)]
    {
        set_nonblocking(stdin)?;
        set_nonblocking(stdout)
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Pipes::{PIPE_NOWAIT, SetNamedPipeHandleState};
        let _ = stdout;
        let mode = PIPE_NOWAIT;
        // SAFETY: the borrowed pipe handle stays owned by stdin; the mode pointer
        // is valid for this call, and both optional buffering pointers are null.
        if unsafe {
            SetNamedPipeHandleState(
                stdin.as_raw_handle(),
                &mode,
                std::ptr::null(),
                std::ptr::null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (stdin, stdout);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "bounded stdio pipes are unsupported on this platform",
        ))
    }
}

#[cfg(unix)]
fn set_nonblocking(file: &impl std::os::fd::AsRawFd) -> io::Result<()> {
    let fd = file.as_raw_fd();
    // SAFETY: fd remains borrowed from a live owned pipe; these operations take
    // integer flags, never pointers, and do not close or transfer ownership.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(crate) fn read_available(stdout: &mut ChildStdout, bytes: &mut [u8]) -> io::Result<usize> {
    use std::io::Read;
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{Foundation::ERROR_BROKEN_PIPE, System::Pipes::PeekNamedPipe};
        let mut available = 0;
        // SAFETY: stdout owns this live read handle. Only this control thread
        // reads it; available remains valid and all unused output pointers are null.
        if unsafe {
            PeekNamedPipe(
                stdout.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        } == 0
        {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                Ok(0)
            } else {
                Err(error)
            };
        }
        if available == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        // No other reader can drain these bytes between peek and read. Do not
        // ask ReadFile for more than was observed available on this private pipe.
        let count = bytes.len().min(available as usize);
        stdout.read(&mut bytes[..count])
    }
    #[cfg(not(windows))]
    {
        stdout.read(bytes)
    }
}

pub(crate) fn wait_ready(
    stdin: &ChildStdin,
    stdout: &ChildStdout,
    direction: PipeDirection,
    deadline: Instant,
) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        let (fd, events) = match direction {
            PipeDirection::Read => (stdout.as_raw_fd(), libc::POLLIN),
            PipeDirection::Write => (stdin.as_raw_fd(), libc::POLLOUT),
        };
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(io::ErrorKind::TimedOut)?;
            // Poll has integer-millisecond granularity. Round upward and recheck
            // the same absolute deadline after timeout/interruption.
            let millis = remaining
                .as_millis()
                .saturating_add(1)
                .min(i32::MAX as u128) as i32;
            let mut entry = libc::pollfd {
                fd,
                events,
                revents: 0,
            };
            // SAFETY: entry is one initialized stack element and its borrowed fd
            // remains alive for the complete call; nfds is exactly one.
            let result = unsafe { libc::poll(&mut entry, 1, millis) };
            if result > 0 {
                return Ok(());
            }
            if result < 0 {
                let error = io::Error::last_os_error();
                if error.kind() != io::ErrorKind::Interrupted {
                    return Err(error);
                }
            }
        }
    }
    #[cfg(windows)]
    {
        let _ = (stdin, stdout, direction);
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or(io::ErrorKind::TimedOut)?;
        // Synchronous offline transport, not overlapped I/O. A one-ms retry
        // cadence avoids spinning while preserving the caller's remaining budget.
        std::thread::sleep(remaining.min(WINDOWS_READY_RETRY));
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (stdin, stdout, direction, deadline);
        Err(io::ErrorKind::Unsupported.into())
    }
}
