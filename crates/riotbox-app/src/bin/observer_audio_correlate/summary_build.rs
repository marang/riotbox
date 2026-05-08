#[derive(Debug, PartialEq)]
struct CorrelationSummary {
    observer_schema: String,
    launch_mode: String,
    audio_runtime_status: String,
    key_outcomes: Vec<String>,
    first_commit: String,
    commit_count: usize,
    commit_boundaries: Vec<String>,
    pack_id: String,
    manifest_result: String,
    artifact_count: usize,
    full_mix_rms: Option<f64>,
    full_mix_low_band_rms: Option<f64>,
    mc202_question_answer_delta_rms: Option<f64>,
    w30_candidate_rms: Option<f64>,
    w30_candidate_active_sample_ratio: Option<f64>,
    w30_rms_delta: Option<f64>,
    source_timing: Option<SourceTimingEvidence>,
    source_timing_malformed: bool,
    source_grid_output_drift: Option<SourceGridOutputDriftEvidence>,
    source_grid_output_drift_malformed: bool,
    lane_recipe_cases: Vec<LaneRecipeCaseEvidence>,
}

#[derive(Debug, PartialEq)]
struct SourceTimingEvidence {
    readiness: String,
    requires_manual_confirm: bool,
    beat_status: String,
    downbeat_status: String,
    primary_downbeat_offset_beats: Option<u64>,
    confidence_result: String,
    drift_status: String,
    phrase_status: String,
    alternate_evidence_count: u64,
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

    let (source_grid_output_drift, source_grid_output_drift_malformed) =
        collect_source_grid_output_drift(&manifest);
    let (source_timing, source_timing_malformed) = collect_source_timing(&manifest);

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
        pack_id: manifest["pack_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        manifest_result: manifest["result"].as_str().unwrap_or("unknown").to_string(),
        artifact_count: manifest["artifacts"].as_array().map_or(0, Vec::len),
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
        source_grid_output_drift,
        source_grid_output_drift_malformed,
        lane_recipe_cases: collect_lane_recipe_cases(&manifest),
    })
}

fn collect_source_timing(manifest: &Value) -> (Option<SourceTimingEvidence>, bool) {
    let Some(source_timing) = manifest.get("source_timing") else {
        return (None, false);
    };
    if !source_timing.is_object() {
        return (None, true);
    }

    let evidence = SourceTimingEvidence {
        readiness: match source_timing_string(source_timing, "readiness") {
            Some(value) => value,
            None => return (None, true),
        },
        requires_manual_confirm: match source_timing["requires_manual_confirm"].as_bool() {
            Some(value) => value,
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
    };

    (Some(evidence), false)
}

fn source_timing_string(source_timing: &Value, field: &str) -> Option<String> {
    source_timing[field]
        .as_str()
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
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
