use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};

use dawproject::prelude::project::{
    ApplicationType, ArrangementType, AudioType, ChannelType, ClipType, ClipTypeContent, ClipsType,
    ContentType, ContentTypeList, FileReferenceType, LanesType, LanesTypeContent, MixerRoleType,
    ProjectScenesElementType, ProjectStructureElementType, ProjectStructureElementTypeContent,
    RealParameterType, TimeSignatureParameterType, TimeUnitType, TrackType, TransportType,
    UnitType,
};
use dawproject::{Dawproject, DawprojectReader, DawprojectWriter, MetaData, Project};
use riotbox_audio::source_audio::SourceAudioCache;
use riotbox_core::{
    TimestampMs,
    action::{
        ActionCommand, ActionDraft, ActionParams, ActionTarget, ActorType,
        DawSessionExportBoundary, Quantization, TargetScope, UndoPolicy,
    },
    export_readiness::{
        EXPORT_READINESS_CONTRACT_SCHEMA, ExportReadinessContract, ExportReadinessStatus,
        ExportScope, ProductExportBoundary, ProductExportDestinationKind, ProductExportRole,
        STEM_PACKAGE_W30_HOOK_LOOP_PACK_ID, W30_HOOK_DAWPROJECT_PACK_ID,
    },
    ids::{ActionId, ExportReceiptId},
    queue::QueueEnqueueResult,
    session::{
        ExportArrangementPlacementRef, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactRole, ExportArtifactSetEntry, ExportDawTempoMapRef,
        ExportReceiptQaGateResult, ExportReceiptState, SessionFile,
    },
    stem_package_writer::{
        W30_HOOK_LOOP_BEATS_PER_BAR, W30_HOOK_LOOP_DURATION_BEATS, W30_HOOK_LOOP_LOOP_START_BEAT,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::jam_app::{JamAppError, JamAppState, product_export::DawSessionExportQueueResult};

pub const W30_HOOK_DAWPROJECT_ACTION_BOUNDARY_ID: &str = "w30_hook_dawproject_v1";
pub const W30_HOOK_DAWPROJECT_PROOF_SCHEMA: &str = "riotbox.w30_hook_dawproject.v1";
const DAWPROJECT_FORMAT_VERSION: &str = "1.0";
const EMBEDDED_AUDIO_PATH: &str = "audio/w30_hook_loop.wav";
const EMBEDDED_PROOF_PATH: &str = "riotbox-proof.json";
const PROJECT_XML_PATH: &str = "project.xml";
const METADATA_XML_PATH: &str = "metadata.xml";
const DAWPROJECT_BEATS_PER_BAR: i32 = W30_HOOK_LOOP_BEATS_PER_BAR as i32;
const EXPECTED_ARCHIVE_PATHS: [&str; 4] = [
    EMBEDDED_AUDIO_PATH,
    METADATA_XML_PATH,
    PROJECT_XML_PATH,
    EMBEDDED_PROOF_PATH,
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct W30HookDawprojectProof {
    pub schema: String,
    pub source_receipt_id: ExportReceiptId,
    pub source_boundary: String,
    pub source_wav_sha256: String,
    pub embedded_audio_path: String,
    pub embedded_audio_sha256: String,
    pub confirmed_bpm_micros: u64,
    pub start_beat: u32,
    pub duration_beats: u32,
    pub beats_per_bar: u32,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub frame_count: u64,
    pub source_graph_ref: riotbox_core::session::ExportArtifactSourceGraphRef,
    pub timing_grid_ref: riotbox_core::session::ExportArtifactTimingGridRef,
    pub source_capture_refs: Vec<riotbox_core::ids::CaptureId>,
    pub lineage_capture_refs: Vec<riotbox_core::ids::CaptureId>,
}

struct W30HookDawprojectInput {
    source_receipt_id: ExportReceiptId,
    source_wav_sha256: String,
    source_wav_bytes: Vec<u8>,
    bpm: f32,
    sample_rate_hz: u32,
    channel_count: u16,
    frame_count: u64,
    source_graph_ref: riotbox_core::session::ExportArtifactSourceGraphRef,
    timing_grid_ref: riotbox_core::session::ExportArtifactTimingGridRef,
    scene_id: riotbox_core::ids::SceneId,
    source_artifact: ExportArtifactSetEntry,
}

struct ValidatedArchive {
    bytes: Vec<u8>,
    archive_sha256: String,
    project_xml_sha256: String,
    proof_sha256: String,
}

impl JamAppState {
    pub fn queue_w30_hook_dawproject_export(
        &mut self,
        requested_at: TimestampMs,
        session_base_dir: Option<&Path>,
        destination_path: Option<String>,
    ) -> DawSessionExportQueueResult {
        let source_receipt_id = latest_w30_hook_receipt(&self.session)
            .map(|receipt| receipt.receipt_id.as_str().to_owned());
        let mut draft = ActionDraft::new(
            ActorType::User,
            ActionCommand::ExportDawSession,
            Quantization::Immediate,
            ActionTarget {
                scope: Some(TargetScope::Session),
                ..Default::default()
            },
        );
        draft.params = ActionParams::DawSessionExport {
            export_scope: ExportScope::DawSession,
            boundary: DawSessionExportBoundary::W30HookDawprojectV1,
            include_manifest: true,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_path: destination_path.clone(),
            receipt_id: source_receipt_id,
        };
        draft.undo_policy = UndoPolicy::NotUndoable {
            reason: "DAWproject export writes a musician file outside musical undo".into(),
        };
        draft.explanation =
            Some("place the accepted W-30 hook in one vendor-neutral two-bar DAWproject".into());

        match self
            .queue
            .enqueue_if_no_pending_command(draft, requested_at)
        {
            QueueEnqueueResult::AlreadyPending { .. } => {
                DawSessionExportQueueResult::AlreadyPending
            }
            QueueEnqueueResult::Enqueued(action_id) => {
                let preflight = destination_path
                    .as_deref()
                    .ok_or_else(|| {
                        JamAppError::InvalidSession(
                            "W-30 DAWproject export requires an explicit destination file".into(),
                        )
                    })
                    .and_then(|destination| {
                        validate_destination(Path::new(destination))?;
                        prepare_input(&self.session, session_base_dir)
                    });
                match preflight {
                    Ok(_) => {
                        self.refresh_view();
                        DawSessionExportQueueResult::Enqueued { action_id }
                    }
                    Err(error) => {
                        let reason = error.to_string();
                        self.queue.reject(action_id, reason.clone());
                        self.refresh_view();
                        DawSessionExportQueueResult::Rejected { reason }
                    }
                }
            }
        }
    }

    pub fn commit_w30_hook_dawproject_export(
        &mut self,
        session_base_dir: Option<&Path>,
        destination_path: impl AsRef<Path>,
        requested_at: TimestampMs,
    ) -> Result<ExportReceiptState, JamAppError> {
        let destination_path = destination_path.as_ref();
        let action_id = match self.pending_w30_hook_dawproject_action_id(destination_path) {
            Some(action_id) => action_id,
            None => match self.queue_w30_hook_dawproject_export(
                requested_at,
                session_base_dir,
                Some(destination_path.to_string_lossy().into_owned()),
            ) {
                DawSessionExportQueueResult::Enqueued { action_id } => action_id,
                DawSessionExportQueueResult::Rejected { reason } => {
                    return Err(JamAppError::InvalidSession(reason));
                }
                DawSessionExportQueueResult::AlreadyPending => {
                    return Err(JamAppError::InvalidSession(
                        "another DAW session export is already pending".into(),
                    ));
                }
            },
        };

        let written = write_w30_hook_dawproject(
            &self.session,
            session_base_dir,
            destination_path,
            action_id,
            requested_at,
        );
        let receipt = match written {
            Ok(receipt) => receipt,
            Err(error) => {
                self.queue.reject(action_id, error.to_string());
                self.refresh_view();
                return Err(error);
            }
        };
        let result_summary = format!(
            "exported two-bar W-30 DAWproject receipt {} sha256 {}",
            receipt.receipt_id, receipt.export_hash
        );
        if let Err(error) = self.commit_export_receipt_after_side_effect(
            action_id,
            requested_at,
            receipt.clone(),
            result_summary,
        ) {
            if super::product_export::sha256_file(destination_path)
                .is_ok_and(|sha256| sha256 == receipt.export_hash)
            {
                let _ = fs::remove_file(destination_path);
            }
            return Err(error);
        }
        Ok(receipt)
    }

    fn pending_w30_hook_dawproject_action_id(&self, destination: &Path) -> Option<ActionId> {
        let expected_destination = destination.to_string_lossy();
        self.queue
            .pending_actions()
            .into_iter()
            .find(|action| {
                action.command == ActionCommand::ExportDawSession
                    && matches!(
                        &action.params,
                        ActionParams::DawSessionExport {
                            boundary: DawSessionExportBoundary::W30HookDawprojectV1,
                            destination_path,
                            ..
                        } if destination_path.as_deref() == Some(expected_destination.as_ref())
                    )
            })
            .map(|action| action.id)
    }
}

fn write_w30_hook_dawproject(
    session: &SessionFile,
    session_base_dir: Option<&Path>,
    destination: &Path,
    action_id: ActionId,
    created_at: TimestampMs,
) -> Result<ExportReceiptState, JamAppError> {
    validate_destination(destination)?;
    let input = prepare_input(session, session_base_dir)?;
    let proof = build_proof(&input);
    let metadata = build_metadata();
    let project = build_project(&input);
    let archive = build_and_validate_archive(&input, &proof, &metadata, &project)?;
    publish_archive(destination, &archive)?;
    let validation =
        validate_published_archive(destination, &archive, &input, &proof, &metadata, &project);
    if let Err(error) = validation {
        remove_owned_destination(destination, &archive.archive_sha256);
        return Err(error);
    }
    match build_receipt(destination, action_id, created_at, &input, &archive) {
        Ok(receipt) => Ok(receipt),
        Err(error) => {
            remove_owned_destination(destination, &archive.archive_sha256);
            Err(error)
        }
    }
}

fn prepare_input(
    session: &SessionFile,
    session_base_dir: Option<&Path>,
) -> Result<W30HookDawprojectInput, JamAppError> {
    let receipt = latest_w30_hook_receipt(session).ok_or_else(|| {
        JamAppError::InvalidSession(
            "W-30 DAWproject export requires a Session-owned V4 hook receipt".into(),
        )
    })?;
    if receipt.export_scope != ExportScope::StemPackage
        || receipt.export_boundary != ProductExportBoundary::StemPackageW30HookLoopV4
        || receipt.pack_id != STEM_PACKAGE_W30_HOOK_LOOP_PACK_ID
        || receipt.export_role != ProductExportRole::PackageManifest
        || !receipt.stem_package_readiness_report().ready()
    {
        return Err(JamAppError::InvalidSession(
            "W-30 DAWproject export source receipt is not the ready V4 semantic hook".into(),
        ));
    }
    let mut hooks = receipt
        .artifact_set
        .iter()
        .filter(|artifact| artifact.role == ExportArtifactRole::W30HookLoop);
    let hook = hooks.next().ok_or_else(|| {
        JamAppError::InvalidSession("V4 hook receipt has no w30_hook_loop artifact".into())
    })?;
    if hooks.next().is_some() || hook.media_type != ExportArtifactMediaType::AudioWav {
        return Err(JamAppError::InvalidSession(
            "V4 hook receipt must contain exactly one WAV hook artifact".into(),
        ));
    }
    let ExportArtifactLocation::LocalPath { path } = &hook.location else {
        return Err(JamAppError::InvalidSession(
            "V4 hook artifact must be an exact local file".into(),
        ));
    };
    let stored_path = Path::new(path);
    let source_wav_path = if stored_path.is_absolute() {
        stored_path.to_owned()
    } else {
        session_base_dir
            .map(|base| base.join(stored_path))
            .ok_or_else(|| {
                JamAppError::InvalidSession(
                    "relative V4 hook artifact requires the Session base directory".into(),
                )
            })?
    };
    if source_wav_path.file_name().and_then(|name| name.to_str()) != Some("w30_hook_loop.wav") {
        return Err(JamAppError::InvalidSession(
            "V4 hook artifact is not the exact registered w30_hook_loop.wav".into(),
        ));
    }
    let metadata = fs::symlink_metadata(&source_wav_path)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(JamAppError::InvalidSession(
            "V4 hook artifact must be a regular non-symlink file".into(),
        ));
    }
    let source_wav_sha256 = super::product_export::sha256_file(&source_wav_path)?;
    if source_wav_sha256 != hook.sha256 {
        return Err(JamAppError::InvalidSession(format!(
            "V4 hook artifact hash drift: expected {} actual {}",
            hook.sha256, source_wav_sha256
        )));
    }
    let source_wav_bytes = fs::read(&source_wav_path)?;
    let audio = SourceAudioCache::load_pcm_wav(&source_wav_path)
        .map_err(|error| JamAppError::InvalidSession(format!("invalid V4 hook WAV: {error}")))?;
    let bpm = session
        .runtime_state
        .source_timing
        .confirmed_bpm
        .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "W-30 DAWproject export requires a positive confirmed Session BPM".into(),
            )
        })?;
    let source_graph_ref = hook.source_graph_ref.as_ref().ok_or_else(|| {
        JamAppError::InvalidSession("V4 hook artifact is missing Source Graph lineage".into())
    })?;
    let timing_grid_ref = hook.timing_grid_ref.as_ref().ok_or_else(|| {
        JamAppError::InvalidSession("V4 hook artifact is missing timing-grid lineage".into())
    })?;
    let confirmed_grid = session
        .runtime_state
        .source_timing
        .confirmed_grid
        .as_ref()
        .ok_or_else(|| {
            JamAppError::InvalidSession("Session confirmed timing grid is missing".into())
        })?;
    let scene_id = session
        .runtime_state
        .scene_state
        .active_scene
        .clone()
        .filter(|scene_id| {
            !scene_id.as_str().trim().is_empty()
                && session.runtime_state.scene_state.scenes.contains(scene_id)
        })
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "W-30 DAWproject export requires an active Session-owned scene".into(),
            )
        })?;
    if source_graph_ref.source_id != timing_grid_ref.source_id
        || confirmed_grid.source_id != timing_grid_ref.source_id
        || confirmed_grid.hypothesis_id != timing_grid_ref.hypothesis_id
        || confirmed_grid.confirmed_by_action != timing_grid_ref.confirmed_by_action
        || confirmed_grid.confirmed_at != timing_grid_ref.confirmed_at
        || hook.source_capture_refs.is_empty()
    {
        return Err(JamAppError::InvalidSession(
            "V4 hook receipt lineage does not match the confirmed Session owner".into(),
        ));
    }
    if audio.sample_rate != 48_000
        || audio.channel_count != 2
        || hook.sample_rate_hz != Some(audio.sample_rate)
        || hook.channel_count != Some(audio.channel_count)
    {
        return Err(JamAppError::InvalidSession(
            "V4 hook WAV must remain 48 kHz stereo".into(),
        ));
    }
    let expected_frames =
        (f64::from(W30_HOOK_LOOP_DURATION_BEATS) * 60.0 * f64::from(audio.sample_rate)
            / f64::from(bpm))
        .round() as usize;
    if audio.frame_count() != expected_frames {
        return Err(JamAppError::InvalidSession(format!(
            "V4 hook WAV frame count {} does not match eight beats at {bpm} BPM ({expected_frames})",
            audio.frame_count()
        )));
    }

    Ok(W30HookDawprojectInput {
        source_receipt_id: receipt.receipt_id.clone(),
        source_wav_sha256,
        source_wav_bytes,
        bpm,
        sample_rate_hz: audio.sample_rate,
        channel_count: audio.channel_count,
        frame_count: audio.frame_count() as u64,
        source_graph_ref: source_graph_ref.clone(),
        timing_grid_ref: timing_grid_ref.clone(),
        scene_id,
        source_artifact: hook.clone(),
    })
}

