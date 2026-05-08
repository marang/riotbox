#[cfg(test)]
mod manifest_assertions {
    use std::path::{Path, PathBuf};

    use super::*;

    pub(super) fn assert_manifest_smoke_gate(manifest: &serde_json::Value, output_dir: &Path) {
        assert_eq!(
            manifest["schema_version"],
            LISTENING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["result"], "pass");
        assert_eq!(manifest["bars"], 2);
        assert_eq!(manifest["grid_bpm_source"], "user_override");
        assert_eq!(manifest["grid_bpm_decision_reason"], "user_override");
        assert!(manifest["source_timing_bpm_delta"].is_number());
        assert_eq!(manifest["feral_scorecard"]["readiness"], "ready");
        assert_eq!(
            manifest["feral_scorecard"]["break_rebuild_potential"],
            "high"
        );
        assert_eq!(manifest["feral_scorecard"]["source_backed"], true);
        assert_eq!(manifest["feral_scorecard"]["fallback_like"], false);
        assert_eq!(
            manifest["feral_scorecard"]["top_reason"],
            "grid-locked generated feral QA pack"
        );
        assert_manifest_source_timing(&manifest["source_timing"]);
        assert_eq!(
            manifest["feral_scorecard"]["lane_gestures"]
                .as_array()
                .expect("lane gestures")
                .len(),
            2
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_signal_rms"],
            MIN_SIGNAL_RMS,
            "min_signal_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["min_low_band_rms"],
            MIN_LOW_BAND_RMS,
            "min_low_band_rms",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["max_source_first_generated_to_source_rms_ratio"],
            MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO,
            "max_source_first_generated_to_source_rms_ratio",
        );
        assert_manifest_f32(
            &manifest["thresholds"]["max_support_generated_to_source_rms_ratio"],
            MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO,
            "max_support_generated_to_source_rms_ratio",
        );

        let artifacts = manifest["artifacts"].as_array().expect("artifacts");
        assert_eq!(artifacts.len(), 6);
        assert_manifest_artifact(
            artifacts,
            "tr909_beat_fill",
            "audio_wav",
            output_dir.join("stems/01_tr909_beat_fill.wav"),
            Some(output_dir.join("stems/01_tr909_beat_fill.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "w30_feral_source_chop",
            "audio_wav",
            output_dir.join("stems/02_w30_feral_source_chop.wav"),
            Some(output_dir.join("stems/02_w30_feral_source_chop.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "source_first_mix",
            "audio_wav",
            output_dir.join("03_riotbox_source_first_mix.wav"),
            Some(output_dir.join("03_riotbox_source_first_mix.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "full_grid_mix",
            "audio_wav",
            output_dir.join("04_riotbox_generated_support_mix.wav"),
            Some(output_dir.join("04_riotbox_generated_support_mix.metrics.md")),
        );
        assert_manifest_artifact(
            artifacts,
            "grid_report",
            "markdown_report",
            output_dir.join("grid-report.md"),
            None,
        );
        assert_manifest_artifact(
            artifacts,
            "readme",
            "markdown_readme",
            output_dir.join("README.md"),
            None,
        );

        assert!(
            manifest["metrics"]["source_first_mix"]["signal"]["rms"]
                .as_f64()
                .expect("source-first mix rms")
                > f64::from(MIN_SIGNAL_RMS)
        );
        assert!(
            manifest["metrics"]["full_grid_mix"]["signal"]["rms"]
                .as_f64()
                .expect("full mix rms")
                > f64::from(MIN_SIGNAL_RMS)
        );
        assert!(
            manifest["metrics"]["full_grid_mix"]["signal"]["event_density_per_bar"]
                .as_f64()
                .expect("full mix event density")
                > 0.0
        );
        assert!(
            manifest["metrics"]["full_grid_mix"]["low_band"]["rms"]
                .as_f64()
                .expect("low-band rms")
                > f64::from(MIN_LOW_BAND_RMS)
        );
        assert!(manifest["metrics"]["mc202_question_answer_delta"].is_null());
        assert!(manifest["metrics"]["mc202_question_answer"].is_null());
        assert_eq!(
            manifest["metrics"]["tr909_source_profile"]["support_context"],
            "transport_bar"
        );
        assert!(
            manifest["metrics"]["tr909_source_profile"]["reason"]
                .as_str()
                .expect("tr909 source reason")
                .starts_with("source_")
        );
        assert!(
            manifest["metrics"]["tr909_source_profile"]["signal_rms"]
                .as_f64()
                .expect("tr909 source signal rms")
                > 0.0
        );
        assert!(
            manifest["metrics"]["w30_source_chop_profile"]["preview_rms"]
                .as_f64()
                .expect("w30 source chop preview rms")
                > 0.0
        );
        assert!(
            manifest["metrics"]["w30_source_chop_profile"]["gain"]
                .as_f64()
                .expect("w30 source chop gain")
                >= 0.85
        );
        assert!(
            manifest["metrics"]["mix_balance"]["source_first_generated_to_source_rms_ratio"]
                .as_f64()
                .expect("source-first generated/source ratio")
                < f64::from(MAX_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO)
        );
        assert!(
            manifest["metrics"]["mix_balance"]["support_generated_to_source_rms_ratio"]
                .as_f64()
                .expect("support generated/source ratio")
                < f64::from(MAX_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO)
        );
        let output_drift = &manifest["metrics"]["source_grid_output_drift"];
        let hit_ratio = output_drift["hit_ratio"].as_f64().expect("hit ratio");
        assert!(hit_ratio >= f64::from(SOURCE_GRID_OUTPUT_MIN_HIT_RATIO));
        assert!(
            manifest["metrics"]["bar_variation"]["source_first_mix"]["bar_similarity"]
                .as_f64()
                .expect("source-first bar similarity")
                <= 1.0
        );
        assert!(
            manifest["metrics"]["bar_variation"]["full_grid_mix"]["bar_similarity"]
                .as_f64()
                .expect("bar similarity")
                <= 1.0
        );
        assert!(
            manifest["metrics"]["bar_variation"]["full_grid_mix"]["identical_bar_run_length"]
                .as_u64()
                .expect("identical bar run")
                >= 1
        );
        assert_spectral_sum(&manifest["metrics"]["spectral_energy"]["source_first_mix"]);
        assert_spectral_sum(&manifest["metrics"]["spectral_energy"]["full_grid_mix"]);
    }

    fn assert_manifest_source_timing(source_timing: &serde_json::Value) {
        assert_eq!(source_timing["schema"], "riotbox.source_timing_probe_readiness.v1");
        assert_eq!(source_timing["schema_version"], 1);
        assert!(source_timing["source_id"]
            .as_str()
            .is_some_and(|value| value.ends_with("source.wav")));
        assert_eq!(
            source_timing["policy_profile"],
            SourceTimingProbeBpmCandidatePolicy::DANCE_LOOP_AUTO_READINESS_PROFILE
        );
        let readiness = source_timing["readiness"].as_str();
        assert!(readiness.is_some_and(|value| matches!(
            value,
            "unavailable" | "weak" | "needs_review" | "ready"
        )));
        assert!(source_timing["requires_manual_confirm"].is_boolean());
        assert!(source_timing["bpm_agrees_with_grid"].is_boolean());
        assert!(
            source_timing["primary_downbeat_offset_beats"].is_null()
                || source_timing["primary_downbeat_offset_beats"]
                    .as_u64()
                    .is_some_and(|value| value < 4)
        );
        assert!(source_timing["beat_status"].as_str().is_some_and(|value| {
            matches!(value, "unavailable" | "weak" | "stable" | "ambiguous")
        }));
        assert!(source_timing["downbeat_status"]
            .as_str()
            .is_some_and(|value| matches!(
                value,
                "unavailable" | "weak" | "stable" | "ambiguous"
            )));
        assert!(source_timing["confidence_result"]
            .as_str()
            .is_some_and(|value| matches!(
                value,
                "degraded" | "candidate_cautious" | "candidate_ambiguous"
            )));
        assert!(source_timing["drift_status"].as_str().is_some_and(|value| {
            matches!(
                value,
                "unavailable" | "not_enough_material" | "stable" | "high"
            )
        }));
        assert!(source_timing["phrase_status"].as_str().is_some_and(|value| {
            matches!(
                value,
                "unavailable"
                    | "not_enough_material"
                    | "ambiguous_downbeat"
                    | "high_drift"
                    | "stable"
            )
        }));
        assert!(source_timing["alternate_evidence_count"].is_u64());
        assert!(source_timing["warning_codes"].is_array());
    }

    fn assert_spectral_sum(spectral: &serde_json::Value) {
        let spectral_sum = spectral["low_band_energy_ratio"]
            .as_f64()
            .expect("low energy")
            + spectral["mid_band_energy_ratio"].as_f64().expect("mid energy")
            + spectral["high_band_energy_ratio"]
                .as_f64()
                .expect("high energy");
        assert!((spectral_sum - 1.0).abs() < 0.000_001);
    }

    fn assert_manifest_artifact(
        artifacts: &[serde_json::Value],
        role: &str,
        kind: &str,
        path: PathBuf,
        metrics_path: Option<PathBuf>,
    ) {
        let artifact = artifacts
            .iter()
            .find(|artifact| artifact["role"] == role)
            .unwrap_or_else(|| panic!("missing artifact role {role}"));

        assert_eq!(artifact["kind"], kind);
        assert_eq!(artifact["path"], path.display().to_string());
        assert!(path.is_file(), "manifest artifact should exist: {path:?}");

        match metrics_path {
            Some(metrics_path) => {
                assert_eq!(artifact["metrics_path"], metrics_path.display().to_string());
                assert!(
                    metrics_path.is_file(),
                    "manifest metrics artifact should exist: {metrics_path:?}"
                );
            }
            None => assert!(artifact["metrics_path"].is_null()),
        }
    }

    fn assert_manifest_f32(value: &serde_json::Value, expected: f32, name: &str) {
        let actual = value.as_f64().unwrap_or_else(|| panic!("{name} missing"));
        assert!(
            (actual - f64::from(expected)).abs() < 0.000_001,
            "{name} expected {expected}, got {actual}"
        );
    }
}
