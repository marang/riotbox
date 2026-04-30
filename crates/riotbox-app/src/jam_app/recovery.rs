use super::*;

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
    pub replay_unsupported_label: String,
    pub guidance: Option<RecoveryCandidateGuidance>,
    pub trust: RecoveryCandidateTrust,
    pub detail: String,
    pub action_hint: &'static str,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecoveryCandidateGuidance {
    ArtifactReadyReplayHydrationBlocked,
}

impl RecoveryCandidateGuidance {
    #[must_use]
    pub const fn help_label(self) -> &'static str {
        match self {
            Self::ArtifactReadyReplayHydrationBlocked => {
                "Artifact note: audio present, but W-30 artifact replay hydration is not built yet"
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

fn recovery_candidate_view(
    candidate: &riotbox_core::persistence::SessionRecoveryCandidate,
) -> SessionRecoveryCandidateView {
    let trust = recovery_candidate_trust(&candidate.kind, &candidate.status);
    let replay_labels = recovery_replay_readiness_labels(candidate);
    let artifact_availability_label = recovery_artifact_availability_label(candidate);
    let guidance = recovery_candidate_guidance(&artifact_availability_label, &replay_labels);
    SessionRecoveryCandidateView {
        kind: candidate.kind.clone(),
        path: candidate.path.clone(),
        kind_label: recovery_kind_label(&candidate.kind),
        status_label: recovery_status_label(&candidate.status),
        artifact_availability_label,
        replay_readiness_label: replay_labels.status,
        payload_readiness_label: replay_labels.payload,
        replay_suffix_label: replay_labels.suffix,
        replay_unsupported_label: replay_labels.unsupported,
        guidance,
        trust,
        detail: recovery_detail(&candidate.kind, &candidate.status),
        action_hint: recovery_action_hint(trust),
    }
}

fn recovery_artifact_availability_label(
    candidate: &riotbox_core::persistence::SessionRecoveryCandidate,
) -> String {
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
    artifact_availability_label: &str,
    replay_labels: &runtime_replay_warnings::ReplayReadinessLabels,
) -> Option<RecoveryCandidateGuidance> {
    let unsupported_artifact_command = replay_labels.unsupported.contains("w30.loop_freeze")
        || replay_labels.unsupported.contains("promote.resample");
    if artifact_availability_label.starts_with("artifacts ready:")
        && is_actionable_replay_unsupported(&replay_labels.unsupported)
        && unsupported_artifact_command
    {
        return Some(RecoveryCandidateGuidance::ArtifactReadyReplayHydrationBlocked);
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
    candidate: &riotbox_core::persistence::SessionRecoveryCandidate,
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
            unsupported: "unsupported unreadable".into(),
        },
    }
}

fn recovery_detail(
    kind: &SessionRecoveryCandidateKind,
    status: &SessionRecoveryCandidateStatus,
) -> String {
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

fn recovery_action_hint(trust: RecoveryCandidateTrust) -> &'static str {
    match trust {
        RecoveryCandidateTrust::NormalLoadTarget => "load normally",
        RecoveryCandidateTrust::RecoverableClue => "review before manual recovery",
        RecoveryCandidateTrust::BrokenClue => "do not recover automatically",
        RecoveryCandidateTrust::MissingTarget => "normal load cannot start from this path",
    }
}
