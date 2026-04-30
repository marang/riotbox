use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use riotbox_audio::listening_manifest::validate_manifest_envelope;
use serde_json::Value;

#[path = "observer_audio_correlate/lane_recipe_output.rs"]
mod lane_recipe_output;
#[path = "observer_audio_correlate/observer_validation.rs"]
mod observer_validation;

use lane_recipe_output::{
    LaneRecipeCaseEvidence, collect_lane_recipe_cases, lane_recipe_metric_failures,
};
use observer_validation::validate_user_session_observer_events;

const STRICT_OUTPUT_METRIC_FLOOR: f64 = 1.0e-6;
const SUMMARY_SCHEMA: &str = "riotbox.observer_audio_summary.v1";
const SUMMARY_SCHEMA_VERSION: u32 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let observer_events = read_observer_events(&args.observer_path)?;
    if args.require_evidence {
        validate_user_session_observer_events(&observer_events)?;
        validate_manifest_envelope_file(&args.manifest_path)?;
    }

    let summary = build_summary_from_events(&observer_events, &args.manifest_path)?;
    let output = if args.json_output {
        render_json(&summary)?
    } else {
        render_markdown(&summary)
    };
    if args.require_evidence {
        validate_required_evidence(&summary)?;
    }

    match args.output_path {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, output)?;
        }
        None => print!("{output}"),
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Args {
    observer_path: PathBuf,
    manifest_path: PathBuf,
    output_path: Option<PathBuf>,
    require_evidence: bool,
    json_output: bool,
    show_help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut observer_path = None;
        let mut manifest_path = None;
        let mut output_path = None;
        let mut require_evidence = false;
        let mut json_output = false;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--require-evidence" => require_evidence = true,
                "--json" => json_output = true,
                "--observer" => {
                    observer_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--observer requires a path".to_string())?,
                    ));
                }
                "--manifest" => {
                    manifest_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--manifest requires a path".to_string())?,
                    ));
                }
                "--output" => {
                    output_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--output requires a path".to_string())?,
                    ));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if show_help {
            return Ok(Self {
                observer_path: PathBuf::new(),
                manifest_path: PathBuf::new(),
                output_path,
                require_evidence,
                json_output,
                show_help,
            });
        }

        Ok(Self {
            observer_path: observer_path.ok_or_else(|| "--observer is required".to_string())?,
            manifest_path: manifest_path.ok_or_else(|| "--manifest is required".to_string())?,
            output_path,
            require_evidence,
            json_output,
            show_help,
        })
    }
}

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
    lane_recipe_cases: Vec<LaneRecipeCaseEvidence>,
}

fn print_help() {
    println!(
        "Usage: observer_audio_correlate --observer PATH --manifest PATH [--output PATH] [--json]\n\
         \n\
         Reads a riotbox-app observer NDJSON file and an audio QA manifest.json,\n\
         then emits a compact Markdown correlation summary, or JSON with --json.\n\
         This is local-first QA bookkeeping, not a live host-session monitor.\n\
         \n\
         Pass --require-evidence to fail when the observer stream is malformed,\n\
         the manifest envelope is unstable, committed control-path evidence is\n\
         missing, or passing output-path manifest evidence is missing."
    );
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
        lane_recipe_cases: collect_lane_recipe_cases(&manifest),
    })
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

fn render_markdown(summary: &CorrelationSummary) -> String {
    format!(
        "# Observer / Audio QA Correlation Summary\n\n\
         ## Control Path\n\n\
         - Observer schema: `{}`\n\
         - Launch mode: `{}`\n\
         - Audio runtime status: `{}`\n\
         - Key outcomes: `{}`\n\
         - First commit: `{}`\n\n\
         - Commit count: `{}`\n\
         - Commit boundaries: `{}`\n\n\
         ## Output Path\n\n\
         - Pack id: `{}`\n\
         - Manifest result: `{}`\n\
         - Artifact count: `{}`\n\
         - Full mix RMS: `{}`\n\
         - Full mix low-band RMS: `{}`\n\
         - MC-202 question/answer delta RMS: `{}`\n\n\
         - W-30 candidate RMS: `{}`\n\
         - W-30 candidate active-sample ratio: `{}`\n\
         - W-30 RMS delta: `{}`\n\n\
         ## Correlation Verdict\n\n\
         - Control path present: `{}`\n\
         - Output path present: `{}`\n\
         - Output path issues: `{}`\n\
         - Needs human listening: `yes`\n",
        summary.observer_schema,
        summary.launch_mode,
        summary.audio_runtime_status,
        if summary.key_outcomes.is_empty() {
            "none".to_string()
        } else {
            summary.key_outcomes.join(", ")
        },
        summary.first_commit,
        summary.commit_count,
        if summary.commit_boundaries.is_empty() {
            "none".to_string()
        } else {
            summary.commit_boundaries.join(", ")
        },
        summary.pack_id,
        summary.manifest_result,
        summary.artifact_count,
        format_optional_f64(summary.full_mix_rms),
        format_optional_f64(summary.full_mix_low_band_rms),
        format_optional_f64(summary.mc202_question_answer_delta_rms),
        format_optional_f64(summary.w30_candidate_rms),
        format_optional_f64(summary.w30_candidate_active_sample_ratio),
        format_optional_f64(summary.w30_rms_delta),
        yes_no(control_path_present(summary)),
        yes_no(output_path_present(summary)),
        format_output_path_issues(summary)
    )
}

