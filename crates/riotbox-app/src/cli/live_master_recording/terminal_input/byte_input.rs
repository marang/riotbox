//! Fixed-work byte ingress. This owner never touches a terminal or audio state.
use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::RecordingInput;

const ESCAPE_AMBIGUITY_V2: Duration = Duration::from_millis(50);
const MAX_SEQUENCE_BYTES_V2: u8 = 64;
const MAX_PASTE_BYTES_V2: usize = 65_536;

pub(super) enum ByteRead {
    Byte(u8),
    Pending,
    Interrupted,
}

pub(super) trait ByteReader {
    fn read_byte(&mut self) -> io::Result<ByteRead>;
}

pub(super) struct ByteInput<R> {
    reader: R,
    started: Instant,
    decoder: Decoder,
    unavailable: bool,
}

impl<R: ByteReader> ByteInput<R> {
    pub(super) fn new(reader: R) -> Self {
        Self {
            reader,
            started: Instant::now(),
            decoder: Decoder::default(),
            unavailable: false,
        }
    }

    pub(super) fn reader_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    pub(super) fn poll_at(&mut self, elapsed: Duration) -> io::Result<Option<KeyEvent>> {
        if self.unavailable {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "recording input decoder is unavailable",
            ));
        }
        if let State::Escape { at } = self.decoder.state
            && elapsed.saturating_sub(at) >= ESCAPE_AMBIGUITY_V2
        {
            self.decoder.state = State::Ground;
            return Ok(Some(token(KeyCode::Esc)));
        }
        let result = self.reader.read_byte().and_then(|value| match value {
            ByteRead::Byte(byte) => self.decoder.feed(byte, elapsed),
            ByteRead::Pending | ByteRead::Interrupted => Ok(None),
        });
        if result.is_err() {
            self.unavailable = true;
        }
        result
    }
}

#[derive(Default)]
struct Decoder {
    state: State,
}

#[derive(Default)]
enum State {
    #[default]
    Ground,
    Escape {
        at: Duration,
    },
    Sequence(Sequence),
    Paste {
        bytes: usize,
        closer: usize,
    },
    Utf8 {
        remaining: u8,
        minimum: u8,
        maximum: u8,
    },
}

#[derive(Clone, Copy)]
enum SequenceKind {
    Csi,
    Ss3,
    EscapeIntermediate,
}

#[derive(Clone, Copy)]
struct Sequence {
    kind: SequenceKind,
    bytes: u8,
    intermediate: bool,
    paste_prefix: Option<u8>,
}

impl Sequence {
    fn new(kind: SequenceKind) -> Self {
        Self {
            kind,
            bytes: 2,
            intermediate: false,
            paste_prefix: matches!(kind, SequenceKind::Csi).then_some(0),
        }
    }

    fn feed(mut self, byte: u8) -> io::Result<(State, Option<KeyEvent>)> {
        if self.bytes == MAX_SEQUENCE_BYTES_V2 {
            return Err(invalid_input("recording input sequence exceeds V2 limit"));
        }
        self.bytes += 1;
        if matches!(self.kind, SequenceKind::EscapeIntermediate) {
            return match byte {
                0x20..=0x2f => Ok((State::Sequence(self), None)),
                0x30..=0x7e => Ok((State::Ground, Some(token(KeyCode::Null)))),
                _ => Err(invalid_input(
                    "invalid recording input Escape intermediate sequence",
                )),
            };
        }
        self.paste_prefix = self
            .paste_prefix
            .filter(|&index| b"200~"[index as usize] == byte)
            .map(|index| index + 1);
        match byte {
            0x30..=0x3f if !self.intermediate => Ok((State::Sequence(self), None)),
            0x20..=0x2f => {
                self.intermediate = true;
                Ok((State::Sequence(self), None))
            }
            0x40..=0x7e => {
                if self.paste_prefix == Some(4) {
                    Ok((
                        State::Paste {
                            bytes: 0,
                            closer: 0,
                        },
                        None,
                    ))
                } else {
                    Ok((State::Ground, Some(token(KeyCode::Null))))
                }
            }
            _ => Err(invalid_input("invalid recording input CSI/SS3 sequence")),
        }
    }
}

