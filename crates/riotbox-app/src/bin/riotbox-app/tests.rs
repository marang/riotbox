#[cfg(test)]
mod tests {
    use super::*;
    use riotbox_core::{
        action::{ActionCommand, ActionTarget, GhostMode, Quantization, TargetScope},
        ghost::{
            GhostSuggestedAction, GhostSuggestionConfidence, GhostSuggestionSafety,
            GhostWatchSuggestion, GhostWatchTool,
        },
        ids::{ActionId, AssetId, BankId, CaptureId, PadId, SceneId, SourceId},
        persistence::save_session_json,
        queue::ActionQueue,
        session::{
            ActionCommitRecord, CaptureRef, CaptureTarget, CaptureType, SessionFile, Snapshot,
            SnapshotPayload,
        },
        source_graph::{
            AnalysisSummary, Candidate, CandidateType, DecodeProfile, GraphProvenance,
            QualityClass, SourceDescriptor, SourceGraph,
        },
    };

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
    fn observer_snapshot_records_recovery_startup_probe_without_selecting_candidate() {
        let temp = tempfile::tempdir().expect("tempdir");
        let session_path = temp.path().join("session.json");
        let autosave_path = temp
            .path()
            .join("session.autosave.artifact-ready-blocked.json");
        let captures_dir = temp.path().join("captures");
        fs::create_dir_all(&captures_dir).expect("create captures dir");
        fs::write(captures_dir.join("cap-01.wav"), [0_u8; 44])
            .expect("write capture artifact");
        save_session_json(
            &session_path,
            &SessionFile::new("canonical", "0.1.0", "2026-04-30T09:58:00Z"),
        )
        .expect("save canonical session");
        save_session_json(
            &autosave_path,
            &artifact_ready_blocked_autosave_session(),
        )
        .expect("save autosave session");

        let mode = LaunchMode::Load {
            session_path: session_path.clone(),
            source_graph_path: None,
        };
        let shell = shell_for_loaded_state(
            JamAppState::from_parts(
                SessionFile::new("loaded", "0.1.0", "2026-04-30T09:58:00Z"),
                None,
                ActionQueue::new(),
            ),
            &mode,
        );

        let snapshot = observer_snapshot(&shell);
        let recovery = &snapshot["recovery"];
        assert_eq!(recovery["present"], true);
        assert_eq!(recovery["has_manual_candidates"], true);
        assert_eq!(recovery["selected_candidate"], serde_json::Value::Null);
        assert_eq!(recovery["candidate_count"], 2);

        let candidates = recovery["candidates"].as_array().expect("candidate array");
        assert_eq!(candidates[0]["kind"], "normal session path");
        assert_eq!(candidates[0]["trust"], "NormalLoadTarget");
        assert_eq!(candidates[1]["kind"], "autosave file");
        let autosave = candidates
            .iter()
            .find(|candidate| {
                candidate["path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("session.autosave.artifact-ready-blocked.json"))
            })
            .expect("autosave recovery candidate");
        assert_eq!(autosave["trust"], "RecoverableClue");
        assert_eq!(autosave["artifact_availability"], "artifacts ready: 1 capture(s)");
        assert_eq!(autosave["payload_readiness"], "payload ready | snapshot restore ok");
        assert_eq!(autosave["replay_unsupported"], "unsupported suffix 1: w30.loop_freeze");
        assert_eq!(
            autosave["guidance"],
            "ArtifactReadyReplayHydrationBlocked"
        );
        assert!(session_path.exists());
        assert!(autosave_path.exists());
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

    fn artifact_ready_blocked_autosave_session() -> SessionFile {
        let mut session = SessionFile::new("autosave", "0.1.0", "2026-04-30T09:58:01Z");
        session.snapshots.push(Snapshot {
            snapshot_id: "snap-1".into(),
            created_at: "2026-04-30T09:58:02Z".into(),
            label: "before unsupported freeze".into(),
            action_cursor: 0,
            payload: Some(SnapshotPayload::from_runtime_state(
                &"snap-1".into(),
                0,
                &session.runtime_state,
            )),
        });
        session.captures.push(CaptureRef {
            capture_id: CaptureId::from("cap-01"),
            capture_type: CaptureType::Pad,
            source_origin_refs: vec!["source-1".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: Some(ActionId(1)),
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: Some(CaptureTarget::W30Pad {
                bank_id: BankId::from("bank-a"),
                pad_id: PadId::from("pad-01"),
            }),
            is_pinned: false,
            notes: None,
        });
        session.action_log.actions.push(riotbox_core::action::Action {
            id: ActionId(88),
            actor: riotbox_core::action::ActorType::User,
            command: ActionCommand::W30LoopFreeze,
            params: riotbox_core::action::ActionParams::Promotion {
                capture_id: Some(CaptureId::from("cap-01")),
                destination: Some("w30:loop_freeze".into()),
            },
            target: ActionTarget {
                scope: Some(TargetScope::LaneW30),
                bank_id: Some(BankId::from("bank-a")),
                pad_id: Some(PadId::from("pad-01")),
                ..Default::default()
            },
            requested_at: 480,
            quantization: Quantization::NextBar,
            status: riotbox_core::action::ActionStatus::Committed,
            committed_at: Some(500),
            result: None,
            undo_policy: riotbox_core::action::UndoPolicy::Undoable,
            explanation: Some("artifact-producing W-30 action".into()),
        });
        session.action_log.commit_records.push(ActionCommitRecord {
            action_id: ActionId(88),
            boundary: riotbox_core::transport::CommitBoundaryState {
                kind: riotbox_core::action::CommitBoundary::Bar,
                beat_index: 40,
                bar_index: 10,
                phrase_index: 2,
                scene_id: Some(SceneId::from("scene-1")),
            },
            commit_sequence: 1,
            committed_at: 500,
        });
        session
    }

    #[test]
    fn ghost_accept_control_reports_queue_or_read_only_status() {
        let mut assist_shell = ghost_shell(GhostMode::Assist);

        accept_current_ghost_suggestion(&mut assist_shell, 123);

        assert!(
            assist_shell
                .status_message
                .starts_with("accepted ghost suggestion | queued action "),
            "{}",
            assist_shell.status_message
        );
        assert!(assist_shell.app.runtime.current_ghost_suggestion.is_none());
        assert_eq!(assist_shell.app.queue.pending_actions().len(), 1);

        let mut watch_shell = ghost_shell(GhostMode::Watch);

        accept_current_ghost_suggestion(&mut watch_shell, 123);

        assert_eq!(
            watch_shell.status_message,
            "ghost accept ignored: ghost accept requires assist mode"
        );
        assert!(watch_shell.app.runtime.current_ghost_suggestion.is_some());
        assert!(watch_shell.app.queue.pending_actions().is_empty());
    }

    #[test]
    fn ghost_reject_control_reports_clear_or_noop_status() {
        let mut shell = ghost_shell(GhostMode::Assist);

        reject_current_ghost_suggestion(&mut shell);

        assert_eq!(shell.status_message, "rejected current ghost suggestion");
        assert!(shell.app.runtime.current_ghost_suggestion.is_none());
        assert!(shell.app.session.ghost_state.suggestion_history[0].rejected);

        reject_current_ghost_suggestion(&mut shell);

        assert_eq!(
            shell.status_message,
            "ghost reject ignored: no current ghost suggestion"
        );
    }

    #[test]
    fn ghost_accept_control_can_request_then_accept_jam_state_suggestion() {
        let mut shell = ghost_feed_shell();

        accept_current_ghost_suggestion(&mut shell, 123);

        assert_eq!(
            shell.status_message,
            "ghost suggestion ready: capture the current source-backed hit"
        );
        assert!(shell.app.runtime.current_ghost_suggestion.is_some());
        assert!(shell.app.queue.pending_actions().is_empty());

        accept_current_ghost_suggestion(&mut shell, 124);

        assert!(
            shell
                .status_message
                .starts_with("accepted ghost suggestion | queued action "),
            "{}",
            shell.status_message
        );
        assert!(shell.app.runtime.current_ghost_suggestion.is_none());
        assert_eq!(shell.app.queue.pending_actions().len(), 1);
        assert_eq!(
            shell.app.queue.pending_actions()[0].command,
            ActionCommand::CaptureNow
        );
    }

    fn ghost_shell(mode: GhostMode) -> JamShellState {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T00:00:00Z");
        session.ghost_state.mode = mode;
        let mut shell = JamShellState::new(
            JamAppState::from_parts(session, None, ActionQueue::new()),
            ShellLaunchMode::Load,
        );
        shell
            .app
            .set_current_ghost_suggestion(sample_ghost_fill_suggestion(mode));
        shell
    }

    fn ghost_feed_shell() -> JamShellState {
        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-29T00:00:00Z");
        session.ghost_state.mode = GhostMode::Assist;
        JamShellState::new(
            JamAppState::from_parts(
                session,
                Some(ghost_capture_candidate_graph()),
                ActionQueue::new(),
            ),
            ShellLaunchMode::Load,
        )
    }

    fn ghost_capture_candidate_graph() -> SourceGraph {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 12.0,
                sample_rate: 44_100,
                channel_count: 2,
                decode_profile: DecodeProfile::Native,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["decoded.wav_baseline".into()],
                generated_at: "2026-04-29T17:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 1,
                run_notes: None,
            },
        );
        graph.candidates.push(Candidate {
            candidate_id: "capture-candidate-a".into(),
            candidate_type: CandidateType::CaptureCandidate,
            asset_ref: AssetId::from("asset-a"),
            score: 0.86,
            confidence: 0.88,
            tags: vec!["capture".into()],
            constraints: vec!["bar_aligned".into()],
            provenance_refs: vec!["provider:decoded.wav_baseline".into()],
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.86,
            timing_quality: QualityClass::Medium,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 0,
            hook_candidate_count: 0,
            break_rebuild_potential: QualityClass::Medium,
            warnings: Vec::new(),
        };
        graph
    }

    fn sample_ghost_fill_suggestion(mode: GhostMode) -> GhostWatchSuggestion {
        GhostWatchSuggestion {
            proposal_id: "ghost-fill-1".into(),
            mode,
            tool_name: GhostWatchTool::SuggestMacroShift,
            summary: "add a next-bar drum answer".into(),
            rationale: "the current loop has room before the next scene move".into(),
            suggested_action: Some(GhostSuggestedAction {
                command: ActionCommand::Tr909FillNext,
                target: ActionTarget {
                    scope: Some(TargetScope::LaneTr909),
                    ..Default::default()
                },
                quantization: Quantization::NextBar,
                intent: "add a next-bar drum answer".into(),
            }),
            confidence: GhostSuggestionConfidence::Medium,
            safety: GhostSuggestionSafety::NeedsAssistAcceptance,
            blockers: Vec::new(),
            created_at: "2026-04-29T17:00:00Z".into(),
        }
    }
}
