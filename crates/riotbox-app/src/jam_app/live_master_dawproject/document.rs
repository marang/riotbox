//! Pure typed DAWproject and receipt construction from an admitted master.

use dawproject::prelude::project::{
    ApplicationType, ArrangementType, AudioType, ChannelType, ClipType, ClipTypeContent, ClipsType,
    ContentType, ContentTypeList, FileReferenceType, LanesType, LanesTypeContent, MixerRoleType,
    ProjectScenesElementType, ProjectStructureElementType, ProjectStructureElementTypeContent,
    RealParameterType, TimeSignatureParameterType, TimeUnitType, TrackType, TransportType,
    UnitType,
};
use dawproject::{MetaData, Project};
use std::path::Path;

use super::{
    EMBEDDED_AUDIO_PATH, LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA, input::LiveMasterDawprojectInput,
};
use crate::jam_app::{
    JamAppError, LiveMasterRecordingProof,
    dawproject_archive::{DAWPROJECT_PROOF_PATH, PublishedDawprojectArchive},
};
use riotbox_core::{
    TimestampMs,
    export_readiness::{
        ExportReadinessContract, ExportReadinessStatus, ExportScope,
        LIVE_MASTER_DAWPROJECT_PACK_ID, ProductExportBoundary, ProductExportRole,
    },
    ids::{ActionId, CaptureId, ExportReceiptId, SceneId},
    session::{
        ExportArrangementPlacementRef, ExportArtifactLocation, ExportArtifactMediaType,
        ExportArtifactRole, ExportArtifactSetEntry, ExportArtifactSourceGraphRef,
        ExportArtifactTimingGridRef, ExportDawTempoMapRef, ExportReceiptQaGateResult,
        ExportReceiptState,
    },
};
use serde::{Deserialize, Serialize};
const DAWPROJECT_FORMAT_VERSION: &str = "1.0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveMasterDawprojectProof {
    pub schema: String,
    pub source_receipt_id: ExportReceiptId,
    pub source_boundary: String,
    pub source_action_id: ActionId,
    pub session_id: String,
    pub source_proof_sha256: String,
    pub source_wav_sha256: String,
    pub embedded_audio_path: String,
    pub embedded_audio_sha256: String,
    pub confirmed_bpm_micros: u64,
    pub start_beat: u32,
    pub duration_beats: u32,
    pub beats_per_bar: u8,
    pub bar_grid_anchor_position_microbeats: u64,
    pub requested_start_position_microbeats: u64,
    pub captured_start_position_microbeats: u64,
    pub captured_end_position_microbeats: u64,
    pub beat_span_per_frame_nanobeats: u64,
    pub host: String,
    pub device: String,
    pub device_sample_format: String,
    pub wav_sample_format: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub frame_count: u64,
    pub callback_count: u64,
    pub peak_amplitude_micros: u32,
    pub rms_amplitude_micros: u32,
    pub clip_count: u64,
    pub sample_payload_sha256: String,
    pub scene_id: SceneId,
    pub source_graph_ref: Option<ExportArtifactSourceGraphRef>,
    pub timing_grid_ref: Option<ExportArtifactTimingGridRef>,
    pub source_capture_refs: Vec<CaptureId>,
    pub lineage_capture_refs: Vec<CaptureId>,
}

