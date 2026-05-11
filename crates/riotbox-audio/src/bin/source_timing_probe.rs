use std::{
    env,
    path::{Path, PathBuf},
    process,
};

#[path = "source_timing_probe/anchor_summary.rs"]
mod anchor_summary;
#[path = "source_timing_probe/groove_summary.rs"]
mod groove_summary;

use anchor_summary::AnchorEvidenceSummary;
use groove_summary::GrooveEvidenceSummary;
use riotbox_audio::{
    source_audio::SourceAudioCache,
    source_timing_probe::{SourceTimingProbeConfig, analyze_source_timing_probe},
};
use riotbox_core::source_graph::{
    MeterHint, SourceTimingCandidateConfidenceResult, SourceTimingCandidateDriftStatus,
    SourceTimingCandidatePhraseStatus, SourceTimingProbeBeatEvidenceReport,
    SourceTimingProbeBeatEvidenceStatus, SourceTimingProbeBpmCandidatePolicy,
    SourceTimingProbeDownbeatEvidenceReport, SourceTimingProbeDownbeatEvidenceStatus,
    SourceTimingProbeReadinessReport, SourceTimingProbeReadinessStatus, TimingWarningCode,
    source_timing_probe_beat_evidence_report, source_timing_probe_downbeat_evidence_report,
    source_timing_probe_readiness_report, timing_model_from_probe_bpm_candidates,
};
use serde::Serialize;

const DEFAULT_BEATS_PER_BAR: u8 = 4;

#[derive(Debug, PartialEq, Eq)]
struct Args {
    source_path: PathBuf,
    json: bool,
}

#[derive(Serialize)]
struct ProbeSummary {
    schema: &'static str,
    schema_version: u32,
    source_path: String,
    source_id: String,
    cue: &'static str,
    readiness: &'static str,
    requires_manual_confirm: bool,
    grid_use: &'static str,
    primary_bpm: Option<f32>,
    primary_beat_score: Option<f32>,
    primary_beat_matched_onset_ratio: Option<f32>,
    primary_beat_median_distance_ratio: Option<f32>,
    primary_downbeat_offset_beats: Option<u8>,
    primary_downbeat_score: Option<f32>,
    beat_status: &'static str,
    downbeat_status: &'static str,
    confidence_result: &'static str,
    drift_status: &'static str,
    phrase_status: &'static str,
    alternate_evidence_count: usize,
    alternate_beat_candidate_count: usize,
    alternate_downbeat_phase_count: usize,
    anchor_evidence: AnchorEvidenceSummary,
    groove_evidence: GrooveEvidenceSummary,
    warning_codes: Vec<&'static str>,
    onset_count: usize,
    onset_density_per_second: f32,
    duration_seconds: f32,
}

fn main() {
    match run(env::args().skip(1)) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let args = args.into_iter().collect::<Vec<_>>();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        println!("{}", usage());
        return Ok(());
    }

    let args = Args::parse(args)?;
    let source = SourceAudioCache::load_pcm_wav(&args.source_path)
        .map_err(|error| format!("failed to load source WAV: {error}"))?;
    let probe = analyze_source_timing_probe(&source, SourceTimingProbeConfig::default());
    let source_id = args.source_path.display().to_string();
    let input = probe.bpm_candidate_input(
        source_id.clone(),
        MeterHint {
            beats_per_bar: DEFAULT_BEATS_PER_BAR,
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
    let summary = ProbeSummary::from_report(
        &args.source_path,
        &readiness,
        &beat,
        &downbeat,
        &timing,
        &probe,
    );

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&summary)
                .map_err(|error| format!("failed to render JSON: {error}"))?
        );
    } else {
        println!("{}", render_text(&summary));
    }

    Ok(())
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut source_path = None;
        let mut json = false;

        for arg in args {
            match arg.as_str() {
                "--json" => json = true,
                value if value.starts_with('-') => {
                    return Err(format!("unknown option {value:?}\n{}", usage()));
                }
                value => {
                    if source_path.replace(PathBuf::from(value)).is_some() {
                        return Err(format!("unexpected extra argument {value:?}\n{}", usage()));
                    }
                }
            }
        }

        let Some(source_path) = source_path else {
            return Err(usage());
        };

        Ok(Self { source_path, json })
    }
}

