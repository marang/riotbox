use std::path::{Path, PathBuf};

use riotbox_core::persistence::{
    SessionRecoveryCandidate, SessionRecoveryCandidateKind, SessionRecoveryCandidateStatus,
    load_session_json, scan_session_recovery_candidates,
};

use super::{
    capture_artifacts, runtime_replay_warnings,
    state::{JamAppError, JamAppState},
};

mod hydration_guidance;
mod payload_guidance;

use hydration_guidance::supported_artifact_replay_hydration_blocker;
use payload_guidance::missing_snapshot_payload_guidance;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRecoverySurface {
    pub target_path: PathBuf,
    pub headline: String,
    pub safety_note: String,
    pub selected_candidate: Option<PathBuf>,
    pub candidates: Vec<SessionRecoveryCandidateView>,
}

impl SessionRecoverySurface {
    #[must_use]
    pub fn has_manual_candidates(&self) -> bool {
        self.candidates.iter().any(|candidate| {
            !matches!(
                candidate.kind,
                SessionRecoveryCandidateKind::CanonicalTarget
            ) && matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue)
        })
    }

    #[must_use]
    pub fn has_non_canonical_clues(&self) -> bool {
        self.candidates.iter().any(|candidate| {
            !matches!(
                candidate.kind,
                SessionRecoveryCandidateKind::CanonicalTarget
            )
        })
    }

    #[must_use]
    pub fn dry_run_manual_choice(
        &self,
        candidate_path: impl AsRef<Path>,
    ) -> Option<ManualRecoveryChoiceDryRun> {
        let candidate_path = candidate_path.as_ref();
        self.candidates
            .iter()
            .find(|candidate| candidate.path == candidate_path)
            .map(ManualRecoveryChoiceDryRun::from_candidate)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualRecoveryChoiceDryRun {
    pub candidate_path: PathBuf,
    pub decision_label: String,
    pub artifact_availability_label: String,
    pub replay_readiness_label: String,
    pub payload_readiness_label: String,
    pub replay_suffix_label: String,
    pub replay_family_label: String,
    pub replay_unsupported_label: String,
    pub guidance_label: Option<String>,
    pub trust: RecoveryCandidateTrust,
    pub action_hint: &'static str,
    pub selected_for_restore: bool,
    pub safety_note: &'static str,
}

impl ManualRecoveryChoiceDryRun {
    fn from_candidate(candidate: &SessionRecoveryCandidateView) -> Self {
        Self {
            candidate_path: candidate.path.clone(),
            decision_label: candidate.decision_label.clone(),
            artifact_availability_label: candidate.artifact_availability_label.clone(),
            replay_readiness_label: candidate.replay_readiness_label.clone(),
            payload_readiness_label: candidate.payload_readiness_label.clone(),
            replay_suffix_label: candidate.replay_suffix_label.clone(),
            replay_family_label: candidate.replay_family_label.clone(),
            replay_unsupported_label: candidate.replay_unsupported_label.clone(),
            guidance_label: candidate
                .guidance
                .as_ref()
                .map(|guidance| guidance.help_label()),
            trust: candidate.trust,
            action_hint: candidate.action_hint,
            selected_for_restore: false,
            safety_note: "Dry-run only: candidate inspected, not selected for restore.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRecoveryCandidateView {
    pub kind: SessionRecoveryCandidateKind,
    pub path: PathBuf,
    pub kind_label: &'static str,
    pub status_label: &'static str,
    pub artifact_availability_label: String,
    pub replay_readiness_label: String,
    pub payload_readiness_label: String,
    pub replay_suffix_label: String,
    pub replay_family_label: String,
    pub replay_unsupported_label: String,
    pub decision_label: String,
    pub guidance: Option<RecoveryCandidateGuidance>,
    pub trust: RecoveryCandidateTrust,
    pub detail: String,
    pub action_hint: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryCandidateGuidance {
    SupportedArtifactReplayHydrationBlocked { detail: String },
    MissingSnapshotPayload { detail: String },
}

impl RecoveryCandidateGuidance {
    #[must_use]
    pub fn help_label(&self) -> String {
        match self {
            Self::SupportedArtifactReplayHydrationBlocked { detail } => {
                format!("Replay hydration note: {detail}")
            }
            Self::MissingSnapshotPayload { detail } => {
                format!("Snapshot payload note: {detail}")
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecoveryCandidateTrust {
    NormalLoadTarget,
    RecoverableClue,
    BrokenClue,
    MissingTarget,
}

impl JamAppState {
    pub fn scan_session_recovery_surface(
        target_path: impl AsRef<Path>,
    ) -> Result<SessionRecoverySurface, JamAppError> {
        let scan = scan_session_recovery_candidates(target_path)?;
        let candidates = scan
            .candidates
            .iter()
            .map(recovery_candidate_view)
            .collect::<Vec<_>>();

        Ok(SessionRecoverySurface {
            target_path: scan.target_path,
            headline: recovery_headline(&candidates),
            safety_note: "Manual recovery only: Riotbox did not choose, load, replace, or delete any candidate.".into(),
            selected_candidate: None,
            candidates,
        })
    }
}

fn recovery_candidate_view(candidate: &SessionRecoveryCandidate) -> SessionRecoveryCandidateView {
    let replay_labels = recovery_replay_readiness_labels(candidate);
    let payload_invalid = replay_labels
        .payload
        .starts_with("payload invalid | snapshot restore blocked");
    let trust = if payload_invalid {
        RecoveryCandidateTrust::BrokenClue
    } else {
        recovery_candidate_trust(&candidate.kind, &candidate.status)
    };
    let artifact_availability_label = if payload_invalid {
        "artifacts unchecked".into()
    } else {
        recovery_artifact_availability_label(candidate)
    };
    let replay_family_label = if payload_invalid {
        "families unchecked".into()
    } else {
        replay_labels.family.clone()
    };
    let guidance = recovery_candidate_guidance(candidate);
    let decision_label = recovery_decision_label(
        trust,
        &artifact_availability_label,
        &replay_labels,
        guidance.as_ref(),
    );
    SessionRecoveryCandidateView {
        kind: candidate.kind.clone(),
        path: candidate.path.clone(),
        kind_label: recovery_kind_label(&candidate.kind),
        status_label: if payload_invalid {
            "app-invalid session"
        } else {
            recovery_status_label(&candidate.status)
        },
        artifact_availability_label,
        replay_readiness_label: replay_labels.status,
        payload_readiness_label: replay_labels.payload,
        replay_suffix_label: replay_labels.suffix,
        replay_family_label,
        replay_unsupported_label: replay_labels.unsupported,
        decision_label,
        guidance,
        trust,
        detail: recovery_detail(&candidate.kind, &candidate.status, payload_invalid),
        action_hint: recovery_action_hint(trust),
    }
}

fn recovery_artifact_availability_label(candidate: &SessionRecoveryCandidate) -> String {
    if !matches!(
        candidate.status,
        SessionRecoveryCandidateStatus::ParseableSession
    ) {
        return "artifacts unchecked".into();
    }

    let Ok(session) = load_session_json(&candidate.path) else {
        return "artifacts unreadable".into();
    };

    let capture_count = session.captures.len();
    if capture_count == 0 {
        return "artifacts n/a | no captures".into();
    }

    let base_dir = candidate.path.parent();
    let mut ready = 0usize;
    let mut missing = 0usize;
    let mut unreadable = 0usize;
    let mut missing_identity = 0usize;

    for capture in &session.captures {
        match capture_artifacts::preflight_capture_artifact_hydration(capture, base_dir) {
            Ok(_) => ready += 1,
            Err(
                capture_artifacts::CaptureArtifactHydrationPreflightError::MissingStoragePath {
                    ..
                }
                | capture_artifacts::CaptureArtifactHydrationPreflightError::MissingSessionFileSet {
                    ..
                },
            ) => missing_identity += 1,
            Err(capture_artifacts::CaptureArtifactHydrationPreflightError::MissingArtifact {
                ..
            }) => missing += 1,
            Err(
                capture_artifacts::CaptureArtifactHydrationPreflightError::UnreadableArtifact {
                    ..
                }
                | capture_artifacts::CaptureArtifactHydrationPreflightError::NotFile { .. },
            ) => unreadable += 1,
        }
    }

    if ready == capture_count {
        return format!("artifacts ready: {capture_count} capture(s)");
    }

    let mut blockers = Vec::new();
    if missing_identity > 0 {
        blockers.push(format!("{missing_identity} missing identity"));
    }
    if missing > 0 {
        blockers.push(format!("{missing} missing"));
    }
    if unreadable > 0 {
        blockers.push(format!("{unreadable} unreadable"));
    }

    format!(
        "artifacts blocked: {} of {} | {}",
        capture_count - ready,
        capture_count,
        blockers.join(", ")
    )
}

fn recovery_candidate_guidance(
    candidate: &SessionRecoveryCandidate,
) -> Option<RecoveryCandidateGuidance> {
    if let Some(detail) = supported_artifact_replay_hydration_blocker(candidate) {
        return Some(RecoveryCandidateGuidance::SupportedArtifactReplayHydrationBlocked { detail });
    }
    if let Some(detail) = missing_snapshot_payload_guidance(candidate) {
        return Some(RecoveryCandidateGuidance::MissingSnapshotPayload { detail });
    }

    None
}

fn is_actionable_replay_unsupported(label: &str) -> bool {
    label.starts_with("unsupported suffix") || label.starts_with("unsupported origin")
}

fn recovery_headline(candidates: &[SessionRecoveryCandidateView]) -> String {
    let parseable_clues = candidates
        .iter()
        .filter(|candidate| {
            !matches!(
                candidate.kind,
                SessionRecoveryCandidateKind::CanonicalTarget
            ) && matches!(candidate.trust, RecoveryCandidateTrust::RecoverableClue)
        })
        .count();

    if parseable_clues == 0 {
        "No manual recovery candidate selected".into()
    } else {
        format!("{parseable_clues} manual recovery candidate(s) need explicit review")
    }
}

fn recovery_kind_label(kind: &SessionRecoveryCandidateKind) -> &'static str {
    match kind {
        SessionRecoveryCandidateKind::CanonicalTarget => "normal session path",
        SessionRecoveryCandidateKind::OrphanTemp => "orphan temp file",
        SessionRecoveryCandidateKind::Autosave => "autosave file",
    }
}

fn recovery_status_label(status: &SessionRecoveryCandidateStatus) -> &'static str {
    match status {
        SessionRecoveryCandidateStatus::Missing => "missing",
        SessionRecoveryCandidateStatus::ParseableSession => "parseable session JSON",
        SessionRecoveryCandidateStatus::InvalidSessionJson { .. } => "invalid session JSON",
        SessionRecoveryCandidateStatus::Unreadable { .. } => "unreadable",
    }
}

fn recovery_candidate_trust(
    kind: &SessionRecoveryCandidateKind,
    status: &SessionRecoveryCandidateStatus,
) -> RecoveryCandidateTrust {
    match (kind, status) {
        (
            SessionRecoveryCandidateKind::CanonicalTarget,
            SessionRecoveryCandidateStatus::Missing,
        ) => RecoveryCandidateTrust::MissingTarget,
        (
            SessionRecoveryCandidateKind::CanonicalTarget,
            SessionRecoveryCandidateStatus::ParseableSession,
        ) => RecoveryCandidateTrust::NormalLoadTarget,
        (_, SessionRecoveryCandidateStatus::ParseableSession) => {
            RecoveryCandidateTrust::RecoverableClue
        }
        _ => RecoveryCandidateTrust::BrokenClue,
    }
}

fn recovery_replay_readiness_labels(
    candidate: &SessionRecoveryCandidate,
) -> runtime_replay_warnings::ReplayReadinessLabels {
    if !matches!(
        candidate.status,
        SessionRecoveryCandidateStatus::ParseableSession
    ) {
        return runtime_replay_warnings::ReplayReadinessLabels {
            status: "replay unchecked".into(),
            anchor: "anchor unchecked".into(),
            payload: "payload unchecked".into(),
            suffix: "suffix unchecked".into(),
            family: "families unchecked".into(),
            unsupported: "unsupported unchecked".into(),
        };
    }

    match load_session_json(&candidate.path) {
        Ok(session) => runtime_replay_warnings::derive_replay_readiness_labels(&session),
        Err(_) => runtime_replay_warnings::ReplayReadinessLabels {
            status: "replay unreadable".into(),
            anchor: "anchor unreadable".into(),
            payload: "payload unreadable".into(),
            suffix: "suffix unreadable".into(),
            family: "families unreadable".into(),
            unsupported: "unsupported unreadable".into(),
        },
    }
}

fn recovery_detail(
    kind: &SessionRecoveryCandidateKind,
    status: &SessionRecoveryCandidateStatus,
    payload_invalid: bool,
) -> String {
    if payload_invalid {
        return "Snapshot payload identity is invalid; normal app load rejects this candidate."
            .into();
    }

    match (kind, status) {
        (
            SessionRecoveryCandidateKind::CanonicalTarget,
            SessionRecoveryCandidateStatus::Missing,
        ) => "Normal session path does not exist; inspect listed siblings manually if any appear."
            .into(),
        (
            SessionRecoveryCandidateKind::CanonicalTarget,
            SessionRecoveryCandidateStatus::ParseableSession,
        ) => "This is the normal deterministic load target.".into(),
        (_, SessionRecoveryCandidateStatus::ParseableSession) => {
            "Looks parseable, but remains untrusted until the user explicitly chooses recovery."
                .into()
        }
        (_, SessionRecoveryCandidateStatus::InvalidSessionJson { reason }) => {
            format!("Cannot parse as a Riotbox session: {reason}")
        }
        (_, SessionRecoveryCandidateStatus::Unreadable { reason }) => {
            format!("Cannot read candidate: {reason}")
        }
        (_, SessionRecoveryCandidateStatus::Missing) => {
            "Candidate path is missing; no recovery action is available.".into()
        }
    }
}

fn recovery_decision_label(
    trust: RecoveryCandidateTrust,
    artifact_availability_label: &str,
    replay_labels: &runtime_replay_warnings::ReplayReadinessLabels,
    guidance: Option<&RecoveryCandidateGuidance>,
) -> String {
    match trust {
        RecoveryCandidateTrust::NormalLoadTarget => "decision: normal load path".into(),
        RecoveryCandidateTrust::BrokenClue => "decision: broken candidate".into(),
        RecoveryCandidateTrust::MissingTarget => "decision: normal target missing".into(),
        RecoveryCandidateTrust::RecoverableClue => {
            let hydration_blocked = matches!(
                guidance,
                Some(RecoveryCandidateGuidance::SupportedArtifactReplayHydrationBlocked { .. })
            );
            let replay_blocked = is_actionable_replay_unsupported(&replay_labels.unsupported);
            let artifacts_blocked = artifact_availability_label.starts_with("artifacts blocked:");
            if hydration_blocked && artifacts_blocked {
                return "decision: blocked | replay hydration and artifacts".into();
            }
            if hydration_blocked {
                return "decision: blocked | replay hydration".into();
            }
            if replay_blocked && artifacts_blocked {
                return "decision: blocked | replay and artifacts".into();
            }
            if replay_blocked {
                return "decision: blocked | replay unsupported".into();
            }
            if artifacts_blocked {
                return "decision: blocked | artifacts unavailable".into();
            }
            if replay_labels.payload.starts_with("payload missing") {
                return "decision: reviewable | full replay required".into();
            }
            "decision: reviewable | explicit user choice required".into()
        }
    }
}

fn recovery_action_hint(trust: RecoveryCandidateTrust) -> &'static str {
    match trust {
        RecoveryCandidateTrust::NormalLoadTarget => "load normally",
        RecoveryCandidateTrust::RecoverableClue => "review before manual recovery",
        RecoveryCandidateTrust::BrokenClue => "do not recover automatically",
        RecoveryCandidateTrust::MissingTarget => "normal load cannot start from this path",
    }
}
