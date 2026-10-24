//! Preparation-only generated control. Never constructs an audio runtime.
//! Core/App own the actual recipe, files, identity, restore and recording plans.
mod contract;
mod windows;

use std::{error::Error, fs, io::Write, path::Path, process::Command};

use riotbox_app::jam_app::JamAppState;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{live_flow::confirmed_source_timing, ready_prefix};
use contract::Contract;
use windows::WindowReport;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PreparationResult {
    Failed,
    Prepared,
}

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    schema_version: u32,
    classification: &'static str,
    pattern_origin: &'static str,
    quality_proof: bool,
    source_intelligence_qualified: bool,
    host_execution_allowed: bool,
    live_controller_executed: bool,
    result: PreparationResult,
    contract_sha256: String,
    identity: Option<Value>,
    windows: Vec<WindowReport>,
    failure_reason: Option<String>,
}

impl Report {
    fn new() -> Self {
        Self {
            schema: "riotbox.generated_live_master_load_preparation.v1",
            schema_version: 1,
            classification: "generated_control_preparation_only",
            pattern_origin: "fixture",
            quality_proof: false,
            source_intelligence_qualified: false,
            host_execution_allowed: false,
            live_controller_executed: false,
            result: PreparationResult::Failed,
            contract_sha256: contract::digest(),
            identity: None,
            windows: Vec::new(),
            failure_reason: None,
        }
    }
}

pub(super) fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let output = admit_args(args)?;
    let contract = Contract::load()?;
    let root = create_owner(output)?;
    let mut report = Report::new();
    let preparation = prepare(&root, &contract, &mut report);
    persist_result(&root, &mut report, &preparation)?;
    serde_json::to_writer(
        std::io::stdout(),
        &json!({
            "mode": "generated_live_master_load_preparation_v1",
            "report_path": root.join("preparation.json"),
            "result": report.result,
            "host_execution_allowed": false,
            "quality_proof": false,
        }),
    )?;
    println!();
    preparation
}

fn create_owner(output: &Path) -> Result<std::path::PathBuf, Box<dyn Error>> {
    if !cfg!(target_os = "linux") {
        return Err("generated load preparation is scoped to Linux".into());
    }
    if !output.is_absolute() {
        return Err("generated load preparation requires an absolute fresh output path".into());
    }
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    // Exclusive ownership, not cleanup/reuse of an earlier phase or user path.
    builder.create(output)?;
    Ok(fs::canonicalize(output)?)
}

fn persist_result(
    root: &Path,
    report: &mut Report,
    preparation: &Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    match preparation {
        Ok(()) => report.result = PreparationResult::Prepared,
        Err(error) => report.failure_reason = Some(error.to_string()),
    }
    // Retain all computed metrics before enforcing aggregate acceptance.
    let report_path = root.join("preparation.json");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&report_path)?;
    serde_json::to_writer_pretty(&mut file, &report)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn admit_args(args: &[String]) -> Result<&Path, Box<dyn Error>> {
    match args {
        [mode, flag, output]
            if mode == "--prepare-generated-live-master-load-v1" && flag == "--output" =>
        {
            Ok(Path::new(output))
        }
        _ => Err("preparation accepts only --prepare-generated-live-master-load-v1 --output <absolute fresh directory>; no source, profile, host or render options".into()),
    }
}

