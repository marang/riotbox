use crate::cli::model::AppLaunch;
use crate::cli::model::LaunchMode;
use crate::cli::stem_package_export::export_artifact_role_label;
use crate::observer::compact_commit;
use crate::observer::observer_snapshot;
use crate::ui::JamShellState;
use serde_json::Value;
use serde_json::json;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(in crate::cli) struct UserSessionObserver {
    writer: BufWriter<File>,
}

impl UserSessionObserver {
    pub(in crate::cli) fn open(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?,
        );
        Ok(Self { writer })
    }

    pub(in crate::cli) fn open_new(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let writer = BufWriter::new(OpenOptions::new().create_new(true).write(true).open(path)?);
        Ok(Self { writer })
    }

    pub(in crate::cli) fn record_launch(
        &mut self,
        raw_args: &[String],
        launch: &AppLaunch,
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "observer_started",
            "schema": "riotbox.user_session_observer.v1",
            "timestamp_ms": timestamp_now(),
            "opt_in": true,
            "capture_context": "interactive_terminal",
            "raw_audio_recording": false,
            "realtime_callback_io": false,
            "argv": raw_args,
            "launch": launch_summary(launch),
            "snapshot": observer_snapshot(shell),
        }))
    }

    pub(in crate::cli) fn record_audio_runtime(
        &mut self,
        status: &str,
        error: Option<&str>,
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "audio_runtime",
            "timestamp_ms": timestamp_now(),
            "status": status,
            "error": error,
            "snapshot": observer_snapshot(shell),
        }))
    }

    pub(in crate::cli) fn record_key_event(
        &mut self,
        timestamp_ms: u64,
        key: &str,
        outcome: &str,
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "key_outcome",
            "timestamp_ms": timestamp_ms,
            "key": key,
            "outcome": outcome,
            "snapshot": observer_snapshot(shell),
        }))
    }

    pub(in crate::cli) fn record_transport_commit(
        &mut self,
        timestamp_ms: u64,
        committed: &[riotbox_core::queue::CommittedActionRef],
        shell: &JamShellState,
    ) -> io::Result<()> {
        self.record(json!({
            "event": "transport_commit",
            "timestamp_ms": timestamp_ms,
            "committed": committed.iter().map(compact_commit).collect::<Vec<_>>(),
            "snapshot": observer_snapshot(shell),
        }))
    }

    pub(in crate::cli) fn record(&mut self, event: Value) -> io::Result<()> {
        serde_json::to_writer(&mut self.writer, &event).map_err(io::Error::other)?;
        writeln!(self.writer)?;
        self.writer.flush()
    }
}

pub(in crate::cli) fn launch_summary(launch: &AppLaunch) -> Value {
    match &launch.mode {
        LaunchMode::Load {
            session_path,
            source_graph_path,
            product_mix_export_handoff,
        } => json!({
            "mode": "load",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "observer_path": launch.observer_path,
            "product_mix_export_handoff": product_mix_export_handoff.as_ref().map(|handoff| json!({
                "proof_path": handoff.proof_path,
                "destination_path": handoff.destination_path,
            })),
        }),
        LaunchMode::Ingest {
            source_path,
            session_path,
            source_graph_path,
            sidecar_script_path,
            analysis_seed,
            explicit_source_bpm,
            explicit_source_downbeat_seconds,
            product_mix_export_handoff,
        } => json!({
            "mode": "ingest",
            "source_path": source_path,
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "sidecar_script_path": sidecar_script_path,
            "analysis_seed": analysis_seed,
            "explicit_source_bpm": explicit_source_bpm,
            "explicit_source_downbeat_seconds": explicit_source_downbeat_seconds,
            "observer_path": launch.observer_path,
            "product_mix_export_handoff": product_mix_export_handoff.as_ref().map(|handoff| json!({
                "proof_path": handoff.proof_path,
                "destination_path": handoff.destination_path,
            })),
        }),
        LaunchMode::StemPackageLocalCiDryRun {
            destination_path,
            claimed_stem_roles,
        } => json!({
            "mode": "stem_package_local_ci_dry_run",
            "destination_path": destination_path,
            "claimed_stem_roles": claimed_stem_roles
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageLocalCiExecute {
            session_path,
            source_graph_path,
            destination_path,
            claimed_stem_roles,
        } => json!({
            "mode": "stem_package_local_ci_execute",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "destination_path": destination_path,
            "claimed_stem_roles": claimed_stem_roles
                .iter()
                .copied()
                .map(export_artifact_role_label)
                .collect::<Vec<_>>(),
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageSourceMatchedExecute {
            session_path,
            source_graph_path,
            handoff_proof_path,
            destination_path,
        } => json!({
            "mode": "stem_package_source_matched_execute",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "handoff_proof_path": handoff_proof_path,
            "destination_path": destination_path,
            "claimed_stem_roles": ["stem_drums", "stem_music", "stem_bass"],
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageW30HookExecute {
            session_path,
            source_graph_path,
            destination_path,
        } => json!({
            "mode": "stem_package_w30_hook_execute",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "destination_path": destination_path,
            "claimed_stem_roles": ["w30_hook_loop"],
            "observer_path": launch.observer_path,
        }),
        LaunchMode::StemPackageLocalCiReport { session_path } => json!({
            "mode": "stem_package_local_ci_report",
            "session_path": session_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::LiveRecordingReadinessReport { session_path } => json!({
            "mode": "live_recording_readiness_report",
            "session_path": session_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::LiveMasterRecordingExecute {
            session_path,
            source_graph_path,
            destination_path,
        } => json!({
            "mode": "live_master_recording_execute",
            "session_path": session_path,
            "source_graph_path": source_graph_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawExportReadinessReport { session_path } => json!({
            "mode": "daw_export_readiness_report",
            "session_path": session_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionJsonPackageExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_json_package_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionJsonPackageEvidenceApply {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_json_package_evidence_apply",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionHostImportProofApply {
            session_path,
            proof_path,
        } => json!({
            "mode": "daw_session_host_import_proof_apply",
            "session_path": session_path,
            "proof_path": proof_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionHostImportProofExportExecute {
            session_path,
            proof_path,
        } => json!({
            "mode": "daw_session_host_import_proof_export_execute",
            "session_path": session_path,
            "proof_path": proof_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionAudibleOutputProofApply {
            session_path,
            proof_path,
        } => json!({
            "mode": "daw_session_audible_output_proof_apply",
            "session_path": session_path,
            "proof_path": proof_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterProofExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_proof_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterProofApply {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_proof_apply",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterExportExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_export_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::W30HookDawprojectExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "w30_hook_dawproject_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::LiveMasterDawprojectExecute {
            session_path,
            destination_path,
        } => json!({
            "mode": "live_master_dawproject_execute",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
        LaunchMode::DawSessionWriterPlan {
            session_path,
            destination_path,
        } => json!({
            "mode": "daw_session_writer_plan",
            "session_path": session_path,
            "destination_path": destination_path,
            "observer_path": launch.observer_path,
        }),
    }
}

pub(in crate::cli) fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