fn render_json(summary: &CorrelationSummary) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": SUMMARY_SCHEMA,
        "schema_version": SUMMARY_SCHEMA_VERSION,
        "control_path": {
            "present": control_path_present(summary),
            "observer_schema": &summary.observer_schema,
            "launch_mode": &summary.launch_mode,
            "audio_runtime_status": &summary.audio_runtime_status,
            "key_outcomes": &summary.key_outcomes,
            "first_commit": &summary.first_commit,
            "commit_count": summary.commit_count,
            "commit_boundaries": &summary.commit_boundaries,
        },
        "output_path": {
            "present": output_path_present(summary),
            "issues": output_path_evidence_failures(summary),
            "pack_id": &summary.pack_id,
            "manifest_result": &summary.manifest_result,
            "artifact_count": summary.artifact_count,
            "metrics": {
                "full_mix_rms": summary.full_mix_rms,
                "full_mix_low_band_rms": summary.full_mix_low_band_rms,
                "mc202_question_answer_delta_rms": summary.mc202_question_answer_delta_rms,
                "w30_candidate_rms": summary.w30_candidate_rms,
                "w30_candidate_active_sample_ratio": summary.w30_candidate_active_sample_ratio,
                "w30_rms_delta": summary.w30_rms_delta,
            },
        },
        "needs_human_listening": true,
    }))
    .map(|json| json + "\n")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| format!("{value:.6}"))
}

fn format_output_path_issues(summary: &CorrelationSummary) -> String {
    let failures = output_path_evidence_failures(summary);
    if failures.is_empty() {
        "none".to_string()
    } else {
        failures.join(", ")
    }
}

fn control_path_present(summary: &CorrelationSummary) -> bool {
    summary.first_commit != "none"
}

fn output_path_present(summary: &CorrelationSummary) -> bool {
    output_path_evidence_failures(summary).is_empty()
}

fn output_path_evidence_failures(summary: &CorrelationSummary) -> Vec<String> {
    let mut failures = Vec::new();

    if summary.manifest_result != "pass" {
        failures.push(format!("manifest_result={}", summary.manifest_result));
    }

    let metric_failures = if summary.pack_id == "w30-preview-smoke" {
        w30_source_preview_metric_failures(summary)
    } else if summary.pack_id == "lane-recipe-listening-pack" {
        lane_recipe_metric_failures(&summary.lane_recipe_cases, STRICT_OUTPUT_METRIC_FLOOR)
    } else {
        feral_grid_metric_failures(summary)
    };
    failures.extend(metric_failures);

    failures
}

fn feral_grid_metric_failures(summary: &CorrelationSummary) -> Vec<String> {
    metric_failures([
        ("full_mix_rms", summary.full_mix_rms),
        ("full_mix_low_band_rms", summary.full_mix_low_band_rms),
        (
            "mc202_question_answer_delta_rms",
            summary.mc202_question_answer_delta_rms,
        ),
    ])
}

fn w30_source_preview_metric_failures(summary: &CorrelationSummary) -> Vec<String> {
    metric_failures([
        ("w30_candidate_rms", summary.w30_candidate_rms),
        (
            "w30_candidate_active_sample_ratio",
            summary.w30_candidate_active_sample_ratio,
        ),
        ("w30_rms_delta", summary.w30_rms_delta),
    ])
}

fn metric_failures(metrics: impl IntoIterator<Item = (&'static str, Option<f64>)>) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, metric) in metrics {
        if let Some(failure) = output_metric_failure(name, metric) {
            failures.push(failure);
        }
    }

    failures
}

fn output_metric_failure(name: &str, metric: Option<f64>) -> Option<String> {
    match metric {
        Some(_) if metric_is_noncollapsed(metric) => None,
        Some(value) => Some(format!(
            "{name}={value:.6} <= floor {STRICT_OUTPUT_METRIC_FLOOR:.6}"
        )),
        None => Some(format!("{name}=missing")),
    }
}

fn metric_is_noncollapsed(metric: Option<f64>) -> bool {
    metric.is_some_and(|value| value > STRICT_OUTPUT_METRIC_FLOOR)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn validate_required_evidence(summary: &CorrelationSummary) -> Result<(), io::Error> {
    if !control_path_present(summary) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "observer/audio correlation is missing committed control-path evidence",
        ));
    }

    let output_failures = output_path_evidence_failures(summary);
    if !output_failures.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "observer/audio correlation is missing passing output-path manifest evidence: {}",
                output_failures.join(", ")
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "observer_audio_correlate/lane_recipe_tests.rs"]
mod lane_recipe_tests;
#[cfg(test)]
#[path = "observer_audio_correlate/tests.rs"]
mod tests;
