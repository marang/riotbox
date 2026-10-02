mod cache;
mod file_io;
mod resource_limits;

pub use cache::{
    SourceAudioCache, SourceAudioError, SourceAudioWindow, pcm16_wave_bytes,
    write_interleaved_pcm16_wav,
};
pub use file_io::{read_opened_wav_bytes, read_source_wav_bytes};
pub use resource_limits::{
    SOURCE_AUDIO_MAX_DECODED_SAMPLES_V1, SOURCE_AUDIO_MAX_ENCODED_BYTES_V1, SourceAudioResource,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod file_admission_tests;
