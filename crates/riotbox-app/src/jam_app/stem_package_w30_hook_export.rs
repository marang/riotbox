use std::path::{Path, PathBuf};

use riotbox_audio::{
    runtime::{
        AudioRuntimeTimingSnapshot, RuntimeMixRenderPlan, RuntimeMixRenderSequenceStep,
        SourceMonitorRenderState,
        render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report,
    },
    w30::{W30PreviewRenderMode, W30PreviewRenderRouting, W30PreviewSourceProfile},
};
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionStatus, ActionTarget, ActorType,
        CaptureLengthIntent, Quantization, SourceMonitorMode, StemPackageExportBoundary,
        StemPackageExportRole, StemPackageFallbackComparisonPolicy, StemPackageLineagePolicy,
        TargetScope, UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    ids::ActionId,
    queue::QueueEnqueueResult,
    session::{
        CaptureTarget, ExportArtifactRole, ExportArtifactSourceGraphRef,
        ExportArtifactTimingGridRef,
    },
    stem_package_writer::{
        W30_HOOK_LOOP_DURATION_BEATS, W30_HOOK_LOOP_SOURCE_TRANSPORT_START_BEAT,
        W30_HOOK_LOOP_STEM_ROLES,
    },
    style::PerformancePresetId,
    w30_damage_policy::latest_committed_w30_damage_intensity,
};

use super::{
    JamAppError, JamAppState, QueueControlResult,
    persistence::source_graph_hash,
    stem_package_writer::{
        StemPackageRenderedStem, StemPackageW30HookWriterInput, write_w30_hook_stem_package,
    },
};

const SAMPLE_RATE_HZ: u32 = 48_000;
const CHANNEL_COUNT: u16 = 2;
const TRANSPORT_START_BEAT: f64 = W30_HOOK_LOOP_SOURCE_TRANSPORT_START_BEAT as f64;
const DURATION_BEATS: f64 = W30_HOOK_LOOP_DURATION_BEATS as f64;
const PRIMARY_CALLBACK_FRAMES: usize = 128;
const PARITY_CALLBACK_FRAMES: usize = 257;

struct W30HookActionParams {
    destination_root: PathBuf,
}

impl JamAppState {
    pub fn queue_stem_package_export_w30_hook_loop(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
    ) -> QueueControlResult {
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportStemPackage,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::StemPackageExport {
            export_scope: ExportScope::StemPackage,
            export_role: StemPackageExportRole::PackageManifest,
            boundary: StemPackageExportBoundary::W30HookLoopV4,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
            handoff_proof_path: None,
            claimed_stem_roles: W30_HOOK_LOOP_STEM_ROLES.to_vec(),
            lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
            fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "W-30 hook export writes files outside musical undo".into(),
        };
        draft.explanation =
            Some("export the current focused W-30 capture as one semantic hook loop".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => QueueControlResult::AlreadyPending,
            QueueEnqueueResult::Enqueued(_) => {
                self.refresh_view();
                QueueControlResult::Enqueued
            }
        }
    }

    pub fn commit_stem_package_export_w30_hook_loop(
        &mut self,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<riotbox_core::session::ExportReceiptState, JamAppError> {
        let destination_dir = destination_dir.as_ref();
        let action_id = match self.pending_w30_hook_stem_package_action_id() {
            Some(action_id) => action_id,
            None => {
                self.queue_stem_package_export_w30_hook_loop(
                    requested_at,
                    Some(destination_dir.to_string_lossy().into_owned()),
                );
                self.pending_w30_hook_stem_package_action_id()
                    .ok_or_else(|| {
                        JamAppError::InvalidSession(
                        "cannot queue W-30 hook export while another stem-package export is pending"
                            .into(),
                    )
                    })?
            }
        };
        let receipt = match self.prepare_and_write_w30_hook_stem_package(action_id, requested_at) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };
        let result_summary = format!(
            "exported semantic W-30 hook stem_package receipt {} role w30_hook_loop artifacts {}",
            receipt.receipt_id,
            receipt.artifact_set.len()
        );
        self.commit_export_receipt_after_side_effect(
            action_id,
            requested_at,
            receipt.clone(),
            result_summary,
        )?;
        Ok(receipt)
    }

