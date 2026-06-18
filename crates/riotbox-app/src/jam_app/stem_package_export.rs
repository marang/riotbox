use std::path::{Path, PathBuf};

use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType, Quantization,
        StemPackageExportBoundary, StemPackageExportRole, StemPackageFallbackComparisonPolicy,
        StemPackageLineagePolicy, TargetScope, UndoPolicy,
    },
    export_readiness::{ExportScope, ProductExportDestinationKind},
    ids::ActionId,
    queue::QueueEnqueueResult,
    session::{
        ActionCommitRecord, ExportArtifactFallbackComparisonEvidence,
        ExportArtifactFallbackComparisonKind, ExportArtifactRole, ExportArtifactSourceGraphRef,
        ExportReceiptState,
    },
    source_graph::SourceGraphVersion,
    transport::CommitBoundaryState,
};

use super::{
    JamAppError, JamAppState, QueueControlResult,
    helpers::update_logged_action_result,
    stem_package_writer::{
        StemPackageFixtureStem, StemPackageFixtureWriterInput, write_ci_safe_stem_package_fixture,
    },
};

const LOCAL_CI_STEM_SAMPLE_RATE: u32 = 48_000;
const LOCAL_CI_STEM_CHANNELS: u16 = 2;

struct LocalCiStemPackageActionParams {
    destination_root: PathBuf,
    claimed_stem_roles: Vec<ExportArtifactRole>,
}

impl JamAppState {
    pub fn queue_stem_package_export_local_ci_package(
        &mut self,
        requested_at: TimestampMs,
        destination_path: Option<String>,
        claimed_stem_roles: Vec<ExportArtifactRole>,
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
            boundary: StemPackageExportBoundary::LocalCiPackageV1,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_path,
            claimed_stem_roles,
            lineage_policy: StemPackageLineagePolicy::RequireAnyCoreLineage,
            fallback_comparison_policy: StemPackageFallbackComparisonPolicy::Required,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "stem package export writes files outside musical undo".into(),
        };
        draft.explanation = Some("export local CI stem package proof".into());

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

    pub fn commit_stem_package_export_local_ci_package(
        &mut self,
        destination_dir: impl AsRef<Path>,
        requested_at: TimestampMs,
        claimed_stem_roles: Vec<ExportArtifactRole>,
    ) -> Result<ExportReceiptState, JamAppError> {
        let destination_dir = destination_dir.as_ref();
        let action_id = self
            .pending_stem_package_export_action_id()
            .unwrap_or_else(|| {
                self.queue_stem_package_export_local_ci_package(
                    requested_at,
                    Some(destination_dir.to_string_lossy().into_owned()),
                    claimed_stem_roles,
                );
                self.pending_stem_package_export_action_id()
                    .expect("queued stem package export action should be pending")
            });
        let params = match self.local_ci_stem_package_action_params(action_id) {
            Ok(params) => params,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };

        let receipt = match write_local_ci_stem_package_for_roles(
            action_id,
            requested_at,
            params.destination_root,
            params.claimed_stem_roles.clone(),
        ) {
            Ok(written) => written,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };
        let role_summary = params
            .claimed_stem_roles
            .iter()
            .map(|role| stem_role_summary_label(*role))
            .collect::<Vec<_>>()
            .join(",");
        let result_summary = format!(
            "exported stem_package receipt {} roles {} artifacts {}",
            receipt.receipt_id,
            role_summary,
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

    fn pending_stem_package_export_action_id(&self) -> Option<ActionId> {
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| action.command == ActionCommand::ExportStemPackage)
            .map(|action| action.id)
    }

    fn local_ci_stem_package_action_params(
        &self,
        action_id: ActionId,
    ) -> Result<LocalCiStemPackageActionParams, JamAppError> {
        let action = self
            .queue
            .pending_actions()
            .into_iter()
            .find(|action| action.id == action_id)
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "queued stem package export action {action_id} is not pending"
                ))
            })?;
        let ActionParams::StemPackageExport {
            export_scope,
            export_role,
            boundary,
            include_manifest,
            destination_kind,
            destination_path,
            claimed_stem_roles,
            lineage_policy,
            fallback_comparison_policy,
        } = &action.params
        else {
            return Err(JamAppError::InvalidSession(format!(
                "queued action {action_id} is not a stem package export"
            )));
        };

        if *export_scope != ExportScope::StemPackage {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} has non-stem export scope {export_scope:?}"
            )));
        }
        if *export_role != StemPackageExportRole::PackageManifest {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} has unsupported export role {export_role:?}"
            )));
        }
        if *boundary != StemPackageExportBoundary::LocalCiPackageV1 {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} has unsupported boundary {boundary:?}"
            )));
        }
        if !*include_manifest {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} must include a manifest"
            )));
        }
        if *destination_kind != ProductExportDestinationKind::LocalArtifactDirectory {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} has unsupported destination {destination_kind:?}"
            )));
        }
        if *lineage_policy != StemPackageLineagePolicy::RequireAnyCoreLineage {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} has unsupported lineage policy {lineage_policy:?}"
            )));
        }
        if *fallback_comparison_policy != StemPackageFallbackComparisonPolicy::Required {
            return Err(JamAppError::InvalidSession(format!(
                "stem package action {action_id} has unsupported fallback policy {fallback_comparison_policy:?}"
            )));
        }
        let destination_root = destination_path
            .as_ref()
            .filter(|path| !path.trim().is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "stem package action {action_id} is missing a local destination path"
                ))
            })?;

        Ok(LocalCiStemPackageActionParams {
            destination_root,
            claimed_stem_roles: claimed_stem_roles.clone(),
        })
    }

    fn commit_export_receipt_after_side_effect(
        &mut self,
        action_id: ActionId,
        requested_at: TimestampMs,
        receipt: ExportReceiptState,
        result_summary: String,
    ) -> Result<(), JamAppError> {
        let boundary = CommitBoundaryState {
            kind: riotbox_core::action::CommitBoundary::Immediate,
            beat_index: self.runtime.transport.beat_index,
            bar_index: self.runtime.transport.bar_index,
            phrase_index: self.runtime.transport.phrase_index,
            scene_id: self.runtime.transport.current_scene.clone(),
        };
        let committed_ref = self
            .queue
            .commit_pending_after_side_effect(
                action_id,
                boundary.clone(),
                requested_at,
                result_summary.clone(),
            )
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "queued export action {action_id} was not ready to commit"
                ))
            })?;
        let action = self
            .queue
            .history_action(committed_ref.action_id)
            .cloned()
            .ok_or_else(|| {
                JamAppError::InvalidSession(format!(
                    "committed export action {} missing from queue history",
                    committed_ref.action_id
                ))
            })?;

        self.session.action_log.actions.push(action);
        self.session
            .action_log
            .commit_records
            .push(ActionCommitRecord {
                action_id,
                boundary: committed_ref.boundary,
                commit_sequence: committed_ref.commit_sequence,
                committed_at: requested_at,
                mc202_source_phrase_plan: None,
            });
        self.session.export_receipts.push(receipt);
        update_logged_action_result(&mut self.session, action_id, result_summary);
        self.runtime.last_commit_boundary = Some(boundary);
        self.refresh_view();

        Ok(())
    }
}

