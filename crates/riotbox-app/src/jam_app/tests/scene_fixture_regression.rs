#[test]
fn scene_fixture_backed_committed_state_regressions_hold() {
    let fixtures: Vec<SceneRegressionFixture> =
        serde_json::from_str(include_str!("../../../tests/fixtures/scene_regression.json"))
            .expect("parse Scene Brain regression fixtures");

    for fixture in fixtures {
        let graph = scene_regression_graph(&fixture.section_labels);
        let mut session = sample_session(&graph);
        session.runtime_state.transport.current_scene = None;
        session.runtime_state.scene_state.active_scene = None;
        session.runtime_state.scene_state.scenes.clear();

        let mut state = JamAppState::from_parts(session, Some(graph), ActionQueue::new());
        seed_scene_fixture_state(&mut state, &fixture);

        match fixture.action {
            SceneRegressionAction::ProjectCandidates => {}
            SceneRegressionAction::SelectNextScene => {
                assert_eq!(
                    state.queue_scene_select(
                        fixture.requested_at.expect("scene select requested_at")
                    ),
                    QueueControlResult::Enqueued,
                    "{} did not enqueue",
                    fixture.name
                );

                let committed = state.commit_ready_actions(
                    fixture
                        .boundary
                        .expect("scene select boundary")
                        .into_commit_boundary_state(),
                    fixture.committed_at.expect("scene select committed_at"),
                );
                assert_eq!(
                    committed.len(),
                    1,
                    "{} did not commit exactly one action",
                    fixture.name
                );
            }
            SceneRegressionAction::RestoreScene => {
                assert_eq!(
                    state.queue_scene_restore(
                        fixture.requested_at.expect("scene restore requested_at")
                    ),
                    QueueControlResult::Enqueued,
                    "{} did not enqueue",
                    fixture.name
                );

                let committed = state.commit_ready_actions(
                    fixture
                        .boundary
                        .expect("scene restore boundary")
                        .into_commit_boundary_state(),
                    fixture.committed_at.expect("scene restore committed_at"),
                );
                assert_eq!(
                    committed.len(),
                    1,
                    "{} did not commit exactly one action",
                    fixture.name
                );
            }
        }

        let actual_scenes = state
            .session
            .runtime_state
            .scene_state
            .scenes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            actual_scenes, fixture.expected.scenes,
            "{} scenes drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .scene_state
                .active_scene
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(fixture.expected.active_scene.as_str()),
            "{} active scene drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .transport
                .current_scene
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(fixture.expected.current_scene.as_str()),
            "{} transport scene drifted",
            fixture.name
        );
        assert_eq!(
            state
                .session
                .runtime_state
                .scene_state
                .restore_scene
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            fixture.expected.restore_scene.as_deref(),
            "{} restore scene drifted",
            fixture.name
        );
        assert_eq!(
            state.jam_view.scene.active_scene.as_deref(),
            Some(fixture.expected.active_scene.as_str()),
            "{} jam view scene drifted",
            fixture.name
        );
        assert_eq!(
            state.jam_view.scene.active_scene_energy.as_deref(),
            Some(fixture.expected.active_scene_energy.as_str()),
            "{} jam view active energy drifted",
            fixture.name
        );
        assert_eq!(
            state.jam_view.scene.restore_scene_energy.as_deref(),
            fixture.expected.restore_scene_energy.as_deref(),
            "{} jam view restore energy drifted",
            fixture.name
        );
        assert_eq!(
            state
                .runtime
                .transport
                .current_scene
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(fixture.expected.current_scene.as_str()),
            "{} runtime transport scene drifted",
            fixture.name
        );

        if let Some(expected_summary) = &fixture.expected.result_summary {
            assert_eq!(
                state
                    .session
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
                    .map(|result| result.summary.as_str()),
                Some(expected_summary.as_str()),
                "{} result summary drifted",
                fixture.name
            );
        }
        if let Some(expected_profile) = fixture.expected.tr909_render_profile.as_deref() {
            assert_eq!(
                state.runtime_view.tr909_render_profile, expected_profile,
                "{} TR-909 profile drifted",
                fixture.name
            );
        }
        if let Some(expected_context) = fixture.expected.tr909_render_support_context.as_deref() {
            assert_eq!(
                state.runtime_view.tr909_render_support_context, expected_context,
                "{} TR-909 support context drifted",
                fixture.name
            );
        }
        if let Some(expected_accent) = fixture.expected.tr909_render_support_accent.as_deref() {
            assert_eq!(
                state.runtime_view.tr909_render_support_accent, expected_accent,
                "{} TR-909 support accent drifted",
                fixture.name
            );
        }

        let tempdir = tempdir().expect("create Scene Brain regression tempdir");
        let session_path = tempdir.path().join(format!("{}.json", fixture.name));
        save_session_json(&session_path, &state.session)
            .expect("save Scene Brain regression session");
        let loaded =
            load_session_json(&session_path).expect("reload Scene Brain regression session");

        let loaded_scenes = loaded
            .runtime_state
            .scene_state
            .scenes
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            loaded_scenes, fixture.expected.scenes,
            "{} scenes did not survive replay roundtrip",
            fixture.name
        );
        assert_eq!(
            loaded
                .runtime_state
                .scene_state
                .active_scene
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(fixture.expected.active_scene.as_str()),
            "{} active scene did not survive replay roundtrip",
            fixture.name
        );
        assert_eq!(
            loaded
                .runtime_state
                .transport
                .current_scene
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some(fixture.expected.current_scene.as_str()),
            "{} transport scene did not survive replay roundtrip",
            fixture.name
        );
        if let Some(expected_summary) = &fixture.expected.result_summary {
            assert_eq!(
                loaded
                    .action_log
                    .actions
                    .last()
                    .and_then(|action| action.result.as_ref())
                    .map(|result| result.summary.as_str()),
                Some(expected_summary.as_str()),
                "{} result summary did not survive replay roundtrip",
                fixture.name
            );
        }
    }
}
