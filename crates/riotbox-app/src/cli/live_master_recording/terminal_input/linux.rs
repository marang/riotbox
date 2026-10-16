//! Linux FD and raw-mode ownership; no terminal event decoder or audio work.
use std::{
    io,
    os::fd::{AsFd, AsRawFd},
};

use super::byte_input::{ByteRead, ByteReader};

enum DescriptorState {
    Active { original_flags: Option<i32> },
    Restored,
}

pub(super) struct NonBlockingReader<R: AsFd> {
    reader: R,
    descriptor: DescriptorState,
    unavailable: bool,
}

impl<R: AsFd> NonBlockingReader<R> {
    pub(super) fn new(reader: R) -> io::Result<Self> {
        let original_flags = get_flags(&reader)?;
        if original_flags & libc::O_NONBLOCK == 0 {
            set_flags(&reader, original_flags | libc::O_NONBLOCK)?;
        }
        Ok(Self {
            reader,
            descriptor: DescriptorState::Active {
                original_flags: (original_flags & libc::O_NONBLOCK == 0).then_some(original_flags),
            },
            unavailable: false,
        })
    }

    pub(super) fn restore(&mut self) -> io::Result<()> {
        self.unavailable = true;
        if let DescriptorState::Active {
            original_flags: Some(flags),
        } = self.descriptor
        {
            set_flags(&self.reader, flags)?;
        }
        self.descriptor = DescriptorState::Restored;
        Ok(())
    }
}

impl<R: AsFd> ByteReader for NonBlockingReader<R> {
    fn read_byte(&mut self) -> io::Result<ByteRead> {
        if self.unavailable || matches!(self.descriptor, DescriptorState::Restored) {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "recording input descriptor is unavailable or restored",
            ));
        }
        let mut byte = [0_u8; 1];
        // SAFETY: the FD is borrowed and the writable buffer has exactly the
        // supplied length. No reference or pointer escapes this syscall.
        let count = unsafe {
            libc::read(
                self.reader.as_fd().as_raw_fd(),
                byte.as_mut_ptr().cast(),
                byte.len(),
            )
        };
        let result = match count {
            1 => Ok(ByteRead::Byte(byte[0])),
            0 => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "live recording terminal input closed",
            )),
            _ => read_error(io::Error::last_os_error()),
        };
        if result.is_err() {
            self.unavailable = true;
        }
        result
    }
}

impl<R: AsFd> Drop for NonBlockingReader<R> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn read_error(error: io::Error) -> io::Result<ByteRead> {
    match error.kind() {
        io::ErrorKind::WouldBlock => Ok(ByteRead::Pending),
        io::ErrorKind::Interrupted => Ok(ByteRead::Interrupted),
        _ => Err(error),
    }
}

fn get_flags(reader: &impl AsFd) -> io::Result<i32> {
    // SAFETY: F_GETFL has no pointer argument; the borrowed FD stays valid.
    let flags = unsafe { libc::fcntl(reader.as_fd().as_raw_fd(), libc::F_GETFL) };
    if flags == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(flags)
    }
}

fn set_flags(reader: &impl AsFd, flags: i32) -> io::Result<()> {
    // SAFETY: F_SETFL takes an integer mask, not a pointer; the FD stays valid.
    if unsafe { libc::fcntl(reader.as_fd().as_raw_fd(), libc::F_SETFL, flags) } == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(super) trait RawModeOps {
    fn enabled(&mut self) -> io::Result<bool>;
    fn enable(&mut self) -> io::Result<()>;
    fn restore(&mut self) -> io::Result<()>;
}

pub(super) struct CrosstermRawMode;

impl RawModeOps for CrosstermRawMode {
    fn enabled(&mut self) -> io::Result<bool> {
        crossterm::terminal::is_raw_mode_enabled()
    }
    fn enable(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()
    }
    fn restore(&mut self) -> io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }
}

pub(super) struct RawModeGuard<Ops: RawModeOps> {
    ops: Ops,
    owned: bool,
}

impl<Ops: RawModeOps> RawModeGuard<Ops> {
    fn new(mut ops: Ops) -> io::Result<Self> {
        let owned = !ops.enabled()?;
        if owned {
            ops.enable()?;
        }
        Ok(Self { ops, owned })
    }

    pub(super) fn restore(&mut self) -> io::Result<()> {
        if self.owned {
            self.ops.restore()?;
            self.owned = false;
        }
        Ok(())
    }
}

impl<Ops: RawModeOps> Drop for RawModeGuard<Ops> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

pub(super) fn acquire<R: AsFd, Ops: RawModeOps>(
    reader: R,
    ops: Ops,
) -> io::Result<(NonBlockingReader<R>, RawModeGuard<Ops>)> {
    let mut reader = NonBlockingReader::new(reader)?;
    match RawModeGuard::new(ops) {
        Ok(raw_mode) => Ok((reader, raw_mode)),
        Err(error) => match reader.restore() {
            Ok(()) => Err(error),
            Err(rollback) => Err(io::Error::new(
                error.kind(),
                format!("{error}; descriptor flag rollback failed: {rollback}"),
            )),
        },
    }
}

pub(super) fn restore_resources(
    restore_flags: impl FnOnce() -> io::Result<()>,
    restore_raw: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    let flags = restore_flags();
    let raw = restore_raw();
    match (flags, raw) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(flags), Ok(())) => Err(io::Error::new(
            flags.kind(),
            format!("descriptor flag restoration failed: {flags}"),
        )),
        (Ok(()), Err(raw)) => Err(io::Error::new(
            raw.kind(),
            format!("raw-mode restoration failed: {raw}"),
        )),
        (Err(flags), Err(raw)) => Err(io::Error::new(
            flags.kind(),
            format!(
                "descriptor flag restoration failed: {flags}; raw-mode restoration failed: {raw}"
            ),
        )),
    }
}

#[cfg(test)]
mod tests;
