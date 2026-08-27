use serde::{Deserialize, Serialize};

use crate::session::ExportArtifactRole;

pub const PRODUCT_STEM_HANDOFF_SCHEMA: &str = "riotbox.product_stem_handoff.v2";
pub const PRODUCT_STEM_HANDOFF_SCHEMA_VERSION: u32 = 2;
pub const PRODUCT_STEM_HANDOFF_BOUNDARY: &str = "feral-grid generated-support product stems";
pub const PRODUCT_STEM_HANDOFF_PACK_ID: &str = "feral-grid-demo";
pub const PRODUCT_STEM_HANDOFF_MATERIAL_STATUS: &str = "development_only";
pub const PRODUCT_STEM_RECONSTRUCTION_SCHEMA: &str = "riotbox.product_stem_reconstruction.v1";
pub const PRODUCT_STEM_RECONSTRUCTION_RULE: &str = "pcm_sum_v1";
pub const MC202_SOURCE_EXPRESSION_SCHEMA: &str = "riotbox.mc202_source_expression_origin.v1";
pub const PRODUCT_STEM_PCM_MAX_ABS_ERROR: f64 = 3.0 / 32_768.0;
pub const PRODUCT_STEM_PCM_MAX_RMS_ERROR: f64 = 1.5 / 32_768.0;
pub const PRODUCT_STEM_CONTRACT_FLOAT_TOLERANCE: f64 = 1.0e-12;
pub const PRODUCT_STEM_DECLARED_METRIC_TOLERANCE: f64 = 1.0e-7;
pub const MC202_MIN_SOURCE_GRID_HIT_RATIO: f64 = 0.5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductStemHandoff {
    pub schema: String,
    pub schema_version: u32,
    pub boundary: String,
    pub pack_id: String,
    pub material_status: String,
    pub release_ready: bool,
    pub musician_export_action_ready: bool,
    pub source_sha256: String,
    pub normalized_manifest_sha256: String,
    pub grid: ProductStemHandoffGrid,
    pub artifacts: Vec<ProductStemHandoffArtifact>,
    pub reconstruction: ProductStemReconstruction,
    pub renderer_status: ProductStemRendererStatus,
}

impl ProductStemHandoff {
    pub fn validate(&self) -> Result<(), ProductStemHandoffError> {
        if self.schema != PRODUCT_STEM_HANDOFF_SCHEMA
            || self.schema_version != PRODUCT_STEM_HANDOFF_SCHEMA_VERSION
        {
            return Err(ProductStemHandoffError::Invalid("schema"));
        }
        if self.boundary != PRODUCT_STEM_HANDOFF_BOUNDARY
            || self.pack_id != PRODUCT_STEM_HANDOFF_PACK_ID
            || self.material_status != PRODUCT_STEM_HANDOFF_MATERIAL_STATUS
        {
            return Err(ProductStemHandoffError::Invalid("boundary/status/pack"));
        }
        if self.release_ready || self.musician_export_action_ready {
            return Err(ProductStemHandoffError::Invalid("readiness flags"));
        }
        if !is_lowercase_sha256(&self.source_sha256)
            || !is_lowercase_sha256(&self.normalized_manifest_sha256)
        {
            return Err(ProductStemHandoffError::Invalid("SHA-256 identity"));
        }
        self.grid.validate()?;
        self.validate_artifacts()?;
        self.reconstruction.validate(&self.grid)?;
        self.renderer_status.validate()?;
        Ok(())
    }

    #[must_use]
    pub fn artifact(
        &self,
        role: ProductStemHandoffArtifactRole,
    ) -> Option<&ProductStemHandoffArtifact> {
        self.artifacts.iter().find(|artifact| artifact.role == role)
    }

    #[must_use]
    pub fn stem_roles(&self) -> Vec<ExportArtifactRole> {
        [
            ProductStemHandoffArtifactRole::StemDrums,
            ProductStemHandoffArtifactRole::StemMusic,
            ProductStemHandoffArtifactRole::StemBass,
        ]
        .into_iter()
        .map(ProductStemHandoffArtifactRole::export_artifact_role)
        .collect()
    }