impl ProbeSummary {
    fn from_report(
        source_path: &Path,
        report: &SourceTimingProbeReadinessReport,
        beat: &SourceTimingProbeBeatEvidenceReport,
        downbeat: &SourceTimingProbeDownbeatEvidenceReport,
        timing: &riotbox_core::source_graph::TimingModel,
        probe: &riotbox_audio::source_timing_probe::SourceTimingProbe,
    ) -> Self {
        Self {
            schema: "riotbox.source_timing_probe_cli.v1",
            schema_version: 1,
            source_path: source_path.display().to_string(),
            source_id: report.source_id.clone(),
            cue: source_timing_readiness_cue(report.readiness, report.requires_manual_confirm),
            readiness: readiness_status_label(report.readiness),
            requires_manual_confirm: report.requires_manual_confirm,
            grid_use: source_timing_grid_use(report),
            primary_bpm: report.primary_bpm,
            primary_beat_score: beat.primary_score,
            primary_beat_matched_onset_ratio: beat.primary_matched_onset_ratio,
            primary_beat_median_distance_ratio: beat.primary_median_distance_ratio,
            primary_downbeat_offset_beats: report.primary_downbeat_offset_beats,
            primary_downbeat_score: downbeat.primary_score,
            beat_status: beat_status_label(report.beat_status),
            downbeat_status: downbeat_status_label(report.downbeat_status),
            confidence_result: confidence_result_label(report.confidence_result),
            drift_status: drift_status_label(report.drift_status),
            phrase_status: phrase_status_label(report.phrase_status),
            alternate_evidence_count: report.alternate_evidence_count,
            alternate_beat_candidate_count: beat.alternate_candidate_count,
            alternate_downbeat_phase_count: downbeat.alternate_phase_count,
            anchor_evidence: AnchorEvidenceSummary::from_timing(timing),
            groove_evidence: GrooveEvidenceSummary::from_timing(timing),
            warning_codes: report
                .warning_codes
                .iter()
                .map(|code| warning_code_label(*code))
                .collect(),
            onset_count: probe.onset_count,
            onset_density_per_second: probe.onset_density_per_second,
            duration_seconds: probe.duration_seconds,
        }
    }
}

fn render_text(summary: &ProbeSummary) -> String {
    let bpm = summary
        .primary_bpm
        .map_or_else(|| "none".to_string(), |bpm| format!("{bpm:.2}"));
    let downbeat = summary
        .primary_downbeat_offset_beats
        .map_or_else(|| "none".to_string(), |offset| offset.to_string());
    let beat_score = summary
        .primary_beat_score
        .map_or_else(|| "none".to_string(), |score| format!("{score:.3}"));
    let downbeat_score = summary
        .primary_downbeat_score
        .map_or_else(|| "none".to_string(), |score| format!("{score:.3}"));
    let warnings = if summary.warning_codes.is_empty() {
        "none".to_string()
    } else {
        summary.warning_codes.join(",")
    };
    let anchors = &summary.anchor_evidence;
    let groove = &summary.groove_evidence;

    format!(
        concat!(
            "Riotbox Source Timing Probe\n",
            "source: {source}\n",
            "cue: {cue}\n",
            "readiness: {readiness} manual_confirm={manual_confirm} grid_use={grid_use}\n",
            "bpm: {bpm}\n",
            "beat: {beat} downbeat: {downbeat_status} offset_beats={downbeat}\n",
            "scores: beat={beat_score} downbeat={downbeat_score}\n",
            "phrase: {phrase} drift: {drift} confidence: {confidence}\n",
            "anchors: total={anchor_total} kick={kick_anchors} backbeat={backbeat_anchors} transient={transient_anchors}\n",
            "groove: residuals={groove_residuals} max_abs_offset_ms={groove_max_abs_offset:.3}\n",
            "alternates: {alternates} warnings: {warnings}\n",
            "onsets: {onsets} density_per_second={density:.3} duration_seconds={duration:.3}"
        ),
        source = summary.source_path,
        cue = summary.cue,
        readiness = summary.readiness,
        manual_confirm = summary.requires_manual_confirm,
        grid_use = summary.grid_use,
        bpm = bpm,
        beat = summary.beat_status,
        downbeat_status = summary.downbeat_status,
        downbeat = downbeat,
        beat_score = beat_score,
        downbeat_score = downbeat_score,
        phrase = summary.phrase_status,
        drift = summary.drift_status,
        confidence = summary.confidence_result,
        anchor_total = anchors.primary_anchor_count,
        kick_anchors = anchors.primary_kick_anchor_count,
        backbeat_anchors = anchors.primary_backbeat_anchor_count,
        transient_anchors = anchors.primary_transient_anchor_count,
        groove_residuals = groove.primary_groove_residual_count,
        groove_max_abs_offset = groove.primary_max_abs_offset_ms,
        alternates = summary.alternate_evidence_count,
        warnings = warnings,
        onsets = summary.onset_count,
        density = summary.onset_density_per_second,
        duration = summary.duration_seconds,
    )
}