impl Decoder {
    fn feed(&mut self, byte: u8, elapsed: Duration) -> io::Result<Option<KeyEvent>> {
        match self.state {
            State::Ground => match byte {
                0x1b => {
                    self.state = State::Escape { at: elapsed };
                    Ok(None)
                }
                b'f' => Ok(Some(token(KeyCode::Char('f')))),
                b'q' => Ok(Some(token(KeyCode::Char('q')))),
                0x03 => Ok(Some(KeyEvent::new(
                    KeyCode::Char('c'),
                    KeyModifiers::CONTROL,
                ))),
                0x00..=0x7f => Ok(Some(token(KeyCode::Null))),
                _ => {
                    self.state = utf8_start(byte)?;
                    Ok(None)
                }
            },
            State::Escape { .. } => {
                self.state = match byte {
                    b'[' => State::Sequence(Sequence::new(SequenceKind::Csi)),
                    b'O' => State::Sequence(Sequence::new(SequenceKind::Ss3)),
                    0x20..=0x2f => State::Sequence(Sequence::new(SequenceKind::EscapeIntermediate)),
                    b']' | b'P' | b'X' | b'^' | b'_' | 0x1b => {
                        return Err(invalid_input("unsupported recording input Escape protocol"));
                    }
                    0x80..=0xff => utf8_start(byte)?,
                    _ => {
                        self.state = State::Ground;
                        return Ok(Some(token(KeyCode::Null)));
                    }
                };
                Ok(None)
            }
            State::Sequence(sequence) => {
                let (state, key) = sequence.feed(byte)?;
                self.state = state;
                Ok(key)
            }
            State::Paste { bytes, closer } => {
                if bytes == MAX_PASTE_BYTES_V2 {
                    return Err(invalid_input("recording input paste exceeds V2 limit"));
                }
                let closer = if b"\x1b[201~"[closer] == byte {
                    closer + 1
                } else {
                    usize::from(byte == 0x1b)
                };
                if closer == 6 {
                    self.state = State::Ground;
                    Ok(Some(token(KeyCode::Null)))
                } else {
                    self.state = State::Paste {
                        bytes: bytes + 1,
                        closer,
                    };
                    Ok(None)
                }
            }
            State::Utf8 {
                remaining,
                minimum,
                maximum,
            } => {
                if !(minimum..=maximum).contains(&byte) {
                    return Err(invalid_input("invalid UTF-8 in recording input"));
                }
                if remaining == 1 {
                    self.state = State::Ground;
                    Ok(Some(token(KeyCode::Null)))
                } else {
                    self.state = State::Utf8 {
                        remaining: remaining - 1,
                        minimum: 0x80,
                        maximum: 0xbf,
                    };
                    Ok(None)
                }
            }
        }
    }
}

fn utf8_start(byte: u8) -> io::Result<State> {
    let (remaining, minimum, maximum) = match byte {
        0xc2..=0xdf => (1, 0x80, 0xbf),
        0xe0 => (2, 0xa0, 0xbf),
        0xe1..=0xec | 0xee..=0xef => (2, 0x80, 0xbf),
        0xed => (2, 0x80, 0x9f),
        0xf0 => (3, 0x90, 0xbf),
        0xf1..=0xf3 => (3, 0x80, 0xbf),
        0xf4 => (3, 0x80, 0x8f),
        _ => {
            return Err(invalid_input(
                "invalid UTF-8 leading byte in recording input",
            ));
        }
    };
    Ok(State::Utf8 {
        remaining,
        minimum,
        maximum,
    })
}

fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn token(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

impl<R: ByteReader> RecordingInput for ByteInput<R> {
    fn poll_event(&mut self) -> io::Result<Option<KeyEvent>> {
        self.poll_at(self.started.elapsed())
    }
}

#[cfg(test)]
mod tests;