fn prepare(root: &Path, contract: &Contract, report: &mut Report) -> Result<(), Box<dyn Error>> {
    let generator = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/write_diverse_test_source_wavs.py");
    // Execute the exact retained producer bytes, not a mutable checkout path.
    let generator_bytes = fs::read(&generator)?;
    let generator_snapshot = root.join("generator.py");
    let mut snapshot = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&generator_snapshot)?;
    snapshot.write_all(&generator_bytes)?;
    snapshot.flush()?;
    report.identity = Some(json!({
        "generator_repository_path": generator,
        "generator_snapshot_path": generator_snapshot,
        "generator_sha256": hex::encode(Sha256::digest(&generator_bytes)),
        // Linux's live executable inode, even if a later build replaces its path.
        "producer_binary_sha256": file_hash(Path::new("/proc/self/exe"))?,
    }));
    let generated = root.join("generated-source");
    let generation = Command::new("python3")
        .arg(&generator_snapshot)
        .arg("--output")
        .arg(&generated)
        .arg("--seconds")
        .arg(contract.source_seconds.to_string())
        .arg("--only")
        .arg(&contract.source_case_id)
        .output()?;
    if !generation.status.success() {
        return Err(format!(
            "fixed generated source failed: {}",
            String::from_utf8_lossy(&generation.stderr)
        )
        .into());
    }
    let source = generated.join("dense_break_132.wav");
    let source_hash = file_hash(&source)?;
    let identity = report
        .identity
        .as_mut()
        .ok_or("generator identity missing")?;
    identity["generated_source_path"] = json!(source);
    identity["generated_source_sha256"] = json!(source_hash);
    let session_path = root.join("session.json");
    let graph_path = root.join("source-graph.json");
    let mut state = JamAppState::analyze_source_file_to_json_with_source_timing_confirmation(
        &source,
        &session_path,
        Some(graph_path.clone()),
        riotbox_sidecar::path::bundled_sidecar_script_path(),
        23,
        Some(contract.source_bpm),
        None,
    )?;
    // Frozen generated V1 is a declared historical control, not a fresh V2
    // chronology qualification. Keep its policy/Session bytes and prefix meaning.
    // Ordinary product ingestion retains the new committed-order default.
    state.session.action_log.replay_policy =
        riotbox_core::session::ReplayPolicy::DeterministicPreferred;
    let cache = state
        .source_audio_cache
        .as_ref()
        .ok_or("generated source did not hydrate")?;
    if cache.sample_rate != contract.source_sample_rate
        || cache.channel_count != contract.source_channels
        || cache.frame_count()
            != contract.source_seconds as usize * contract.source_sample_rate as usize
        || state
            .source_graph
            .as_ref()
            .ok_or("generated graph missing")?
            .source
            .content_hash
            .trim()
            .trim_start_matches("sha256:")
            != source_hash
    {
        return Err("generated source identity/geometry disagrees with the fixed input".into());
    }
    let timing = confirmed_source_timing(&state, contract.source_bpm)?;
    let prefix = ready_prefix::prepare(&mut state, &timing)?;
    state.save()?;
    let saved_session = serde_json::to_vec(&state.session)?;
    let session_file_hash = file_hash(&session_path)?;
    let identity = report
        .identity
        .as_mut()
        .ok_or("generated identity missing")?;
    identity["session_path"] = json!(session_path);
    identity["source_graph_path"] = json!(graph_path);
    identity["session_file_sha256"] = json!(session_file_hash);
    identity["source_graph_file_sha256"] = json!(file_hash(&graph_path)?);
    identity["source_graph_ref"] = json!(state.session.source_graph_refs);
    identity["committed_prefix"] = json!({
        "capture_action_id": prefix.capture_journey_proof.capture_action_id,
        "raw_audition_action_id": prefix.capture_journey_proof.raw_audition_action_id,
        "promotion_action_id": prefix.capture_journey_proof.promotion_action_id,
        "preset_action_id": prefix.preset_action_id,
        "preset": prefix.preset_id,
        "position_beats": prefix.phrase_cursor,
        "scripted_fixture_actions": true,
    });
    drop(state);
    // Exercise ordinary Session-relative graph resolution and real capture hydration.
    let restored = JamAppState::from_json_files(&session_path, Option::<&Path>::None)?;
    if serde_json::to_vec(&restored.session)? != saved_session {
        return Err("saved Core Session identity changed on ordinary restore".into());
    }
    let capture = restored
        .session
        .runtime_state
        .lane_state
        .w30
        .last_capture
        .as_ref()
        .ok_or("ready prefix has no W-30 capture")?;
    if restored.source_audio_cache.is_none() || !restored.capture_audio_cache.contains_key(capture)
    {
        return Err("ready source/capture did not hydrate on restore".into());
    }
    identity["save_restore_identity_verified"] = json!(true);
    identity["capture_artifacts"] = json!(restored.session.captures);
    identity["core_session_sha256"] = json!(hex::encode(Sha256::digest(&saved_session)));
    identity["contributors"] = json!([
        "tr909",
        "mc202",
        "w30_preview",
        "w30_resample_tap",
        "source_monitor"
    ]);
    identity["contributor_scope"] =
        json!("named composite, not isolated lanes; generated fixture origin");
    fs::create_dir(root.join("offline"))?;
    measure_windows(root, contract, &saved_session, &session_file_hash, report)
}

fn measure_windows(
    root: &Path,
    contract: &Contract,
    saved_session: &[u8],
    session_file_hash: &str,
    report: &mut Report,
) -> Result<(), Box<dyn Error>> {
    for rate in contract.output_sample_rates {
        for duration in [
            riotbox_core::action::LiveRecordingDuration::TwoBars,
            riotbox_core::action::LiveRecordingDuration::EightBars,
            riotbox_core::action::LiveRecordingDuration::SixteenBars,
        ] {
            let measured = windows::measure(
                root,
                contract,
                rate,
                duration,
                saved_session,
                session_file_hash,
            )?;
            report.windows.push(measured);
        }
    }
    if report.windows.iter().any(|window| !window.passed) {
        return Err(
            "post-restore generated recording window failed its technical preparation gates".into(),
        );
    }
    Ok(())
}

pub(super) fn file_hash(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex::encode(digest.finalize()))
}

#[cfg(test)]
mod tests;
