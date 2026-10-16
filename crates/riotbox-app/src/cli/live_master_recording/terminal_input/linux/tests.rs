//! Owned anonymous descriptors only: never stdin, a TTY or an audio runtime.
use std::{
    cell::{Cell, RefCell},
    fs::File,
    io::{self, Write},
    os::fd::{FromRawFd, OwnedFd},
    rc::Rc,
    sync::mpsc,
    thread,
    time::Duration,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    NonBlockingReader, RawModeGuard, RawModeOps, acquire, get_flags, read_error, restore_resources,
    set_flags,
};
use crate::cli::live_master_recording::terminal_input::byte_input::{
    ByteInput, ByteRead, ByteReader,
};

fn pipe() -> (OwnedFd, File) {
    let mut descriptors = [-1; 2];
    // SAFETY: pipe2 initializes two FDs on success; each is transferred into
    // exactly one owning Rust value immediately after the successful call.
    assert_eq!(
        unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) },
        0
    );
    // SAFETY: the successful call initialized two distinct, still-owned FDs.
    unsafe {
        (
            OwnedFd::from_raw_fd(descriptors[0]),
            File::from_raw_fd(descriptors[1]),
        )
    }
}

#[test]
fn fragmented_escape_pipe_returns_pending_without_a_writer_continuation() {
    let (read, mut write) = pipe();
    assert_eq!(get_flags(&read).unwrap() & libc::O_NONBLOCK, 0);
    write.write_all(b"\x1b[").unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let mut reader = NonBlockingReader::new(read).unwrap();
        assert!(matches!(reader.read_byte().unwrap(), ByteRead::Byte(0x1b)));
        assert!(matches!(reader.read_byte().unwrap(), ByteRead::Byte(b'[')));
        entered_tx.send(()).unwrap();
        let next = reader.read_byte();
        done_tx.send(next).unwrap();
        reader
    });
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let early = done_rx.recv_timeout(Duration::from_millis(250));
    let returned_without_continuation = early.is_ok();
    let result = match early {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Release a genuinely blocked read before asserting RED. An outer
            // process watchdog is a final bound, not the expected failure mode.
            write.write_all(b"A").unwrap();
            done_rx.recv_timeout(Duration::from_secs(2)).unwrap()
        }
        Err(error) => panic!("reader exited without a result: {error}"),
    };
    let _reader = worker.join().unwrap();
    assert!(
        returned_without_continuation,
        "the supervisor did not regain control until synthetic continuation A"
    );
    assert!(matches!(result.unwrap(), ByteRead::Pending));
}

#[test]
fn native_pipe_and_decoder_suppress_fragmented_csi_before_a_standalone_fill() {
    let (read, mut write) = pipe();
    write.write_all(b"\x1b[").unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let mut input = ByteInput::new(NonBlockingReader::new(read).unwrap());
        assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
        assert_eq!(input.poll_at(Duration::from_millis(10)).unwrap(), None);
        entered_tx.send(()).unwrap();
        let pending = input.poll_at(Duration::from_millis(20));
        done_tx.send(pending).unwrap();
        input
    });
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let early = done_rx.recv_timeout(Duration::from_millis(250));
    let returned_without_continuation = early.is_ok();
    let pending = match early {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Regression protection only: release and join a blocked native
            // read before failing; no detached reader outlives this fixture.
            write.write_all(b"A").unwrap();
            done_rx.recv_timeout(Duration::from_secs(2)).unwrap()
        }
        Err(error) => panic!("composed reader exited without a result: {error}"),
    };
    let mut input = worker.join().unwrap();
    assert!(returned_without_continuation);
    assert_eq!(pending.unwrap(), None);

    // The first f is CSI's final byte, not a Fill. Even with the subsequent
    // standalone f already available, one attempt must not drain both bytes.
    write.write_all(b"ff").unwrap();
    assert_eq!(
        input.poll_at(Duration::from_millis(30)).unwrap(),
        Some(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE))
    );
    assert_eq!(
        input.poll_at(Duration::from_millis(40)).unwrap(),
        Some(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE))
    );
    assert_eq!(input.poll_at(Duration::from_millis(50)).unwrap(), None);
    drop(write);
    assert_eq!(
        input.poll_at(Duration::from_millis(60)).unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert_eq!(
        input.poll_at(Duration::from_millis(70)).unwrap_err().kind(),
        io::ErrorKind::NotConnected
    );
}

