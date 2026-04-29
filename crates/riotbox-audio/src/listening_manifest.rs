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
}
