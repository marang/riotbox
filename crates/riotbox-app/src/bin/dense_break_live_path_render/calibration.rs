//! Bounded diagnostic adapter; source admission belongs to the preregistered caller.

mod protocol;

use std::{
    error::Error,
    fmt, fs,
    io::{Read, Write},
    path::PathBuf,
};

use riotbox_app::jam_app::{JamAppState, JamFileSet};
use riotbox_audio::runtime::{
    MasterBusLimiterReport, OfflineAudioMetrics, RuntimeMixRenderPlan,
    RuntimeMixRenderSequenceStep,
    limiter_calibration::{self, Policy, PolicyOutput, RuntimeMixEvidence},
};
use riotbox_core::source_graph::{DecodeProfile, SourceGraph};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::calibration_preparation;
pub(super) use protocol::CalibrationVersion;
use protocol::{HistoricalControls, pcm_hash};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MAX_REQUEST_BYTES: usize = 32 * 1024 * 1024;
const MAX_WAV_BYTES: usize = 6 * 1024 * 1024;
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u16 = 2;
const DURATION_BEATS: f64 = 8.0;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(super) enum Case {
    #[serde(rename = "dense_beat03_130")]
    Dense,
    #[serde(rename = "tonal_rusharp_120")]
    Tonal,
    #[serde(rename = "sparse_kicksnr_120")]
    Sparse,
}

impl Case {
    pub(super) fn bpm(self) -> f32 {
        if self == Self::Dense { 130.0 } else { 120.0 }
    }

    pub(super) fn start_beat(self) -> f64 {
        if self == Self::Sparse { 17.0 } else { 8.0 }
    }

    fn identity(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Dense => (
                "/home/markus/Dev/riotbox/data/test_audio/examples/Beat03_130BPM(Full).wav",
                "sha256:e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a",
                "src-e752819f53f7",
            ),
            Self::Tonal => (
                "/home/markus/Dev/riotbox/data/test_audio/examples/DH_RushArp_120_A.wav",
                "sha256:ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e",
                "src-ec2a0c930eb3",
            ),
            Self::Sparse => (
                "/home/markus/Dev/riotbox/data/test_audio/examples/DH_BeatC_KickSnr_120-01.wav",
                "sha256:8a970e5d7bd9b29771aba85f75e697c7510940d4404714bfb1e55e210c15f46c",
                "src-8a970e5d7bd9",
            ),
        }
    }

    fn validate_graph(self, graph: &SourceGraph) -> Result<()> {
        let (path, hash, source_id) = self.identity();
        let source = &graph.source;
        let duration = if self == Self::Dense { 3.692 } else { 4.0 };
        if source.path != path
            || source.content_hash != hash
            || source.source_id.as_str() != source_id
            || graph.provenance.source_hash != hash
            || source.sample_rate != 44_100
            || source.channel_count != 2
            || source.decode_profile != DecodeProfile::Native
            || source.duration_seconds != duration
        {
            return Err(
                "calibration case source identity/native format does not match registration".into(),
            );
        }
        let hypothesis_id = if self == Self::Dense {
            "probe-bpm-primary"
        } else {
            "manual-source-grid-v1-42f00000-00000000"
        };
        let primary = graph
            .timing
            .primary_hypothesis()
            .ok_or("calibration graph has no primary timing hypothesis")?;
        let graph_bpm = if self == Self::Dense {
            130.28494
        } else {
            120.0
        };
        if graph.timing.primary_hypothesis_id.as_deref() != Some(hypothesis_id)
            || primary.bpm != graph_bpm
            || primary.meter.beats_per_bar != 4
        {
            return Err(
                "calibration graph timing differs from registered hypothesis/BPM/meter".into(),
            );
        }
        // Do not manufacture a zero-phase fallback for the historical graph.
        let anchor = primary
            .transport_bar_grid_anchor()
            .ok_or("calibration timing has no evidenced transport bar anchor")?;
        if self != Self::Dense && (anchor.bar_index != 1 || anchor.beat_cursor != 0) {
            return Err("calibration manual zero-phase source anchor changed".into());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    case_id: Case,
    graph: SourceGraph,
    source_wav_bytes: Vec<u8>,
    output_dir: PathBuf,
}

fn read_request(reader: impl Read) -> Result<Request> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err("calibration request exceeds 32 MiB".into());
    }
    let request: Request = serde_json::from_slice(&bytes)?;
    if request.source_wav_bytes.is_empty() || request.source_wav_bytes.len() > MAX_WAV_BYTES {
        return Err("calibration WAV payload must contain 1..=6 MiB".into());
    }
    request.case_id.validate_graph(&request.graph)?;
    Ok(request)
}

