use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    if args.show_help {
        print_help();
        return Ok(());
    }

    let summary = build_summary(&args.observer_path, &args.manifest_path)?;
    let markdown = render_markdown(&summary);

    match args.output_path {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, markdown)?;
        }
        None => print!("{markdown}"),
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct Args {
    observer_path: PathBuf,
    manifest_path: PathBuf,
    output_path: Option<PathBuf>,
    show_help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut observer_path = None;
        let mut manifest_path = None;
        let mut output_path = None;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
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
                show_help,
            });
        }

        Ok(Self {
            observer_path: observer_path.ok_or_else(|| "--observer is required".to_string())?,
            manifest_path: manifest_path.ok_or_else(|| "--manifest is required".to_string())?,
            output_path,
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
    pack_id: String,
    manifest_result: String,
    artifact_count: usize,
    full_mix_rms: Option<f64>,
    full_mix_low_band_rms: Option<f64>,
    mc202_question_answer_delta_rms: Option<f64>,
}

fn print_help() {
    println!(
        "Usage: observer_audio_correlate --observer PATH --manifest PATH [--output PATH]\n\
         \n\
         Reads a riotbox-app observer NDJSON file and an audio QA manifest.json,\n\
         then emits a compact Markdown correlation summary. This is local-first\n\
         QA bookkeeping, not a live host-session monitor."
    );
}

fn build_summary(
    observer_path: &Path,
    manifest_path: &Path,
) -> Result<CorrelationSummary, Box<dyn std::error::Error>> {
    let observer_events = read_observer_events(observer_path)?;
    let manifest: Value = serde_json::from_str(&fs::read_to_string(manifest_path)?)?;

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

fn render_markdown(summary: &CorrelationSummary) -> String {
    format!(
        "# Observer / Audio QA Correlation Summary\n\n\
         ## Control Path\n\n\
         - Observer schema: `{}`\n\
         - Launch mode: `{}`\n\
         - Audio runtime status: `{}`\n\
         - Key outcomes: `{}`\n\
         - First commit: `{}`\n\n\
         ## Output Path\n\n\
         - Pack id: `{}`\n\
         - Manifest result: `{}`\n\
         - Artifact count: `{}`\n\
         - Full mix RMS: `{}`\n\
         - Full mix low-band RMS: `{}`\n\
         - MC-202 question/answer delta RMS: `{}`\n\n\
         ## Correlation Verdict\n\n\
         - Control path present: `{}`\n\
         - Output path present: `{}`\n\
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
        summary.pack_id,
        summary.manifest_result,
        summary.artifact_count,
        format_optional_f64(summary.full_mix_rms),
        format_optional_f64(summary.full_mix_low_band_rms),
        format_optional_f64(summary.mc202_question_answer_delta_rms),
        if summary.first_commit == "none" {
            "no"
        } else {
            "yes"
        },
        if summary.manifest_result == "pass" {
            "yes"
        } else {
            "no"
        }
    )
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(|| "unknown".to_string(), |value| format!("{value:.6}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_paths_and_output() {
        let parsed = Args::parse([
            "--observer".to_string(),
            "events.ndjson".to_string(),
            "--manifest".to_string(),
            "manifest.json".to_string(),
            "--output".to_string(),
            "summary.md".to_string(),
        ])
        .expect("parse args");

        assert_eq!(parsed.observer_path, PathBuf::from("events.ndjson"));
        assert_eq!(parsed.manifest_path, PathBuf::from("manifest.json"));
        assert_eq!(parsed.output_path, Some(PathBuf::from("summary.md")));
    }

    #[test]
    fn rejects_missing_required_paths() {
        assert!(Args::parse(Vec::<String>::new()).is_err());
    }

    #[test]
    fn accepts_help_without_required_paths() {
        let parsed = Args::parse(["--help".to_string()]).expect("parse help");

        assert!(parsed.show_help);
    }

    #[test]
    fn summarizes_synthetic_observer_and_manifest() {
        let temp = tempfile::tempdir().expect("tempdir");
        let observer_path = temp.path().join("events.ndjson");
        let manifest_path = temp.path().join("manifest.json");
        fs::write(&observer_path, synthetic_observer()).expect("write observer");
        fs::write(&manifest_path, synthetic_manifest()).expect("write manifest");

        let summary = build_summary(&observer_path, &manifest_path).expect("summary");
        let markdown = render_markdown(&summary);

        assert_eq!(summary.observer_schema, "riotbox.user_session_observer.v1");
        assert_eq!(summary.launch_mode, "ingest");
        assert_eq!(summary.audio_runtime_status, "started");
        assert_eq!(
            summary.key_outcomes,
            ["space -> transport started", "f -> queued"]
        );
        assert!(summary.first_commit.contains("action 2 at NextBar"));
        assert_eq!(summary.pack_id, "feral-grid-demo");
        assert_eq!(summary.manifest_result, "pass");
        assert_eq!(summary.artifact_count, 6);
        assert_eq!(summary.full_mix_rms, Some(0.1));
        assert!(markdown.contains("Control path present: `yes`"));
        assert!(markdown.contains("Output path present: `yes`"));
    }

    fn synthetic_observer() -> String {
        [
            r#"{"event":"observer_started","schema":"riotbox.user_session_observer.v1","launch":{"mode":"ingest"}}"#,
            r#"{"event":"audio_runtime","status":"started"}"#,
            r#"{"event":"key_outcome","key":"space","outcome":"transport started"}"#,
            r#"{"event":"key_outcome","key":"f","outcome":"queued"}"#,
            r#"{"event":"transport_commit","committed":[{"action_id":2,"boundary":"NextBar","beat_index":8,"bar_index":2,"phrase_index":0,"commit_sequence":1}]}"#,
        ]
        .join("\n")
            + "\n"
    }

    fn synthetic_manifest() -> String {
        r#"{
  "pack_id": "feral-grid-demo",
  "result": "pass",
  "artifacts": [{}, {}, {}, {}, {}, {}],
  "metrics": {
    "full_grid_mix": {
      "signal": { "rms": 0.1 },
      "low_band": { "rms": 0.08 }
    },
    "mc202_question_answer_delta": { "rms": 0.01 }
  }
}"#
        .to_string()
    }
}
