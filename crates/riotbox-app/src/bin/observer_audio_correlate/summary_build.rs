#[derive(Debug, PartialEq)]
struct CorrelationSummary {
    observer_schema: String,
    launch_mode: String,
    audio_runtime_status: String,
    key_outcomes: Vec<String>,
    first_commit: String,
    commit_count: usize,
    commit_boundaries: Vec<String>,
    observer_source_timing: Option<ObserverSourceTimingReadiness>,
    observer_source_timing_malformed: bool,
    pack_id: String,
    manifest_result: String,
    artifact_count: usize,
    grid_bpm_source: String,
    grid_bpm_decision_reason: String,
    source_timing_bpm_delta: Option<f64>,
    full_mix_rms: Option<f64>,
    full_mix_low_band_rms: Option<f64>,
    mc202_question_answer_delta_rms: Option<f64>,
    w30_candidate_rms: Option<f64>,
    w30_candidate_active_sample_ratio: Option<f64>,
    w30_rms_delta: Option<f64>,
    source_timing: Option<SourceTimingEvidence>,
    source_timing_malformed: bool,
    source_timing_alignment: Option<SourceTimingAlignmentEvidence>,
    source_grid_output_drift: Option<SourceGridOutputDriftEvidence>,
    source_grid_output_drift_malformed: bool,
    lane_recipe_cases: Vec<LaneRecipeCaseEvidence>,
}

