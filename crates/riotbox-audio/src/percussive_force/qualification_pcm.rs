//! Source-blind in-memory WAV/PCM binding scaffold for Stage-A qualification.
//!
//! This module has no filesystem seam. It binds already-authorized raw WAV
//! bytes to their frozen registry identity and format before any mechanism may
//! consume decoded samples. It does not qualify an event, render a candidate,
//! or establish perceptual force. The Stage-A v1 qualification runner used an
//! independent Python binding path; this Rust module is intentionally private
//! until one future versioned runner owns a reachable Gate -> Bind -> Render
//! integration. Its tests are format/hash evidence, never source-access proof.

use std::{
    error::Error,
    fmt,
    path::{Component, Path},
};

use sha2::{Digest, Sha256};

use super::{
    common::{FrozenEventInput, FrozenEventRegion, PercussiveForceError},
    f3_dynamic::{
        F3DynamicRenderSet, F3PcmEncoding, render_f3_causal_envelope_contrast_dynamic_residual_v2,
    },
};

pub const STAGE_A_PCM_F32LE_HASH_DOMAIN: &str = "riotbox.percussive_force_pcm_f32le.v1";
pub const STAGE_A_SOURCE_REGISTRY_PATH: &str = "docs/benchmarks/source_holdout_rotation_v2.json";
pub const STAGE_A_SOURCE_REGISTRY_SCHEMA: &str = "riotbox.source_holdout_rotation.v2";
pub const STAGE_A_SOURCE_REGISTRY_SHA256: &str =
    "af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812";
pub const STAGE_A_QUALIFICATION_SESSION_KIND: &str = "StageAQualificationSession";
pub const STAGE_A_DEVELOPMENT_ACCESS_LOG_SCHEMA: &str =
    "riotbox.source_holdout_development_access_log.v3";

const MINIMUM_SAMPLE_RATE_HZ: u32 = 32_000;
const MAXIMUM_SAMPLE_RATE_HZ: u32 = 192_000;
const STAGE_A_MAXIMUM_DURATION_SECONDS: u32 = 16;
const PCM_FORMAT_TAG: u16 = 1;
const DEVELOPMENT_PARTITION: &str = "development";
const VERIFIED_AND_DELIVERED_ACCESS_STATUS: &str = "verified_and_delivered_to_owner";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageAPcmEncoding {
    SignedPcm16,
    SignedPcm24,
}

impl StageAPcmEncoding {
    pub const fn valid_bits(self) -> u16 {
        match self {
            Self::SignedPcm16 => 16,
            Self::SignedPcm24 => 24,
        }
    }

