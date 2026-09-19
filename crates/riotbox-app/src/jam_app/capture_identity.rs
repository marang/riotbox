//! Capture integrity is persisted in Core; these statuses describe this load only.
use riotbox_audio::source_audio::{SourceAudioCache, pcm16_wave_bytes};
use riotbox_core::session::{CaptureAudioIdentity, CaptureAudioIdentityProvenance, CaptureRef};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs::OpenOptions, io::Read, path::Path};

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

pub(super) fn write_wav(
    path: &Path,
    rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<Vec<u8>, String> {
    let bytes = pcm16_wave_bytes(rate, channels, samples).map_err(|error| error.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(path, &bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}