fn latest_w30_hook_receipt(session: &SessionFile) -> Option<&ExportReceiptState> {
    session.export_receipts.iter().rev().find(|receipt| {
        receipt.export_scope == ExportScope::StemPackage
            && receipt.export_boundary == ProductExportBoundary::StemPackageW30HookLoopV4
    })
}

fn build_proof(input: &W30HookDawprojectInput) -> W30HookDawprojectProof {
    W30HookDawprojectProof {
        schema: W30_HOOK_DAWPROJECT_PROOF_SCHEMA.into(),
        source_receipt_id: input.source_receipt_id.clone(),
        source_boundary: ProductExportBoundary::StemPackageW30HookLoopV4
            .as_proof_str()
            .into(),
        source_wav_sha256: input.source_wav_sha256.clone(),
        embedded_audio_path: EMBEDDED_AUDIO_PATH.into(),
        embedded_audio_sha256: input.source_wav_sha256.clone(),
        confirmed_bpm_micros: bpm_micros(input.bpm),
        start_beat: W30_HOOK_LOOP_LOOP_START_BEAT,
        duration_beats: W30_HOOK_LOOP_DURATION_BEATS,
        beats_per_bar: W30_HOOK_LOOP_BEATS_PER_BAR,
        sample_rate_hz: input.sample_rate_hz,
        channel_count: input.channel_count,
        frame_count: input.frame_count,
        source_graph_ref: input.source_graph_ref.clone(),
        timing_grid_ref: input.timing_grid_ref.clone(),
        source_capture_refs: input.source_artifact.source_capture_refs.clone(),
        lineage_capture_refs: input.source_artifact.lineage_capture_refs.clone(),
    }
}