fn stem_role_summary_label(role: ExportArtifactRole) -> &'static str {
    match role {
        ExportArtifactRole::StemDrums => "stem_drums",
        ExportArtifactRole::StemBass => "stem_bass",
        ExportArtifactRole::StemMusic => "stem_music",
        ExportArtifactRole::StemVocals => "stem_vocals",
        _ => "non_stem",
    }
}

fn write_local_ci_stem_package_for_roles(
    created_by_action: ActionId,
    created_at: TimestampMs,
    destination_root: PathBuf,
    claimed_stem_roles: Vec<ExportArtifactRole>,
) -> Result<ExportReceiptState, JamAppError> {
    let stems = claimed_stem_roles
        .into_iter()
        .map(local_ci_stem_for_role)
        .collect::<Result<Vec<_>, _>>()?;
    let written = write_ci_safe_stem_package_fixture(StemPackageFixtureWriterInput {
        created_by_action,
        created_at,
        destination_root,
        stems,
    })?;
    Ok(written.receipt)
}

fn local_ci_stem_for_role(role: ExportArtifactRole) -> Result<StemPackageFixtureStem, JamAppError> {
    let samples = match role {
        ExportArtifactRole::StemDrums => local_ci_drums_samples(),
        ExportArtifactRole::StemBass => local_ci_bass_samples(),
        other => {
            return Err(JamAppError::InvalidSession(format!(
                "unsupported local CI stem package role: {other:?}"
            )));
        }
    };
    let role_label = match role {
        ExportArtifactRole::StemDrums => "drums",
        ExportArtifactRole::StemBass => "bass",
        _ => "stem",
    };
    Ok(StemPackageFixtureStem {
        role,
        samples,
        source_graph_ref: local_ci_source_graph_ref(),
        fallback_comparison: local_ci_fallback_comparison(role_label),
    })
}

fn local_ci_source_graph_ref() -> ExportArtifactSourceGraphRef {
    ExportArtifactSourceGraphRef {
        source_id: riotbox_core::ids::SourceId::new("source-stem-package-ci-fixture"),
        graph_version: SourceGraphVersion::V1,
        graph_hash: "stem-package-ci-fixture-graph-sha".into(),
    }
}

fn local_ci_fallback_comparison(role: &str) -> ExportArtifactFallbackComparisonEvidence {
    ExportArtifactFallbackComparisonEvidence {
        comparison_kind: ExportArtifactFallbackComparisonKind::SourceVsFallback,
        reference_identity: format!("fallback://stem-package-ci-fixture/{role}"),
        rms_difference_micros: Some(180_000),
        normalized_correlation_micros: Some(260_000),
    }
}

fn local_ci_drums_samples() -> Vec<f32> {
    let frames = 48_000;
    let mut samples = vec![0.0; frames * usize::from(LOCAL_CI_STEM_CHANNELS)];
    for frame in (0..frames).step_by(6_000) {
        for transient in 0..96 {
            let amp = 0.92 * (1.0 - transient as f32 / 96.0);
            let index = (frame + transient) * usize::from(LOCAL_CI_STEM_CHANNELS);
            samples[index] = amp;
            samples[index + 1] = -amp * 0.75;
        }
    }
    samples
}

fn local_ci_bass_samples() -> Vec<f32> {
    let frames = 48_000;
    let mut samples = Vec::with_capacity(frames * usize::from(LOCAL_CI_STEM_CHANNELS));
    for frame in 0..frames {
        let phase = frame as f32 / LOCAL_CI_STEM_SAMPLE_RATE as f32;
        let amp = (phase * 55.0 * std::f32::consts::TAU).sin() * 0.44;
        samples.push(amp);
        samples.push(amp * 0.96);
    }
    samples
}