pub(super) fn run(
    reader: impl Read,
    writer: impl Write,
    version: CalibrationVersion,
) -> Result<()> {
    let request = read_request(reader)?;
    // Only the caller-created output directory is inspected. Source and graph paths
    // remain opaque metadata; the constructor receives the admitted bytes directly.
    let output_metadata = fs::symlink_metadata(&request.output_dir)?;
    if !output_metadata.is_dir() || output_metadata.file_type().is_symlink() {
        return Err("calibration requires a caller-created ordinary output directory".into());
    }
    let mut state = JamAppState::from_limiter_calibration_graph_bytes(
        request.graph,
        &request.source_wav_bytes,
        JamFileSet {
            session_path: request.output_dir.join("session.json"),
            source_graph_path: Some(request.output_dir.join("source-graph.json")),
        },
    )?;
    let plan = match calibration_preparation::prepare(&mut state, request.case_id) {
        Ok(plan) => plan,
        Err(error) => {
            write_failure_record(
                error.as_ref(),
                "preparation",
                Some(preparation_snapshot(&state, request.case_id)),
                std::io::stderr().lock(),
            )?;
            return Err(error);
        }
    };
    let frame_count = (DURATION_BEATS * 60.0 * f64::from(SAMPLE_RATE)
        / f64::from(request.case_id.bpm()))
    .round() as usize;
    // Fail a changed capture binding before performing any diagnostic render.
    let preparation = match calibration_preparation::evidence(&state, request.case_id) {
        Ok(preparation) => preparation,
        Err(error) => {
            write_failure_record(
                error.as_ref(),
                "preparation_evidence",
                Some(preparation_snapshot(&state, request.case_id)),
                std::io::stderr().lock(),
            )?;
            return Err(error);
        }
    };
    let comparison = match version {
        CalibrationVersion::V1 => compare_plan(&plan, frame_count, request.case_id),
        CalibrationVersion::V2 => compare_plan_with_history(
            &plan,
            frame_count,
            request.case_id,
            Some(&HistoricalControls::registered(request.case_id)),
        ),
    };
    let mut result = match comparison {
        Ok(result) => result,
        Err(error) => {
            write_failure_record(
                error.as_ref(),
                "comparison",
                Some(preparation),
                std::io::stderr().lock(),
            )?;
            return Err(error);
        }
    };
    result["case_id"] = serde_json::to_value(request.case_id)?;
    result["preparation"] = preparation;
    serde_json::to_writer(writer, &result)?;
    Ok(())
}

fn render_once(
    plan: &RuntimeMixRenderPlan,
    frames: usize,
    callback: usize,
) -> Result<RuntimeMixEvidence> {
    let mut results = limiter_calibration::render_sequence(
        &[RuntimeMixRenderSequenceStep::new(plan, frames)],
        SAMPLE_RATE,
        CHANNELS,
        callback,
    );
    if results.len() != 1 {
        return Err("calibration render did not produce exactly one interval".into());
    }
    Ok(results.remove(0))
}

fn bits_equal(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(a, b)| a.to_bits() == b.to_bits())
}

fn validate_samples(samples: &[f32], frames: usize) -> Result<()> {
    if frames == 0
        || samples.len() != frames * usize::from(CHANNELS)
        || samples.iter().any(|sample| !sample.is_finite())
    {
        return Err(
            "calibration PCM is empty, nonfinite, or not aligned to the frozen stereo interval"
                .into(),
        );
    }
    Ok(())
}

#[derive(Serialize)]
struct ComparisonFailure {
    stage: &'static str,
    reason: String,
    diagnostics: Value,
}