    fn validate_artifacts(&self) -> Result<(), ProductStemHandoffError> {
        let contracts = [
            ProductStemHandoffArtifactContract::drums(),
            ProductStemHandoffArtifactContract::music(),
            ProductStemHandoffArtifactContract::bass(),
            ProductStemHandoffArtifactContract::full_mix(),
        ];
        if self.artifacts.len() != contracts.len() {
            return Err(ProductStemHandoffError::Invalid("artifact count"));
        }
        for contract in contracts {
            let matches = self
                .artifacts
                .iter()
                .filter(|artifact| artifact.role == contract.role)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(ProductStemHandoffError::Invalid("artifact roles"));
            }
            let artifact = matches[0];
            if artifact.source_role != contract.source_role
                || artifact.path != contract.path
                || artifact.media_type != ProductStemHandoffMediaType::AudioWav
                || artifact.origin != contract.origin
                || !is_lowercase_sha256(&artifact.sha256)
            {
                return Err(ProductStemHandoffError::Invalid("artifact contract"));
            }
            if !is_contained_relative_path(&artifact.path) {
                return Err(ProductStemHandoffError::Invalid("artifact path"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductStemHandoffGrid {
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub bpm: f64,
    pub beats_per_bar: u32,
    pub bars: u32,
    pub total_beats: u32,
    pub frame_count: u64,
    pub duration_seconds: f64,
}

impl ProductStemHandoffGrid {
    fn validate(&self) -> Result<(), ProductStemHandoffError> {
        if self.sample_rate_hz == 0
            || self.channel_count == 0
            || !self.bpm.is_finite()
            || self.bpm <= 0.0
            || self.beats_per_bar == 0
            || self.bars == 0
            || self.total_beats == 0
            || self.frame_count == 0
            || !self.duration_seconds.is_finite()
            || self.duration_seconds <= 0.0
        {
            return Err(ProductStemHandoffError::Invalid("grid values"));
        }
        if self.total_beats != self.beats_per_bar.saturating_mul(self.bars) {
            return Err(ProductStemHandoffError::Invalid("grid beat/bar identity"));
        }
        let expected_frames = (f64::from(self.total_beats) * f64::from(self.sample_rate_hz) * 60.0
            / self.bpm)
            .round() as u64;
        if self.frame_count.abs_diff(expected_frames) > 1 {
            return Err(ProductStemHandoffError::Invalid(
                "grid frame/tempo identity",
            ));
        }
        let expected_duration = self.frame_count as f64 / f64::from(self.sample_rate_hz);
        if (self.duration_seconds - expected_duration).abs() > 1.0 / f64::from(self.sample_rate_hz)
        {
            return Err(ProductStemHandoffError::Invalid("grid duration identity"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductStemHandoffArtifact {
    pub role: ProductStemHandoffArtifactRole,
    pub source_role: ProductStemHandoffSourceRole,
    pub path: String,
    pub media_type: ProductStemHandoffMediaType,
    pub sha256: String,
    pub origin: ProductStemHandoffOrigin,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStemHandoffArtifactRole {
    StemDrums,
    StemMusic,
    StemBass,
    FullGridMix,
}

impl ProductStemHandoffArtifactRole {
    #[must_use]
    pub const fn export_artifact_role(self) -> ExportArtifactRole {
        match self {
            Self::StemDrums => ExportArtifactRole::StemDrums,
            Self::StemMusic => ExportArtifactRole::StemMusic,
            Self::StemBass => ExportArtifactRole::StemBass,
            Self::FullGridMix => ExportArtifactRole::FullGridMix,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStemHandoffSourceRole {
    ProductStemDrums,
    ProductStemMusic,
    ProductStemBass,
    FullGridMix,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductStemHandoffMediaType {
    #[serde(rename = "audio/wav")]
    AudioWav,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStemHandoffOrigin {
    SourceDerived,
    Composite,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductStemReconstruction {
    pub schema: String,
    pub rule: String,
    pub passed: bool,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub frame_count: u64,
    pub max_abs_error: f64,
    pub rms_error: f64,
    pub max_allowed_abs_error: f64,
    pub max_allowed_rms_error: f64,
}

impl ProductStemReconstruction {
    fn validate(&self, grid: &ProductStemHandoffGrid) -> Result<(), ProductStemHandoffError> {
        if self.schema != PRODUCT_STEM_RECONSTRUCTION_SCHEMA
            || self.rule != PRODUCT_STEM_RECONSTRUCTION_RULE
            || !self.passed
            || self.sample_rate_hz != grid.sample_rate_hz
            || self.channel_count != grid.channel_count
            || self.frame_count != grid.frame_count
        {
            return Err(ProductStemHandoffError::Invalid("reconstruction identity"));
        }
        for value in [
            self.max_abs_error,
            self.rms_error,
            self.max_allowed_abs_error,
            self.max_allowed_rms_error,
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(ProductStemHandoffError::Invalid("reconstruction metrics"));
            }
        }
        if (self.max_allowed_abs_error - PRODUCT_STEM_PCM_MAX_ABS_ERROR).abs()
            > PRODUCT_STEM_CONTRACT_FLOAT_TOLERANCE
            || (self.max_allowed_rms_error - PRODUCT_STEM_PCM_MAX_RMS_ERROR).abs()
                > PRODUCT_STEM_CONTRACT_FLOAT_TOLERANCE
            || self.max_abs_error > PRODUCT_STEM_PCM_MAX_ABS_ERROR
            || self.rms_error > PRODUCT_STEM_PCM_MAX_RMS_ERROR
        {
            return Err(ProductStemHandoffError::Invalid("reconstruction tolerance"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductStemRendererStatus {
    pub mc202_source_expression: Mc202SourceExpressionOrigin,
    pub limitations: Vec<String>,
}

impl ProductStemRendererStatus {
    fn validate(&self) -> Result<(), ProductStemHandoffError> {
        if !self.limitations.is_empty() {
            return Err(ProductStemHandoffError::Invalid("renderer limitations"));
        }
        self.mc202_source_expression.validate()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mc202SourceExpressionOrigin {
    pub schema: String,
    pub pattern_origin: String,
    pub bass_pressure_applied: bool,
    pub bass_pressure_reason: String,
    pub source_expression_render_plan_applied: bool,
    pub source_expression_role: Mc202SourceExpressionRole,
    pub source_failure_fallback: bool,
    pub source_contour_origin: String,
    pub source_contour_applied: bool,
    pub source_contour_delta_rms: f64,
    pub source_contour_min_required_delta_rms: f64,
    pub source_grid_hit_ratio: f64,
    pub source_grid_min_required_hit_ratio: f64,
}

impl Mc202SourceExpressionOrigin {
    fn validate(&self) -> Result<(), ProductStemHandoffError> {
        if self.schema != MC202_SOURCE_EXPRESSION_SCHEMA
            || self.pattern_origin != "source_derived"
            || !self.bass_pressure_applied
            || self.bass_pressure_reason != "mc202_source_grid_proof_renderer"
            || !self.source_expression_render_plan_applied
            || self.source_failure_fallback
            || self.source_contour_origin != "source_derived_contour"
            || !self.source_contour_applied
        {
            return Err(ProductStemHandoffError::Invalid("MC-202 source expression"));
        }
        for value in [
            self.source_contour_delta_rms,
            self.source_contour_min_required_delta_rms,
            self.source_grid_hit_ratio,
            self.source_grid_min_required_hit_ratio,
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(ProductStemHandoffError::Invalid("MC-202 metrics"));
            }
        }
        if self.source_contour_delta_rms < self.source_contour_min_required_delta_rms
            || self.source_grid_hit_ratio > 1.0
            || self.source_grid_hit_ratio < MC202_MIN_SOURCE_GRID_HIT_RATIO
            || (self.source_grid_min_required_hit_ratio - MC202_MIN_SOURCE_GRID_HIT_RATIO).abs()
                > f64::EPSILON
        {
            return Err(ProductStemHandoffError::Invalid("MC-202 thresholds"));
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mc202SourceExpressionRole {
    BassPressure,
    AnswerLift,
    HookRestraintHold,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductStemHandoffError {
    Invalid(&'static str),
}

struct ProductStemHandoffArtifactContract {
    role: ProductStemHandoffArtifactRole,
    source_role: ProductStemHandoffSourceRole,
    path: &'static str,
    origin: ProductStemHandoffOrigin,
}

impl ProductStemHandoffArtifactContract {
    const fn drums() -> Self {
        Self {
            role: ProductStemHandoffArtifactRole::StemDrums,
            source_role: ProductStemHandoffSourceRole::ProductStemDrums,
            path: "stems/stem_drums.wav",
            origin: ProductStemHandoffOrigin::SourceDerived,
        }
    }

    const fn music() -> Self {
        Self {
            role: ProductStemHandoffArtifactRole::StemMusic,
            source_role: ProductStemHandoffSourceRole::ProductStemMusic,
            path: "stems/stem_music.wav",
            origin: ProductStemHandoffOrigin::SourceDerived,
        }
    }

    const fn bass() -> Self {
        Self {
            role: ProductStemHandoffArtifactRole::StemBass,
            source_role: ProductStemHandoffSourceRole::ProductStemBass,
            path: "stems/stem_bass.wav",
            origin: ProductStemHandoffOrigin::SourceDerived,
        }
    }

    const fn full_mix() -> Self {
        Self {
            role: ProductStemHandoffArtifactRole::FullGridMix,
            source_role: ProductStemHandoffSourceRole::FullGridMix,
            path: "full_grid_mix.wav",
            origin: ProductStemHandoffOrigin::Composite,
        }
    }
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_contained_relative_path(value: &str) -> bool {
    let path = std::path::Path::new(value);
    !path.is_absolute()
        && !value.is_empty()
        && path.components().all(|component| {
            matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_handoff() -> ProductStemHandoff {
        ProductStemHandoff {
            schema: PRODUCT_STEM_HANDOFF_SCHEMA.into(),
            schema_version: PRODUCT_STEM_HANDOFF_SCHEMA_VERSION,
            boundary: PRODUCT_STEM_HANDOFF_BOUNDARY.into(),
            pack_id: PRODUCT_STEM_HANDOFF_PACK_ID.into(),
            material_status: PRODUCT_STEM_HANDOFF_MATERIAL_STATUS.into(),
            release_ready: false,
            musician_export_action_ready: false,
            source_sha256: "a".repeat(64),
            normalized_manifest_sha256: "b".repeat(64),
            grid: ProductStemHandoffGrid {
                sample_rate_hz: 48_000,
                channel_count: 2,
                bpm: 120.0,
                beats_per_bar: 4,
                bars: 1,
                total_beats: 4,
                frame_count: 96_000,
                duration_seconds: 2.0,
            },
            artifacts: vec![
                artifact(
                    ProductStemHandoffArtifactRole::StemDrums,
                    ProductStemHandoffSourceRole::ProductStemDrums,
                    "stems/stem_drums.wav",
                    ProductStemHandoffOrigin::SourceDerived,
                    '1',
                ),
                artifact(
                    ProductStemHandoffArtifactRole::StemMusic,
                    ProductStemHandoffSourceRole::ProductStemMusic,
                    "stems/stem_music.wav",
                    ProductStemHandoffOrigin::SourceDerived,
                    '2',
                ),
                artifact(
                    ProductStemHandoffArtifactRole::StemBass,
                    ProductStemHandoffSourceRole::ProductStemBass,
                    "stems/stem_bass.wav",
                    ProductStemHandoffOrigin::SourceDerived,
                    '3',
                ),
                artifact(
                    ProductStemHandoffArtifactRole::FullGridMix,
                    ProductStemHandoffSourceRole::FullGridMix,
                    "full_grid_mix.wav",
                    ProductStemHandoffOrigin::Composite,
                    '4',
                ),
            ],
            reconstruction: ProductStemReconstruction {
                schema: PRODUCT_STEM_RECONSTRUCTION_SCHEMA.into(),
                rule: PRODUCT_STEM_RECONSTRUCTION_RULE.into(),
                passed: true,
                sample_rate_hz: 48_000,
                channel_count: 2,
                frame_count: 96_000,
                max_abs_error: 0.0,
                rms_error: 0.0,
                max_allowed_abs_error: PRODUCT_STEM_PCM_MAX_ABS_ERROR,
                max_allowed_rms_error: PRODUCT_STEM_PCM_MAX_RMS_ERROR,
            },
            renderer_status: ProductStemRendererStatus {
                mc202_source_expression: Mc202SourceExpressionOrigin {
                    schema: MC202_SOURCE_EXPRESSION_SCHEMA.into(),
                    pattern_origin: "source_derived".into(),
                    bass_pressure_applied: true,
                    bass_pressure_reason: "mc202_source_grid_proof_renderer".into(),
                    source_expression_render_plan_applied: true,
                    source_expression_role: Mc202SourceExpressionRole::BassPressure,
                    source_failure_fallback: false,
                    source_contour_origin: "source_derived_contour".into(),
                    source_contour_applied: true,
                    source_contour_delta_rms: 0.01,
                    source_contour_min_required_delta_rms: 0.00025,
                    source_grid_hit_ratio: 1.0,
                    source_grid_min_required_hit_ratio: MC202_MIN_SOURCE_GRID_HIT_RATIO,
                },
                limitations: Vec::new(),
            },
        }
    }

    fn artifact(
        role: ProductStemHandoffArtifactRole,
        source_role: ProductStemHandoffSourceRole,
        path: &str,
        origin: ProductStemHandoffOrigin,
        hash_character: char,
    ) -> ProductStemHandoffArtifact {
        ProductStemHandoffArtifact {
            role,
            source_role,
            path: path.into(),
            media_type: ProductStemHandoffMediaType::AudioWav,
            sha256: hash_character.to_string().repeat(64),
            origin,
        }
    }

    #[test]
    fn exact_v2_handoff_contract_validates() {
        let handoff = valid_handoff();
        handoff.validate().expect("valid v2 handoff");
        assert_eq!(
            handoff.stem_roles(),
            vec![
                ExportArtifactRole::StemDrums,
                ExportArtifactRole::StemMusic,
                ExportArtifactRole::StemBass,
            ]
        );
    }

    #[test]
    fn v2_handoff_accepts_frozen_tolerances_at_published_json_precision() {
        let mut json = serde_json::to_value(valid_handoff()).expect("serialize valid handoff");
        json["reconstruction"]["max_allowed_abs_error"] = serde_json::json!(0.000_091_552_734);
        json["reconstruction"]["max_allowed_rms_error"] = serde_json::json!(0.000_045_776_367);
        let handoff: ProductStemHandoff =
            serde_json::from_value(json).expect("deserialize published V2 precision");

        handoff
            .validate()
            .expect("published V2 JSON precision preserves the frozen tolerances");
    }

    #[test]
    fn v2_handoff_rejects_material_reconstruction_tolerance_drift() {
        let mut handoff = valid_handoff();
        handoff.reconstruction.max_allowed_abs_error += 1.0e-9;

        assert_eq!(
            handoff.validate(),
            Err(ProductStemHandoffError::Invalid("reconstruction tolerance"))
        );
    }

    #[test]
    fn v2_handoff_keeps_exact_metric_limits_when_declared_fields_round_up() {
        let mut handoff = valid_handoff();
        handoff.reconstruction.max_allowed_abs_error =
            PRODUCT_STEM_PCM_MAX_ABS_ERROR + PRODUCT_STEM_CONTRACT_FLOAT_TOLERANCE / 2.0;
        handoff.reconstruction.max_abs_error =
            PRODUCT_STEM_PCM_MAX_ABS_ERROR + PRODUCT_STEM_CONTRACT_FLOAT_TOLERANCE / 4.0;

        assert_eq!(
            handoff.validate(),
            Err(ProductStemHandoffError::Invalid("reconstruction tolerance"))
        );
    }

    #[test]
    fn v2_handoff_rejects_unsafe_origin_and_threshold_mutations() {
        let mut ready = valid_handoff();
        ready.musician_export_action_ready = true;
        assert!(ready.validate().is_err());

        let mut fallback = valid_handoff();
        fallback
            .renderer_status
            .mc202_source_expression
            .source_failure_fallback = true;
        assert!(fallback.validate().is_err());

        let mut traversal = valid_handoff();
        traversal.artifacts[0].path = "../stem_drums.wav".into();
        assert!(traversal.validate().is_err());

        let mut weak_grid = valid_handoff();
        weak_grid
            .renderer_status
            .mc202_source_expression
            .source_grid_hit_ratio = 0.49;
        assert!(weak_grid.validate().is_err());
    }
}
