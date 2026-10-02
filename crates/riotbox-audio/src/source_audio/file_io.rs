//! Original-source file admission. Symlinks to regular files are supported;
//! capture/export artifacts deliberately have separate no-follow contracts.

use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    path::Path,
};

use super::{
    SourceAudioError,
    resource_limits::{SourceAudioLimits, SourceAudioResource, check_limit, reserve_payload},
};

/// Read one admitted original-source file for same-buffer decode and hashing.
/// On Unix, nonblocking open prevents a FIFO from waiting for a producer before
/// the opened descriptor can be rejected. Regular-file I/O itself is not timed.
pub fn read_source_wav_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, SourceAudioError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(io_error)?;
    read_opened_wav_bytes(file)
}

/// Read an already opened regular WAV descriptor under the V1 byte budget.
/// Callers own path/no-follow admission; this never reopens the path. Reads
/// begin at the descriptor's current position; its total size must fit V1.
pub fn read_opened_wav_bytes(mut file: File) -> Result<Vec<u8>, SourceAudioError> {
    let metadata = file.metadata().map_err(io_error)?;
    if !metadata.is_file() {
        return Err(SourceAudioError::Io(
            "source WAV is not a regular file".into(),
        ));
    }
    read_bounded_bytes(
        &mut file,
        metadata.len(),
        SourceAudioLimits::V1.encoded_bytes,
    )
}

fn read_bounded_bytes(
    mut reader: impl Read,
    declared_bytes: u64,
    limit: usize,
) -> Result<Vec<u8>, SourceAudioError> {
    check_limit(SourceAudioResource::EncodedBytes, declared_bytes, limit)?;
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        // An overrun probe lives on the stack, never in the admitted payload.
        let remaining = limit - bytes.len();
        let read_len = chunk.len().min(remaining.saturating_add(1));
        let count = match reader.read(&mut chunk[..read_len]) {
            Ok(0) => return Ok(bytes),
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(io_error(error)),
        };
        let needed = bytes.len() + count;
        check_limit(SourceAudioResource::EncodedBytes, needed as u64, limit)?;
        if needed > bytes.capacity() {
            let capacity = bytes.capacity().saturating_mul(2).max(needed).min(limit);
            reserve_payload(&mut bytes, capacity, SourceAudioResource::EncodedBytes)?;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}

fn io_error(error: std::io::Error) -> SourceAudioError {
    SourceAudioError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::read_source_wav_bytes;
    use crate::source_audio::{
        SOURCE_AUDIO_MAX_ENCODED_BYTES_V1, SourceAudioError, SourceAudioResource,
    };
    use std::{
        fs,
        io::{self, Cursor, Read},
    };

    #[test]
    fn memory_limit_rejects_understated_metadata_without_returning_truncated_bytes() {
        let bytes = b"12345";
        let error = super::read_bounded_bytes(&bytes[..], 1, 4).unwrap_err();
        assert_eq!(
            error,
            SourceAudioError::ResourceLimitExceeded {
                resource: SourceAudioResource::EncodedBytes,
                limit: 4,
                required: 5,
            }
        );
    }

    #[test]
    fn memory_limit_exact_boundary_and_empty_input_preserve_all_bytes() {
        for bytes in [b"".as_slice(), b"1234".as_slice()] {
            let result = super::read_bounded_bytes(bytes, bytes.len() as u64, bytes.len()).unwrap();
            assert_eq!(result, bytes);
            assert!(result.capacity() <= bytes.len());
        }
    }

    #[test]
    fn memory_limit_rejects_metadata_before_attempting_any_read() {
        struct MustNotRead;
        impl Read for MustNotRead {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                panic!("metadata rejection must precede reading")
            }
        }
        assert_eq!(
            super::read_bounded_bytes(MustNotRead, 5, 4),
            Err(SourceAudioError::ResourceLimitExceeded {
                resource: SourceAudioResource::EncodedBytes,
                limit: 4,
                required: 5,
            })
        );
    }

    #[test]
    fn memory_limit_short_reads_and_interrupted_overrun_probe_stop_at_one_extra_byte() {
        struct ShortReader {
            bytes: Cursor<Vec<u8>>,
            interrupt_at: u64,
            interrupted: bool,
        }
        impl Read for ShortReader {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                if self.bytes.position() == self.interrupt_at && !self.interrupted {
                    self.interrupted = true;
                    return Err(io::ErrorKind::Interrupted.into());
                }
                self.bytes.read(&mut output[..1])
            }
        }
        for interrupt_at in [0, 4] {
            let mut reader = ShortReader {
                bytes: Cursor::new(b"123456789".to_vec()),
                interrupt_at,
                interrupted: false,
            };
            let error = super::read_bounded_bytes(&mut reader, 1, 4).unwrap_err();
            assert_eq!(reader.bytes.position(), 5);
            assert!(reader.interrupted);
            assert_eq!(
                error,
                SourceAudioError::ResourceLimitExceeded {
                    resource: SourceAudioResource::EncodedBytes,
                    limit: 4,
                    required: 5,
                }
            );
        }
    }

    #[test]
    fn memory_limit_multi_chunk_read_preserves_bytes_and_bounded_capacity() {
        let original = vec![42; 20_003];
        let bytes = super::read_bounded_bytes(&original[..], 0, original.len()).unwrap();
        assert_eq!(bytes, original);
        assert!(bytes.capacity() <= original.len());
    }

    #[test]
    fn memory_limit_read_error_does_not_return_partial_success() {
        struct FaultyReader(bool);
        impl Read for FaultyReader {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                if self.0 {
                    return Err(io::Error::other("injected read failure"));
                }
                self.0 = true;
                output[0] = 42;
                Ok(1)
            }
        }
        assert_eq!(
            super::read_bounded_bytes(FaultyReader(false), 1, 4),
            Err(SourceAudioError::Io("injected read failure".into()))
        );
    }

    #[test]
    fn memory_limit_production_reader_rejects_sparse_oversized_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("generated-sparse.wav");
        let file = fs::File::create(&path).unwrap();
        file.set_len(SOURCE_AUDIO_MAX_ENCODED_BYTES_V1 as u64 + 1)
            .unwrap();
        drop(file);
        assert_eq!(
            read_source_wav_bytes(&path),
            Err(SourceAudioError::ResourceLimitExceeded {
                resource: SourceAudioResource::EncodedBytes,
                limit: SOURCE_AUDIO_MAX_ENCODED_BYTES_V1 as u64,
                required: SOURCE_AUDIO_MAX_ENCODED_BYTES_V1 as u64 + 1,
            })
        );
        assert_eq!(
            fs::metadata(path).unwrap().len(),
            SOURCE_AUDIO_MAX_ENCODED_BYTES_V1 as u64 + 1
        );
    }

    #[test]
    fn read_returns_the_original_bytes_for_hash_and_decode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synthetic.wav");
        let bytes = crate::source_audio::pcm16_wave_bytes(48_000, 1, &[0.25, -0.25]).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert_eq!(read_source_wav_bytes(path).unwrap(), bytes);
    }

    #[cfg(unix)]
    #[test]
    fn opened_descriptor_is_not_replaced_by_a_changed_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("synthetic.wav");
        let original = b"original opened bytes";
        fs::write(&path, original).unwrap();
        let file = fs::File::open(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        // The internal descriptor seam has no path to reopen or re-resolve.
        assert_eq!(super::read_opened_wav_bytes(file).unwrap(), original);
    }
}
