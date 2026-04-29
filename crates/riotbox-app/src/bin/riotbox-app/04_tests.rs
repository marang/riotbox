#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_core::{ids::SceneId, queue::ActionQueue, session::SessionFile};

    #[test]
    fn parse_args_builds_ingest_mode() {
        let mode = parse_args([
            "--source".into(),
            "input.wav".into(),
            "--session".into(),
            "session.json".into(),
            "--graph".into(),
            "graph.json".into(),
        ])
        .expect("parse ingest mode");

        assert_eq!(mode.observer_path, None);
        match mode.mode {
            LaunchMode::Ingest {
                source_path,
                session_path,
                source_graph_path,
                analysis_seed,
                ..
            } => {
                assert_eq!(source_path, PathBuf::from("input.wav"));
                assert_eq!(session_path, PathBuf::from("session.json"));
                assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
                assert_eq!(analysis_seed, 19);
            }
            LaunchMode::Load { .. } => panic!("expected ingest mode"),
        }
    }

    #[test]
    fn parse_args_defaults_ingest_to_embedded_graph_storage() {
        let mode = parse_args([
            "--source".into(),
            "input.wav".into(),
            "--session".into(),
            "session.json".into(),
        ])
        .expect("parse ingest mode");

        match mode.mode {
            LaunchMode::Ingest {
                source_graph_path, ..
            } => {
                assert_eq!(source_graph_path, None);
            }
            LaunchMode::Load { .. } => panic!("expected ingest mode"),
        }
    }

    #[test]
    fn parse_args_builds_load_mode() {
        let mode = parse_args([
            "--session".into(),
            "session.json".into(),
            "--graph".into(),
            "graph.json".into(),
        ])
        .expect("parse load mode");

        match mode.mode {
            LaunchMode::Load {
                session_path,
                source_graph_path,
            } => {
                assert_eq!(session_path, PathBuf::from("session.json"));
                assert_eq!(source_graph_path, Some(PathBuf::from("graph.json")));
            }
            LaunchMode::Ingest { .. } => panic!("expected load mode"),
        }
    }

    #[test]
    fn parse_args_allows_session_only_for_load_mode() {
        let mode =
            parse_args(["--session".into(), "session.json".into()]).expect("session-only load");

        match mode.mode {
            LaunchMode::Load {
                session_path,
                source_graph_path,
            } => {
                assert_eq!(session_path, PathBuf::from("session.json"));
                assert_eq!(source_graph_path, None);
            }
            LaunchMode::Ingest { .. } => panic!("expected load mode"),
        }
    }

    #[test]
    fn parse_args_accepts_observer_path() {
        let launch = parse_args([
            "--session".into(),
            "session.json".into(),
            "--observer".into(),
            "artifacts/audio_qa/live/events.ndjson".into(),
        ])
        .expect("parse observer path");

        assert_eq!(
            launch.observer_path,
            Some(PathBuf::from("artifacts/audio_qa/live/events.ndjson"))
        );
    }

    #[test]
    fn user_session_observer_writes_launch_and_key_events() {
        let temp = tempfile::tempdir().expect("tempdir");
        let observer_path = temp.path().join("observer/events.ndjson");
        let launch = AppLaunch {
            mode: LaunchMode::Load {
                session_path: PathBuf::from("session.json"),
                source_graph_path: None,
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

    #[test]
    fn scene_select_unavailable_status_explains_waiting_for_scene_material() {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-25T00:00:00Z");
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));

        let shell = JamShellState::new(
            JamAppState::from_parts(session, None, ActionQueue::new()),
            ShellLaunchMode::Load,
        );

        assert_eq!(
            scene_select_unavailable_status(&shell),
            "scene jump waits for 2 scenes"
        );
    }
}