fn build_metadata() -> MetaData {
    MetaData {
        title: Some("Riotbox W-30 Hook".into()),
        artist: None,
        album: None,
        original_artist: None,
        composer: None,
        songwriter: None,
        producer: Some("Riotbox".into()),
        arranger: Some("Riotbox".into()),
        year: None,
        genre: Some("rave-punk breakbeat".into()),
        copyright: None,
        website: None,
        comment: Some(
            "Qualified two-bar W-30 hook; embedded audio is byte-identical to its Riotbox V4 receipt."
                .into(),
        ),
    }
}

fn build_project(input: &W30HookDawprojectInput) -> Project {
    const HOOK_TRACK_ID: &str = "riotbox-hook-track";
    const HOOK_CHANNEL_ID: &str = "riotbox-hook-channel";
    const MASTER_TRACK_ID: &str = "riotbox-master-track";
    const MASTER_CHANNEL_ID: &str = "riotbox-master-channel";
    let audio_duration_seconds = input.frame_count as f64 / f64::from(input.sample_rate_hz);
    let hook_track = TrackType {
        name: Some("W-30 Hook".into()),
        color: Some("#ff3b1f".into()),
        comment: Some("Qualified Riotbox semantic hook loop".into()),
        id: Some(HOOK_TRACK_ID.into()),
        content_types: Some(ContentTypeList(vec![ContentType::Audio])),
        loaded: Some(true),
        channel: Some(ChannelType {
            name: Some("W-30 Hook".into()),
            color: Some("#ff3b1f".into()),
            comment: None,
            id: Some(HOOK_CHANNEL_ID.into()),
            audio_channels: Some(i32::from(input.channel_count)),
            destination: Some(MASTER_CHANNEL_ID.into()),
            role: Some(MixerRoleType::Regular),
            solo: Some(false),
            devices: None,
            mute: None,
            pan: None,
            sends: None,
            volume: None,
        }),
        track: Vec::new(),
    };
    let master_track = TrackType {
        name: Some("Master".into()),
        color: None,
        comment: None,
        id: Some(MASTER_TRACK_ID.into()),
        content_types: Some(ContentTypeList(vec![ContentType::Audio])),
        loaded: Some(true),
        channel: Some(ChannelType {
            name: Some("Master".into()),
            color: None,
            comment: None,
            id: Some(MASTER_CHANNEL_ID.into()),
            audio_channels: Some(i32::from(input.channel_count)),
            destination: None,
            role: Some(MixerRoleType::Master),
            solo: Some(false),
            devices: None,
            mute: None,
            pan: None,
            sends: None,
            volume: None,
        }),
        track: Vec::new(),
    };
    let audio = AudioType {
        name: Some("w30_hook_loop.wav".into()),
        color: None,
        comment: Some("Byte-identical Riotbox V4 hook audio".into()),
        id: Some("riotbox-hook-audio".into()),
        time_unit: Some(TimeUnitType::Seconds),
        track: Some(HOOK_TRACK_ID.into()),
        duration: audio_duration_seconds,
        algorithm: None,
        channels: i32::from(input.channel_count),
        sample_rate: input.sample_rate_hz as i32,
        file: FileReferenceType {
            path: EMBEDDED_AUDIO_PATH.into(),
            external: Some(false),
        },
    };
    let hook_clip = ClipType {
        name: Some("W-30 Hook — 2 bars".into()),
        color: Some("#ff3b1f".into()),
        comment: None,
        time: f64::from(W30_HOOK_LOOP_LOOP_START_BEAT),
        duration: Some(f64::from(W30_HOOK_LOOP_DURATION_BEATS)),
        content_time_unit: Some(TimeUnitType::Seconds),
        play_start: Some(0.0),
        play_stop: Some(audio_duration_seconds),
        loop_start: None,
        loop_end: None,
        fade_time_unit: Some(TimeUnitType::Beats),
        fade_in_time: Some(0.0),
        fade_out_time: Some(0.0),
        enable: Some(true),
        reference: None,
        content: Some(ClipTypeContent::Audio(audio)),
    };
    let hook_lanes = LanesType {
        name: Some("W-30 Hook".into()),
        color: Some("#ff3b1f".into()),
        comment: None,
        id: Some("riotbox-hook-lanes".into()),
        time_unit: Some(TimeUnitType::Beats),
        track: Some(HOOK_TRACK_ID.into()),
        content: vec![LanesTypeContent::Clips(ClipsType {
            name: None,
            color: None,
            comment: None,
            id: Some("riotbox-hook-clips".into()),
            time_unit: Some(TimeUnitType::Beats),
            track: Some(HOOK_TRACK_ID.into()),
            clip: vec![hook_clip],
        })],
    };
    let master_lanes = LanesType {
        name: Some("Master".into()),
        color: None,
        comment: None,
        id: Some("riotbox-master-lanes".into()),
        time_unit: Some(TimeUnitType::Beats),
        track: Some(MASTER_TRACK_ID.into()),
        content: vec![LanesTypeContent::Clips(ClipsType {
            name: None,
            color: None,
            comment: None,
            id: Some("riotbox-master-clips".into()),
            time_unit: Some(TimeUnitType::Beats),
            track: Some(MASTER_TRACK_ID.into()),
            clip: Vec::new(),
        })],
    };

    Project {
        version: DAWPROJECT_FORMAT_VERSION.into(),
        application: ApplicationType {
            name: "Riotbox".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        transport: Some(TransportType {
            tempo: Some(RealParameterType {
                name: Some("Tempo".into()),
                color: None,
                comment: Some("Confirmed Riotbox Session tempo".into()),
                id: Some("riotbox-tempo".into()),
                parameter_id: None,
                max: None,
                min: None,
                unit: UnitType::Bpm,
                value: Some(format_bpm(input.bpm)),
            }),
            time_signature: Some(TimeSignatureParameterType {
                name: Some("Time Signature".into()),
                color: None,
                comment: None,
                id: Some("riotbox-time-signature".into()),
                parameter_id: None,
                denominator: 4,
                numerator: DAWPROJECT_BEATS_PER_BAR,
            }),
        }),
        structure: Some(ProjectStructureElementType {
            content: vec![
                ProjectStructureElementTypeContent::Track(hook_track),
                ProjectStructureElementTypeContent::Track(master_track),
            ],
        }),
        arrangement: Some(ArrangementType {
            name: Some("W-30 Hook Arrangement".into()),
            color: None,
            comment: Some("One qualified hook at beat zero for exactly two bars".into()),
            id: Some("riotbox-arrangement".into()),
            lanes: Some(LanesType {
                name: Some("Arrangement".into()),
                color: None,
                comment: None,
                id: Some("riotbox-arrangement-lanes".into()),
                time_unit: Some(TimeUnitType::Beats),
                track: None,
                content: vec![
                    LanesTypeContent::Lanes(hook_lanes),
                    LanesTypeContent::Lanes(master_lanes),
                ],
            }),
            markers: None,
            tempo_automation: None,
            time_signature_automation: None,
        }),
        scenes: Some(ProjectScenesElementType { scene: Vec::new() }),
    }
}

fn build_and_validate_archive(
    input: &W30HookDawprojectInput,
    proof: &W30HookDawprojectProof,
    metadata: &MetaData,
    project: &Project,
) -> Result<ValidatedArchive, JamAppError> {
    let proof_bytes = serde_json::to_vec_pretty(proof)?;
    let cursor = Cursor::new(Vec::new());
    let mut writer = DawprojectWriter::new(cursor)
        .map_err(|error| invalid_dawproject("could not create archive writer", error))?;
    writer
        .write_dawproject(&Dawproject::new(metadata.clone(), project.clone()))
        .map_err(|error| invalid_dawproject("could not serialize DAWproject model", error))?;
    writer
        .write_file(EMBEDDED_AUDIO_PATH, &input.source_wav_bytes)
        .map_err(|error| invalid_dawproject("could not embed W-30 hook audio", error))?;
    writer
        .write_file(EMBEDDED_PROOF_PATH, &proof_bytes)
        .map_err(|error| invalid_dawproject("could not embed Riotbox proof", error))?;
    let bytes = writer
        .finish()
        .map_err(|error| invalid_dawproject("could not finish DAWproject archive", error))?
        .into_inner();
    let (project_xml_sha256, proof_sha256) =
        validate_archive_bytes(&bytes, input, proof, metadata, project)?;
    Ok(ValidatedArchive {
        archive_sha256: sha256_bytes(&bytes),
        bytes,
        project_xml_sha256,
        proof_sha256,
    })
}

fn validate_archive_bytes(
    bytes: &[u8],
    input: &W30HookDawprojectInput,
    proof: &W30HookDawprojectProof,
    expected_metadata: &MetaData,
    expected_project: &Project,
) -> Result<(String, String), JamAppError> {
    let mut reader = DawprojectReader::new(Cursor::new(bytes))
        .map_err(|error| invalid_dawproject("archive is not readable", error))?;
    let paths = reader.file_names().map(str::to_owned).collect::<Vec<_>>();
    if paths != EXPECTED_ARCHIVE_PATHS {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject archive paths differ from the frozen set: {paths:?}"
        )));
    }
    reader
        .read_dawproject()
        .map_err(|error| invalid_dawproject("project or metadata XML did not parse", error))?;
    let parsed = reader.build_dawproject().ok_or_else(|| {
        JamAppError::InvalidSession("DAWproject reader produced no typed model".into())
    })?;
    if &parsed.metadata != expected_metadata || &parsed.project != expected_project {
        return Err(JamAppError::InvalidSession(
            "DAWproject typed read-back differs from the frozen model".into(),
        ));
    }
    let embedded_audio = read_archive_file(&mut reader, EMBEDDED_AUDIO_PATH)?;
    if embedded_audio != input.source_wav_bytes
        || sha256_bytes(&embedded_audio) != input.source_wav_sha256
    {
        return Err(JamAppError::InvalidSession(
            "DAWproject embedded audio is not byte-identical to the V4 hook".into(),
        ));
    }
    let proof_bytes = read_archive_file(&mut reader, EMBEDDED_PROOF_PATH)?;
    let parsed_proof: W30HookDawprojectProof = serde_json::from_slice(&proof_bytes)?;
    if &parsed_proof != proof {
        return Err(JamAppError::InvalidSession(
            "DAWproject embedded Riotbox proof differs from its source contract".into(),
        ));
    }
    let project_xml = read_archive_file(&mut reader, PROJECT_XML_PATH)?;
    Ok((sha256_bytes(&project_xml), sha256_bytes(&proof_bytes)))
}