#[derive(Debug, PartialEq)]
struct ObserverSourceTimingReadiness {
    source_id: String,
    cue: String,
    bpm_estimate: Option<f64>,
    bpm_confidence: f64,
    quality: String,
    degraded_policy: String,
    primary_hypothesis_id: Option<String>,
    hypothesis_count: u64,
    primary_warning_code: Option<String>,
    warning_codes: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct SourceTimingEvidence {
    source_id: String,
    policy_profile: String,
    readiness: String,
    requires_manual_confirm: bool,
    primary_bpm: Option<f64>,
    bpm_agrees_with_grid: Option<bool>,
    beat_status: String,
    downbeat_status: String,
    primary_downbeat_offset_beats: Option<u64>,
    confidence_result: String,
    drift_status: String,
    phrase_status: String,
    alternate_evidence_count: u64,
    warning_codes: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct SourceGridOutputDriftEvidence {
    hit_ratio: f64,
    max_peak_offset_ms: f64,
    max_allowed_peak_offset_ms: f64,
}

#[cfg(test)]
fn build_summary(
    observer_path: &Path,
    manifest_path: &Path,
) -> Result<CorrelationSummary, Box<dyn std::error::Error>> {
    let observer_events = read_observer_events(observer_path)?;
    build_summary_from_events(&observer_events, manifest_path)
}

fn build_summary_from_events(
    observer_events: &[Value],
    manifest_path: &Path,
) -> Result<CorrelationSummary, Box<dyn std::error::Error>> {
    let manifest = read_manifest(manifest_path)?;

    let launch = observer_events
        .iter()
        .find(|event| event["event"] == "observer_started");
    let audio_runtime = observer_events
        .iter()
        .rev()
        .find(|event| event["event"] == "audio_runtime");
    let key_outcomes = observer_events
        .iter()
        .filter(|event| event["event"] == "key_outcome")
        .map(|event| {
            format!(
                "{} -> {}",
                string_field(event, "key"),
                string_field(event, "outcome")
            )
        })
        .collect::<Vec<_>>();
    let first_commit = observer_events
        .iter()
        .find(|event| event["event"] == "transport_commit")
        .and_then(format_first_commit)
        .unwrap_or_else(|| "none".to_string());
    let (commit_count, commit_boundaries) = collect_commit_summary(observer_events);
    let (observer_source_timing, observer_source_timing_malformed) =
        collect_observer_source_timing(observer_events);

    let (source_grid_output_drift, source_grid_output_drift_malformed) =
        collect_source_grid_output_drift(&manifest);
    let (source_timing, source_timing_malformed) = collect_source_timing(&manifest);
    let source_timing_alignment = collect_source_timing_alignment(
        observer_source_timing.as_ref(),
        source_timing.as_ref(),
        observer_source_timing_malformed,
        source_timing_malformed,
    );

    Ok(CorrelationSummary {
        observer_schema: launch
            .and_then(|event| event["schema"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        launch_mode: launch
            .and_then(|event| event["launch"]["mode"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        audio_runtime_status: audio_runtime
            .and_then(|event| event["status"].as_str())
            .unwrap_or("unknown")
            .to_string(),
        key_outcomes,
        first_commit,
        commit_count,
        commit_boundaries,
        observer_source_timing,
        observer_source_timing_malformed,
        pack_id: manifest["pack_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        manifest_result: manifest["result"].as_str().unwrap_or("unknown").to_string(),
        artifact_count: manifest["artifacts"].as_array().map_or(0, Vec::len),
        grid_bpm_source: manifest["grid_bpm_source"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        grid_bpm_decision_reason: manifest["grid_bpm_decision_reason"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        source_timing_bpm_delta: manifest["source_timing_bpm_delta"].as_f64(),
        full_mix_rms: manifest["metrics"]["full_grid_mix"]["signal"]["rms"].as_f64(),
        full_mix_low_band_rms: manifest["metrics"]["full_grid_mix"]["low_band"]["rms"].as_f64(),
        mc202_question_answer_delta_rms: manifest["metrics"]["mc202_question_answer_delta"]["rms"]
            .as_f64(),
        w30_candidate_rms: manifest["metrics"]["candidate"]["rms"].as_f64(),
        w30_candidate_active_sample_ratio: manifest["metrics"]["candidate"]["active_sample_ratio"]
            .as_f64(),
        w30_rms_delta: manifest["metrics"]["deltas"]["rms"].as_f64(),
        source_timing,
        source_timing_malformed,
        source_timing_alignment,
        source_grid_output_drift,
        source_grid_output_drift_malformed,
        lane_recipe_cases: collect_lane_recipe_cases(&manifest),
    })
}

fn collect_observer_source_timing(
    events: &[Value],
) -> (Option<ObserverSourceTimingReadiness>, bool) {
    let Some(source_timing) = events
        .iter()
        .filter_map(|event| event.get("snapshot"))
        .find_map(|snapshot| snapshot.get("source_timing"))
    else {
        return (None, false);
    };
    if source_timing.is_null() {
        return (None, false);
    }
    if !source_timing.is_object() {
        return (None, true);
    }

    let cue = match non_empty_string(source_timing, "cue") {
        Some(value) => value,
        None => return (None, true),
    };
    let degraded_policy = match non_empty_string(source_timing, "degraded_policy") {
        Some(value) => value,
        None => return (None, true),
    };
    let Some(expected_cue) = observer_source_timing_policy_cue(&degraded_policy) else {
        return (None, true);
    };
    if cue != expected_cue {
        return (None, true);
    }

    let evidence = ObserverSourceTimingReadiness {
        source_id: match non_empty_string(source_timing, "source_id") {
            Some(value) => value,
            None => return (None, true),
        },
        cue,
        bpm_estimate: match source_timing.get("bpm_estimate") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_f64() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        bpm_confidence: match source_timing["bpm_confidence"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        quality: match non_empty_string(source_timing, "quality") {
            Some(value) => value,
            None => return (None, true),
        },
        degraded_policy,
        primary_hypothesis_id: match source_timing.get("primary_hypothesis_id") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_str().filter(|value| !value.is_empty()) {
                Some(value) => Some(value.to_string()),
                None => return (None, true),
            },
            None => return (None, true),
        },
        hypothesis_count: match source_timing["hypothesis_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        primary_warning_code: match source_timing.get("primary_warning_code") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_str().filter(|value| !value.is_empty()) {
                Some(value) => Some(value.to_string()),
                None => return (None, true),
            },
            None => return (None, true),
        },
        warning_codes: match string_list(source_timing, "warning_codes") {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

fn observer_source_timing_policy_cue(policy: &str) -> Option<&'static str> {
    match policy {
        "locked" | "manual_confirm" | "cautious" | "fallback_grid" | "disabled" | "unknown" => {
            Some(riotbox_app::source_timing_cues::source_timing_policy_cue_label(
                policy,
            ))
        }
        _ => None,
    }
}

fn collect_source_timing(manifest: &Value) -> (Option<SourceTimingEvidence>, bool) {
    let Some(source_timing) = manifest.get("source_timing") else {
        return (None, false);
    };
    if !source_timing.is_object() {
        return (None, true);
    }

    let evidence = SourceTimingEvidence {
        source_id: match source_timing_string(source_timing, "source_id") {
            Some(value) => value,
            None => return (None, true),
        },
        policy_profile: match source_timing_string(source_timing, "policy_profile") {
            Some(value) => value,
            None => return (None, true),
        },
        readiness: match source_timing_string(source_timing, "readiness") {
            Some(value) => value,
            None => return (None, true),
        },
        requires_manual_confirm: match source_timing["requires_manual_confirm"].as_bool() {
            Some(value) => value,
            None => return (None, true),
        },
        primary_bpm: match source_timing.get("primary_bpm") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_f64() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        bpm_agrees_with_grid: match source_timing.get("bpm_agrees_with_grid") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_bool() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        beat_status: match source_timing_string(source_timing, "beat_status") {
            Some(value) => value,
            None => return (None, true),
        },
        downbeat_status: match source_timing_string(source_timing, "downbeat_status") {
            Some(value) => value,
            None => return (None, true),
        },
        primary_downbeat_offset_beats: match source_timing.get("primary_downbeat_offset_beats") {
            Some(value) if value.is_null() => None,
            Some(value) => match value.as_u64() {
                Some(value) => Some(value),
                None => return (None, true),
            },
            None => return (None, true),
        },
        confidence_result: match source_timing_string(source_timing, "confidence_result") {
            Some(value) => value,
            None => return (None, true),
        },
        drift_status: match source_timing_string(source_timing, "drift_status") {
            Some(value) => value,
            None => return (None, true),
        },
        phrase_status: match source_timing_string(source_timing, "phrase_status") {
            Some(value) => value,
            None => return (None, true),
        },
        alternate_evidence_count: match source_timing["alternate_evidence_count"].as_u64() {
            Some(value) => value,
            None => return (None, true),
        },
        warning_codes: match string_list(source_timing, "warning_codes") {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

fn source_timing_string(source_timing: &Value, field: &str) -> Option<String> {
    non_empty_string(source_timing, field)
}

fn non_empty_string(value: &Value, field: &str) -> Option<String> {
    value[field]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn string_list(value: &Value, field: &str) -> Option<Vec<String>> {
    value[field]
        .as_array()?
        .iter()
        .map(|item| item.as_str().filter(|value| !value.is_empty()).map(str::to_string))
        .collect()
}

fn collect_source_grid_output_drift(manifest: &Value) -> (Option<SourceGridOutputDriftEvidence>, bool) {
    let Some(metrics) = manifest.get("metrics").and_then(Value::as_object) else {
        return (None, false);
    };
    let Some(metric) = metrics.get("source_grid_output_drift") else {
        return (None, false);
    };

    let evidence = SourceGridOutputDriftEvidence {
        hit_ratio: match metric["hit_ratio"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_peak_offset_ms: match metric["max_peak_offset_ms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
        max_allowed_peak_offset_ms: match metric["max_allowed_peak_offset_ms"].as_f64() {
            Some(value) => value,
            None => return (None, true),
        },
    };

    (Some(evidence), false)
}

fn read_observer_events(path: &Path) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    fs::read_to_string(path)?
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str::<Value>(line).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid observer JSON on line {}: {error}", index + 1),
                )
                .into()
            })
        })
        .collect()
}

fn validate_manifest_envelope_file(path: &Path) -> Result<(), io::Error> {
    let manifest = read_manifest(path)?;
    validate_manifest_envelope(&manifest).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid audio QA manifest envelope: {error}"),
        )
    })
}

fn read_manifest(path: &Path) -> Result<Value, io::Error> {
    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid audio QA manifest JSON: {error}"),
        )
    })
}

fn string_field(event: &Value, field: &str) -> String {
    event[field].as_str().unwrap_or("unknown").to_string()
}

fn format_first_commit(event: &Value) -> Option<String> {
    let commit = event["committed"].as_array()?.first()?;
    Some(format!(
        "action {} at {} beat {} bar {} phrase {} sequence {}",
        commit["action_id"].as_u64().unwrap_or_default(),
        commit["boundary"].as_str().unwrap_or("unknown"),
        commit["beat_index"].as_u64().unwrap_or_default(),
        commit["bar_index"].as_u64().unwrap_or_default(),
        commit["phrase_index"].as_u64().unwrap_or_default(),
        commit["commit_sequence"].as_u64().unwrap_or_default()
    ))
}

fn collect_commit_summary(events: &[Value]) -> (usize, Vec<String>) {
    let mut count = 0;
    let mut boundaries = Vec::new();

    for commit in events
        .iter()
        .filter(|event| event["event"] == "transport_commit")
        .filter_map(|event| event["committed"].as_array())
        .flatten()
    {
        count += 1;
        let boundary = commit["boundary"].as_str().unwrap_or("unknown").to_string();
        if !boundaries.contains(&boundary) {
            boundaries.push(boundary);
        }
    }

    (count, boundaries)
}
