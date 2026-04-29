use std::{fs, path::Path};

use serde::Serialize;

use crate::runtime::OfflineAudioMetrics;

/// Current local audio QA manifest schema version.
///
/// Version 1 freezes the stable top-level contract shared by all generated
/// Riotbox audio QA manifests: `schema_version`, `pack_id`, `artifacts`, and
/// `result`. Pack-specific fields such as thresholds, metrics, source windows,
/// grid settings, and case details may vary while the local pack conventions are
/// still evolving.
pub const LISTENING_MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ListeningPackArtifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_id: Option<&'static str>,
    pub role: &'static str,
    pub kind: &'static str,
    pub path: String,
    pub metrics_path: Option<String>,
}

impl ListeningPackArtifact {
    pub fn audio_wav(role: &'static str, path: &Path, metrics_path: Option<&Path>) -> Self {
        Self::new(None, role, "audio_wav", path, metrics_path)
    }

    pub fn case_audio_wav(
        case_id: &'static str,
        role: &'static str,
        path: &Path,
        metrics_path: Option<&Path>,
    ) -> Self {
        Self::new(Some(case_id), role, "audio_wav", path, metrics_path)
    }

    pub fn markdown_report(role: &'static str, path: &Path) -> Self {
        Self::new(None, role, "markdown_report", path, None)
    }

    pub fn markdown_readme(role: &'static str, path: &Path) -> Self {
        Self::new(None, role, "markdown_readme", path, None)
    }

    pub fn case_markdown_report(case_id: &'static str, role: &'static str, path: &Path) -> Self {
        Self::new(Some(case_id), role, "markdown_report", path, None)
    }

