    #[test]
    fn derives_scene_energy_from_projected_scene_id() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "src-1".into(),
                path: "audio/test.wav".into(),
                content_hash: "graph-1".into(),
                duration_seconds: 32.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "graph-1".into(),
                analysis_seed: 7,
                run_notes: Some("scene-energy-test".into()),
            },
        );
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-a".into(),
            label_hint: crate::source_graph::SectionLabelHint::Intro,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::Medium,
            confidence: 0.9,
            tags: vec![],
        });
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-b".into(),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.timing.bpm_estimate = Some(128.0);
        graph.timing.degraded_policy = crate::source_graph::TimingDegradedPolicy::Locked;

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-02-drop"));
        session.runtime_state.scene_state.restore_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-intro"),
            SceneId::from("scene-02-drop"),
        ];

        let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

        assert_eq!(vm.scene.active_scene.as_deref(), Some("scene-02-drop"));
        assert_eq!(vm.scene.restore_scene.as_deref(), Some("scene-01-intro"));
        assert_eq!(vm.scene.next_scene.as_deref(), Some("scene-01-intro"));
        assert_eq!(
            vm.scene.scene_jump_availability,
            SceneJumpAvailabilityView::Ready
        );
        assert_eq!(
            vm.scene.arrangement_contract.readiness,
            ArrangementSceneContractReadinessView::Ready
        );
        assert_eq!(
            vm.scene.arrangement_contract.truth_source,
            ArrangementSceneTruthSourceView::ProductSpine
        );
        assert_eq!(
            vm.scene.arrangement_contract.timing_readiness,
            SourceTimingConsumerReadiness::AnalyzerLocked
        );
        assert!(vm.scene.arrangement_contract.has_next_scene);
        assert!(vm
            .scene
            .arrangement_contract
            .requires_output_path_proof_for_audible_changes);
        assert_eq!(vm.scene.active_scene_energy.as_deref(), Some("high"));
        assert_eq!(vm.scene.restore_scene_energy.as_deref(), Some("medium"));
        assert_eq!(vm.scene.next_scene_energy.as_deref(), Some("medium"));
        assert_eq!(
            vm.scene.next_scene_policy,
            Some(SceneTransitionPolicyView {
                kind: SceneTransitionKindView::Launch,
                direction: SceneTransitionDirectionView::Drop,
                tr909_intent: SceneTransitionLaneIntentView::Release,
                mc202_intent: SceneTransitionLaneIntentView::Anchor,
                intensity: 0.55,
            })
        );
        assert_eq!(
            vm.scene.restore_scene_policy,
            Some(SceneTransitionPolicyView {
                kind: SceneTransitionKindView::Restore,
                direction: SceneTransitionDirectionView::Drop,
                tr909_intent: SceneTransitionLaneIntentView::Release,
                mc202_intent: SceneTransitionLaneIntentView::Anchor,
                intensity: 0.55,
            })
        );

        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-01-intro")];

        let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

        assert_eq!(vm.scene.next_scene, None);
        assert_eq!(
            vm.scene.scene_jump_availability,
            SceneJumpAvailabilityView::WaitingForMoreScenes
        );
        assert_eq!(
            vm.scene.arrangement_contract.readiness,
            ArrangementSceneContractReadinessView::NeedsSceneMaterial
        );
        assert_eq!(vm.scene.next_scene_energy, None);
        assert_eq!(vm.scene.next_scene_policy, None);
    }

    #[test]
    fn arrangement_scene_contract_preserves_timing_trust_matrix() {
        struct TimingTrustCase {
            name: &'static str,
            policy: crate::source_graph::TimingDegradedPolicy,
            bpm: Option<f32>,
            confirmed_grid: bool,
            expected_readiness: ArrangementSceneContractReadinessView,
            expected_timing: SourceTimingConsumerReadiness,
            expected_source_locked_scene_movement: bool,
        }

        let cases = [
            TimingTrustCase {
                name: "locked analyzer timing",
                policy: crate::source_graph::TimingDegradedPolicy::Locked,
                bpm: Some(128.0),
                confirmed_grid: false,
                expected_readiness: ArrangementSceneContractReadinessView::Ready,
                expected_timing: SourceTimingConsumerReadiness::AnalyzerLocked,
                expected_source_locked_scene_movement: true,
            },
            TimingTrustCase {
                name: "manual confirm without user trust",
                policy: crate::source_graph::TimingDegradedPolicy::ManualConfirm,
                bpm: Some(128.0),
                confirmed_grid: false,
                expected_readiness: ArrangementSceneContractReadinessView::NeedsTimingConfirmation,
                expected_timing: SourceTimingConsumerReadiness::NeedsUserConfirmation,
                expected_source_locked_scene_movement: false,
            },
            TimingTrustCase {
                name: "manual confirm with user trust",
                policy: crate::source_graph::TimingDegradedPolicy::ManualConfirm,
                bpm: Some(128.0),
                confirmed_grid: true,
                expected_readiness: ArrangementSceneContractReadinessView::Ready,
                expected_timing: SourceTimingConsumerReadiness::UserConfirmed,
                expected_source_locked_scene_movement: true,
            },
            TimingTrustCase {
                name: "fallback grid",
                policy: crate::source_graph::TimingDegradedPolicy::FallbackGrid,
                bpm: Some(128.0),
                confirmed_grid: false,
                expected_readiness: ArrangementSceneContractReadinessView::FallbackTimingOnly,
                expected_timing: SourceTimingConsumerReadiness::FallbackGrid,
                expected_source_locked_scene_movement: false,
            },
            TimingTrustCase {
                name: "disabled timing",
                policy: crate::source_graph::TimingDegradedPolicy::Disabled,
                bpm: Some(128.0),
                confirmed_grid: false,
                expected_readiness: ArrangementSceneContractReadinessView::NeedsTimingEvidence,
                expected_timing: SourceTimingConsumerReadiness::Unavailable,
                expected_source_locked_scene_movement: false,
            },
            TimingTrustCase {
                name: "missing bpm",
                policy: crate::source_graph::TimingDegradedPolicy::Locked,
                bpm: None,
                confirmed_grid: false,
                expected_readiness: ArrangementSceneContractReadinessView::NeedsTimingEvidence,
                expected_timing: SourceTimingConsumerReadiness::Unavailable,
                expected_source_locked_scene_movement: false,
            },
        ];

        for case in cases {
            let mut graph = sample_graph_with_sections(&["intro".into(), "drop".into()]);
            graph.timing.bpm_estimate = case.bpm;
            graph.timing.primary_hypothesis_id = Some("scene-grid".into());
            graph.timing.degraded_policy = case.policy;

            let mut session = SessionFile::new("session-1", "0.1.0", "2026-05-30T08:20:00Z");
            session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-intro"));
            session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-intro"));
            session.runtime_state.scene_state.scenes = vec![
                SceneId::from("scene-01-intro"),
                SceneId::from("scene-02-drop"),
            ];
            if case.confirmed_grid {
                session.runtime_state.source_timing.confirmed_grid =
                    Some(crate::session::SourceTimingGridConfirmationState {
                        source_id: graph.source.source_id.clone(),
                        hypothesis_id: Some("scene-grid".into()),
                        confirmed_by_action: crate::ids::ActionId(7),
                        confirmed_at: 1_777_777,
                    });
            }

            let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

            assert_eq!(
                vm.scene.arrangement_contract.readiness, case.expected_readiness,
                "{} readiness",
                case.name
            );
            assert_eq!(
                vm.scene.arrangement_contract.timing_readiness, case.expected_timing,
                "{} timing readiness",
                case.name
            );
            assert_eq!(
                vm.scene
                    .arrangement_contract
                    .can_use_source_locked_scene_movement,
                case.expected_source_locked_scene_movement,
                "{} source-locked scene movement gate",
                case.name
            );
            assert!(vm.scene.arrangement_contract.requires_p012_source_grid_gate);
            assert!(vm
                .scene
                .arrangement_contract
                .requires_p013_musical_quality_gate);
        }

        let session = SessionFile::new("session-1", "0.1.0", "2026-05-30T08:20:00Z");
        let vm = JamViewModel::build(&session, &ActionQueue::new(), None);

        assert_eq!(
            vm.scene.arrangement_contract.readiness,
            ArrangementSceneContractReadinessView::MissingSourceGraph
        );
        assert_eq!(
            vm.scene.arrangement_contract.timing_readiness,
            SourceTimingConsumerReadiness::Unavailable
        );
        assert!(!vm
            .scene
            .arrangement_contract
            .can_use_source_locked_scene_movement);
    }

    #[test]
    fn prefers_contrast_next_scene_when_energy_data_is_available() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: "src-1".into(),
                path: "audio/test.wav".into(),
                content_hash: "graph-1".into(),
                duration_seconds: 48.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-25T00:00:00Z".into(),
                source_hash: "graph-1".into(),
                analysis_seed: 7,
                run_notes: Some("scene-contrast-test".into()),
            },
        );
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-a".into(),
            label_hint: crate::source_graph::SectionLabelHint::Drop,
            start_seconds: 0.0,
            end_seconds: 16.0,
            bar_start: 1,
            bar_end: 8,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-b".into(),
            label_hint: crate::source_graph::SectionLabelHint::Break,
            start_seconds: 16.0,
            end_seconds: 32.0,
            bar_start: 9,
            bar_end: 16,
            energy_class: crate::source_graph::EnergyClass::High,
            confidence: 0.9,
            tags: vec![],
        });
        graph.sections.push(crate::source_graph::Section {
            section_id: "sec-c".into(),
            label_hint: crate::source_graph::SectionLabelHint::Intro,
            start_seconds: 32.0,
            end_seconds: 48.0,
            bar_start: 17,
            bar_end: 24,
            energy_class: crate::source_graph::EnergyClass::Medium,
            confidence: 0.9,
            tags: vec![],
        });

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-25T00:00:00Z");
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.transport.current_scene = Some(SceneId::from("scene-01-drop"));
        session.runtime_state.scene_state.scenes = vec![
            SceneId::from("scene-01-drop"),
            SceneId::from("scene-02-break"),
            SceneId::from("scene-03-intro"),
        ];

        let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

        assert_eq!(vm.scene.next_scene.as_deref(), Some("scene-03-intro"));
        assert_eq!(vm.scene.next_scene_energy.as_deref(), Some("medium"));
        assert_eq!(
            vm.scene.next_scene_policy.map(|policy| (
                policy.direction,
                policy.tr909_intent,
                policy.mc202_intent
            )),
            Some((
                SceneTransitionDirectionView::Drop,
                SceneTransitionLaneIntentView::Release,
                SceneTransitionLaneIntentView::Anchor,
            ))
        );
        assert_eq!(
            next_scene_launch_candidate_with_reason(&session, Some(&graph))
                .map(|candidate| (candidate.scene_id.to_string(), candidate.reason,)),
            Some((
                "scene-03-intro".into(),
                SceneLaunchTargetReason::EnergyContrast,
            ))
        );

        let mut graph_with_unknown_current_energy = graph.clone();
        graph_with_unknown_current_energy.sections[0].energy_class =
            crate::source_graph::EnergyClass::Unknown;
        let vm = JamViewModel::build(
            &session,
            &ActionQueue::new(),
            Some(&graph_with_unknown_current_energy),
        );

        assert_eq!(vm.scene.next_scene.as_deref(), Some("scene-02-break"));
        assert_eq!(
            next_scene_launch_candidate_with_reason(
                &session,
                Some(&graph_with_unknown_current_energy)
            )
            .map(|candidate| candidate.reason),
            Some(SceneLaunchTargetReason::Ordered)
        );
    }

    #[derive(Debug, Deserialize)]
    struct SceneEnergyProjectionFixture {
        name: String,
        section_labels: Vec<String>,
        expected: SceneEnergyProjectionExpected,
    }

    #[derive(Debug, Deserialize)]
    struct SceneEnergyProjectionExpected {
        scenes: Vec<String>,
        active_scene: String,
        current_scene: String,
        active_scene_energy: String,
        #[serde(default)]
        restore_scene: Option<String>,
        #[serde(default)]
        restore_scene_energy: Option<String>,
    }

    #[test]
    fn fixture_backed_scene_energy_projection_holds() {
        let fixtures: Vec<SceneEnergyProjectionFixture> = serde_json::from_str(include_str!(
            "../../../../../riotbox-app/tests/fixtures/scene_regression.json"
        ))
        .expect("parse scene energy projection fixtures");

        for fixture in fixtures {
            let graph = sample_graph_with_sections(&fixture.section_labels);
            let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
            session.runtime_state.scene_state.scenes = fixture
                .expected
                .scenes
                .iter()
                .map(|scene| scene.as_str().into())
                .collect();
            session.runtime_state.scene_state.active_scene =
                Some(fixture.expected.active_scene.as_str().into());
            session.runtime_state.transport.current_scene =
                Some(fixture.expected.current_scene.as_str().into());
            session.runtime_state.scene_state.restore_scene =
                fixture.expected.restore_scene.as_deref().map(Into::into);

            let vm = JamViewModel::build(&session, &ActionQueue::new(), Some(&graph));

            assert_eq!(
                vm.scene.active_scene.as_deref(),
                Some(fixture.expected.active_scene.as_str()),
                "{} active scene drifted",
                fixture.name
            );
            assert_eq!(
                vm.scene.restore_scene.as_deref(),
                fixture.expected.restore_scene.as_deref(),
                "{} restore scene drifted",
                fixture.name
            );
            assert_eq!(
                vm.scene.active_scene_energy.as_deref(),
                Some(fixture.expected.active_scene_energy.as_str()),
                "{} active energy drifted",
                fixture.name
            );
            assert_eq!(
                vm.scene.restore_scene_energy.as_deref(),
                fixture.expected.restore_scene_energy.as_deref(),
                "{} restore energy drifted",
                fixture.name
            );
        }
    }
