//! Original-source file admission. Symlinks to regular files are supported;
//! capture/export artifacts deliberately have separate no-follow contracts.

use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

use super::SourceAudioError;

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
    read_opened_source(file)
}

fn read_opened_source(mut file: File) -> Result<Vec<u8>, SourceAudioError> {
    if !file.metadata().map_err(io_error)?.is_file() {
        return Err(SourceAudioError::Io(
            "source WAV is not a regular file".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(io_error)?;
    Ok(bytes)
}

fn io_error(error: std::io::Error) -> SourceAudioError {
    SourceAudioError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::read_source_wav_bytes;
    use std::fs;

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
        assert_eq!(super::read_opened_source(file).unwrap(), original);
    }
}
