//! Source-agnostic, offline Stage-A percussive-force mechanisms.
//!
//! These mechanisms are experimental render seams. They are not wired into the
//! Riotbox runtime and cannot by themselves establish a musician-facing
//! `percussive_hard` product claim.

mod common;
mod f1;
mod f2;
#[allow(dead_code, clippy::manual_range_contains)]
mod f3;
mod f3_dynamic;
mod qualification_pcm;

pub use common::{
    EffectiveRegionEnergy, EqualPowerMasks, FrozenEventInput, FrozenEventRegion, InvalidEventInput,
    PercussiveForceError, PercussiveForceRefusal,
};
pub use f1::{
    F1_AB_ENERGY_REDISTRIBUTION_V1, F1EnergyRedistributionPolicy, F1RenderSet,
    render_f1_ab_energy_redistribution_v1,
};
pub use f2::{
    F2_EXACT_COMPLEMENTARY_THREE_BAND_V1, F2BandPolicy, F2BandRole, F2ComplementaryPolicy,
    F2RenderSet, render_f2_exact_complementary_three_band_v1,
};
// The old 4x nonlinear family is deliberately not exported as a candidate
// renderer. Its only public surface is the executable, immutable rejected
// preflight record.
pub use f3::{
    ResamplerPreflight as RejectedF3Os4PreflightV1,
    run_f3_resampler_preflight as run_rejected_f3_os4_preflight_v1,
};
pub use f3_dynamic::{
    F3_CAUSAL_ENVELOPE_CONTRAST_DYNAMIC_RESIDUAL_V2, F3ControllerHashes,
    F3DynamicNearIdentityRecord, F3DynamicPolicy, F3DynamicPreflightPolicyIdentity,
    F3DynamicRenderSet, F3DynamicSourceResponseDiversityIdentity, F3DynamicSyntheticOutcome,
    F3DynamicSyntheticPreflight, F3DynamicSyntheticPreflightAtRate, F3DynamicSyntheticRunIdentity,
    F3DynamicSyntheticRunRecord, F3PcmEncoding,
    f3_source_response_identities_are_diversity_separated,
    render_f3_causal_envelope_contrast_dynamic_residual_v2, run_f3_dynamic_synthetic_preflight_v2,
};
pub use qualification_pcm::{
    STAGE_A_DEVELOPMENT_ACCESS_LOG_SCHEMA, STAGE_A_PCM_F32LE_HASH_DOMAIN,
    STAGE_A_QUALIFICATION_SESSION_KIND, STAGE_A_SOURCE_REGISTRY_PATH,
    STAGE_A_SOURCE_REGISTRY_SCHEMA, STAGE_A_SOURCE_REGISTRY_SHA256, StageAAuthorizedSourceAccess,
    StageABoundPcm, StageADevelopmentAccessProvenance, StageAPcmEncoding,
    StageAPcmFormatProvenance, StageAQualificationPcmError, StageAQualificationPcmProvenance,
    StageAQualificationSessionProvenance, StageARegistryBindingProvenance,
    bind_stage_a_registry_pcm_wav, render_f3_from_stage_a_bound_pcm_v2,
};
