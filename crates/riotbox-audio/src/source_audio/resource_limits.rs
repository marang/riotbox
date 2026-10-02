//! Per-admission WAV/PCM payload budgets, not a process-wide memory ceiling.

use super::SourceAudioError;

/// Version 1 admits at most 256 MiB of encoded WAV bytes, including headers.
pub const SOURCE_AUDIO_MAX_ENCODED_BYTES_V1: usize = 256 * 1024 * 1024;
/// Version 1 admits at most 256 MiB of interleaved decoded `f32` payload.
pub const SOURCE_AUDIO_MAX_DECODED_SAMPLES_V1: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceAudioResource {
    EncodedBytes,
    DecodedSamples,
}

impl SourceAudioResource {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::EncodedBytes => "encoded WAV bytes",
            Self::DecodedSamples => "decoded interleaved samples",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SourceAudioLimits {
    pub encoded_bytes: usize,
    pub decoded_samples: usize,
}

impl SourceAudioLimits {
    pub const V1: Self = Self {
        encoded_bytes: SOURCE_AUDIO_MAX_ENCODED_BYTES_V1,
        decoded_samples: SOURCE_AUDIO_MAX_DECODED_SAMPLES_V1,
    };
}

pub(super) fn check_limit(
    resource: SourceAudioResource,
    required: u64,
    limit: usize,
) -> Result<(), SourceAudioError> {
    if required > limit as u64 {
        return Err(SourceAudioError::ResourceLimitExceeded {
            resource,
            limit: limit as u64,
            required,
        });
    }
    Ok(())
}

pub(super) fn reserve_payload<T>(
    buffer: &mut Vec<T>,
    capacity: usize,
    resource: SourceAudioResource,
) -> Result<(), SourceAudioError> {
    buffer
        .try_reserve_exact(capacity.saturating_sub(buffer.len()))
        .map_err(|_| SourceAudioError::AllocationFailed { resource })
}

#[cfg(test)]
mod tests {
    use super::{SourceAudioError, SourceAudioResource, reserve_payload};

    #[test]
    fn impossible_reservation_returns_typed_failure_without_allocating() {
        let mut samples = Vec::<f32>::new();
        assert_eq!(
            reserve_payload(
                &mut samples,
                usize::MAX,
                SourceAudioResource::DecodedSamples
            ),
            Err(SourceAudioError::AllocationFailed {
                resource: SourceAudioResource::DecodedSamples
            }),
        );
        assert_eq!(samples.capacity(), 0);
    }
}