fn usage() -> String {
    "usage: source_timing_probe [--json] <source.wav>".to_string()
}

fn source_timing_readiness_cue(
    readiness: SourceTimingProbeReadinessStatus,
    requires_manual_confirm: bool,
) -> &'static str {
    if requires_manual_confirm {
        return "needs confirm";
    }
    match readiness {
        SourceTimingProbeReadinessStatus::Ready => "grid locked",
        SourceTimingProbeReadinessStatus::NeedsReview | SourceTimingProbeReadinessStatus::Weak => {
            "listen first"
        }
        SourceTimingProbeReadinessStatus::Unavailable => "not available",
    }
}

fn source_timing_grid_use(report: &SourceTimingProbeReadinessReport) -> &'static str {
    if report.primary_bpm.is_none()
        || report.readiness == SourceTimingProbeReadinessStatus::Unavailable
    {
        return "unavailable";
    }
    if report.readiness == SourceTimingProbeReadinessStatus::Ready
        && !report.requires_manual_confirm
    {
        return "locked_grid";
    }
    if is_stable_short_loop_manual_confirm(report) {
        return "short_loop_manual_confirm";
    }
    if report.requires_manual_confirm {
        return "manual_confirm_only";
    }
    "fallback_grid"
}

fn is_stable_short_loop_manual_confirm(report: &SourceTimingProbeReadinessReport) -> bool {
    report.readiness == SourceTimingProbeReadinessStatus::NeedsReview
        && report.requires_manual_confirm
        && report.primary_bpm.is_some()
        && report.beat_status == SourceTimingProbeBeatEvidenceStatus::Stable
        && report.downbeat_status == SourceTimingProbeDownbeatEvidenceStatus::Stable
        && report.phrase_status == SourceTimingCandidatePhraseStatus::NotEnoughMaterial
        && report.confidence_result == SourceTimingCandidateConfidenceResult::CandidateCautious
        && report.alternate_evidence_count == 0
}

fn readiness_status_label(status: SourceTimingProbeReadinessStatus) -> &'static str {
    match status {
        SourceTimingProbeReadinessStatus::Unavailable => "unavailable",
        SourceTimingProbeReadinessStatus::Weak => "weak",
        SourceTimingProbeReadinessStatus::NeedsReview => "needs_review",
        SourceTimingProbeReadinessStatus::Ready => "ready",
    }
}

fn beat_status_label(status: SourceTimingProbeBeatEvidenceStatus) -> &'static str {
    match status {
        SourceTimingProbeBeatEvidenceStatus::Unavailable => "unavailable",
        SourceTimingProbeBeatEvidenceStatus::Weak => "weak",
        SourceTimingProbeBeatEvidenceStatus::Stable => "stable",
        SourceTimingProbeBeatEvidenceStatus::Ambiguous => "ambiguous",
    }
}

