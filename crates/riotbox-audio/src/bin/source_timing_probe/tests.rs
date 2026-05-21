use super::*;
use riotbox_audio::source_audio::write_interleaved_pcm16_wav;
use tempfile::tempdir;

#[test]
fn parses_json_probe_args() {
    let args = Args::parse(["--json".to_string(), "beat.wav".to_string()]).expect("args");

    assert_eq!(args.source_path, PathBuf::from("beat.wav"));
    assert!(args.json);
}

#[test]
fn renders_probe_summary_for_accented_loop() {
    let tempdir = tempdir().expect("tempdir");
    let source_path = tempdir.path().join("accented_loop.wav");
    write_interleaved_pcm16_wav(&source_path, 1_000, 1, &accented_loop_samples())
        .expect("write source");
    let source = SourceAudioCache::load_pcm_wav(&source_path).expect("load source");
    let probe = analyze_source_timing_probe(
        &source,
        SourceTimingProbeConfig {
            window_size_frames: 50,
            hop_size_frames: 50,
            onset_threshold_ratio: 0.20,
            min_onset_flux: 0.01,
        },
    );
    let input = probe.bpm_candidate_input(
        source_path.display().to_string(),
        MeterHint {
            beats_per_bar: 4,
            beat_unit: 4,
        },
    );
    let policy = SourceTimingProbeBpmCandidatePolicy::dance_loop_auto_readiness();
    let readiness = source_timing_probe_readiness_report(&input, policy);
    let beat = source_timing_probe_beat_evidence_report(&input, policy);
    let downbeat = source_timing_probe_downbeat_evidence_report(
        &input,
        readiness.primary_bpm.unwrap_or(f32::NAN),
        policy,
    );
    let timing = timing_model_from_probe_bpm_candidates(&input, policy);
    let summary =
        ProbeSummary::from_report(&source_path, &readiness, &beat, &downbeat, &timing, &probe);
    let text = render_text(&summary);
    let json = serde_json::to_value(&summary).expect("json");

    assert_eq!(summary.schema, "riotbox.source_timing_probe_cli.v1");
    assert_eq!(summary.schema_version, 1);
    assert!(
        summary
            .primary_bpm
            .is_some_and(|bpm| (bpm - 120.0).abs() <= 0.1)
    );
    assert!(summary.primary_beat_score.is_some_and(|score| score > 0.0));
    assert!(
        summary
            .primary_downbeat_score
            .is_some_and(|score| score > 0.0)
    );
    assert!(text.contains("cue: "));
    assert!(text.contains("action: "));
    assert!(text.contains("grid_use="));
    assert!(text.contains("beat: stable"));
    assert!(text.contains("downbeat: "));
    assert!(text.contains("scores: beat="));
    assert!(text.contains("anchors: total="));
    assert!(text.contains("groove: residuals="));
    assert!(json["schema"].is_string());
    assert!(json["actionability"].is_string());
    assert!(json["grid_use"].is_string());
    assert!(json["primary_beat_score"].is_number());
    assert!(json["primary_downbeat_score"].is_number());
    assert!(json["anchor_evidence"]["primary_anchor_count"].is_number());
    assert!(json["anchor_evidence"]["primary_anchor_preview"].is_array());
    assert!(json["groove_evidence"]["primary_groove_residual_count"].is_number());
    assert!(json["groove_evidence"]["primary_groove_preview"].is_array());
}

#[test]
fn source_timing_grid_use_marks_short_loop_manual_confirm() {
    let report = readiness_report(SourceTimingProbeReadinessStatus::NeedsReview, true);

    assert_eq!(
        source_timing_grid_use(&report).label(),
        "short_loop_manual_confirm"
    );
}

#[test]
fn source_timing_grid_use_marks_locked_grid_only_without_manual_confirm() {
    let report = readiness_report(SourceTimingProbeReadinessStatus::Ready, false);

    assert_eq!(source_timing_grid_use(&report).label(), "locked_grid");
}

#[test]
fn source_timing_grid_use_keeps_ambiguous_loop_manual_only() {
    let mut report = readiness_report(SourceTimingProbeReadinessStatus::Weak, true);
    report.downbeat_status = SourceTimingProbeDownbeatEvidenceStatus::Weak;
    report.confidence_result = SourceTimingCandidateConfidenceResult::CandidateAmbiguous;

    assert_eq!(
        source_timing_grid_use(&report).label(),
        "manual_confirm_only"
    );
}

#[test]
fn source_timing_grid_use_marks_missing_bpm_unavailable() {
    let mut report = readiness_report(SourceTimingProbeReadinessStatus::Unavailable, true);
    report.primary_bpm = None;

    assert_eq!(source_timing_grid_use(&report).label(), "unavailable");
}

fn readiness_report(
    readiness: SourceTimingProbeReadinessStatus,
    requires_manual_confirm: bool,
) -> SourceTimingProbeReadinessReport {
    SourceTimingProbeReadinessReport {
        schema: "riotbox.source_timing_probe_readiness.v1",
        schema_version: 1,
        source_id: "test-source".into(),
        primary_bpm: Some(128.0),
        primary_downbeat_offset_beats: Some(0),
        primary_downbeat_score: Some(0.75),
        alternate_downbeat_phase_count: 0,
        beat_status: SourceTimingProbeBeatEvidenceStatus::Stable,
        downbeat_status: SourceTimingProbeDownbeatEvidenceStatus::Stable,
        confidence_result: SourceTimingCandidateConfidenceResult::CandidateCautious,
        drift_status: SourceTimingCandidateDriftStatus::NotEnoughMaterial,
        phrase_status: SourceTimingCandidatePhraseStatus::NotEnoughMaterial,
        alternate_evidence_count: 0,
        warning_codes: vec![TimingWarningCode::PhraseUncertain],
        requires_manual_confirm,
        readiness,
    }
}

fn accented_loop_samples() -> Vec<f32> {
    let mut samples = vec![0.0; 16_000];
    for beat in 0..32 {
        let frame = beat * 500;
        let amplitude = if beat % 4 == 0 { 1.0 } else { 0.55 };
        add_click(&mut samples, frame, amplitude);
    }
    samples
}

fn add_click(samples: &mut [f32], frame: usize, amplitude: f32) {
    for offset in 0..16 {
        if let Some(sample) = samples.get_mut(frame + offset) {
            *sample += amplitude * (1.0 - offset as f32 / 16.0);
        }
    }
}
