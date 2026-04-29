    #[test]
    fn builds_minimal_jam_view_model() {
        let mut graph = SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 120.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into(), "section".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 7,
                run_notes: None,
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
        graph.assets.push(Asset {
            asset_id: "asset-a".into(),
            asset_type: AssetType::LoopWindow,
            start_seconds: 0.0,
            end_seconds: 4.0,
            start_bar: 1,
            end_bar: 2,
            confidence: 0.8,
            tags: vec![],
            source_refs: vec![],
        });
        graph.assets.push(Asset {
            asset_id: "asset-hook".into(),
            asset_type: AssetType::HookFragment,
            start_seconds: 4.0,
            end_seconds: 5.0,
            start_bar: 2,
            end_bar: 2,
            confidence: 0.82,
            tags: vec!["hook".into()],
            source_refs: vec!["asset-a".into()],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-loop".into(),
            candidate_type: CandidateType::LoopCandidate,
            asset_ref: "asset-a".into(),
            score: 0.9,
            confidence: 0.9,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-hook".into(),
            candidate_type: CandidateType::HookCandidate,
            asset_ref: "asset-a".into(),
            score: 0.7,
            confidence: 0.8,
            tags: vec![],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.candidates.push(Candidate {
            candidate_id: "cand-capture".into(),
            candidate_type: CandidateType::CaptureCandidate,
            asset_ref: "asset-hook".into(),
            score: 0.86,
            confidence: 0.77,
            tags: vec!["feral".into()],
            constraints: vec![],
            provenance_refs: vec![],
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::SupportsBreakRebuild,
            from_id: "asset-hook".into(),
            to_id: "asset-a".into(),
            weight: 0.8,
            notes: Some("hook supports loop rebuild".into()),
        });
        graph.relationships.push(Relationship {
            relation_type: RelationshipType::HighQuoteRiskWith,
            from_id: "asset-hook".into(),
            to_id: "src-1".into(),
            weight: 0.6,
            notes: Some("recognizable hook".into()),
        });
        graph.analysis_summary = AnalysisSummary {
            overall_confidence: 0.85,
            timing_quality: QualityClass::High,
            section_quality: QualityClass::Medium,
            loop_candidate_count: 1,
            hook_candidate_count: 1,
            break_rebuild_potential: QualityClass::High,
            warnings: vec![],
        };

        let mut session = SessionFile::new("session-1", "0.1.0", "2026-04-12T18:00:00Z");
        session.runtime_state.transport.is_playing = true;
        session.runtime_state.transport.position_beats = 16.0;
        session.runtime_state.scene_state.active_scene = Some(SceneId::from("scene-1"));
        session.runtime_state.scene_state.scenes = vec![SceneId::from("scene-1")];
        session.runtime_state.lane_state.mc202.role = Some("follower".into());
        session.runtime_state.lane_state.w30.active_bank = Some(BankId::from("bank-a"));
        session.runtime_state.lane_state.w30.focused_pad = Some("pad-01".into());
        session.runtime_state.lane_state.w30.last_capture = Some("cap-01".into());
        session.runtime_state.lane_state.tr909.takeover_enabled = true;
        session.runtime_state.lane_state.tr909.takeover_profile =
            Some(Tr909TakeoverProfileState::SceneLockTakeover);
        session.runtime_state.lane_state.tr909.slam_enabled = true;
        session.runtime_state.lane_state.tr909.last_fill_bar = Some(8);
        session.runtime_state.lane_state.tr909.reinforcement_mode =
            Some(Tr909ReinforcementModeState::Takeover);
        session.ghost_state.mode = GhostMode::Assist;
        session.ghost_state.suggestion_history = vec![GhostSuggestionRecord {
            proposal_id: "gp-1".into(),
            summary: "capture next bar".into(),
            accepted: false,
        }];
        session.action_log = ActionLog {
            actions: vec![],
            replay_policy: crate::session::ReplayPolicy::DeterministicPreferred,
        };
        session.source_graph_refs = vec![SourceGraphRef {
            source_id: SourceId::from("src-1"),
            graph_version: crate::source_graph::SourceGraphVersion::V1,
            graph_hash: "graph-1".into(),
            storage_mode: crate::session::GraphStorageMode::Embedded,
            embedded_graph: Some(graph.clone()),
            external_path: None,
            provenance: graph.provenance.clone(),
        }];
        session.runtime_state = RuntimeState {
            transport: session.runtime_state.transport.clone(),
            macro_state: session.runtime_state.macro_state.clone(),
            lane_state: session.runtime_state.lane_state.clone(),
            mixer_state: session.runtime_state.mixer_state.clone(),
            scene_state: session.runtime_state.scene_state.clone(),
            lock_state: session.runtime_state.lock_state.clone(),
            pending_policy: session.runtime_state.pending_policy.clone(),
            undo_state: session.runtime_state.undo_state.clone(),
        };
        session.captures.push(crate::session::CaptureRef {
            capture_id: "cap-01".into(),
            capture_type: crate::session::CaptureType::Pad,
            source_origin_refs: vec!["asset-a".into(), "src-1".into()],
            source_window: None,
            lineage_capture_refs: Vec::new(),
            resample_generation_depth: 0,
            created_from_action: None,
            storage_path: "captures/cap-01.wav".into(),
            assigned_target: Some(crate::session::CaptureTarget::W30Pad {
                bank_id: "bank-a".into(),
                pad_id: "pad-01".into(),
            }),
            is_pinned: false,
            notes: Some("keeper capture".into()),
        });

        let mut queue = ActionQueue::new();
        let mut draft = ActionDraft::new(
            ActorType::Ghost,
            ActionCommand::CaptureNow,
            Quantization::NextBar,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        draft.undo_policy = UndoPolicy::Undoable;
        draft.explanation = Some("capture current break".into());
        queue.enqueue(draft, 100);
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Mc202SetRole,
                Quantization::NextPhrase,
                ActionTarget {
                    scope: Some(TargetScope::LaneMc202),
                    object_id: Some("leader".into()),
                    ..Default::default()
                },
            ),
            101,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30LiveRecall,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-a".into()),
                    pad_id: Some("pad-02".into()),
                    ..Default::default()
                },
            ),
            102,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30SwapBank,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-c".into()),
                    pad_id: Some("pad-01".into()),
                    ..Default::default()
                },
            ),
            103,
        );
        queue.enqueue(
            {
                let mut draft = ActionDraft::new(
                    ActorType::User,
                    ActionCommand::W30BrowseSlicePool,
                    Quantization::NextBeat,
                    ActionTarget {
                        scope: Some(TargetScope::LaneW30),
                        bank_id: Some("bank-a".into()),
                        pad_id: Some("pad-04".into()),
                        ..Default::default()
                    },
                );
                draft.params = ActionParams::Mutation {
                    intensity: 1.0,
                    target_id: Some("cap-02".into()),
                };
                draft
            },
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30ApplyDamageProfile,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-d".into()),
                    pad_id: Some("pad-03".into()),
                    ..Default::default()
                },
            ),
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30TriggerPad,
                Quantization::NextBeat,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-a".into()),
                    pad_id: Some("pad-03".into()),
                    ..Default::default()
                },
            ),
            103,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::W30StepFocus,
                Quantization::NextBeat,
                ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    bank_id: Some("bank-c".into()),
                    pad_id: Some("pad-01".into()),
                    ..Default::default()
                },
            ),
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Tr909Release,
                Quantization::NextPhrase,
                ActionTarget {
                    scope: Some(TargetScope::LaneTr909),
                    ..Default::default()
                },
            ),
            104,
        );
        queue.enqueue(
            ActionDraft::new(
                ActorType::User,
                ActionCommand::Tr909FillNext,
                Quantization::NextBar,
                ActionTarget {
                    scope: Some(TargetScope::LaneTr909),
                    ..Default::default()
                },
            ),
            105,
        );
        let mut resample_draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::PromoteResample,
            Quantization::NextPhrase,
            ActionTarget {
                scope: Some(TargetScope::LaneW30),
                ..Default::default()
            },
        );
        resample_draft.params = crate::action::ActionParams::Promotion {
            capture_id: Some("cap-01".into()),
            destination: Some("w30:resample".into()),
        };
        queue.enqueue(resample_draft, 106);

        let vm = JamViewModel::build(&session, &queue, Some(&graph));

        assert!(vm.transport.is_playing);
        assert_eq!(vm.source.loop_candidate_count, 1);
        assert_eq!(vm.source.hook_candidate_count, 1);
        assert_eq!(vm.source.feral_scorecard.readiness, "ready");
        assert_eq!(vm.source.feral_scorecard.break_rebuild_potential, "high");
        assert_eq!(vm.source.feral_scorecard.hook_fragment_count, 1);
        assert_eq!(vm.source.feral_scorecard.break_support_count, 1);
        assert_eq!(vm.source.feral_scorecard.quote_risk_count, 1);
        assert_eq!(vm.source.feral_scorecard.capture_candidate_count, 1);
        assert_eq!(
            vm.source.feral_scorecard.top_reason,
            "use capture before quoting"
        );
        assert_eq!(
            vm.source.feral_scorecard.warnings,
            vec!["quote risk 1".to_string()]
        );
        assert_eq!(vm.scene.scene_count, 1);
        assert_eq!(vm.scene.restore_scene, None);
        assert_eq!(
            vm.scene.scene_jump_availability,
            SceneJumpAvailabilityView::WaitingForMoreScenes
        );
        assert_eq!(vm.scene.active_scene_energy.as_deref(), Some("high"));
        assert_eq!(vm.scene.restore_scene_energy, None);
        assert_eq!(vm.capture.capture_count, 1);
        assert_eq!(vm.capture.pinned_capture_count, 0);
        assert_eq!(vm.capture.promoted_capture_count, 1);
        assert_eq!(vm.capture.unassigned_capture_count, 0);
        assert_eq!(vm.capture.pending_capture_count, 2);
        assert_eq!(vm.capture.last_capture_id.as_deref(), Some("cap-01"));
        assert_eq!(
            vm.capture.last_capture_target.as_deref(),
            Some("pad bank-a/pad-01")
        );
        assert_eq!(
            vm.capture.last_capture_target_kind,
            Some(CaptureTargetKindView::W30Pad)
        );
        assert_eq!(
            vm.capture.last_capture_handoff_readiness,
            Some(CaptureHandoffReadinessView::Fallback)
        );
        assert_eq!(
            vm.capture.last_promotion_result.as_deref(),
            Some("promoted to pad bank-a/pad-01")
        );
        assert_eq!(
            vm.capture.latest_w30_promoted_capture_label.as_deref(),
            Some("cap-01 -> bank-a/pad-01")
        );
        assert_eq!(
            vm.capture.recent_capture_rows,
            vec!["cap-01 | bank-a/pad-01 | 2 origins"]
        );
        assert_eq!(
            vm.capture.latest_capture_provenance_lines,
            vec![
                "file captures/cap-01.wav",
                "from action manual or unknown",
                "origins asset-a, src-1",
            ]
        );
        assert!(vm.capture.pinned_capture_ids.is_empty());
        assert_eq!(vm.capture.pending_capture_items.len(), 2);
        assert_eq!(vm.capture.pending_capture_items[0].command, "capture.now");
        assert_eq!(vm.capture.pending_capture_items[0].target, "lanew30");
        assert_eq!(
            vm.capture.pending_capture_items[0].explanation.as_deref(),
            Some("capture current break")
        );
        assert_eq!(
            vm.capture.pending_capture_items[1].command,
            "promote.resample"
        );
        assert_eq!(vm.capture.pending_capture_items[1].target, "lanew30");
        assert_eq!(vm.lanes.mc202_pending_role.as_deref(), Some("leader"));
        assert!(!vm.lanes.mc202_pending_follower_generation);
        assert!(!vm.lanes.mc202_pending_answer_generation);
        assert_eq!(vm.lanes.mc202_phrase_ref, None);
        assert_eq!(vm.lanes.w30_active_bank.as_deref(), Some("bank-a"));
        assert_eq!(vm.lanes.w30_focused_pad.as_deref(), Some("pad-01"));
        assert_eq!(
            vm.lanes.w30_pending_trigger_target.as_deref(),
            Some("bank-a/pad-03")
        );
        assert_eq!(
            vm.lanes.w30_pending_recall_target.as_deref(),
            Some("bank-a/pad-02")
        );
        assert_eq!(
            vm.lanes.w30_pending_bank_swap_target.as_deref(),
            Some("bank-c/pad-01")
        );
        assert_eq!(
            vm.lanes.w30_pending_slice_pool_target.as_deref(),
            Some("bank-a/pad-04")
        );
        assert_eq!(
            vm.lanes.w30_pending_slice_pool_capture_id.as_deref(),
            Some("cap-02")
        );
        assert_eq!(
            vm.lanes.w30_pending_slice_pool_reason.as_deref(),
            Some("cycle")
        );
        assert_eq!(
            vm.lanes.w30_pending_damage_profile_target.as_deref(),
            Some("bank-d/pad-03")
        );
        assert_eq!(vm.lanes.w30_pending_audition_target, None);
        assert_eq!(
            vm.lanes.w30_pending_focus_step_target.as_deref(),
            Some("bank-c/pad-01")
        );
        assert_eq!(
            vm.lanes.w30_pending_resample_capture_id.as_deref(),
            Some("cap-01")
        );
        assert!(vm.lanes.tr909_takeover_enabled);
        assert_eq!(vm.lanes.tr909_takeover_pending_target, Some(false));
        assert_eq!(vm.lanes.tr909_takeover_pending_profile, None);
        assert_eq!(
            vm.lanes.tr909_takeover_profile,
            Some(Tr909TakeoverProfileState::SceneLockTakeover)
        );
        assert!(vm.lanes.tr909_fill_armed_next_bar);
        assert_eq!(vm.lanes.tr909_last_fill_bar, Some(8));
        assert_eq!(
            vm.lanes.tr909_reinforcement_mode,
            Some(Tr909ReinforcementModeState::Takeover)
        );
        assert_eq!(vm.pending_actions.len(), 11);
        assert_eq!(vm.ghost.mode, "assist");
    }

