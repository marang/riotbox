use crate::cli::launch::recovery_surface_for_launch;
use crate::cli::launch::refresh_recovery_surface_for_launch;
use crate::cli::launch::shell_for_loaded_state;
use crate::cli::model::AppLaunch;
use crate::cli::model::LaunchMode;
use crate::cli::observer::UserSessionObserver;
use crate::jam_app::JamAppState;
use crate::jam_app::SessionRecoverySurface;
use crate::ui::JamShellState;
use crate::ui::ShellLaunchMode;
use riotbox_core::persistence::save_session_json;
use riotbox_core::queue::ActionQueue;
use riotbox_core::session::SessionFile;
use std::fs;
use std::path::PathBuf;

mod fixtures;
mod live_master_recording_cli;
mod w30_hook_dawproject_cli;

#[test]
fn load_mode_collects_manual_recovery_surface_without_selecting_candidate() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let autosave_path = temp.path().join("session.autosave.2026-04-29T211500Z.json");
    save_session_json(
        &session_path,
        &SessionFile::new("canonical", "0.1.0", "2026-04-29T21:15:00Z"),
    )
    .expect("save canonical session");
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "0.1.0", "2026-04-29T21:15:01Z"),
    )
    .expect("save autosave session");

    let surface = recovery_surface_for_launch(&LaunchMode::Load {
        session_path,
        source_graph_path: None,
        product_mix_export_handoff: None,
    })
    .expect("load mode scans recovery candidates");

    assert!(surface.has_manual_candidates());
    assert_eq!(surface.selected_candidate, None);
    assert!(
        surface
            .candidates
            .iter()
            .any(|candidate| candidate.path == autosave_path)
    );
}

#[test]
fn loaded_shell_attaches_and_refreshes_manual_recovery_surface() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_path = temp.path().join("session.json");
    let autosave_path = temp.path().join("session.autosave.2026-04-29T211501Z.json");
    save_session_json(
        &session_path,
        &SessionFile::new("canonical", "0.1.0", "2026-04-29T21:15:00Z"),
    )
    .expect("save canonical session");
    save_session_json(
        &autosave_path,
        &SessionFile::new("autosave", "0.1.0", "2026-04-29T21:15:01Z"),
    )
    .expect("save autosave session");
    let mode = LaunchMode::Load {
        session_path,
        source_graph_path: None,
        product_mix_export_handoff: None,
    };

    let mut shell = shell_for_loaded_state(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-04-29T21:15:00Z"),
            None,
            ActionQueue::new(),
        ),
        &mode,
    );
    assert!(
        shell
            .recovery_surface
            .as_ref()
            .is_some_and(SessionRecoverySurface::has_manual_candidates)
    );

    fs::remove_file(autosave_path).expect("remove autosave");
    refresh_recovery_surface_for_launch(&mut shell, &mode);

    assert!(shell.recovery_surface.is_none());
}

#[test]
fn user_session_observer_writes_launch_and_key_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let observer_path = temp.path().join("observer/events.ndjson");
    let launch = AppLaunch {
        mode: LaunchMode::Load {
            session_path: PathBuf::from("session.json"),
            source_graph_path: None,
            product_mix_export_handoff: None,
        },
        observer_path: Some(observer_path.clone()),
    };
    let shell = JamShellState::new(
        JamAppState::from_parts(
            SessionFile::new("session-1", "0.1.0", "2026-04-26T00:00:00Z"),
            None,
            ActionQueue::new(),
        ),
        ShellLaunchMode::Load,
    );
    let mut observer = UserSessionObserver::open(&observer_path).expect("open observer");

    observer
        .record_launch(
            &[
                "riotbox-app".into(),
                "--session".into(),
                "session.json".into(),
                "--observer".into(),
                observer_path.display().to_string(),
            ],
            &launch,
            &shell,
        )
        .expect("record launch");
    observer
        .record_key_event(123, "space", "toggle_transport", &shell)
        .expect("record key");
    drop(observer);

    let content = fs::read_to_string(observer_path).expect("read observer");

    assert!(content.contains("\"event\":\"observer_started\""));
    assert!(content.contains("\"event\":\"key_outcome\""));
    assert!(content.contains("\"outcome\":\"toggle_transport\""));
    assert!(content.contains("\"raw_audio_recording\":false"));
    assert!(content.contains("\"realtime_callback_io\":false"));
}

#[path = "tests/source_timing_observer.rs"]
mod source_timing_observer;

#[path = "tests/stem_package_export_cli.rs"]
mod stem_package_export_cli;

#[path = "tests/stem_package_report_cli.rs"]
mod stem_package_report_cli;

#[path = "tests/live_recording_report_cli.rs"]
mod live_recording_report_cli;

#[path = "tests/daw_export_report_cli.rs"]
mod daw_export_report_cli;

#[path = "tests/daw_export_surface_gate_cli.rs"]
mod daw_export_surface_gate_cli;

#[path = "tests/daw_session_writer_plan_cli.rs"]
mod daw_session_writer_plan_cli;

#[path = "tests/daw_session_json_package_cli.rs"]
mod daw_session_json_package_cli;

#[path = "tests/daw_session_json_package_evidence_cli.rs"]
mod daw_session_json_package_evidence_cli;

#[path = "tests/daw_session_host_import_proof_cli.rs"]
mod daw_session_host_import_proof_cli;

#[path = "tests/daw_session_audible_output_proof_cli.rs"]
mod daw_session_audible_output_proof_cli;

#[path = "tests/source_timing_confirm_control.rs"]
mod source_timing_confirm_control;

mod source_monitor_control;

mod transport_control;

#[path = "tests/source_map_navigation_control.rs"]
mod source_map_navigation_control;

#[path = "tests/source_map_bucket_ingest.rs"]
mod source_map_bucket_ingest;

#[path = "tests/capture_length_control.rs"]
mod capture_length_control;

#[path = "tests/recovery_observer.rs"]
mod recovery_observer;

#[path = "tests/export_observer.rs"]
mod export_observer;

#[path = "tests/product_export_control.rs"]
mod product_export_control;

#[path = "tests/export_arrangement_observer.rs"]
mod export_arrangement_observer;

#[path = "tests/export_daw_session_observer.rs"]
mod export_daw_session_observer;

#[path = "tests/export_live_recording_observer.rs"]
mod export_live_recording_observer;

#[path = "tests/export_stem_package_observer.rs"]
mod export_stem_package_observer;

mod launch_args;
mod performer_controls;
mod runtime_health;