    fn prepare_and_write_w30_hook_stem_package(
        &self,
        action_id: ActionId,
        requested_at: TimestampMs,
    ) -> Result<riotbox_core::session::ExportReceiptState, JamAppError> {
        let params = self.w30_hook_stem_package_action_params(action_id)?;
        let graph = self.source_graph.as_ref().ok_or_else(|| {
            JamAppError::InvalidSession(
                "W-30 hook export requires an active Source Graph identity".into(),
            )
        })?;
        let source_sha256 = normalized_sha256(&graph.source.content_hash);
        if source_sha256.len() != 64 {
            return Err(JamAppError::InvalidSession(
                "W-30 hook export active source SHA-256 is invalid".into(),
            ));
        }
        let graph_hash = source_graph_hash(graph)?;
        let source_graph_ref = self
            .session
            .source_graph_refs
            .iter()
            .find(|graph_ref| {
                graph_ref.source_id == graph.source.source_id
                    && graph_ref.graph_version == graph.graph_version
                    && graph_ref.graph_hash == graph_hash
            })
            .map(|graph_ref| ExportArtifactSourceGraphRef {
                source_id: graph_ref.source_id.clone(),
                graph_version: graph_ref.graph_version,
                graph_hash: graph_ref.graph_hash.clone(),
            })
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "W-30 hook export requires exact active Session Source Graph lineage".into(),
                )
            })?;
        let confirmed_grid = self
            .session
            .runtime_state
            .source_timing
            .confirmed_grid
            .as_ref()
            .filter(|grid| grid.source_id == graph.source.source_id)
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "W-30 hook export requires a confirmed active-source timing grid".into(),
                )
            })?;
        let bpm = self
            .session
            .runtime_state
            .source_timing
            .confirmed_bpm
            .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "W-30 hook export requires a positive committed source BPM".into(),
                )
            })?;
        let timing_grid_ref = ExportArtifactTimingGridRef {
            source_id: confirmed_grid.source_id.clone(),
            hypothesis_id: confirmed_grid.hypothesis_id.clone(),
            confirmed_by_action: confirmed_grid.confirmed_by_action,
            confirmed_at: confirmed_grid.confirmed_at,
        };
        let capture = self.focused_w30_capture().ok_or_else(|| {
            JamAppError::InvalidSession(
                "W-30 hook export requires a current focused W-30 capture".into(),
            )
        })?;
        let CaptureTarget::W30Pad { bank_id, pad_id } =
            capture.assigned_target.as_ref().ok_or_else(|| {
                JamAppError::InvalidSession(
                    "focused W-30 capture is not assigned to a W-30 pad".into(),
                )
            })?
        else {
            return Err(JamAppError::InvalidSession(
                "focused W-30 capture is not assigned to a W-30 pad".into(),
            ));
        };
        if capture
            .source_window
            .as_ref()
            .is_none_or(|window| window.source_id != graph.source.source_id)
        {
            return Err(JamAppError::InvalidSession(
                "focused W-30 capture does not belong to the active source".into(),
            ));
        }

        let render = &self.runtime.w30_preview;
        let pad_playback = render.pad_playback.as_ref();
        if render.mode != W30PreviewRenderMode::LiveRecall
            || render.routing != W30PreviewRenderRouting::MusicBusPreview
            || !matches!(
                render.source_profile,
                Some(
                    W30PreviewSourceProfile::PromotedRecall | W30PreviewSourceProfile::PinnedRecall
                )
            )
            || render.capture_id.as_deref() != Some(capture.capture_id.as_str())
            || pad_playback.is_none()
            || pad_playback.is_some_and(|playback| {
                playback.hook_articulation.is_some()
                    || (playback.playback_rate - 1.0).abs() > f32::EPSILON
                    || playback.reverse
                    || playback.gate_step_fraction.abs() > f32::EPSILON
            })
            || latest_committed_w30_damage_intensity(&self.session, &capture.capture_id)
                .is_some_and(|intensity| intensity > 0.0)
        {
            return Err(JamAppError::InvalidSession(
                "W-30 hook export requires an ordinary promoted or pinned live-recall path".into(),
            ));
        }
        let committed_actions = self
            .session
            .action_log
            .actions
            .iter()
            .filter(|action| action.status == ActionStatus::Committed)
            .collect::<Vec<_>>();
        let [
            timing_action,
            preset_action,
            length_action,
            capture_action,
            promotion_action,
            trigger_action,
        ] = committed_actions.as_slice()
        else {
            return Err(JamAppError::InvalidSession(
                "W-30 hook V4 export requires the exact six-action w30_live_path_render owner"
                    .into(),
            ));
        };
        let session_target = ActionTarget {
            scope: Some(TargetScope::Session),
            ..ActionTarget::default()
        };
        let w30_target = ActionTarget {
            scope: Some(TargetScope::LaneW30),
            bank_id: Some(bank_id.clone()),
            pad_id: Some(pad_id.clone()),
            ..ActionTarget::default()
        };
        let expected_destination = format!("w30:{bank_id}/{pad_id}");
        let expected_capture_id = capture.capture_id.clone();
        let expected_trigger_intensity = if capture.is_pinned { 0.72 } else { 0.84 };
        let timing_params_match = matches!(
            &timing_action.params,
            ActionParams::SourceTimingGrid {
                source_id: Some(source_id),
                hypothesis_id,
                confirmed_bpm: Some(action_bpm),
            } if source_id == &graph.source.source_id
                && hypothesis_id == &confirmed_grid.hypothesis_id
                && (*action_bpm - bpm).abs() <= 0.0001
        );
        let exact_owner_state = confirmed_grid.confirmed_by_action == timing_action.id
            && capture.created_from_action == Some(capture_action.id)
            && self.session.runtime_state.style.active_preset
                == Some(PerformancePresetId::FeralBreakAlphaV2)
            && self.session.runtime_state.capture.length_intent == CaptureLengthIntent::OneBar;
        let exact_owner_actions = [
            (
                *timing_action,
                ActionCommand::SourceTimingConfirmGrid,
                Quantization::Immediate,
            ),
            (
                *preset_action,
                ActionCommand::PresetActivate,
                Quantization::Immediate,
            ),
            (
                *length_action,
                ActionCommand::CaptureSetLength,
                Quantization::Immediate,
            ),
            (
                *capture_action,
                ActionCommand::CaptureBarGroup,
                Quantization::NextBar,
            ),
            (
                *promotion_action,
                ActionCommand::PromoteCaptureToPad,
                Quantization::NextBar,
            ),
            (
                *trigger_action,
                ActionCommand::W30TriggerPad,
                Quantization::NextBeat,
            ),
        ]
        .into_iter()
        .all(|(action, command, quantization)| {
            action.actor == ActorType::User
                && action.command == command
                && action.quantization == quantization
        });
        let timing_target = ActionTarget {
            object_id: confirmed_grid.hypothesis_id.clone(),
            ..session_target.clone()
        };
        let exact_owner_params = timing_params_match
            && timing_action.target == timing_target
            && matches!(
                preset_action.params,
                ActionParams::Preset {
                    preset_id: PerformancePresetId::FeralBreakAlphaV2
                }
            )
            && preset_action.target
                == ActionTarget {
                    object_id: Some(PerformancePresetId::FeralBreakAlphaV2.contract_id().into()),
                    ..session_target.clone()
                }
            && matches!(
                length_action.params,
                ActionParams::CaptureLength {
                    intent: Some(CaptureLengthIntent::OneBar)
                }
            )
            && length_action.target
                == ActionTarget {
                    object_id: Some("capture-length".into()),
                    ..session_target
                }
            && capture_action.params == ActionParams::Capture { bars: None }
            && capture_action.target
                == ActionTarget {
                    scope: Some(TargetScope::LaneW30),
                    ..ActionTarget::default()
                }
            && matches!(
                &promotion_action.params,
                ActionParams::Promotion {
                    capture_id: Some(capture_id),
                    destination: Some(destination),
                } if capture_id == &expected_capture_id && destination == &expected_destination
            )
            && promotion_action.target == w30_target
            && matches!(
                &trigger_action.params,
                ActionParams::Mutation {
                    intensity,
                    target_id: Some(target_id),
                } if (*intensity - expected_trigger_intensity).abs() <= f32::EPSILON
                    && target_id == expected_capture_id.as_str()
            )
            && trigger_action.target == w30_target;
        if !exact_owner_actions || !exact_owner_params || !exact_owner_state {
            return Err(JamAppError::InvalidSession(
                "W-30 hook V4 export requires the exact six-action parameters, targets, and state lineage from w30_live_path_render"
                    .into(),
            ));
        }

        let frame_count =
            (DURATION_BEATS * 60.0 * f64::from(SAMPLE_RATE_HZ) / f64::from(bpm)).round() as usize;
        if frame_count == 0 {
            return Err(JamAppError::InvalidSession(
                "W-30 hook export duration resolved to zero frames".into(),
            ));
        }
        let plan = RuntimeMixRenderPlan {
            transport: AudioRuntimeTimingSnapshot {
                is_transport_running: true,
                tempo_bpm: bpm,
                position_beats: TRANSPORT_START_BEAT,
            },
            tr909_render: Default::default(),
            mc202_render: Default::default(),
            w30_preview_render: render.clone(),
            w30_resample_tap: Default::default(),
            source_monitor_render: SourceMonitorRenderState::control_only(
                SourceMonitorMode::Riotbox,
            ),
        };
        let primary = render_once(&plan, frame_count, PRIMARY_CALLBACK_FRAMES)?;
        let parity = render_once(&plan, frame_count, PARITY_CALLBACK_FRAMES)?;
        if primary.samples != parity.samples {
            return Err(JamAppError::InvalidSession(
                "W-30 hook render changed across callback partitions".into(),
            ));
        }
        for report in [&primary.limiter, &parity.limiter] {
            if report.applied
                || report.limited_sample_count != 0
                || report.pre.clip_count != 0
                || report.post.clip_count != 0
            {
                return Err(JamAppError::InvalidSession(
                    "W-30 hook render clipped or engaged the product limiter".into(),
                ));
            }
            if report.post.active_samples == 0 {
                return Err(JamAppError::InvalidSession(
                    "W-30 hook render is silent".into(),
                ));
            }
        }

        let mut missing_plan = plan.clone();
        missing_plan.w30_preview_render = Default::default();
        let missing = render_once(&missing_plan, frame_count, PRIMARY_CALLBACK_FRAMES)?;
        if missing.limiter.post.active_samples != 0 {
            return Err(JamAppError::InvalidSession(
                "W-30 hook missing-source control emitted audio".into(),
            ));
        }

        let written = write_w30_hook_stem_package(StemPackageW30HookWriterInput {
            created_by_action: action_id,
            created_at: requested_at,
            destination_root: params.destination_root,
            source_sha256,
            stem: StemPackageRenderedStem {
                role: ExportArtifactRole::W30HookLoop,
                samples: primary.samples,
                sample_rate_hz: SAMPLE_RATE_HZ,
                channel_count: CHANNEL_COUNT,
                source_graph_ref,
                timing_grid_ref,
                source_capture_refs: vec![capture.capture_id.clone()],
                lineage_capture_refs: capture.lineage_capture_refs.clone(),
                fallback_reference_identity: format!(
                    "w30-hook-loop:{}#missing-source-silence-v1",
                    capture.capture_id
                ),
            },
        })?;
        Ok(written.receipt)
    }

    fn pending_w30_hook_stem_package_action_id(&self) -> Option<ActionId> {
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportStemPackage
                    && matches!(
                        action.params,
                        ActionParams::StemPackageExport {
                            boundary: StemPackageExportBoundary::W30HookLoopV4,
                            ..
                        }
                    )
            })
            .map(|action| action.id)
    }

    fn w30_hook_stem_package_action_params(
        &self,
        action_id: ActionId,
    ) -> Result<W30HookActionParams, JamAppError> {
        let action = self
            .queue
            .pending_actions()
            .into_iter()
            .find(|action| action.id == action_id)
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "queued W-30 hook stem-package action {action_id} is not pending"
                ))
            })?;
        let ActionParams::StemPackageExport {
            export_scope,
            export_role,
            boundary,
            include_manifest,
            destination_kind,
            destination_path,
            handoff_proof_path,
            claimed_stem_roles,
            lineage_policy,
            fallback_comparison_policy,
        } = &action.params
        else {
            return Err(JamAppError::InvalidSession(format!(
                "queued action {action_id} is not a stem-package export"
            )));
        };
        if *export_scope != ExportScope::StemPackage
            || *export_role != StemPackageExportRole::PackageManifest
            || *boundary != StemPackageExportBoundary::W30HookLoopV4
            || !*include_manifest
            || *destination_kind != ProductExportDestinationKind::LocalArtifactDirectory
            || handoff_proof_path.is_some()
            || claimed_stem_roles != W30_HOOK_LOOP_STEM_ROLES
            || *lineage_policy != StemPackageLineagePolicy::RequireAnyCoreLineage
            || *fallback_comparison_policy != StemPackageFallbackComparisonPolicy::Required
        {
            return Err(JamAppError::InvalidSession(format!(
                "W-30 hook stem-package action {action_id} has unsupported parameters"
            )));
        }
        let destination_root = destination_path
            .as_deref()
            .filter(|path| !path.trim().is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "W-30 hook stem-package destination path is required".into(),
                )
            })?;
        Ok(W30HookActionParams { destination_root })
    }
}

fn render_once(
    plan: &RuntimeMixRenderPlan,
    frame_count: usize,
    callback_frame_count: usize,
) -> Result<riotbox_audio::runtime::RuntimeMixRenderOutput, JamAppError> {
    render_runtime_mix_plan_sequence_realtime_simulation_offline_with_report(
        &[RuntimeMixRenderSequenceStep::new(plan, frame_count)],
        SAMPLE_RATE_HZ,
        CHANNEL_COUNT,
        callback_frame_count,
    )
    .pop()
    .ok_or_else(|| JamAppError::InvalidSession("W-30 hook render produced no output".into()))
}

fn normalized_sha256(value: &str) -> String {
    value
        .trim()
        .strip_prefix("sha256:")
        .unwrap_or(value.trim())
        .to_ascii_lowercase()
}