fn read_archive_file<R: Read + std::io::Seek>(
    reader: &mut DawprojectReader<R>,
    path: &str,
) -> Result<Vec<u8>, JamAppError> {
    let mut file = reader
        .by_name(path)
        .map_err(|error| invalid_dawproject("required archive member is missing", error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn validate_destination(destination: &Path) -> Result<(), JamAppError> {
    if destination
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("dawproject")
    {
        return Err(JamAppError::InvalidSession(
            "W-30 DAWproject destination must end in .dawproject".into(),
        ));
    }
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "DAWproject destination requires an explicit parent directory".into(),
            )
        })?;
    if !parent.is_dir() {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject destination parent does not exist: {}",
            parent.display()
        )));
    }
    if destination.exists() || fs::symlink_metadata(destination).is_ok() {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject destination already exists: {}",
            destination.display()
        )));
    }
    Ok(())
}

fn publish_archive(destination: &Path, archive: &ValidatedArchive) -> Result<(), JamAppError> {
    let parent = destination.parent().ok_or_else(|| {
        JamAppError::InvalidSession("DAWproject destination has no parent".into())
    })?;
    let mut staging = tempfile::Builder::new()
        .prefix(".riotbox-dawproject-staging-")
        .tempfile_in(parent)?;
    staging.write_all(&archive.bytes)?;
    staging.as_file().sync_all()?;
    staging
        .persist_noclobber(destination)
        .map_err(|error| error.error)?;
    Ok(())
}