pub(super) fn build_proof(input: &LiveMasterDawprojectInput) -> LiveMasterDawprojectProof {
    let source: &LiveMasterRecordingProof = &input.proof;
    LiveMasterDawprojectProof {
        schema: LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA.into(),
        source_receipt_id: input.source_receipt_id.clone(),
        source_boundary: ProductExportBoundary::LiveRecordingRuntimeMasterBarWindowV2
            .as_proof_str()
            .into(),
        source_action_id: source.action_id,
        session_id: source.session_id.clone(),
        source_proof_sha256: input.source_proof_sha256.clone(),
        source_wav_sha256: input.source_wav_sha256.clone(),
        embedded_audio_path: EMBEDDED_AUDIO_PATH.into(),
        embedded_audio_sha256: input.source_wav_sha256.clone(),
        confirmed_bpm_micros: source.confirmed_bpm_micros,
        start_beat: 0,
        duration_beats: source.duration_beats,
        beats_per_bar: source.beats_per_bar,
        bar_grid_anchor_position_microbeats: source.bar_grid_anchor_position_microbeats,
        requested_start_position_microbeats: source.requested_start_position_microbeats,
        captured_start_position_microbeats: source.captured_start_position_microbeats,
        captured_end_position_microbeats: source.captured_end_position_microbeats,
        beat_span_per_frame_nanobeats: source.beat_span_per_frame_nanobeats,
        host: source.host.clone(),
        device: source.device.clone(),
        device_sample_format: source.device_sample_format.clone(),
        wav_sample_format: source.wav_sample_format.clone(),
        sample_rate_hz: source.sample_rate_hz,
        channel_count: source.channel_count,
        frame_count: source.frame_count,
        callback_count: source.callback_count,
        peak_amplitude_micros: source.peak_amplitude_micros,
        rms_amplitude_micros: source.rms_amplitude_micros,
        clip_count: source.clip_count,
        sample_payload_sha256: source.sample_payload_sha256.clone(),
        scene_id: source.scene_id.clone(),
        source_graph_ref: source.source_graph_ref.clone(),
        timing_grid_ref: source.timing_grid_ref.clone(),
        source_capture_refs: source.source_capture_refs.clone(),
        lineage_capture_refs: source.lineage_capture_refs.clone(),
    }
}

pub(super) fn build_metadata() -> MetaData {
    MetaData {
        title: Some("Riotbox Live Master".into()),
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
            "Recorded Riotbox live master; embedded audio is byte-identical to its V2 receipt."
                .into(),
        ),
    }
}

