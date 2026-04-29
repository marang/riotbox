use super::*;

#[derive(Debug)]
pub enum JamAppError {
    Io(io::Error),
    Persistence(PersistenceError),
    Serialization(serde_json::Error),
    Sidecar(SidecarClientError),
    InvalidSession(String),
}

impl Display for JamAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Persistence(error) => write!(f, "persistence error: {error}"),
            Self::Serialization(error) => write!(f, "serialization error: {error}"),
            Self::Sidecar(error) => write!(f, "sidecar error: {error}"),
            Self::InvalidSession(message) => write!(f, "invalid session: {message}"),
        }
    }
}

impl Error for JamAppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Persistence(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::Sidecar(error) => Some(error),
            Self::InvalidSession(_) => None,
        }
    }
}

impl From<io::Error> for JamAppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<PersistenceError> for JamAppError {
    fn from(value: PersistenceError) -> Self {
        Self::Persistence(value)
    }
}

impl From<serde_json::Error> for JamAppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

impl From<SidecarClientError> for JamAppError {
    fn from(value: SidecarClientError) -> Self {
        Self::Sidecar(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JamFileSet {
    pub session_path: PathBuf,
    pub source_graph_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppRuntimeState {
    pub audio: Option<AudioRuntimeHealth>,
    pub sidecar: SidecarState,
    pub transport: TransportClockState,
    pub transport_driver: TransportDriverState,
    pub tr909_render: Tr909RenderState,
    pub mc202_render: Mc202RenderState,
    pub w30_preview: W30PreviewRenderState,
    pub w30_resample_tap: W30ResampleTapState,
    pub current_ghost_suggestion: Option<GhostWatchSuggestion>,
    pub last_commit_boundary: Option<CommitBoundaryState>,
}

impl Default for AppRuntimeState {
    fn default() -> Self {
        Self {
            audio: None,
            sidecar: SidecarState::Unknown,
            transport: TransportClockState::default(),
            transport_driver: TransportDriverState::default(),
            tr909_render: Tr909RenderState::default(),
            mc202_render: Mc202RenderState::default(),
            w30_preview: W30PreviewRenderState::default(),
            w30_resample_tap: W30ResampleTapState::default(),
            current_ghost_suggestion: None,
            last_commit_boundary: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportDriverState {
    pub last_audio_position_beats: Option<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum QueueControlResult {
    Enqueued,
    AlreadyPending,
    AlreadyInState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SidecarState {
    Unknown,
    Ready {
        version: Option<String>,
        transport: String,
    },
    Unavailable {
        reason: String,
    },
    Degraded {
        reason: String,
    },
}

#[derive(Clone, Debug)]
pub struct JamAppState {
    pub files: Option<JamFileSet>,
    pub session: SessionFile,
    pub source_graph: Option<SourceGraph>,
    pub source_audio_cache: Option<SourceAudioCache>,
    pub capture_audio_cache: BTreeMap<CaptureId, SourceAudioCache>,
    pub queue: ActionQueue,
    pub runtime: AppRuntimeState,
    pub jam_view: JamViewModel,
    pub runtime_view: JamRuntimeView,
}

pub(in crate::jam_app) struct W30BusPrintInput {
    pub(in crate::jam_app) sample_rate: u32,
    pub(in crate::jam_app) channel_count: u16,
    pub(in crate::jam_app) samples: Vec<f32>,
}
