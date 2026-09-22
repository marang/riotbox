//! Capture integrity is persisted in Core; these statuses describe this load only.
use riotbox_audio::source_audio::{SourceAudioCache, pcm16_wave_bytes};
use riotbox_core::session::{CaptureAudioIdentity, CaptureAudioIdentityProvenance, CaptureRef};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CaptureAudioStatus {
    Loaded,
    LegacyUnverified,
    InvalidIdentity,
    Changed,
    Unavailable { reason: String },
}

pub(super) fn read_wav(path: &Path) -> Result<(Vec<u8>, SourceAudioCache), String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("capture artifact is not a regular file".into());
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let cache =
        SourceAudioCache::from_pcm_wav_bytes(path, &bytes).map_err(|error| error.to_string())?;
    Ok((bytes, cache))
}

pub(super) fn identity(
    bytes: &[u8],
    provenance: CaptureAudioIdentityProvenance,
) -> CaptureAudioIdentity {
    CaptureAudioIdentity {
        sha256: format!("sha256:{:x}", Sha256::digest(bytes)),
        provenance,
    }
}

pub(super) fn load_verified(
    capture: &CaptureRef,
    path: &Path,
) -> Result<SourceAudioCache, CaptureAudioStatus> {
    let expected = capture
        .audio_identity
        .as_ref()
        .ok_or(CaptureAudioStatus::LegacyUnverified)?;
    if !expected.is_valid() {
        return Err(CaptureAudioStatus::InvalidIdentity);
    }
    let (bytes, cache) =
        read_wav(path).map_err(|reason| CaptureAudioStatus::Unavailable { reason })?;
    if format!("sha256:{:x}", Sha256::digest(&bytes)) != expected.sha256 {
        return Err(CaptureAudioStatus::Changed);
    }
    Ok(cache)
}

/// Allocate a fresh artifact; the old locator supplies only its directory.
/// The Session may publish the returned locator only after this succeeds.
pub(super) fn write_new_wav(
    locator: &Path,
    rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<(PathBuf, Vec<u8>), String> {
    let bytes = pcm16_wave_bytes(rate, channels, samples).map_err(|error| error.to_string())?;
    publish_new_wav(locator, bytes, |file, bytes| file.write_all(bytes))
}

fn publish_new_wav(
    locator: &Path,
    bytes: Vec<u8>,
    write: impl FnOnce(&mut File, &[u8]) -> io::Result<()>,
) -> Result<(PathBuf, Vec<u8>), String> {
    let parent = locator
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    // create_new-backed allocation never opens or replaces another Session's
    // artifact. Keep this exclusive filename: no rename/copy/hardlink fallback.
    let mut artifact = tempfile::Builder::new()
        .prefix("capture-")
        .suffix(".wav")
        .tempfile_in(parent)
        .map_err(|error| error.to_string())?;
    write(artifact.as_file_mut(), &bytes).map_err(|error| error.to_string())?;
    let (_file, path) = artifact.keep().map_err(|error| error.to_string())?;
    Ok((path, bytes))
}

#[cfg(test)]
mod tests {
    use super::{publish_new_wav, write_new_wav};
    use std::{
        fs,
        io::{self, Write},
    };

    #[test]
    fn partial_write_failure_discards_only_its_own_allocation() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("cap-01.wav");
        fs::write(&existing, b"previous capture").unwrap();
        let result = publish_new_wav(&existing, vec![1, 2, 3, 4], |file, bytes| {
            file.write_all(&bytes[..2])?;
            Err(io::Error::other("injected short write failure"))
        });
        assert!(result.unwrap_err().contains("injected short write failure"));
        assert_eq!(fs::read(&existing).unwrap(), b"previous capture");
        let entries = fs::read_dir(dir.path())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1, "partial allocation was not cleaned up");
        assert_eq!(entries[0].path(), existing);
    }

    #[test]
    fn repeated_content_still_uses_exclusive_files_without_replacing_legacy_path() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("cap-01.wav");
        fs::write(&existing, b"previous capture").unwrap();
        let (first_path, first_bytes) = write_new_wav(&existing, 48_000, 1, &[0.1; 32]).unwrap();
        let (second_path, second_bytes) = write_new_wav(&existing, 48_000, 1, &[0.1; 32]).unwrap();
        assert_ne!(first_path, second_path);
        assert_eq!(first_bytes, second_bytes);
        assert_eq!(fs::read(first_path).unwrap(), first_bytes);
        assert_eq!(fs::read(second_path).unwrap(), second_bytes);
        assert_eq!(fs::read(existing).unwrap(), b"previous capture");
    }

    #[test]
    fn invalid_encoding_does_not_allocate_or_replace_an_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("cap-01.wav");
        fs::write(&existing, b"previous capture").unwrap();
        assert!(write_new_wav(&existing, 48_000, 0, &[0.1; 32]).is_err());
        assert_eq!(fs::read(existing).unwrap(), b"previous capture");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