    pub fn input_lsb(self) -> f64 {
        2.0_f64.powi(1 - i32::from(self.valid_bits()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageARegistryPcmFormat {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub sample_width_bits: u16,
    pub maximum_duration_seconds: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StageARegistryPcmExpectation {
    case_id: String,
    logical_source_path: String,
    expected_raw_wav_sha256: String,
    format: StageARegistryPcmFormat,
}

/// Metadata delivered by the hardened development-access callback.
///
/// This receipt is crate-private and is not authorization by itself. The safe
/// gate validates every field against the frozen Stage-A registry before it can
/// mint the opaque, single-use [`StageAAuthorizedSourceAccess`] capability.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub(crate) struct StageASafeAccessGateReceipt<'a> {
    pub registry_path: &'a str,
    pub registry_schema: &'a str,
    pub registry_sha256: &'a str,
    pub partition: &'a str,
    pub session_kind: &'a str,
    pub session_id: &'a str,
    pub access_log_schema: &'a str,
    pub access_log_path: &'a str,
    pub access_record_index: usize,
    pub access_record_status: &'a str,
    pub case_id: &'a str,
    pub logical_source_path: &'a str,
    pub expected_raw_wav_sha256: &'a str,
    pub accessed_raw_wav_sha256: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageARegistryBindingProvenance {
    pub path: String,
    pub schema: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageAQualificationSessionProvenance {
    pub kind: String,
    pub session_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageADevelopmentAccessProvenance {
    pub partition: String,
    pub access_log_schema: String,
    pub access_log_path: String,
    pub access_record_index: usize,
    pub access_record_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageAQualificationPcmProvenance {
    pub registry: StageARegistryBindingProvenance,
    pub session: StageAQualificationSessionProvenance,
    pub access: StageADevelopmentAccessProvenance,
}

/// Opaque, single-use proof that the hardened safe-access gate admitted one
/// exact frozen development record. Its fields and constructor are private.
#[derive(Debug, PartialEq, Eq)]
pub struct StageAAuthorizedSourceAccess {
    expectation: StageARegistryPcmExpectation,
    provenance: StageAQualificationPcmProvenance,
    _seal: StageAAuthorizationSeal,
}

#[derive(Debug, PartialEq, Eq)]
struct StageAAuthorizationSeal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageAPcmFormatProvenance {
    pub format_tag: u16,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub byte_rate: u32,
    pub block_align: u16,
    pub container_bits: u16,
    pub valid_bits: u16,
    pub encoding: StageAPcmEncoding,
    pub input_lsb: f64,
    pub maximum_duration_seconds: u32,
}

#[derive(Debug, PartialEq)]
pub struct StageABoundPcm {
    case_id: String,
    /// Registry identity only. This path is never opened by this module.
    logical_source_path: String,
    raw_wav_sha256: String,
    pcm_f32le_sha256: String,
    format: StageAPcmFormatProvenance,
    qualification_provenance: StageAQualificationPcmProvenance,
    frame_count: usize,
    interleaved_samples: Vec<f32>,
    _seal: StageABoundPcmSeal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StageABoundPcmSeal;

impl StageABoundPcm {
    pub(crate) fn case_id(&self) -> &str {
        &self.case_id
    }

    pub(crate) fn logical_source_path(&self) -> &str {
        &self.logical_source_path
    }

    pub(crate) fn raw_wav_sha256(&self) -> &str {
        &self.raw_wav_sha256
    }

    pub(crate) fn pcm_f32le_sha256(&self) -> &str {
        &self.pcm_f32le_sha256
    }

    pub(crate) fn format(&self) -> StageAPcmFormatProvenance {
        self.format
    }

    pub(crate) fn qualification_provenance(&self) -> &StageAQualificationPcmProvenance {
        &self.qualification_provenance
    }

    pub(crate) fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub(crate) fn interleaved_samples(&self) -> &[f32] {
        &self.interleaved_samples
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StageAQualificationPcmError {
    EmptyRegistryField {
        field: &'static str,
    },
    UnsafeLogicalSourcePath,
    InvalidExpectedRawSha256,
    AccessProvenanceMismatch {
        field: &'static str,
    },
    InvalidQualificationSessionId,
    UnauthorizedDevelopmentSourceIdentity,
    RegistryFormatOutOfContract {
        field: &'static str,
        value: u64,
    },
    RawWavSha256Mismatch {
        expected: String,
        actual: String,
    },
    InvalidRiffWave(&'static str),
    DuplicateChunk(&'static str),
    MissingChunk(&'static str),
    UnsupportedFormatTag(u16),
    UnsupportedChannels(u16),
    UnsupportedSampleRate(u32),
    UnsupportedSampleWidth(u16),
    InvalidBlockAlign {
        actual: u16,
        expected: u16,
    },
    InvalidByteRate {
        actual: u32,
        expected: u32,
    },
    RegistryFormatMismatch {
        field: &'static str,
        expected: u64,
        actual: u64,
    },
    EmptyPcmData,
    MisalignedPcmData {
        byte_count: usize,
        block_align: u16,
    },
    PcmDurationExceedsRegistry {
        frame_count: usize,
        maximum_frame_count: u64,
    },
}

impl fmt::Display for StageAQualificationPcmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegistryField { field } => {
                write!(formatter, "Stage-A registry field {field} is empty")
            }
            Self::UnsafeLogicalSourcePath => {
                write!(
                    formatter,
                    "Stage-A logical source path is not safe and relative"
                )
            }
            Self::InvalidExpectedRawSha256 => {
                write!(
                    formatter,
                    "Stage-A expected raw WAV SHA-256 is not canonical"
                )
            }
            Self::AccessProvenanceMismatch { field } => {
                write!(
                    formatter,
                    "Stage-A safe-access provenance mismatch: {field}"
                )
            }
            Self::InvalidQualificationSessionId => {
                write!(formatter, "Stage-A qualification session ID is invalid")
            }
            Self::UnauthorizedDevelopmentSourceIdentity => write!(
                formatter,
                "Stage-A source identity is not one of the four frozen development records"
            ),
            Self::RegistryFormatOutOfContract { field, value } => {
                write!(
                    formatter,
                    "Stage-A registry format {field}={value} is unsupported"
                )
            }
            Self::RawWavSha256Mismatch { expected, actual } => write!(
                formatter,
                "Stage-A raw WAV SHA-256 mismatch: expected {expected}, got {actual}"
            ),
            Self::InvalidRiffWave(reason) => {
                write!(formatter, "invalid Stage-A RIFF/WAVE bytes: {reason}")
            }
            Self::DuplicateChunk(chunk) => {
                write!(formatter, "duplicate Stage-A WAV {chunk} chunk")
            }
            Self::MissingChunk(chunk) => write!(formatter, "missing Stage-A WAV {chunk} chunk"),
            Self::UnsupportedFormatTag(tag) => {
                write!(formatter, "unsupported Stage-A WAV format tag {tag}")
            }
            Self::UnsupportedChannels(channels) => {
                write!(
                    formatter,
                    "unsupported Stage-A WAV channel count {channels}"
                )
            }
            Self::UnsupportedSampleRate(rate) => {
                write!(formatter, "unsupported Stage-A WAV sample rate {rate}")
            }
            Self::UnsupportedSampleWidth(bits) => {
                write!(formatter, "unsupported Stage-A WAV sample width {bits}")
            }
            Self::InvalidBlockAlign { actual, expected } => write!(
                formatter,
                "invalid Stage-A WAV block align {actual}; expected {expected}"
            ),
            Self::InvalidByteRate { actual, expected } => write!(
                formatter,
                "invalid Stage-A WAV byte rate {actual}; expected {expected}"
            ),
            Self::RegistryFormatMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "Stage-A WAV {field} mismatch: registry {expected}, header {actual}"
            ),
            Self::EmptyPcmData => write!(formatter, "Stage-A WAV PCM data is empty"),
            Self::MisalignedPcmData {
                byte_count,
                block_align,
            } => write!(
                formatter,
                "Stage-A WAV PCM byte count {byte_count} is not aligned to {block_align}"
            ),
            Self::PcmDurationExceedsRegistry {
                frame_count,
                maximum_frame_count,
            } => write!(
                formatter,
                "Stage-A WAV has {frame_count} frames; registry allows at most {maximum_frame_count}"
            ),
        }
    }
}

impl Error for StageAQualificationPcmError {}

#[derive(Clone, Copy)]
struct FrozenStageADevelopmentIdentity {
    access_record_index: usize,
    case_id: &'static str,
    logical_source_path: &'static str,
    expected_raw_wav_sha256: &'static str,
    format: StageARegistryPcmFormat,
}

fn frozen_stage_a_development_identity(case_id: &str) -> Option<FrozenStageADevelopmentIdentity> {
    let legacy_pcm16 = StageARegistryPcmFormat {
        sample_rate_hz: 48_000,
        channels: 2,
        sample_width_bits: 16,
        maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
    };
    match case_id {
        "oga_cinameng_can_be_so_beautiful" => Some(FrozenStageADevelopmentIdentity {
            access_record_index: 0,
            case_id: "oga_cinameng_can_be_so_beautiful",
            logical_source_path: concat!(
                "data/test_audio/external/RIOTBOX-1423/wav/",
                "dense_oga_cinameng_can_be_so_beautiful.wav"
            ),
            expected_raw_wav_sha256: "bf5fa8c5bc15e39d79cb51a08a54ccc4d663ab4996149b29153bd0e1febebd6f",
            format: legacy_pcm16,
        }),
        "oga_marwan_cinematic_percussion" => Some(FrozenStageADevelopmentIdentity {
            access_record_index: 1,
            case_id: "oga_marwan_cinematic_percussion",
            logical_source_path: concat!(
                "data/test_audio/external/RIOTBOX-1423/wav/",
                "sparse_oga_marwan_cinematic_percussion.wav"
            ),
            expected_raw_wav_sha256: "9373f577cf09135e2b7e3ce0e946ce5af6ea333f5a7462ab9126f6802f9986f3",
            format: legacy_pcm16,
        }),
        "oga_william_hector_horde_war_drums" => Some(FrozenStageADevelopmentIdentity {
            access_record_index: 2,
            case_id: "oga_william_hector_horde_war_drums",
            logical_source_path: concat!(
                "data/test_audio/external/RIOTBOX-1423/wav/",
                "sparse_oga_william_hector_horde_war_drums.wav"
            ),
            expected_raw_wav_sha256: "a4d95514029dd928e5637c3b9edd659b8eaf14fa78d8afb2ab7ec4da064e4417",
            format: StageARegistryPcmFormat {
                sample_rate_hz: 44_100,
                channels: 2,
                sample_width_bits: 24,
                maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
            },
        }),
        "oga_frosty_ham_osdrums" => Some(FrozenStageADevelopmentIdentity {
            access_record_index: 3,
            case_id: "oga_frosty_ham_osdrums",
            logical_source_path: concat!(
                "data/test_audio/external/RIOTBOX-1423/wav/",
                "sparse_oga_frosty_ham_osdrums.wav"
            ),
            expected_raw_wav_sha256: "7e412dd16e701d1f2b3a8c0d66fbb24ec0164691e6761a93eca8b4bb60d32bb2",
            format: StageARegistryPcmFormat {
                sample_rate_hz: 44_100,
                channels: 2,
                sample_width_bits: 16,
                maximum_duration_seconds: STAGE_A_MAXIMUM_DURATION_SECONDS,
            },
        }),
        _ => None,
    }
}

/// Validate one callback receipt from the hardened access gate and mint the
/// opaque capability consumed by [`bind_stage_a_registry_pcm_wav`].
///
/// This is intentionally crate-private: raw strings are not authorization, and
/// no public caller may construct a binding capability from provenance claims.
#[allow(dead_code)]
pub(crate) fn authorize_stage_a_development_access(
    receipt: StageASafeAccessGateReceipt<'_>,
) -> Result<StageAAuthorizedSourceAccess, StageAQualificationPcmError> {
    require_access_provenance(
        receipt.registry_path == STAGE_A_SOURCE_REGISTRY_PATH,
        "registry_path",
    )?;
    require_access_provenance(
        receipt.registry_schema == STAGE_A_SOURCE_REGISTRY_SCHEMA,
        "registry_schema",
    )?;
    require_access_provenance(
        receipt.registry_sha256 == STAGE_A_SOURCE_REGISTRY_SHA256,
        "registry_sha256",
    )?;
    require_access_provenance(receipt.partition == DEVELOPMENT_PARTITION, "partition")?;
    require_access_provenance(
        receipt.session_kind == STAGE_A_QUALIFICATION_SESSION_KIND,
        "session_kind",
    )?;
    if !is_safe_session_id(receipt.session_id) {
        return Err(StageAQualificationPcmError::InvalidQualificationSessionId);
    }
    require_access_provenance(
        receipt.access_log_schema == STAGE_A_DEVELOPMENT_ACCESS_LOG_SCHEMA,
        "access_log_schema",
    )?;
    require_access_provenance(!receipt.access_log_path.is_empty(), "access_log_path")?;
    require_access_provenance(
        receipt.access_record_status == VERIFIED_AND_DELIVERED_ACCESS_STATUS,
        "access_record_status",
    )?;

    let frozen = frozen_stage_a_development_identity(receipt.case_id)
        .ok_or(StageAQualificationPcmError::UnauthorizedDevelopmentSourceIdentity)?;
    if receipt.access_record_index != frozen.access_record_index
        || receipt.case_id != frozen.case_id
        || receipt.logical_source_path != frozen.logical_source_path
        || receipt.expected_raw_wav_sha256 != frozen.expected_raw_wav_sha256
        || receipt.accessed_raw_wav_sha256 != frozen.expected_raw_wav_sha256
    {
        return Err(StageAQualificationPcmError::UnauthorizedDevelopmentSourceIdentity);
    }

    let expectation = StageARegistryPcmExpectation {
        case_id: frozen.case_id.to_owned(),
        logical_source_path: frozen.logical_source_path.to_owned(),
        expected_raw_wav_sha256: frozen.expected_raw_wav_sha256.to_owned(),
        format: frozen.format,
    };
    validate_registry_expectation(&expectation)?;
    Ok(StageAAuthorizedSourceAccess {
        expectation,
        provenance: StageAQualificationPcmProvenance {
            registry: StageARegistryBindingProvenance {
                path: receipt.registry_path.to_owned(),
                schema: receipt.registry_schema.to_owned(),
                sha256: receipt.registry_sha256.to_owned(),
            },
            session: StageAQualificationSessionProvenance {
                kind: receipt.session_kind.to_owned(),
                session_id: receipt.session_id.to_owned(),
            },
            access: StageADevelopmentAccessProvenance {
                partition: receipt.partition.to_owned(),
                access_log_schema: receipt.access_log_schema.to_owned(),
                access_log_path: receipt.access_log_path.to_owned(),
                access_record_index: receipt.access_record_index,
                access_record_status: receipt.access_record_status.to_owned(),
            },
        },
        _seal: StageAAuthorizationSeal,
    })
}

pub(crate) fn bind_stage_a_registry_pcm_wav(
    authorization: StageAAuthorizedSourceAccess,
    raw_wav_bytes: &[u8],
) -> Result<StageABoundPcm, StageAQualificationPcmError> {
    let StageAAuthorizedSourceAccess {
        expectation,
        provenance,
        _seal: _,
    } = authorization;
    validate_registry_expectation(&expectation)?;
    let raw_wav_sha256 = sha256_hex(raw_wav_bytes);
    if raw_wav_sha256 != expectation.expected_raw_wav_sha256 {
        return Err(StageAQualificationPcmError::RawWavSha256Mismatch {
            expected: expectation.expected_raw_wav_sha256.clone(),
            actual: raw_wav_sha256,
        });
    }

    let parsed = parse_strict_pcm_wave(raw_wav_bytes)?;
    require_registry_match(expectation.format, parsed.format)?;
    let frame_count = parsed.data.len() / usize::from(parsed.format.block_align);
    let maximum_frame_count = u64::from(expectation.format.sample_rate_hz)
        .checked_mul(u64::from(expectation.format.maximum_duration_seconds))
        .and_then(|frames| frames.checked_add(1))
        .ok_or(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "maximum_duration_seconds",
            value: u64::from(expectation.format.maximum_duration_seconds),
        })?;
    if u64::try_from(frame_count).map_or(true, |frames| frames > maximum_frame_count) {
        return Err(StageAQualificationPcmError::PcmDurationExceedsRegistry {
            frame_count,
            maximum_frame_count,
        });
    }
    let interleaved_samples = decode_signed_pcm(parsed.data, parsed.format.bits_per_sample);
    let encoding = match parsed.format.bits_per_sample {
        16 => StageAPcmEncoding::SignedPcm16,
        24 => StageAPcmEncoding::SignedPcm24,
        _ => unreachable!("strict format validation admits only signed PCM16/24"),
    };
    let pcm_f32le_sha256 = pcm_f32le_sha256(
        parsed.format.sample_rate_hz,
        parsed.format.channels,
        frame_count,
        &interleaved_samples,
    );

    Ok(StageABoundPcm {
        case_id: expectation.case_id.to_owned(),
        logical_source_path: expectation.logical_source_path.to_owned(),
        raw_wav_sha256,
        pcm_f32le_sha256,
        format: StageAPcmFormatProvenance {
            format_tag: parsed.format.format_tag,
            sample_rate_hz: parsed.format.sample_rate_hz,
            channels: parsed.format.channels,
            byte_rate: parsed.format.byte_rate,
            block_align: parsed.format.block_align,
            container_bits: parsed.format.bits_per_sample,
            valid_bits: parsed.format.bits_per_sample,
            encoding,
            input_lsb: encoding.input_lsb(),
            maximum_duration_seconds: expectation.format.maximum_duration_seconds,
        },
        qualification_provenance: provenance,
        frame_count,
        interleaved_samples,
        _seal: StageABoundPcmSeal,
    })
}

fn validate_registry_expectation(
    expectation: &StageARegistryPcmExpectation,
) -> Result<(), StageAQualificationPcmError> {
    if expectation.case_id.is_empty() {
        return Err(StageAQualificationPcmError::EmptyRegistryField { field: "case_id" });
    }
    if expectation.logical_source_path.is_empty() {
        return Err(StageAQualificationPcmError::EmptyRegistryField {
            field: "logical_source_path",
        });
    }
    let path = Path::new(&expectation.logical_source_path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(StageAQualificationPcmError::UnsafeLogicalSourcePath);
    }
    if !is_lowercase_sha256(&expectation.expected_raw_wav_sha256) {
        return Err(StageAQualificationPcmError::InvalidExpectedRawSha256);
    }
    validate_registry_format(expectation.format)
}

fn validate_registry_format(
    format: StageARegistryPcmFormat,
) -> Result<(), StageAQualificationPcmError> {
    if !(MINIMUM_SAMPLE_RATE_HZ..=MAXIMUM_SAMPLE_RATE_HZ).contains(&format.sample_rate_hz) {
        return Err(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "sample_rate_hz",
            value: u64::from(format.sample_rate_hz),
        });
    }
    if !matches!(format.channels, 1 | 2) {
        return Err(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "channels",
            value: u64::from(format.channels),
        });
    }
    if !matches!(format.sample_width_bits, 16 | 24) {
        return Err(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "sample_width_bits",
            value: u64::from(format.sample_width_bits),
        });
    }
    if format.maximum_duration_seconds != STAGE_A_MAXIMUM_DURATION_SECONDS {
        return Err(StageAQualificationPcmError::RegistryFormatOutOfContract {
            field: "maximum_duration_seconds",
            value: u64::from(format.maximum_duration_seconds),
        });
    }
    Ok(())
}

fn require_access_provenance(
    condition: bool,
    field: &'static str,
) -> Result<(), StageAQualificationPcmError> {
    if condition {
        Ok(())
    } else {
        Err(StageAQualificationPcmError::AccessProvenanceMismatch { field })
    }
}

fn is_safe_session_id(session_id: &str) -> bool {
    !session_id.is_empty()
        && session_id.len() <= 128
        && session_id.trim() == session_id
        && session_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.:".contains(&byte))
}

/// Render F3-v2 from an authorized bound source without exposing a second PCM
/// encoding or LSB input. The existing diagnostic renderer remains unchanged.
pub(crate) fn render_f3_from_stage_a_bound_pcm_v2(
    bound_pcm: &StageABoundPcm,
    region: FrozenEventRegion,
) -> Result<F3DynamicRenderSet, PercussiveForceError> {
    let pcm_encoding = match bound_pcm.format.encoding {
        StageAPcmEncoding::SignedPcm16 => F3PcmEncoding::SignedPcm16,
        StageAPcmEncoding::SignedPcm24 => F3PcmEncoding::SignedPcm24,
    };
    render_f3_causal_envelope_contrast_dynamic_residual_v2(
        FrozenEventInput {
            interleaved_samples: &bound_pcm.interleaved_samples,
            sample_rate_hz: bound_pcm.format.sample_rate_hz,
            channel_count: usize::from(bound_pcm.format.channels),
            region,
        },
        pcm_encoding,
    )
}

#[cfg(test)]
fn authorize_synthetic_source_for_test(
    expectation: StageARegistryPcmExpectation,
) -> Result<StageAAuthorizedSourceAccess, StageAQualificationPcmError> {
    validate_registry_expectation(&expectation)?;
    Ok(StageAAuthorizedSourceAccess {
        provenance: StageAQualificationPcmProvenance {
            registry: StageARegistryBindingProvenance {
                path: STAGE_A_SOURCE_REGISTRY_PATH.to_owned(),
                schema: STAGE_A_SOURCE_REGISTRY_SCHEMA.to_owned(),
                sha256: STAGE_A_SOURCE_REGISTRY_SHA256.to_owned(),
            },
            session: StageAQualificationSessionProvenance {
                kind: STAGE_A_QUALIFICATION_SESSION_KIND.to_owned(),
                session_id: "synthetic-source-blind-test-session".to_owned(),
            },
            access: StageADevelopmentAccessProvenance {
                partition: DEVELOPMENT_PARTITION.to_owned(),
                access_log_schema: STAGE_A_DEVELOPMENT_ACCESS_LOG_SCHEMA.to_owned(),
                access_log_path: "/tmp/synthetic-source-blind-access.json".to_owned(),
                access_record_index: 0,
                access_record_status: VERIFIED_AND_DELIVERED_ACCESS_STATUS.to_owned(),
            },
        },
        expectation,
        _seal: StageAAuthorizationSeal,
    })
}

#[derive(Clone, Copy)]
struct ParsedPcmFormat {
    format_tag: u16,
    channels: u16,
    sample_rate_hz: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

struct ParsedPcmWave<'a> {
    format: ParsedPcmFormat,
    data: &'a [u8],
}

fn parse_strict_pcm_wave(bytes: &[u8]) -> Result<ParsedPcmWave<'_>, StageAQualificationPcmError> {
    if bytes.len() < 12 {
        return Err(StageAQualificationPcmError::InvalidRiffWave(
            "header shorter than RIFF/WAVE",
        ));
    }
    if &bytes[0..4] != b"RIFF" {
        return Err(StageAQualificationPcmError::InvalidRiffWave(
            "container is not little-endian RIFF",
        ));
    }
    if &bytes[8..12] != b"WAVE" {
        return Err(StageAQualificationPcmError::InvalidRiffWave(
            "missing WAVE form type",
        ));
    }
    let declared_riff_size = usize::try_from(read_u32_le(bytes, 4)?).map_err(|_| {
        StageAQualificationPcmError::InvalidRiffWave("RIFF size does not fit usize")
    })?;
    if declared_riff_size.checked_add(8) != Some(bytes.len()) {
        return Err(StageAQualificationPcmError::InvalidRiffWave(
            "declared RIFF size does not equal byte length",
        ));
    }

    let mut cursor = 12_usize;
    let mut format = None;
    let mut data = None;
    while cursor < bytes.len() {
        let chunk_header_end =
            cursor
                .checked_add(8)
                .ok_or(StageAQualificationPcmError::InvalidRiffWave(
                    "chunk header offset overflow",
                ))?;
        if chunk_header_end > bytes.len() {
            return Err(StageAQualificationPcmError::InvalidRiffWave(
                "truncated chunk header",
            ));
        }
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_size = usize::try_from(read_u32_le(bytes, cursor + 4)?).map_err(|_| {
            StageAQualificationPcmError::InvalidRiffWave("chunk size does not fit usize")
        })?;
        let chunk_start = chunk_header_end;
        let chunk_end = chunk_start.checked_add(chunk_size).ok_or(
            StageAQualificationPcmError::InvalidRiffWave("chunk length overflow"),
        )?;
        let padded_end = chunk_end.checked_add(chunk_size & 1).ok_or(
            StageAQualificationPcmError::InvalidRiffWave("chunk padding overflow"),
        )?;
        if padded_end > bytes.len() {
            return Err(StageAQualificationPcmError::InvalidRiffWave(
                "chunk extends past RIFF boundary",
            ));
        }
        let payload = &bytes[chunk_start..chunk_end];
        match chunk_id {
            b"fmt " => {
                if format.is_some() {
                    return Err(StageAQualificationPcmError::DuplicateChunk("fmt"));
                }
                format = Some(parse_pcm_format(payload)?);
            }
            b"data" => {
                if format.is_none() {
                    return Err(StageAQualificationPcmError::InvalidRiffWave(
                        "data chunk precedes fmt chunk",
                    ));
                }
                if data.is_some() {
                    return Err(StageAQualificationPcmError::DuplicateChunk("data"));
                }
                data = Some(payload);
            }
            _ => {}
        }
        cursor = padded_end;
    }

    let format = format.ok_or(StageAQualificationPcmError::MissingChunk("fmt"))?;
    let data = data.ok_or(StageAQualificationPcmError::MissingChunk("data"))?;
    if data.is_empty() {
        return Err(StageAQualificationPcmError::EmptyPcmData);
    }
    if !data.len().is_multiple_of(usize::from(format.block_align)) {
        return Err(StageAQualificationPcmError::MisalignedPcmData {
            byte_count: data.len(),
            block_align: format.block_align,
        });
    }
    Ok(ParsedPcmWave { format, data })
}

fn parse_pcm_format(bytes: &[u8]) -> Result<ParsedPcmFormat, StageAQualificationPcmError> {
    if bytes.len() != 16 {
        return Err(StageAQualificationPcmError::InvalidRiffWave(
            "fmt chunk must be exactly 16 bytes",
        ));
    }
    let format = ParsedPcmFormat {
        format_tag: read_u16_le(bytes, 0)?,
        channels: read_u16_le(bytes, 2)?,
        sample_rate_hz: read_u32_le(bytes, 4)?,
        byte_rate: read_u32_le(bytes, 8)?,
        block_align: read_u16_le(bytes, 12)?,
        bits_per_sample: read_u16_le(bytes, 14)?,
    };
    if format.format_tag != PCM_FORMAT_TAG {
        return Err(StageAQualificationPcmError::UnsupportedFormatTag(
            format.format_tag,
        ));
    }
    if !matches!(format.channels, 1 | 2) {
        return Err(StageAQualificationPcmError::UnsupportedChannels(
            format.channels,
        ));
    }
    if !(MINIMUM_SAMPLE_RATE_HZ..=MAXIMUM_SAMPLE_RATE_HZ).contains(&format.sample_rate_hz) {
        return Err(StageAQualificationPcmError::UnsupportedSampleRate(
            format.sample_rate_hz,
        ));
    }
    if !matches!(format.bits_per_sample, 16 | 24) {
        return Err(StageAQualificationPcmError::UnsupportedSampleWidth(
            format.bits_per_sample,
        ));
    }

    let bytes_per_sample = format.bits_per_sample / 8;
    let expected_block_align = format.channels.checked_mul(bytes_per_sample).ok_or(
        StageAQualificationPcmError::InvalidRiffWave("block-align overflow"),
    )?;
    if format.block_align != expected_block_align {
        return Err(StageAQualificationPcmError::InvalidBlockAlign {
            actual: format.block_align,
            expected: expected_block_align,
        });
    }
    let expected_byte_rate = format
        .sample_rate_hz
        .checked_mul(u32::from(expected_block_align))
        .ok_or(StageAQualificationPcmError::InvalidRiffWave(
            "byte-rate overflow",
        ))?;
    if format.byte_rate != expected_byte_rate {
        return Err(StageAQualificationPcmError::InvalidByteRate {
            actual: format.byte_rate,
            expected: expected_byte_rate,
        });
    }
    Ok(format)
}

fn require_registry_match(
    expected: StageARegistryPcmFormat,
    actual: ParsedPcmFormat,
) -> Result<(), StageAQualificationPcmError> {
    for (field, expected, actual) in [
        (
            "sample_rate_hz",
            u64::from(expected.sample_rate_hz),
            u64::from(actual.sample_rate_hz),
        ),
        (
            "channels",
            u64::from(expected.channels),
            u64::from(actual.channels),
        ),
        (
            "sample_width_bits",
            u64::from(expected.sample_width_bits),
            u64::from(actual.bits_per_sample),
        ),
    ] {
        if expected != actual {
            return Err(StageAQualificationPcmError::RegistryFormatMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn decode_signed_pcm(bytes: &[u8], bits_per_sample: u16) -> Vec<f32> {
    match bits_per_sample {
        16 => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|sample| f32::from(i16::from_le_bytes([sample[0], sample[1]])) / 32_768.0)
            .collect(),
        24 => bytes
            .as_chunks::<3>()
            .0
            .iter()
            .map(|sample| {
                let unsigned = i32::from(sample[0])
                    | (i32::from(sample[1]) << 8)
                    | (i32::from(sample[2]) << 16);
                let signed = if unsigned & 0x80_0000 == 0 {
                    unsigned
                } else {
                    unsigned | !0xFF_FFFF
                };
                signed as f32 / 8_388_608.0
            })
            .collect(),
        _ => unreachable!("strict format validation admits only signed PCM16/24"),
    }
}

fn pcm_f32le_sha256(
    sample_rate_hz: u32,
    channels: u16,
    frame_count: usize,
    samples: &[f32],
) -> String {
    let mut digest = Sha256::new();
    digest.update((STAGE_A_PCM_F32LE_HASH_DOMAIN.len() as u32).to_le_bytes());
    digest.update(STAGE_A_PCM_F32LE_HASH_DOMAIN.as_bytes());
    digest.update(sample_rate_hz.to_le_bytes());
    digest.update(u32::from(channels).to_le_bytes());
    digest.update((frame_count as u64).to_le_bytes());
    for sample in samples {
        digest.update(sample.to_bits().to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, StageAQualificationPcmError> {
    let end = offset
        .checked_add(2)
        .ok_or(StageAQualificationPcmError::InvalidRiffWave(
            "chunk offset overflow",
        ))?;
    let value = bytes
        .get(offset..end)
        .ok_or(StageAQualificationPcmError::InvalidRiffWave(
            "unexpected end of chunk",
        ))?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, StageAQualificationPcmError> {
    let end = offset
        .checked_add(4)
        .ok_or(StageAQualificationPcmError::InvalidRiffWave(
            "chunk offset overflow",
        ))?;
    let value = bytes
        .get(offset..end)
        .ok_or(StageAQualificationPcmError::InvalidRiffWave(
            "unexpected end of chunk",
        ))?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

#[cfg(test)]
mod tests;
