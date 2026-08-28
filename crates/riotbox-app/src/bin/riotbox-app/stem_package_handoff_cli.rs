fn run_stem_package_source_matched_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    write_stem_package_source_matched_execute_output(launch, raw_args, &mut stdout)
}

fn write_stem_package_source_matched_execute_output(
    launch: &AppLaunch,
    raw_args: &[String],
    output: &mut impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = stem_package_source_matched_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "stem_package_source_matched_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_cli",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        }))?;
    }
    serde_json::to_writer_pretty(&mut *output, &summary)?;
    writeln!(output)?;
    Ok(())
}

fn stem_package_source_matched_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::StemPackageSourceMatchedExecute {
        session_path,
        source_graph_path,
        handoff_proof_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a source-matched stem package execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let summary = match state.commit_stem_package_export_from_product_handoff(
        handoff_proof_path,
        destination_path,
        timestamp_now(),
    ) {
        Ok(receipt) => {
            state.save()?;
            json!({
                "mode": "stem_package_source_matched_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "boundary": "stem_package.source_matched_handoff_v1",
                "session_path": session_path,
                "source_graph_path": source_graph_path,
                "handoff_proof_path": handoff_proof_path,
                "destination_path": destination_path,
                "claimed_stem_roles": ["stem_drums", "stem_music", "stem_bass"],
                "readiness_blockers": [],
                "receipt": stem_package_receipt_summary(&receipt),
            })
        }
        Err(error) => json!({
            "mode": "stem_package_source_matched_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "boundary": "stem_package.source_matched_handoff_v1",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "handoff_proof_path": handoff_proof_path,
            "destination_path": destination_path,
            "claimed_stem_roles": ["stem_drums", "stem_music", "stem_bass"],
            "readiness_blockers": [error.to_string()],
            "receipt": null,
        }),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

fn run_stem_package_w30_hook_execute(
    launch: &AppLaunch,
    raw_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    write_stem_package_w30_hook_execute_output(launch, raw_args, &mut stdout)
}

fn write_stem_package_w30_hook_execute_output(
    launch: &AppLaunch,
    raw_args: &[String],
    output: &mut impl std::io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (summary, shell) = stem_package_w30_hook_execute_summary(launch)?;
    if let Some(path) = launch.observer_path.as_deref() {
        let mut observer = UserSessionObserver::open(path)?;
        observer.record(json!({
            "event": "stem_package_w30_hook_execute",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "non_interactive_cli",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "summary": summary.clone(),
            "snapshot": observer_snapshot(&shell),
        }))?;
    }
    serde_json::to_writer_pretty(&mut *output, &summary)?;
    writeln!(output)?;
    Ok(())
}

fn stem_package_w30_hook_execute_summary(
    launch: &AppLaunch,
) -> Result<(Value, JamShellState), Box<dyn std::error::Error>> {
    let LaunchMode::StemPackageW30HookExecute {
        session_path,
        source_graph_path,
        destination_path,
    } = &launch.mode
    else {
        return Err("not a W-30 hook stem package execute launch".into());
    };

    let mut state = JamAppState::from_json_files(session_path, source_graph_path.as_deref())?;
    let summary = match state
        .commit_stem_package_export_w30_hook_loop(destination_path, timestamp_now())
    {
        Ok(receipt) => {
            let musician_handoff = w30_hook_musician_handoff_summary(&state, &receipt);
            state.save()?;
            json!({
                "mode": "stem_package_w30_hook_execute",
                "status": "ready",
                "ready": true,
                "writes_files": true,
                "boundary": "stem_package.w30_hook_loop_v4",
                "session_path": session_path,
                "source_graph_path": source_graph_path,
                "destination_path": destination_path,
                "claimed_stem_roles": ["w30_hook_loop"],
                "readiness_blockers": [],
                "musician_handoff": musician_handoff,
                "receipt": stem_package_receipt_summary(&receipt),
            })
        }
        Err(error) => json!({
            "mode": "stem_package_w30_hook_execute",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "boundary": "stem_package.w30_hook_loop_v4",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "destination_path": destination_path,
            "claimed_stem_roles": ["w30_hook_loop"],
            "readiness_blockers": [error.to_string()],
            "musician_handoff": null,
            "receipt": null,
        }),
    };
    let shell = JamShellState::new(state, ShellLaunchMode::Load);
    Ok((summary, shell))
}

const W30_HOOK_MUSICIAN_HANDOFF_SCHEMA: &str = "riotbox.w30_hook_musician_handoff.v1";

#[derive(serde::Serialize)]
pub(crate) struct W30HookMusicianHandoffSummary<'a> {
    schema: &'static str,
    purpose: [&'static str; 3],
    wav_path: Option<&'a str>,
    manifest_path: Option<&'a str>,
    proof_path: Option<&'a str>,
    confirmed_bpm: Option<f32>,
    loop_start_beat: u32,
    source_transport_start_beat: u32,
    duration_beats: u32,
    beats_per_bar: u32,
    duration_bars: u32,
    sample_rate_hz: Option<u32>,
    channel_count: Option<u16>,
    duration_ms: Option<u64>,
    source_graph_ref: Option<&'a riotbox_core::session::ExportArtifactSourceGraphRef>,
    timing_grid_ref: Option<&'a riotbox_core::session::ExportArtifactTimingGridRef>,
    source_capture_refs: &'a [riotbox_core::ids::CaptureId],
    lineage_capture_refs: &'a [riotbox_core::ids::CaptureId],
    receipt_id: &'a riotbox_core::ids::ExportReceiptId,
    pack_id: &'a str,
    boundary: &'static str,
    canonical_truth: &'static str,
}

pub(crate) fn w30_hook_musician_handoff_summary<'a>(
    state: &'a JamAppState,
    receipt: &'a riotbox_core::session::ExportReceiptState,
) -> W30HookMusicianHandoffSummary<'a> {
    use riotbox_core::{
        session::ExportArtifactRole,
        stem_package_writer::{
            W30_HOOK_LOOP_BEATS_PER_BAR, W30_HOOK_LOOP_DURATION_BARS,
            W30_HOOK_LOOP_DURATION_BEATS, W30_HOOK_LOOP_LOOP_START_BEAT,
            W30_HOOK_LOOP_SOURCE_TRANSPORT_START_BEAT,
        },
    };

    let hook = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::W30HookLoop);
    let manifest = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::ExportManifest);
    let proof = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == ExportArtifactRole::ProductExportProof);

    W30HookMusicianHandoffSummary {
        schema: W30_HOOK_MUSICIAN_HANDOFF_SCHEMA,
        purpose: ["loop", "arrange", "process"],
        wav_path: hook.map(|artifact| artifact.location_identity()),
        manifest_path: manifest.map(|artifact| artifact.location_identity()),
        proof_path: proof.map(|artifact| artifact.location_identity()),
        confirmed_bpm: state.session.runtime_state.source_timing.confirmed_bpm,
        loop_start_beat: W30_HOOK_LOOP_LOOP_START_BEAT,
        source_transport_start_beat: W30_HOOK_LOOP_SOURCE_TRANSPORT_START_BEAT,
        duration_beats: W30_HOOK_LOOP_DURATION_BEATS,
        beats_per_bar: W30_HOOK_LOOP_BEATS_PER_BAR,
        duration_bars: W30_HOOK_LOOP_DURATION_BARS,
        sample_rate_hz: hook.and_then(|artifact| artifact.sample_rate_hz),
        channel_count: hook.and_then(|artifact| artifact.channel_count),
        duration_ms: hook.and_then(|artifact| artifact.duration_ms),
        source_graph_ref: hook.and_then(|artifact| artifact.source_graph_ref.as_ref()),
        timing_grid_ref: hook.and_then(|artifact| artifact.timing_grid_ref.as_ref()),
        source_capture_refs: hook.map_or(&[], |artifact| artifact.source_capture_refs.as_slice()),
        lineage_capture_refs: hook
            .map_or(&[], |artifact| artifact.lineage_capture_refs.as_slice()),
        receipt_id: &receipt.receipt_id,
        pack_id: &receipt.pack_id,
        boundary: receipt.export_boundary.as_proof_str(),
        canonical_truth: "session_export_receipt_and_stem_package_manifest",
    }
}
