use std::path::Path;

use riotbox_audio::{
    source_audio::SourceAudioCache,
    source_timing_probe::{SourceTimingProbeConfig, analyze_source_timing_probe},
};
use riotbox_core::source_graph::{
    MeterHint, SourceTimingProbeReadinessReport, source_timing_probe_readiness_report,
    timing_model_from_probe_bpm_candidates,
};

use super::{
    config::DEFAULT_BEATS_PER_BAR,
    source_timing_policy_profile::SOURCE_TIMING_POLICY_PROFILE,
    timing_evidence::{ManifestSourceTimingAnchorEvidence, ManifestSourceTimingGrooveEvidence},
};

pub(super) struct SourceTimingAnalysisForManifest {
    pub(super) readiness: SourceTimingProbeReadinessReport,
    pub(super) anchor_evidence: ManifestSourceTimingAnchorEvidence,
    pub(super) groove_evidence: ManifestSourceTimingGrooveEvidence,
}

pub(super) fn source_timing_analysis_for_source(
    source: &SourceAudioCache,
    source_path: &Path,
) -> SourceTimingAnalysisForManifest {
    let probe = analyze_source_timing_probe(source, SourceTimingProbeConfig::default());
    let input = probe.bpm_candidate_input(
        source_path.display().to_string(),
        MeterHint {
            beats_per_bar: DEFAULT_BEATS_PER_BAR as u8,
            beat_unit: 4,
        },
    );
    let readiness = source_timing_probe_readiness_report(
        &input,
        SOURCE_TIMING_POLICY_PROFILE.bpm_candidate_policy,
    );
    let timing = timing_model_from_probe_bpm_candidates(
        &input,
        SOURCE_TIMING_POLICY_PROFILE.bpm_candidate_policy,
    );
    SourceTimingAnalysisForManifest {
        readiness,
        anchor_evidence: ManifestSourceTimingAnchorEvidence::from_timing(&timing),
        groove_evidence: ManifestSourceTimingGrooveEvidence::from_timing(&timing),
    }
}
