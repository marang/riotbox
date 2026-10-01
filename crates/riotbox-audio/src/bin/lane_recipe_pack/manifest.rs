use super::args::Args;
use super::config::CHANNEL_COUNT;
use super::config::PACK_ID;
use super::config::SAMPLE_RATE;
use super::report_model::CaseReport;
use super::report_model::Mc202PhraseGridTimingMetrics;
use super::report_model::Mc202SourcePhraseSlotMetrics;
use super::signal_delta::rms_delta;
use riotbox_audio::listening_manifest::LISTENING_MANIFEST_SCHEMA_VERSION;
use riotbox_audio::listening_manifest::ListeningPackArtifact as ManifestArtifact;
use riotbox_audio::listening_manifest::ListeningPackSignalMetrics as ManifestSignalMetrics;
use riotbox_audio::listening_manifest::write_manifest_json;
use serde::Serialize;
use std::path::Path;

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
    primitive_renderer_boundary: PrimitiveRendererBoundary,
    cases: Vec<ManifestCase>,
    result: &'static str,
}

#[derive(Serialize)]
struct ManifestCase {
    id: &'static str,
    title: &'static str,
    pattern_origin: &'static str,
    evidence_role: &'static str,
    product_output_allowed: bool,
    quality_proof: bool,
    demo_readiness: &'static str,
    promotion_blocked: bool,
    recipe_refs: &'static str,
    baseline_label: &'static str,
    candidate_label: &'static str,
    thresholds: ManifestThresholds,
    metrics: ManifestCaseMetrics,
    result: &'static str,
}

#[derive(Serialize)]
struct PrimitiveRendererBoundary {
    schema: &'static str,
    evidence_role: &'static str,
    product_output_allowed: bool,
    quality_proof: bool,
    demo_readiness: &'static str,
    promotion_blocked: bool,
    affected_paths: Vec<String>,
    musician_message: &'static str,
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

pub(super) fn write_manifest(
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
        primitive_renderer_boundary: primitive_renderer_boundary(reports.len()),
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

fn primitive_renderer_boundary(case_count: usize) -> PrimitiveRendererBoundary {
    PrimitiveRendererBoundary {
        schema: "riotbox.primitive_renderer_boundary.v1",
        evidence_role: "non_product_diagnostic_control",
        product_output_allowed: false,
        quality_proof: false,
        demo_readiness: "unverified",
        promotion_blocked: true,
        affected_paths: (0..case_count)
            .map(|index| format!("cases[{index}].pattern_origin"))
            .collect(),
        musician_message: "Lane recipe primitive renderer cases are regression controls; they must not be promoted as source-derived product output.",
    }
}

impl From<&CaseReport> for ManifestCase {
    fn from(report: &CaseReport) -> Self {
        Self {
            id: report.id,
            title: report.title,
            pattern_origin: "primitive_renderer",
            evidence_role: "non_product_diagnostic_control",
            product_output_allowed: false,
            quality_proof: false,
            demo_readiness: "unverified",
            promotion_blocked: true,
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