#[test]
fn eof_latches_reader_unavailable_instead_of_reentering_the_descriptor() {
    let (read, write) = pipe();
    let mut reader = NonBlockingReader::new(read).unwrap();
    drop(write);
    assert_eq!(
        reader.read_byte().err().unwrap().kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert_eq!(
        reader.read_byte().err().unwrap().kind(),
        io::ErrorKind::NotConnected
    );
}

#[test]
fn available_bytes_are_read_one_at_a_time_and_empty_pipe_yields_control() {
    let (read, mut write) = pipe();
    write.write_all(b"fq").unwrap();
    let mut reader = NonBlockingReader::new(read).unwrap();
    assert!(matches!(reader.read_byte().unwrap(), ByteRead::Byte(b'f')));
    assert!(matches!(reader.read_byte().unwrap(), ByteRead::Byte(b'q')));
    assert!(matches!(reader.read_byte().unwrap(), ByteRead::Pending));
}

#[test]
fn original_flags_survive_explicit_restoration_including_preexisting_nonblocking() {
    for prior_nonblocking in [false, true] {
        let (read, _write) = pipe();
        let observer = read.try_clone().unwrap();
        let mut flags = get_flags(&observer).unwrap() | libc::O_APPEND;
        if prior_nonblocking {
            flags |= libc::O_NONBLOCK;
        }
        set_flags(&observer, flags).unwrap();
        let original = get_flags(&observer).unwrap();
        let mut reader = NonBlockingReader::new(read).unwrap();
        assert_eq!(get_flags(&observer).unwrap(), original | libc::O_NONBLOCK);
        reader.restore().unwrap();
        reader.restore().unwrap();
        assert_eq!(get_flags(&observer).unwrap(), original);
        assert_eq!(
            reader.read_byte().err().unwrap().kind(),
            io::ErrorKind::NotConnected
        );
        drop(reader);
        assert_eq!(get_flags(&observer).unwrap(), original);
    }
}

#[test]
fn descriptor_drop_restores_its_original_flags() {
    let (read, _write) = pipe();
    let observer = read.try_clone().unwrap();
    let original = get_flags(&observer).unwrap();
    let reader = NonBlockingReader::new(read).unwrap();
    assert_eq!(get_flags(&observer).unwrap(), original | libc::O_NONBLOCK);
    drop(reader);
    assert_eq!(get_flags(&observer).unwrap(), original);
}

#[test]
fn interrupted_and_would_block_yield_while_other_errors_remain_errors() {
    // Error classification is a pure syscall-result seam, not a claim that
    // this fixture delivered an OS signal or interrupted an actual read.
    assert!(matches!(
        read_error(io::ErrorKind::WouldBlock.into()).unwrap(),
        ByteRead::Pending
    ));
    assert!(matches!(
        read_error(io::ErrorKind::Interrupted.into()).unwrap(),
        ByteRead::Interrupted
    ));
    assert_eq!(
        read_error(io::ErrorKind::PermissionDenied.into())
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::PermissionDenied
    );
}

#[test]
fn unreadable_owned_descriptor_latches_the_actual_syscall_error() {
    let (_read, write) = pipe();
    // A pipe's write end is a valid owned descriptor but cannot be read. This
    // produces EBADF without fabricating or closing a borrowed descriptor.
    let mut reader = NonBlockingReader::new(write).unwrap();
    assert_eq!(
        reader.read_byte().err().unwrap().raw_os_error(),
        Some(libc::EBADF)
    );
    assert_eq!(
        reader.read_byte().err().unwrap().kind(),
        io::ErrorKind::NotConnected
    );
}

#[derive(Default)]
struct RawState {
    prior_raw: bool,
    fail_query: bool,
    fail_enable: bool,
    restore_failures: usize,
    calls: Vec<&'static str>,
}

struct FakeRawMode(Rc<RefCell<RawState>>);

impl RawModeOps for FakeRawMode {
    fn enabled(&mut self) -> io::Result<bool> {
        let mut state = self.0.borrow_mut();
        state.calls.push("query");
        if state.fail_query {
            return Err(io::Error::other("generated raw query failure"));
        }
        Ok(state.prior_raw)
    }

    fn enable(&mut self) -> io::Result<()> {
        let mut state = self.0.borrow_mut();
        state.calls.push("enable");
        if state.fail_enable {
            Err(io::Error::other("generated raw setup failure"))
        } else {
            Ok(())
        }
    }

    fn restore(&mut self) -> io::Result<()> {
        let mut state = self.0.borrow_mut();
        state.calls.push("restore");
        if state.restore_failures > 0 {
            state.restore_failures -= 1;
            Err(io::Error::other("generated raw restore failure"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn raw_setup_failure_rolls_back_acquired_descriptor_flags() {
    for query_failure in [false, true] {
        let (read, _write) = pipe();
        let observer = read.try_clone().unwrap();
        let original = get_flags(&observer).unwrap();
        let state = Rc::new(RefCell::new(RawState {
            fail_query: query_failure,
            fail_enable: !query_failure,
            ..RawState::default()
        }));
        let error = acquire(read, FakeRawMode(state.clone())).err().unwrap();
        assert!(error.to_string().contains(if query_failure {
            "raw query"
        } else {
            "raw setup"
        }));
        assert_eq!(get_flags(&observer).unwrap(), original);
        assert_eq!(
            state.borrow().calls,
            if query_failure {
                vec!["query"]
            } else {
                vec!["query", "enable"]
            }
        );
    }
}

#[test]
fn raw_guard_preserves_preexisting_mode_and_retries_only_unfinished_restoration() {
    for prior_raw in [false, true] {
        let state = Rc::new(RefCell::new(RawState {
            prior_raw,
            restore_failures: usize::from(!prior_raw),
            ..RawState::default()
        }));
        {
            let mut guard = RawModeGuard::new(FakeRawMode(state.clone())).unwrap();
            if prior_raw {
                guard.restore().unwrap();
                guard.restore().unwrap();
            } else {
                assert!(guard.restore().is_err());
            }
        }
        assert_eq!(
            state.borrow().calls,
            if prior_raw {
                vec!["query"]
            } else {
                vec!["query", "enable", "restore", "restore"]
            }
        );
    }
}

#[test]
fn cleanup_attempts_both_resources_and_retains_each_failure() {
    for (flags_fail, raw_fail) in [(false, false), (true, false), (false, true), (true, true)] {
        let flag_attempts = Cell::new(0);
        let raw_attempts = Cell::new(0);
        let result = restore_resources(
            || {
                flag_attempts.set(flag_attempts.get() + 1);
                if flags_fail {
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "generated flags failure",
                    ))
                } else {
                    Ok(())
                }
            },
            || {
                raw_attempts.set(raw_attempts.get() + 1);
                if raw_fail {
                    Err(io::Error::other("generated raw failure"))
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(flag_attempts.get(), 1);
        assert_eq!(raw_attempts.get(), 1);
        assert_eq!(result.is_err(), flags_fail || raw_fail);
        if let Err(error) = result {
            let message = error.to_string();
            assert_eq!(message.contains("generated flags failure"), flags_fail);
            assert_eq!(message.contains("generated raw failure"), raw_fail);
        }
    }
}

#[test]
fn raw_cleanup_failure_does_not_skip_real_flag_restoration_or_drop_retry() {
    let (read, _write) = pipe();
    let observer = read.try_clone().unwrap();
    let original = get_flags(&observer).unwrap();
    let state = Rc::new(RefCell::new(RawState {
        restore_failures: 1,
        ..RawState::default()
    }));
    {
        let (mut reader, mut raw) = acquire(read, FakeRawMode(state.clone())).unwrap();
        let error = restore_resources(|| reader.restore(), || raw.restore()).unwrap_err();
        assert!(error.to_string().contains("generated raw restore failure"));
        assert_eq!(get_flags(&observer).unwrap(), original);
        assert_eq!(
            reader.read_byte().err().unwrap().kind(),
            io::ErrorKind::NotConnected
        );
    }
    assert_eq!(
        state.borrow().calls,
        ["query", "enable", "restore", "restore"]
    );
    assert_eq!(get_flags(&observer).unwrap(), original);
}
