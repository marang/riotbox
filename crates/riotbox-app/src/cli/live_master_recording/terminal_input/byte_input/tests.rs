use std::{collections::VecDeque, io, time::Duration};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{ByteInput, ByteRead, ByteReader};

struct GeneratedReader {
    bytes: VecDeque<u8>,
    reads: usize,
    next_result: Option<io::Result<ByteRead>>,
}

impl GeneratedReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.iter().copied().collect(),
            reads: 0,
            next_result: None,
        }
    }
}

impl ByteReader for GeneratedReader {
    fn read_byte(&mut self) -> io::Result<ByteRead> {
        self.reads += 1;
        if let Some(result) = self.next_result.take() {
            return result;
        }
        Ok(self
            .bytes
            .pop_front()
            .map_or(ByteRead::Pending, ByteRead::Byte))
    }
}

#[test]
fn plain_ascii_fill_consumes_exactly_one_byte_without_draining_the_next_control() {
    let mut input = ByteInput::new(GeneratedReader::new(b"fq"));
    let key = input.poll_at(Duration::ZERO).unwrap();
    assert_eq!(
        key,
        Some(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE))
    );
    assert_eq!(input.reader_mut().reads, 1);
    assert_eq!(input.reader_mut().bytes.front(), Some(&b'q'));
}

#[test]
fn fragmented_csi_never_reinterprets_its_final_f_as_fill() {
    let mut input = ByteInput::new(GeneratedReader::new(b"\x1b[fq"));
    assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
    assert_eq!(input.poll_at(Duration::from_millis(10)).unwrap(), None);
    let ignored = input.poll_at(Duration::from_millis(20)).unwrap();
    assert_eq!(
        ignored,
        Some(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE))
    );
    assert_eq!(input.reader_mut().reads, 3);
    assert_eq!(input.reader_mut().bytes.front(), Some(&b'q'));
}

#[test]
fn standalone_escape_expires_before_reading_a_late_continuation() {
    let mut input = ByteInput::new(GeneratedReader::new(b"\x1bf"));
    assert_eq!(input.poll_at(Duration::from_millis(100)).unwrap(), None);
    assert_eq!(
        input.poll_at(Duration::from_millis(150)).unwrap(),
        Some(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
    );
    assert_eq!(input.reader_mut().reads, 1);
    assert_eq!(input.reader_mut().bytes.front(), Some(&b'f'));
}

#[test]
fn marked_paste_is_one_ignored_token_and_never_executes_its_control_bytes() {
    let bytes = b"\x1b[200~fq\x03\x1b[201~f";
    let mut input = ByteInput::new(GeneratedReader::new(bytes));
    let mut events = Vec::new();
    for _ in bytes {
        if let Some(key) = input.poll_at(Duration::ZERO).unwrap() {
            events.push(key.code);
        }
    }
    assert_eq!(events, [KeyCode::Null, KeyCode::Char('f')]);
    assert_eq!(input.reader_mut().reads, bytes.len());
}

#[test]
fn malformed_utf8_cannot_release_an_ascii_control_and_latches_input_unavailable() {
    let mut input = ByteInput::new(GeneratedReader::new(b"\xc2fq"));
    assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(input.reader_mut().reads, 2);
    assert_eq!(input.reader_mut().bytes.front(), Some(&b'q'));
}

#[test]
fn continuously_available_unfinished_csi_fails_closed_at_the_frozen_byte_limit() {
    let mut bytes = b"\x1b[".to_vec();
    bytes.extend(std::iter::repeat_n(b'1', 62));
    bytes.extend_from_slice(b"fq");
    let mut input = ByteInput::new(GeneratedReader::new(&bytes));
    for expected_reads in 1..=64 {
        assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
        assert_eq!(input.reader_mut().reads, expected_reads);
    }
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(input.reader_mut().reads, 65);
    assert_eq!(input.reader_mut().bytes.front(), Some(&b'q'));
    assert!(std::mem::size_of::<super::Decoder>() <= 64);
}

fn decode(bytes: &[u8]) -> io::Result<Vec<KeyEvent>> {
    let mut input = ByteInput::new(GeneratedReader::new(bytes));
    let mut tokens = Vec::new();
    for expected_reads in 1..=bytes.len() {
        if let Some(key) = input.poll_at(Duration::ZERO)? {
            tokens.push(key);
        }
        assert_eq!(input.reader_mut().reads, expected_reads);
    }
    Ok(tokens)
}

#[test]
fn received_ascii_controls_are_narrow_and_uppercase_is_never_normalized() {
    for byte in 0..=127 {
        if byte == 0x1b {
            continue;
        }
        let expected = match byte {
            b'f' | b'q' => KeyEvent::new(KeyCode::Char(byte as char), KeyModifiers::NONE),
            3 => KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            _ => KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
        };
        assert_eq!(decode(&[byte]).unwrap(), [expected], "ASCII {byte}");
    }
}

#[test]
fn alt_csi_ss3_and_escape_intermediates_are_ignored_as_whole_tokens() {
    for sequence in [
        b"\x1bf".as_slice(),
        b"\x1bq",
        b"\x1b\x03",
        b"\x1b[1;2f",
        b"\x1b[20;5q",
        b"\x1bOq",
        b"\x1bO1;2f",
        b"\x1b(f",
        b"\x1b( f",
    ] {
        let mut bytes = sequence.to_vec();
        bytes.push(b'f');
        assert_eq!(
            decode(&bytes).unwrap(),
            [
                KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)
            ],
            "{sequence:?}"
        );
    }
}