pub(super) fn build_project(proof: &LiveMasterRecordingProof) -> Project {
    const TRACK: &str = "riotbox-live-master-track";
    const CHANNEL: &str = "riotbox-live-master-channel";
    const MASTER_TRACK: &str = "riotbox-master-track";
    const MASTER_CHANNEL: &str = "riotbox-master-channel";
    let seconds = proof.frame_count as f64 / f64::from(proof.sample_rate_hz);
    let track = TrackType {
        name: Some("Live Master".into()),
        color: Some("#ff3b1f".into()),
        comment: Some("Byte-identical Riotbox live runtime-master capture".into()),
        id: Some(TRACK.into()),
        content_types: Some(ContentTypeList(vec![ContentType::Audio])),
        loaded: Some(true),
        channel: Some(ChannelType {
            name: Some("Live Master".into()),
            color: Some("#ff3b1f".into()),
            comment: None,
            id: Some(CHANNEL.into()),
            audio_channels: Some(i32::from(proof.channel_count)),
            destination: Some(MASTER_CHANNEL.into()),
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
    let master = TrackType {
        name: Some("Master".into()),
        color: None,
        comment: None,
        id: Some(MASTER_TRACK.into()),
        content_types: Some(ContentTypeList(vec![ContentType::Audio])),
        loaded: Some(true),
        channel: Some(ChannelType {
            name: Some("Master".into()),
            color: None,
            comment: None,
            id: Some(MASTER_CHANNEL.into()),
            audio_channels: Some(i32::from(proof.channel_count)),
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
    let clip = ClipType {
        name: Some("Live Master — 2 bars".into()),
        color: Some("#ff3b1f".into()),
        comment: None,
        time: 0.0,
        duration: Some(f64::from(proof.duration_beats)),
        content_time_unit: Some(TimeUnitType::Seconds),
        play_start: Some(0.0),
        play_stop: Some(seconds),
        loop_start: None,
        loop_end: None,
        fade_time_unit: Some(TimeUnitType::Beats),
        fade_in_time: Some(0.0),
        fade_out_time: Some(0.0),
        enable: Some(true),
        reference: None,
        content: Some(ClipTypeContent::Audio(AudioType {
            name: Some("live_master.wav".into()),
            color: None,
            comment: Some("Byte-identical recorded live master".into()),
            id: Some("riotbox-live-master-audio".into()),
            time_unit: Some(TimeUnitType::Seconds),
            track: Some(TRACK.into()),
            duration: seconds,
            algorithm: None,
            channels: i32::from(proof.channel_count),
            sample_rate: proof.sample_rate_hz as i32,
            file: FileReferenceType {
                path: EMBEDDED_AUDIO_PATH.into(),
                external: Some(false),
            },
        })),
    };
    let lanes = |name: &str, id: &str, track_id: Option<String>, clips: Vec<ClipType>| LanesType {
        name: Some(name.into()),
        color: None,
        comment: None,
        id: Some(id.into()),
        time_unit: Some(TimeUnitType::Beats),
        track: track_id,
        content: vec![LanesTypeContent::Clips(ClipsType {
            name: None,
            color: None,
            comment: None,
            id: Some(format!("{id}-clips")),
            time_unit: Some(TimeUnitType::Beats),
            track: None,
            clip: clips,
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
                comment: Some("Recorded Riotbox live-master tempo".into()),
                id: Some("riotbox-tempo".into()),
                parameter_id: None,
                max: None,
                min: None,
                unit: UnitType::Bpm,
                value: Some(format!(
                    "{:.6}",
                    proof.confirmed_bpm_micros as f64 / 1_000_000.0
                )),
            }),
            time_signature: Some(TimeSignatureParameterType {
                name: Some("Time Signature".into()),
                color: None,
                comment: None,
                id: Some("riotbox-time-signature".into()),
                parameter_id: None,
                denominator: 4,
                numerator: i32::from(proof.beats_per_bar),
            }),
        }),
        structure: Some(ProjectStructureElementType {
            content: vec![
                ProjectStructureElementTypeContent::Track(track),
                ProjectStructureElementTypeContent::Track(master),
            ],
        }),
        arrangement: Some(ArrangementType {
            name: Some("Live Master Arrangement".into()),
            color: None,
            comment: Some("One recorded two-bar live master at beat zero".into()),
            id: Some("riotbox-arrangement".into()),
            lanes: Some(LanesType {
                name: Some("Arrangement".into()),
                color: None,
                comment: None,
                id: Some("riotbox-arrangement-lanes".into()),
                time_unit: Some(TimeUnitType::Beats),
                track: None,
                content: vec![
                    LanesTypeContent::Lanes(lanes(
                        "Live Master",
                        "riotbox-live-master-lanes",
                        Some(TRACK.into()),
                        vec![clip],
                    )),
                    LanesTypeContent::Lanes(lanes(
                        "Master",
                        "riotbox-master-lanes",
                        Some(MASTER_TRACK.into()),
                        Vec::new(),
                    )),
                ],
            }),
            markers: None,
            tempo_automation: None,
            time_signature_automation: None,
        }),
        scenes: Some(ProjectScenesElementType { scene: Vec::new() }),
    }
}

pub(super) fn build_receipt(
    destination: &Path,
    action_id: ActionId,
    created_at: TimestampMs,
    input: &LiveMasterDawprojectInput,
    archive: &PublishedDawprojectArchive,
) -> Result<ExportReceiptState, JamAppError> {
    let destination = destination.to_string_lossy().into_owned();
    let contract = ExportReadinessContract {
        schema: riotbox_core::export_readiness::EXPORT_READINESS_CONTRACT_SCHEMA.into(),
        status: ExportReadinessStatus::Reproducible,
        proof_schema: LIVE_MASTER_DAWPROJECT_PROOF_SCHEMA.into(),
        export_scope: ExportScope::DawSession,
        boundary: ProductExportBoundary::DawSessionLiveMasterDawprojectV1,
        pack_id: LIVE_MASTER_DAWPROJECT_PACK_ID.into(),
        export_role: ProductExportRole::ArrangementManifest,
        export_artifact: destination.clone(),
        source_sha256: input.source_wav_sha256.clone(),
        export_sha256: archive.archive_sha256.clone(),
        normalized_manifest_sha256: archive.project_xml_sha256.clone(),
        unsupported_scopes: Vec::new(),
    };
    let mut receipt = ExportReceiptState::from_readiness_contract(
        action_id,
        created_at,
        &contract,
        destination.clone(),
        destination.clone(),
        Some(destination.clone()),
    );
    let proof_uri = format!("{destination}#{DAWPROJECT_PROOF_PATH}");
    let project_uri = format!("{destination}#project.xml");
    let audio_uri = format!("{destination}#{EMBEDDED_AUDIO_PATH}");
    let source = &input.source_artifact;
    receipt.artifact_set = vec![
        artifact(
            ExportArtifactRole::DawProjectFile,
            ExportArtifactLocation::LocalPath { path: destination },
            ExportArtifactMediaType::DawProjectZip,
            archive.archive_sha256.clone(),
            Some(archive.project_xml_sha256.clone()),
            source,
        ),
        artifact(
            ExportArtifactRole::ExportManifest,
            ExportArtifactLocation::Uri { uri: project_uri },
            ExportArtifactMediaType::Xml,
            archive.project_xml_sha256.clone(),
            Some(archive.project_xml_sha256.clone()),
            source,
        ),
        artifact(
            ExportArtifactRole::LiveRecordingCapture,
            ExportArtifactLocation::Uri { uri: audio_uri },
            ExportArtifactMediaType::AudioWav,
            input.source_wav_sha256.clone(),
            source.normalized_manifest_hash.clone(),
            source,
        ),
        artifact(
            ExportArtifactRole::DawProjectProof,
            ExportArtifactLocation::Uri { uri: proof_uri },
            ExportArtifactMediaType::Json,
            archive.proof_sha256.clone(),
            None,
            source,
        ),
    ];
    receipt.qa_gates = vec![
        ExportReceiptQaGateResult::live_master_dawproject_archive_readback(),
        ExportReceiptQaGateResult::dawproject_xml_document(),
    ];
    let timing = input.proof.timing_grid_ref.as_ref().ok_or_else(|| {
        JamAppError::InvalidSession("admitted live master has no timing-grid lineage".into())
    })?;
    receipt.arrangement_placement_refs = vec![ExportArrangementPlacementRef::scene_range(
        input.proof.scene_id.clone(),
        Some(timing.source_id.clone()),
        1,
        2,
        0,
        u64::from(input.proof.duration_beats),
    )];
    let tempo_bpm_micros = u32::try_from(input.proof.confirmed_bpm_micros).map_err(|_| {
        JamAppError::InvalidSession("recorded BPM exceeds DAW tempo-map range".into())
    })?;
    receipt.daw_tempo_map_ref = Some(ExportDawTempoMapRef::confirmed_grid(
        timing.source_id.clone(),
        timing.hypothesis_id.clone(),
        timing.confirmed_by_action,
        timing.confirmed_at,
        0,
        u64::from(input.proof.duration_beats),
        tempo_bpm_micros,
    ));
    Ok(receipt)
}

fn artifact(
    role: ExportArtifactRole,
    location: ExportArtifactLocation,
    media_type: ExportArtifactMediaType,
    sha256: String,
    normalized_manifest_hash: Option<String>,
    source: &ExportArtifactSetEntry,
) -> ExportArtifactSetEntry {
    ExportArtifactSetEntry {
        role,
        location,
        media_type,
        sha256,
        normalized_manifest_hash,
        source_graph_ref: source.source_graph_ref.clone(),
        timing_grid_ref: source.timing_grid_ref.clone(),
        source_capture_refs: source.source_capture_refs.clone(),
        lineage_capture_refs: source.lineage_capture_refs.clone(),
        fallback_comparison: None,
        audio_metrics: if role == ExportArtifactRole::LiveRecordingCapture {
            source.audio_metrics.clone()
        } else {
            None
        },
        sample_rate_hz: if role == ExportArtifactRole::LiveRecordingCapture {
            source.sample_rate_hz
        } else {
            None
        },
        channel_count: if role == ExportArtifactRole::LiveRecordingCapture {
            source.channel_count
        } else {
            None
        },
        duration_ms: if role == ExportArtifactRole::LiveRecordingCapture {
            source.duration_ms
        } else {
            None
        },
    }
}
