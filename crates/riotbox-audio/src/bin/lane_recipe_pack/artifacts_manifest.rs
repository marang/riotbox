#[derive(Serialize)]
struct ListeningPackManifest {
    schema_version: u32,
    pack_id: &'static str,
    date: String,
    sample_rate: u32,
    channel_count: u16,
    duration_seconds: f32,
    case_count: usize,
    artifacts: Vec<ManifestArtifact>,
    cases: Vec<ManifestCase>,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestCase {
    id: &'static str,
    title: &'static str,
    recipe_refs: &'static str,
    baseline_label: &'static str,
    candidate_label: &'static str,
    thresholds: ManifestThresholds,
    metrics: ManifestCaseMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestThresholds {
    min_rms_delta: f32,
    min_signal_delta_rms: f32,
}

#[derive(Serialize)]
struct ManifestCaseMetrics {
    baseline: ManifestSignalMetrics,
    candidate: ManifestSignalMetrics,
    signal_delta: ManifestSignalMetrics,
    rms_delta: f32,
    mc202_phrase_grid: Option<Mc202PhraseGridTimingMetrics>,
    mc202_source_phrase_slot: Option<Mc202SourcePhraseSlotMetrics>,
}

fn write_manifest(
    path: &Path,
    args: &Args,
    output_dir: &Path,
    reports: &[CaseReport],
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = ListeningPackManifest {
        schema_version: LISTENING_MANIFEST_SCHEMA_VERSION,
        pack_id: PACK_ID,
        date: args.date.clone(),
        sample_rate: SAMPLE_RATE,
        channel_count: CHANNEL_COUNT,
        duration_seconds: args.duration_seconds,
        case_count: reports.len(),
        artifacts: manifest_artifacts(output_dir, reports),
        cases: reports.iter().map(ManifestCase::from).collect(),
        result: "pass",
    };

    write_manifest_json(path, &manifest)?;
    Ok(())
}

fn manifest_artifacts(output_dir: &Path, reports: &[CaseReport]) -> Vec<ManifestArtifact> {
    let mut artifacts = Vec::new();
    for report in reports {
        let case_dir = output_dir.join(report.id);
        let baseline_path = case_dir.join("baseline.wav");
        let baseline_metrics_path = case_dir.join("baseline.metrics.md");
        let candidate_path = case_dir.join("candidate.wav");
        let candidate_metrics_path = case_dir.join("candidate.metrics.md");
        let comparison_path = case_dir.join("comparison.md");
        artifacts.push(ManifestArtifact::case_audio_wav(
            report.id,
            "baseline",
            &baseline_path,
            Some(&baseline_metrics_path),
        ));
        artifacts.push(ManifestArtifact::case_audio_wav(
            report.id,
            "candidate",
            &candidate_path,
            Some(&candidate_metrics_path),
        ));
        artifacts.push(ManifestArtifact::case_markdown_report(
            report.id,
            "comparison",
            &comparison_path,
        ));
    }
    artifacts.push(ManifestArtifact::case_markdown_report(
        "pack",
        "summary",
        &output_dir.join("pack-summary.md"),
    ));
    artifacts
}

impl From<&CaseReport> for ManifestCase {
    fn from(report: &CaseReport) -> Self {
        Self {
            id: report.id,
            title: report.title,
            recipe_refs: report.recipe_refs,
            baseline_label: report.baseline_label,
            candidate_label: report.candidate_label,
            thresholds: ManifestThresholds {
                min_rms_delta: report.min_rms_delta,
                min_signal_delta_rms: report.min_signal_delta_rms,
            },
            metrics: ManifestCaseMetrics {
                baseline: report.baseline_metrics.into(),
                candidate: report.candidate_metrics.into(),
                signal_delta: report.signal_delta_metrics.into(),
                rms_delta: rms_delta(report.baseline_metrics, report.candidate_metrics),
                mc202_phrase_grid: report.mc202_phrase_grid,
                mc202_source_phrase_slot: report.mc202_source_phrase_slot.clone(),
            },
            result: if report.passed { "pass" } else { "fail" },
        }
    }
}

fn write_pcm16_wav(
    path: &Path,
    sample_rate: u32,
    channel_count: u16,
    samples: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let data_len = samples.len() * 2;
    let riff_len = 36 + data_len;
    let byte_rate = sample_rate * u32::from(channel_count) * 2;
    let block_align = channel_count * 2;

    let mut bytes = Vec::with_capacity(44 + data_len);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(riff_len as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_len as u32).to_le_bytes());

    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }

    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Args, BEATS_PER_BAR, CHANNEL_COUNT, DEFAULT_BPM, LISTENING_MANIFEST_SCHEMA_VERSION, PACK_ID,
        SAMPLE_RATE, MC202_SOURCE_PHRASE_SLOT_CONTRACT, pack_cases, render_pack, render_pair,
        signal_delta_metrics, signal_metrics_with_grid,
    };
    use std::{fs, path::PathBuf};

    #[test]
    fn parses_default_args() {
        let args = Args::parse(Vec::<String>::new()).expect("parse args");

        assert_eq!(args.date, "local");
        assert_eq!(args.output_dir, None);
        assert_eq!(args.duration_seconds, 2.0);
        assert!(!args.show_help);
        assert_eq!(
            args.output_dir(),
            PathBuf::from("artifacts/audio_qa/local").join(PACK_ID)
        );
    }

    #[test]
    fn parses_custom_args() {
        let args = Args::parse([
            "--date".to_string(),
            "audit".to_string(),
            "--duration-seconds".to_string(),
            "1.5".to_string(),
            "--output-dir".to_string(),
            "tmp/pack".to_string(),
        ])
        .expect("parse args");

        assert_eq!(args.date, "audit");
        assert_eq!(args.duration_seconds, 1.5);
        assert_eq!(args.output_dir(), PathBuf::from("tmp/pack"));
    }

    #[test]
    fn rejects_invalid_duration() {
        assert!(Args::parse(["--duration-seconds".to_string(), "0".to_string()]).is_err());
    }

    #[test]
    fn pack_cases_produce_distinct_audio_metrics() {
        let cases = pack_cases();

        assert_eq!(cases.len(), 10);
        for case in cases {
            let (baseline, candidate) = render_pair(&case.render_pair, 88_200);
            let baseline_metrics = signal_metrics_with_grid(
                &baseline,
                SAMPLE_RATE,
                CHANNEL_COUNT,
                DEFAULT_BPM,
                BEATS_PER_BAR,
            );
            let candidate_metrics = signal_metrics_with_grid(
                &candidate,
                SAMPLE_RATE,
                CHANNEL_COUNT,
                DEFAULT_BPM,
                BEATS_PER_BAR,
            );
            let signal_delta_metrics = signal_delta_metrics(&baseline, &candidate);

            assert!(
                baseline_metrics.active_samples > 0,
                "{} baseline silent",
                case.id
            );
            assert!(
                candidate_metrics.active_samples > 0,
                "{} candidate silent",
                case.id
            );
            assert!(
                (baseline_metrics.rms - candidate_metrics.rms).abs() >= case.min_rms_delta,
                "{} did not produce required RMS delta {}",
                case.id,
                case.min_rms_delta
            );
            assert!(
                signal_delta_metrics.rms >= case.min_signal_delta_rms,
                "{} did not produce required signal delta RMS {}",
                case.id,
                case.min_signal_delta_rms
            );
            assert!(
                baseline_metrics.onset_count > 0,
                "{} baseline has no detected onsets",
                case.id
            );
            assert!(
                candidate_metrics.event_density_per_bar > 0.0,
                "{} candidate has no event density",
                case.id
            );
        }
    }

    #[test]
    fn render_pack_writes_machine_readable_manifest() {
        let temp = tempfile::tempdir().expect("tempdir");
        let output_dir = temp.path().join("lane-pack");
        let args = Args {
            date: "manifest-smoke".into(),
            output_dir: Some(output_dir.clone()),
            duration_seconds: 2.0,
            show_help: false,
        };

        render_pack(&args).expect("render pack");

        assert!(output_dir.join("pack-summary.md").is_file());
        assert!(output_dir.join("manifest.json").is_file());

        let manifest = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
        let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("parse manifest");

        assert_eq!(
            manifest["schema_version"],
            LISTENING_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["pack_id"], PACK_ID);
        assert_eq!(manifest["date"], "manifest-smoke");
        assert_eq!(manifest["result"], "pass");
        assert_eq!(manifest["case_count"], 10);

        let cases = manifest["cases"].as_array().expect("cases");
        assert_eq!(cases.len(), 10);
        let first_case = &cases[0];
        assert_eq!(first_case["id"], "tr909-support-to-fill");
        assert_eq!(first_case["result"], "pass");
        assert!(
            first_case["metrics"]["baseline"]["rms"]
                .as_f64()
                .expect("baseline rms")
                > 0.0
        );
        assert!(
            first_case["metrics"]["candidate"]["rms"]
                .as_f64()
                .expect("candidate rms")
                > 0.0
        );
        assert!(
            first_case["metrics"]["candidate"]["event_density_per_bar"]
                .as_f64()
                .expect("candidate event density")
                > 0.0
        );
        assert!(
            first_case["metrics"]["signal_delta"]["rms"]
                .as_f64()
                .expect("signal delta rms")
                >= first_case["thresholds"]["min_signal_delta_rms"]
                    .as_f64()
                    .expect("min signal delta")
        );
        assert!(first_case["metrics"]["mc202_phrase_grid"].is_null());

        let mc202_case = cases
            .iter()
            .find(|case| case["id"] == "mc202-follower-to-answer")
            .expect("mc202 case");
        assert_eq!(mc202_case["metrics"]["mc202_phrase_grid"]["passed"], true);
        assert_eq!(
            mc202_case["metrics"]["mc202_source_phrase_slot"]["passed"],
            true
        );
        assert_eq!(
            mc202_case["metrics"]["mc202_source_phrase_slot"]["contract"],
            MC202_SOURCE_PHRASE_SLOT_CONTRACT
        );
        assert_eq!(
            mc202_case["metrics"]["mc202_source_phrase_slot"]["phrase_index"],
            3
        );
        assert_eq!(
            mc202_case["metrics"]["mc202_phrase_grid"]["resolution"],
            "sixteenth"
        );
        assert!(
            mc202_case["metrics"]["mc202_phrase_grid"]["starts_on_phrase_boundary"]
                .as_bool()
                .expect("phrase boundary")
        );
        assert!(
            mc202_case["metrics"]["mc202_phrase_grid"]["hit_ratio"]
                .as_f64()
                .expect("hit ratio")
                >= 0.95
        );

        let artifacts = manifest["artifacts"].as_array().expect("artifacts");
        assert_eq!(artifacts.len(), 31);
        for artifact in artifacts {
            let path = PathBuf::from(artifact["path"].as_str().expect("artifact path"));
            assert!(path.is_file(), "{} missing", path.display());
            if let Some(metrics_path) = artifact["metrics_path"].as_str() {
                let metrics_path = PathBuf::from(metrics_path);
                assert!(metrics_path.is_file(), "{} missing", metrics_path.display());
            }
        }
    }
}