#[test]
fn valid_unicode_scalar_boundaries_and_alt_unicode_are_ignored() {
    for scalar in [
        '\u{80}',
        '\u{7ff}',
        '\u{800}',
        '\u{d7ff}',
        '\u{e000}',
        '\u{ffff}',
        '\u{10000}',
        '\u{10ffff}',
    ] {
        for alt in [false, true] {
            let mut bytes = Vec::new();
            if alt {
                bytes.push(0x1b);
            }
            bytes.extend_from_slice(scalar.encode_utf8(&mut [0; 4]).as_bytes());
            bytes.push(b'f');
            assert_eq!(
                decode(&bytes).unwrap(),
                [
                    KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
                    KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)
                ],
                "scalar {scalar:?}, alt={alt}"
            );
        }
    }
}

#[test]
fn malformed_or_unsupported_tokens_latch_before_any_embedded_control_can_escape() {
    for bad in [
        b"\x80".as_slice(),
        b"\xc0f",
        b"\xc1q",
        b"\xe0\x9f\x80",
        b"\xed\xa0\x80",
        b"\xf0\x8f\xbf\xbf",
        b"\xf4\x90\x80\x80",
        b"\xf5",
        b"\xff",
        b"\x1b]",
        b"\x1bP",
        b"\x1bX",
        b"\x1b^",
        b"\x1b_",
        b"\x1b\x1b",
        b"\x1b[ 1",
        b"\x1b[1\x03",
        b"\x1bO1\x1b",
        b"\x1b(\x03",
    ] {
        let mut bytes = bad.to_vec();
        bytes.extend_from_slice(b"fq");
        let mut input = ByteInput::new(GeneratedReader::new(&bytes));
        let error = loop {
            match input.poll_at(Duration::ZERO) {
                Ok(None) => {}
                Ok(Some(key)) => panic!("invalid token {bad:?} released {key:?}"),
                Err(error) => break error,
            }
            assert!(input.reader_mut().reads <= bad.len());
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{bad:?}");
        let reads = input.reader_mut().reads;
        assert_eq!(
            input.poll_at(Duration::from_secs(1)).unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
        assert_eq!(input.reader_mut().reads, reads);
    }
}

#[test]
fn each_sequence_kind_accepts_exactly_64_bytes_but_never_a_65th() {
    for (prefix, body) in [
        (b"\x1b[".as_slice(), b'1'),
        (b"\x1bO".as_slice(), b'1'),
        (b"\x1b(".as_slice(), b' '),
    ] {
        let mut bytes = prefix.to_vec();
        bytes.extend(std::iter::repeat_n(body, 61));
        bytes.push(b'f');
        assert_eq!(bytes.len(), 64);
        assert_eq!(
            decode(&bytes).unwrap(),
            [KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)]
        );
        bytes.insert(2, body);
        assert_eq!(
            decode(&bytes).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn escape_expiry_boundary_and_early_alt_continuation_preserve_the_read_budget() {
    let mut input = ByteInput::new(GeneratedReader::new(b"\x1b"));
    assert_eq!(input.poll_at(Duration::from_millis(100)).unwrap(), None);
    assert_eq!(input.poll_at(Duration::from_millis(149)).unwrap(), None);
    input.reader_mut().bytes.push_back(b'f');
    assert_eq!(
        input.poll_at(Duration::from_millis(149)).unwrap(),
        Some(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE))
    );
    assert_eq!(input.reader_mut().reads, 3);
}

#[test]
fn interrupted_and_pending_reads_yield_once_and_fatal_reads_latch_without_retry() {
    let mut input = ByteInput::new(GeneratedReader::new(b"f"));
    input.reader_mut().next_result = Some(Ok(ByteRead::Interrupted));
    assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
    assert_eq!(input.reader_mut().reads, 1);
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap().unwrap().code,
        KeyCode::Char('f')
    );
    assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
    assert_eq!(input.reader_mut().reads, 3);
    input.reader_mut().next_result = Some(Err(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "generated EOF",
    )));
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(input.reader_mut().reads, 4);
}

#[test]
fn completed_prefixes_do_not_inherit_the_standalone_escape_deadline() {
    for prefix in [b"\x1b[".as_slice(), b"\x1bO", b"\xc2", b"\x1b[200~"] {
        let mut input = ByteInput::new(GeneratedReader::new(prefix));
        for _ in prefix {
            assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
        }
        assert_eq!(input.poll_at(Duration::from_secs(3600)).unwrap(), None);
    }
}

#[test]
fn marked_paste_close_prefix_overlap_never_releases_payload_controls() {
    assert_eq!(
        decode(b"\x1b[200~fq\x03\x1b[20f\x1b\x1b[201~f").unwrap(),
        [
            KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE)
        ]
    );
}

#[test]
fn marked_paste_limit_counts_the_closer_and_overrun_latches() {
    let mut bytes = b"\x1b[200~".to_vec();
    bytes.extend(std::iter::repeat_n(b'q', 65_530));
    bytes.extend_from_slice(b"\x1b[201~");
    assert_eq!(
        decode(&bytes).unwrap(),
        [KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)]
    );
    let mut too_large = b"\x1b[200~".to_vec();
    too_large.extend(std::iter::repeat_n(b'q', 65_536));
    too_large.extend_from_slice(b"fq");
    let mut input = ByteInput::new(GeneratedReader::new(&too_large));
    for _ in 0..65_542 {
        assert_eq!(input.poll_at(Duration::ZERO).unwrap(), None);
    }
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        input.poll_at(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::NotConnected
    );
    assert_eq!(input.reader_mut().bytes.front(), Some(&b'q'));
}