fn remove_owned_destination(destination: &Path, expected_sha256: &str) {
    if super::product_export::sha256_file(destination).is_ok_and(|sha256| sha256 == expected_sha256)
    {
        let _ = fs::remove_file(destination);
    }
}

fn validate_published_archive(
    destination: &Path,
    expected_archive: &ValidatedArchive,
    input: &W30HookDawprojectInput,
    proof: &W30HookDawprojectProof,
    metadata: &MetaData,
    project: &Project,
) -> Result<(), JamAppError> {
    let bytes = fs::read(destination)?;
    if bytes != expected_archive.bytes || sha256_bytes(&bytes) != expected_archive.archive_sha256 {
        return Err(JamAppError::InvalidSession(
            "published DAWproject bytes differ from the validated staging archive".into(),
        ));
    }
    validate_archive_bytes(&bytes, input, proof, metadata, project)?;
    Ok(())
}

fn build_receipt(
    destination: &Path,
    action_id: ActionId,
    created_at: TimestampMs,
    input: &W30HookDawprojectInput,
    archive: &ValidatedArchive,
) -> Result<ExportReceiptState, JamAppError> {
    let destination_string = destination.to_string_lossy().into_owned();
    let proof_uri = format!("{destination_string}#{EMBEDDED_PROOF_PATH}");
    let project_uri = format!("{destination_string}#{PROJECT_XML_PATH}");
    let audio_uri = format!("{destination_string}#{EMBEDDED_AUDIO_PATH}");
    let contract = ExportReadinessContract {
        schema: EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: W30_HOOK_DAWPROJECT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::DawSession,
        boundary: ProductExportBoundary::DawSessionW30HookDawprojectV1,
        pack_id: W30_HOOK_DAWPROJECT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_artifact: destination_string.clone(),
        source_sha256: input.source_wav_sha256.clone(),
        export_sha256: archive.archive_sha256.clone(),
        normalized_manifest_sha256: archive.project_xml_sha256.clone(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        action_id,
        created_at,
        &contract,
        destination_string.clone(),
        destination_string.clone(),
        Some(destination_string.clone()),
    );
    receipt.artifact_set = vec![
        ExportArtifactSetEntry {
            role: ExportArtifactRole::DawProjectFile,
            location: ExportArtifactLocation::LocalPath {
                path: destination_string,
            },
            media_type: ExportArtifactMediaType::DawProjectZip,
            sha256: archive.archive_sha256.clone(),
            normalized_manifest_hash: Some(archive.project_xml_sha256.clone()),
            source_graph_ref: input.source_artifact.source_graph_ref.clone(),
            timing_grid_ref: input.source_artifact.timing_grid_ref.clone(),
            source_capture_refs: input.source_artifact.source_capture_refs.clone(),
            lineage_capture_refs: input.source_artifact.lineage_capture_refs.clone(),
            fallback_comparison: None,
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        },
        ExportArtifactSetEntry {
            role: ExportArtifactRole::ExportManifest,
            location: ExportArtifactLocation::Uri { uri: project_uri },
            media_type: ExportArtifactMediaType::Xml,
            sha256: archive.project_xml_sha256.clone(),
            normalized_manifest_hash: Some(archive.project_xml_sha256.clone()),
            source_graph_ref: None,
            timing_grid_ref: None,
            source_capture_refs: Vec::new(),
            lineage_capture_refs: Vec::new(),
            fallback_comparison: None,
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        },
        ExportArtifactSetEntry {
            role: ExportArtifactRole::W30HookLoop,
            location: ExportArtifactLocation::Uri { uri: audio_uri },
            media_type: ExportArtifactMediaType::AudioWav,
            sha256: input.source_wav_sha256.clone(),
            normalized_manifest_hash: input.source_artifact.normalized_manifest_hash.clone(),
            source_graph_ref: input.source_artifact.source_graph_ref.clone(),
            timing_grid_ref: input.source_artifact.timing_grid_ref.clone(),
            source_capture_refs: input.source_artifact.source_capture_refs.clone(),
            lineage_capture_refs: input.source_artifact.lineage_capture_refs.clone(),
            fallback_comparison: input.source_artifact.fallback_comparison.clone(),
            audio_metrics: input.source_artifact.audio_metrics.clone(),
            sample_rate_hz: Some(input.sample_rate_hz),
            channel_count: Some(input.channel_count),
            duration_ms: input.source_artifact.duration_ms,
        },
        ExportArtifactSetEntry {
            role: ExportArtifactRole::DawProjectProof,
            location: ExportArtifactLocation::Uri { uri: proof_uri },
            media_type: ExportArtifactMediaType::Json,
            sha256: archive.proof_sha256.clone(),
            normalized_manifest_hash: None,
            source_graph_ref: input.source_artifact.source_graph_ref.clone(),
            timing_grid_ref: input.source_artifact.timing_grid_ref.clone(),
            source_capture_refs: input.source_artifact.source_capture_refs.clone(),
            lineage_capture_refs: input.source_artifact.lineage_capture_refs.clone(),
            fallback_comparison: None,
            audio_metrics: None,
            sample_rate_hz: None,
            channel_count: None,
            duration_ms: None,
        },
    ];
    receipt.qa_gates = vec![ExportReceiptQaGateResult::dawproject_archive_readback()];
    receipt.arrangement_placement_refs = vec![ExportArrangementPlacementRef::scene_range(
        input.scene_id.clone(),
        Some(input.source_graph_ref.source_id.clone()),
        1,
        2,
        u64::from(W30_HOOK_LOOP_LOOP_START_BEAT),
        u64::from(W30_HOOK_LOOP_DURATION_BEATS),
    )];
    let tempo_bpm_micros = u32::try_from(bpm_micros(input.bpm)).map_err(|_| {
        JamAppError::InvalidSession("confirmed Session BPM exceeds DAW tempo-map range".into())
    })?;
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        input.timing_grid_ref.source_id.clone(),
        input.timing_grid_ref.hypothesis_id.clone(),
        input.timing_grid_ref.confirmed_by_action,
        input.timing_grid_ref.confirmed_at,
        u64::from(W30_HOOK_LOOP_LOOP_START_BEAT),
        u64::from(W30_HOOK_LOOP_DURATION_BEATS),
        tempo_bpm_micros,
    ));
    Ok(receipt)
}

fn bpm_micros(bpm: f32) -> u64 {
    (f64::from(bpm) * 1_000_000.0).round() as u64
}

fn format_bpm(bpm: f32) -> String {
    format!("{bpm:.6}")
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn invalid_dawproject(context: &str, error: impl std::fmt::Display) -> JamAppError {
    JamAppError::InvalidSession(format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_archive_paths_are_sorted_for_exact_reader_comparison() {
        assert_eq!(
            EXPECTED_ARCHIVE_PATHS,
            [
                "audio/w30_hook_loop.wav",
                "metadata.xml",
                "project.xml",
                "riotbox-proof.json",
            ]
        );
    }
}