fn downbeat_status_label(status: SourceTimingProbeDownbeatEvidenceStatus) -> &'static str {
    match status {
        SourceTimingProbeDownbeatEvidenceStatus::Unavailable => "unavailable",
        SourceTimingProbeDownbeatEvidenceStatus::Weak => "weak",
        SourceTimingProbeDownbeatEvidenceStatus::Stable => "stable",
        SourceTimingProbeDownbeatEvidenceStatus::Ambiguous => "ambiguous",
    }
}

fn confidence_result_label(result: SourceTimingCandidateConfidenceResult) -> &'static str {
    match result {
        SourceTimingCandidateConfidenceResult::Degraded => "degraded",
        SourceTimingCandidateConfidenceResult::CandidateCautious => "candidate_cautious",
        SourceTimingCandidateConfidenceResult::CandidateAmbiguous => "candidate_ambiguous",
    }
}

fn drift_status_label(status: SourceTimingCandidateDriftStatus) -> &'static str {
    match status {
        SourceTimingCandidateDriftStatus::Unavailable => "unavailable",
        SourceTimingCandidateDriftStatus::NotEnoughMaterial => "not_enough_material",
        SourceTimingCandidateDriftStatus::Stable => "stable",
        SourceTimingCandidateDriftStatus::High => "high",
    }
}

fn phrase_status_label(status: SourceTimingCandidatePhraseStatus) -> &'static str {
    match status {
        SourceTimingCandidatePhraseStatus::Unavailable => "unavailable",
        SourceTimingCandidatePhraseStatus::NotEnoughMaterial => "not_enough_material",
        SourceTimingCandidatePhraseStatus::AmbiguousDownbeat => "ambiguous_downbeat",
        SourceTimingCandidatePhraseStatus::HighDrift => "high_drift",
        SourceTimingCandidatePhraseStatus::Stable => "stable",
    }
}

fn warning_code_label(code: TimingWarningCode) -> &'static str {
    match code {
        TimingWarningCode::WeakKickAnchor => "weak_kick_anchor",
        TimingWarningCode::WeakBackbeatAnchor => "weak_backbeat_anchor",
        TimingWarningCode::AmbiguousDownbeat => "ambiguous_downbeat",
        TimingWarningCode::HalfTimePossible => "half_time_possible",
        TimingWarningCode::DoubleTimePossible => "double_time_possible",
        TimingWarningCode::DriftHigh => "drift_high",
        TimingWarningCode::PhraseUncertain => "phrase_uncertain",
        TimingWarningCode::LowTimingConfidence => "low_timing_confidence",
    }
}

#[cfg(test)]
mod tests {
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
        assert!(text.contains("grid_use="));
        assert!(text.contains("beat: stable"));
        assert!(text.contains("downbeat: "));
        assert!(text.contains("scores: beat="));
        assert!(text.contains("anchors: total="));
        assert!(text.contains("groove: residuals="));
        assert!(json["schema"].is_string());
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

        assert_eq!(source_timing_grid_use(&report), "short_loop_manual_confirm");
    }

    #[test]
    fn source_timing_grid_use_marks_locked_grid_only_without_manual_confirm() {
        let report = readiness_report(SourceTimingProbeReadinessStatus::Ready, false);

        assert_eq!(source_timing_grid_use(&report), "locked_grid");
    }

    #[test]
    fn source_timing_grid_use_keeps_ambiguous_loop_manual_only() {
        let mut report = readiness_report(SourceTimingProbeReadinessStatus::Weak, true);
        report.downbeat_status = SourceTimingProbeDownbeatEvidenceStatus::Weak;
        report.confidence_result = SourceTimingCandidateConfidenceResult::CandidateAmbiguous;

        assert_eq!(source_timing_grid_use(&report), "manual_confirm_only");
    }

    #[test]
    fn source_timing_grid_use_marks_missing_bpm_unavailable() {
        let mut report = readiness_report(SourceTimingProbeReadinessStatus::Unavailable, true);
        report.primary_bpm = None;

        assert_eq!(source_timing_grid_use(&report), "unavailable");
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
}