impl fmt::Display for ComparisonFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "calibration {}: {}", self.stage, self.reason)
    }
}

impl fmt::Debug for ComparisonFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl Error for ComparisonFailure {}

pub(super) fn preparation_snapshot(state: &JamAppState, case: Case) -> Value {
    json!({"case_id": case, "action_log": state.session.action_log,
        "captures": state.session.captures, "source_timing": state.session.runtime_state.source_timing})
}

pub(super) fn write_failure_record(
    error: &(dyn Error + 'static),
    stage: &'static str,
    preparation: Option<Value>,
    mut writer: impl Write,
) -> Result<()> {
    let mut record = if let Some(failure) = error.downcast_ref::<ComparisonFailure>() {
        serde_json::to_value(failure)?
    } else {
        json!({"stage": stage, "reason": error.to_string(), "diagnostics": {}})
    };
    if let Some(preparation) = preparation {
        record["preparation"] = preparation;
    }
    // The ordinary eleven-action record is retained in full. An unexpectedly
    // large error still keeps actual commit/capture identities, with explicit
    // truncation, rather than overflowing the caller's 64 KiB stderr budget.
    let encoded = serde_json::to_string(&record)?;
    let encoded = if encoded.len() > 60 * 1024 {
        record["preparation"] = compact_failure_preparation(&record["preparation"]);
        record["reason"] = json!(
            record["reason"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .take(1_024)
                .collect::<String>()
        );
        record["unabridged_record_bytes"] = json!(encoded.len());
        serde_json::to_string(&record)?
    } else {
        encoded
    };
    if encoded.len() > 63 * 1024 {
        return Err("calibration failure record exceeds its bounded metadata budget".into());
    }
    writeln!(writer, "RIOTBOX_LIMITER_CALIBRATION_FAILURE {encoded}")?;
    Ok(())
}

fn compact_failure_preparation(preparation: &Value) -> Value {
    let actions = preparation
        .get("committed_actions")
        .unwrap_or(&preparation["action_log"]["actions"]);
    let records = preparation
        .get("commit_records")
        .unwrap_or(&preparation["action_log"]["commit_records"]);
    let empty = Vec::new();
    json!({
        "truncated": true, "source_timing": preparation["source_timing"],
        "action_count": actions.as_array().map(Vec::len),
        "actions": actions.as_array().unwrap_or(&empty).iter().take(16).map(|action| json!({
            "id": action["id"], "command": action["command"], "status": action["status"],
            "requested_at": action["requested_at"], "committed_at": action["committed_at"],
        })).collect::<Vec<_>>(),
        "commit_records": records.as_array().unwrap_or(&empty).iter().take(16).map(|record| json!({
            "action_id": record["action_id"], "commit_sequence": record["commit_sequence"],
            "committed_at": record["committed_at"], "kind": record["boundary"]["kind"],
            "beat_index": record["boundary"]["beat_index"], "bar_index": record["boundary"]["bar_index"],
            "phrase_index": record["boundary"]["phrase_index"],
        })).collect::<Vec<_>>(),
        "capture_window": preparation["capture_window"],
        "capture_audio_identity": preparation["capture_audio_identity"],
        "captures": preparation["captures"].as_array().unwrap_or(&empty).iter().take(2).map(|capture| json!({
            "capture_id": capture["capture_id"], "created_from_action": capture["created_from_action"],
            "audio_identity": capture["audio_identity"], "source_window": capture["source_window"],
        })).collect::<Vec<_>>(),
    })
}

fn failed(stage: &'static str, reason: impl fmt::Display, diagnostics: &Value) -> Box<dyn Error> {
    Box::new(ComparisonFailure {
        stage,
        reason: reason.to_string(),
        diagnostics: diagnostics.clone(),
    })
}

fn render_diagnostics(evidence: &RuntimeMixEvidence) -> Value {
    json!({
        "limiter": report_value(evidence.baseline.limiter),
        "pre_sample_count": evidence.pre_samples.len(),
        "baseline_sample_count": evidence.baseline.samples.len(),
        "pre_nonfinite_count": evidence.pre_samples.iter().filter(|sample| !sample.is_finite()).count(),
        "baseline_nonfinite_count": evidence.baseline.samples.iter().filter(|sample| !sample.is_finite()).count(),
    })
}

pub(super) fn compare_plan(
    plan: &RuntimeMixRenderPlan,
    frames: usize,
    case: Case,
) -> Result<Value> {
    compare_plan_with_history(plan, frames, case, None)
}

// Only V2 supplies the closed per-case historical controls. Tests may inject
// synthetic references here; neither CLI nor stdin exposes a reference override.
fn compare_plan_with_history(
    plan: &RuntimeMixRenderPlan,
    frames: usize,
    case: Case,
    historical: Option<&HistoricalControls>,
) -> Result<Value> {
    let primary = render_once(plan, frames, 128)?;
    let repeat = render_once(plan, frames, 128)?;
    let partition = render_once(plan, frames, 257)?;
    let mut diagnostics = json!({
        "primary_128": render_diagnostics(&primary),
        "repeat_128": render_diagnostics(&repeat),
        "partition_257": render_diagnostics(&partition),
    });
    validate_samples(&primary.pre_samples, frames)
        .and_then(|()| validate_samples(&primary.baseline.samples, frames))
        .map_err(|error| failed("input_alignment", error, &diagnostics))?;
    for alternative in [&repeat, &partition] {
        if !bits_equal(&primary.pre_samples, &alternative.pre_samples)
            || !bits_equal(&primary.baseline.samples, &alternative.baseline.samples)
            || primary.baseline.limiter != alternative.baseline.limiter
        {
            return Err(failed(
                "baseline_parity",
                "repeat/callback-partition parity failed",
                &diagnostics,
            ));
        }
    }
    let clean = limiter_calibration::compare(&primary.pre_samples)
        .map_err(|error| failed("clean_comparison", error, &diagnostics))?;
    for output in &clean {
        diagnostics[format!("clean_{}", policy_name(output.policy))] = report_value(output.limiter);
    }
    if !bits_equal(&clean[0].samples, &primary.baseline.samples)
        || clean[0].limiter != primary.baseline.limiter
    {
        return Err(failed(
            "baseline_api_parity",
            "policy A differs from the unchanged product baseline",
            &diagnostics,
        ));
    }
    for output in &clean {
        if output.limiter.applied
            || output.limiter.limited_sample_count != 0
            || output.limiter.pre.clip_count != 0
            || output.limiter.post.clip_count != 0
            || !bits_equal(&output.samples, &primary.pre_samples)
        {
            return Err(failed(
                "clean_gate",
                "clean interval clipped or a policy modified it",
                &diagnostics,
            ));
        }
    }
    if clean[0].limiter.post.active_samples == 0 {
        return Err(failed(
            "clean_silence",
            "clean interval has no active samples under the existing recipe gate",
            &diagnostics,
        ));
    }
    if case == Case::Sparse && clean[0].limiter.post.rms < 0.01 {
        return Err(failed(
            "clean_sparse_rms",
            "sparse ordinary mix is below the existing 0.01 RMS gate",
            &diagnostics,
        ));
    }
    // All clean output/metric validation also precedes any stress comparison.
    let clean_outputs = outputs_json(clean, frames)
        .map_err(|error| failed("clean_output_validation", error, &diagnostics))?;
    let stress_samples: Vec<f32> = primary
        .pre_samples
        .iter()
        .map(|sample| *sample * 2.0)
        .collect();
    let stress = limiter_calibration::compare(&stress_samples)
        .map_err(|error| failed("stress_comparison", error, &diagnostics))?;
    for output in &stress {
        diagnostics[format!("stress_2x_{}", policy_name(output.policy))] =
            report_value(output.limiter);
    }
    let actual_history = historical.map(|_| HistoricalControls {
        pre: pcm_hash(&primary.pre_samples),
        stress_2x: std::array::from_fn(|index| pcm_hash(&stress[index].samples)),
    });
    let stress_outputs = outputs_json(stress, frames)
        .map_err(|error| failed("stress_output_validation", error, &diagnostics))?;
    let mut conditions = vec![
        json!({"condition": "clean", "outputs": clean_outputs}),
        json!({"condition": "stress_2x", "outputs": stress_outputs}),
    ];
    if let Some(expected) = historical {
        // Clean A/B/C are already bit-identical to pre, so the pre hash binds
        // all four clean controls without rehashing or changing the V1 gates.
        let actual = actual_history
            .as_ref()
            .expect("V2 historical hashes computed");
        diagnostics["historical_v1_controls"] = expected.diagnostics(actual);
        if actual != expected {
            return Err(failed(
                "historical_controls",
                "clean/pre or 2x PCM differs from the frozen V1 report",
                &diagnostics,
            ));
        }
        // No 4x samples are constructed until every historical control passes.
        conditions.push(fourfold_condition(
            &primary.pre_samples,
            frames,
            &mut diagnostics,
        )?);
    }
    let mut result = json!({
        "sample_rate_hz": SAMPLE_RATE, "channels": CHANNELS, "frame_count": frames,
        "controls": {
            "repeat_128_bit_exact": true, "partition_257_bit_exact": true,
            "baseline_api_bit_exact": true,
        },
        "pre_samples": primary.pre_samples,
        "conditions": conditions,
    });
    if historical.is_some() {
        result["protocol_version"] = json!("v2");
    }
    Ok(result)
}

fn fourfold_condition(pre: &[f32], frames: usize, diagnostics: &mut Value) -> Result<Value> {
    // Fixed diagnostic overload only: never derive from limited or doubled PCM.
    let samples: Vec<f32> = pre.iter().map(|sample| *sample * 4.0_f32).collect();
    let outputs = limiter_calibration::compare(&samples)
        .map_err(|error| failed("stress_4x_comparison", error, diagnostics))?;
    for output in &outputs {
        diagnostics[format!("stress_4x_{}", policy_name(output.policy))] =
            report_value(output.limiter);
    }
    let outputs = outputs_json(outputs, frames)
        .map_err(|error| failed("stress_4x_output_validation", error, diagnostics))?;
    Ok(json!({"condition": "stress_4x", "outputs": outputs}))
}

fn outputs_json(outputs: [PolicyOutput; 3], frames: usize) -> Result<Vec<Value>> {
    outputs.into_iter().map(|output| {
        validate_samples(&output.samples, frames)?;
        let policy = policy_name(output.policy);
        Ok(json!({"policy": policy, "samples": output.samples, "limiter": report_json(output.limiter)?}))
    }).collect()
}

fn policy_name(policy: Policy) -> &'static str {
    match policy {
        Policy::BaselineA => "A",
        Policy::LaterKneeB => "B",
        Policy::LowerCeilingC => "C",
    }
}

fn report_json(report: MasterBusLimiterReport) -> Result<Value> {
    if !report.threshold.is_finite() || !report.ceiling.is_finite() {
        return Err("calibration limiter bounds are nonfinite".into());
    }
    metrics_json(report.pre)?;
    metrics_json(report.post)?;
    Ok(report_value(report))
}

// Failure evidence retains the actual available report, including null for any
// nonfinite JSON number. Only report_json's finite check may emit success data.
fn report_value(report: MasterBusLimiterReport) -> Value {
    json!({
        "threshold_bits": report.threshold.to_bits(), "ceiling_bits": report.ceiling.to_bits(),
        "limited_sample_count": report.limited_sample_count, "applied": report.applied,
        "pre": metrics_value(report.pre), "post": metrics_value(report.post),
    })
}

fn metrics_json(metrics: OfflineAudioMetrics) -> Result<Value> {
    if [
        metrics.peak_abs,
        metrics.rms,
        metrics.dc_offset,
        metrics.headroom_to_full_scale,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err("calibration limiter metrics are nonfinite".into());
    }
    Ok(metrics_value(metrics))
}

fn metrics_value(metrics: OfflineAudioMetrics) -> Value {
    json!({
        "active_samples": metrics.active_samples,
        "peak_abs": metrics.peak_abs, "rms": metrics.rms, "dc_offset": metrics.dc_offset,
        "clip_count": metrics.clip_count, "near_clip_count": metrics.near_clip_count,
        "headroom_to_full_scale": metrics.headroom_to_full_scale,
    })
}

#[cfg(test)]
mod tests;