    fn new(
        case_id: Option<&'static str>,
        role: &'static str,
        kind: &'static str,
        path: &Path,
        metrics_path: Option<&Path>,
    ) -> Self {
        Self {
            case_id,
            role,
            kind,
            path: path.display().to_string(),
            metrics_path: metrics_path.map(|path| path.display().to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ListeningPackRenderMetrics {
    pub signal: ListeningPackSignalMetrics,
    pub low_band: ListeningPackSignalMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ListeningPackSignalMetrics {
    pub active_samples: usize,
    pub peak_abs: f32,
    pub rms: f32,
    pub sum: f32,
    pub mean_abs: f32,
    pub zero_crossings: usize,
    pub crest_factor: f32,
    pub active_sample_ratio: f32,
    pub silence_ratio: f32,
    pub dc_offset: f32,
    pub onset_count: usize,
    pub event_density_per_bar: f32,
}

impl From<OfflineAudioMetrics> for ListeningPackSignalMetrics {
    fn from(metrics: OfflineAudioMetrics) -> Self {
        Self {
            active_samples: metrics.active_samples,
            peak_abs: metrics.peak_abs,
            rms: metrics.rms,
            sum: metrics.sum,
            mean_abs: metrics.mean_abs,
            zero_crossings: metrics.zero_crossings,
            crest_factor: metrics.crest_factor,
            active_sample_ratio: metrics.active_sample_ratio,
            silence_ratio: metrics.silence_ratio,
            dc_offset: metrics.dc_offset,
            onset_count: metrics.onset_count,
            event_density_per_bar: metrics.event_density_per_bar,
        }
    }
}

pub fn write_manifest_json(
    path: &Path,
    manifest: &impl Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_string_pretty(manifest)? + "\n")?;
    Ok(())
}

pub fn validate_manifest_envelope(manifest: &serde_json::Value) -> Result<(), String> {
    if manifest["schema_version"].as_u64() != Some(u64::from(LISTENING_MANIFEST_SCHEMA_VERSION)) {
        return Err("manifest schema_version must match current listening manifest schema".into());
    }

    require_non_empty_string(manifest, "pack_id")?;

    match manifest["result"].as_str() {
        Some("pass" | "fail") => {}
        _ => return Err("manifest result must be pass or fail".into()),
    }

    let artifacts = manifest["artifacts"]
        .as_array()
        .ok_or_else(|| "manifest artifacts must be an array".to_string())?;
    if artifacts.is_empty() {
        return Err("manifest artifacts must not be empty".into());
    }

    for (index, artifact) in artifacts.iter().enumerate() {
        require_artifact_string(artifact, index, "role")?;
        require_artifact_string(artifact, index, "kind")?;
        require_artifact_string(artifact, index, "path")?;

        if !artifact["metrics_path"].is_null() && artifact["metrics_path"].as_str().is_none() {
            return Err(format!(
                "manifest artifact {index} metrics_path must be null or a string"
            ));
        }

        if !artifact["case_id"].is_null() && artifact["case_id"].as_str().is_none() {
            return Err(format!(
                "manifest artifact {index} case_id must be null or a string"
            ));
        }
    }

    Ok(())
}

fn require_non_empty_string(manifest: &serde_json::Value, field: &str) -> Result<(), String> {
    match manifest[field].as_str() {
        Some(value) if !value.trim().is_empty() => Ok(()),
        _ => Err(format!("manifest {field} must be a non-empty string")),
    }
}

fn require_artifact_string(
    artifact: &serde_json::Value,
    index: usize,
    field: &str,
) -> Result<(), String> {
    match artifact[field].as_str() {
        Some(value) if !value.trim().is_empty() => Ok(()),
        _ => Err(format!(
            "manifest artifact {index} {field} must be a non-empty string"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_schema_version_stays_at_v1_until_the_contract_changes() {
        assert_eq!(LISTENING_MANIFEST_SCHEMA_VERSION, 1);
    }

    #[test]
    fn artifact_without_case_id_omits_case_id_field() {
        let artifact = ListeningPackArtifact::audio_wav(
            "full_mix",
            Path::new("out/full_mix.wav"),
            Some(Path::new("out/full_mix.metrics.md")),
        );

        let json = serde_json::to_value(artifact).expect("json");

        assert!(json.get("case_id").is_none());
        assert_eq!(json["role"], "full_mix");
        assert_eq!(json["kind"], "audio_wav");
        assert_eq!(json["path"], "out/full_mix.wav");
        assert_eq!(json["metrics_path"], "out/full_mix.metrics.md");
    }

    #[test]
    fn artifact_with_case_id_keeps_case_id_field() {
        let artifact = ListeningPackArtifact::case_markdown_report(
            "mc202-follower-to-answer",
            "comparison",
            Path::new("out/comparison.md"),
        );

        let json = serde_json::to_value(artifact).expect("json");

        assert_eq!(json["case_id"], "mc202-follower-to-answer");
        assert_eq!(json["role"], "comparison");
        assert_eq!(json["kind"], "markdown_report");
        assert_eq!(json["metrics_path"], serde_json::Value::Null);
    }

    #[test]
    fn manifest_envelope_accepts_current_producer_shapes() {
        for manifest in [
            minimal_manifest("w30-preview-smoke", "baseline", None),
            minimal_manifest(
                "lane-recipe-listening-pack",
                "candidate",
                Some("mc202-answer"),
            ),
            minimal_manifest("feral-before-after", "riotbox_after", None),
            minimal_manifest("feral-grid-demo", "full_grid_mix", None),
        ] {
            validate_manifest_envelope(&manifest).expect("valid manifest envelope");
        }
    }

    #[test]
    fn manifest_envelope_rejects_missing_stable_fields() {
        let mut manifest = minimal_manifest("feral-grid-demo", "full_grid_mix", None);
        manifest["pack_id"] = serde_json::Value::Null;

        let error = validate_manifest_envelope(&manifest).expect_err("missing pack id");

        assert!(error.contains("pack_id"));
    }

    #[test]
    fn manifest_envelope_rejects_invalid_artifact_records() {
        let mut manifest = minimal_manifest("feral-grid-demo", "full_grid_mix", None);
        manifest["artifacts"][0]["metrics_path"] = serde_json::json!(42);

        let error = validate_manifest_envelope(&manifest).expect_err("invalid metrics path");

        assert!(error.contains("metrics_path"));
    }

    fn minimal_manifest(
        pack_id: &'static str,
        artifact_role: &'static str,
        case_id: Option<&'static str>,
    ) -> serde_json::Value {
        serde_json::json!({
            "schema_version": LISTENING_MANIFEST_SCHEMA_VERSION,
            "pack_id": pack_id,
            "result": "pass",
            "artifacts": [{
                "case_id": case_id,
                "role": artifact_role,
                "kind": "audio_wav",
                "path": format!("out/{artifact_role}.wav"),
                "metrics_path": format!("out/{artifact_role}.metrics.md")
            }],
            "metrics": {}
        })
    }
}
