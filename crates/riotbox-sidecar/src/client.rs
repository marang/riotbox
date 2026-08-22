use crate::protocol::{
    AnalyzeSourceFilePayload, BuildSourceGraphStubPayload, PROTOCOL_VERSION, PingPayload,
    PongPayload, SidecarErrorPayload, SidecarRequest, SidecarResponse, decode_json_line,
    encode_json_line,
};
use riotbox_core::source_graph::{SourceDescriptor, SourceGraph};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::Duration,
};

const DEFAULT_CONTROL_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_ANALYSIS_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidecarTimeoutPolicy {
    pub control: Duration,
    pub analysis: Duration,
}

impl Default for SidecarTimeoutPolicy {
    fn default() -> Self {
        Self {
            control: DEFAULT_CONTROL_TIMEOUT,
            analysis: DEFAULT_ANALYSIS_TIMEOUT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidecarOperation {
    Control,
    Analysis,
}

impl Display for SidecarOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Control => f.write_str("control"),
            Self::Analysis => f.write_str("analysis"),
        }
    }
}

#[derive(Debug)]
pub enum ClientError {
    ScriptUnavailable {
        path: PathBuf,
        source: io::Error,
    },
    Spawn(io::Error),
    MissingStdin,
    MissingStdout,
    Io(io::Error),
    Protocol(crate::protocol::ProtocolError),
    UnexpectedEof,
    Sidecar(SidecarErrorPayload),
    UnexpectedResponse(&'static str),
    ResponseTimeout {
        operation: SidecarOperation,
        timeout: Duration,
    },
    RequestIdMismatch {
        expected: String,
        received: Option<String>,
    },
    ProtocolVersionMismatch {
        expected: String,
        received: String,
    },
    UntrustedAnalysisProvider {
        providers: Vec<String>,
    },
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScriptUnavailable { path, source } => write!(
                f,
                "sidecar script is unavailable at {}: {source}",
                path.display()
            ),
            Self::Spawn(error) => write!(f, "failed to spawn sidecar: {error}"),
            Self::MissingStdin => write!(f, "spawned sidecar without piped stdin"),
            Self::MissingStdout => write!(f, "spawned sidecar without piped stdout"),
            Self::Io(error) => write!(f, "stdio transport failed: {error}"),
            Self::Protocol(error) => write!(f, "{error}"),
            Self::UnexpectedEof => write!(f, "sidecar closed stdout before replying"),
            Self::Sidecar(error) => write!(f, "sidecar returned {}: {}", error.code, error.message),
            Self::UnexpectedResponse(kind) => write!(f, "unexpected sidecar response: {kind}"),
            Self::ResponseTimeout { operation, timeout } => write!(
                f,
                "sidecar {operation} operation did not reply within {:.1}s",
                timeout.as_secs_f32()
            ),
            Self::RequestIdMismatch { expected, received } => write!(
                f,
                "sidecar response request_id mismatch: expected {expected}, got {}",
                received.as_deref().unwrap_or("<none>")
            ),
            Self::ProtocolVersionMismatch { expected, received } => write!(
                f,
                "sidecar protocol version mismatch: expected {expected}, received {received}"
            ),
            Self::UntrustedAnalysisProvider { providers } => write!(
                f,
                "source analysis returned an untrusted provider set: {}",
                if providers.is_empty() {
                    "<empty>".to_string()
                } else {
                    providers.join(", ")
                }
            ),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ScriptUnavailable { source, .. } | Self::Spawn(source) | Self::Io(source) => {
                Some(source)
            }
            Self::Protocol(error) => Some(error),
            Self::MissingStdin
            | Self::MissingStdout
            | Self::UnexpectedEof
            | Self::Sidecar(_)
            | Self::UnexpectedResponse(_)
            | Self::ResponseTimeout { .. }
            | Self::RequestIdMismatch { .. }
            | Self::ProtocolVersionMismatch { .. }
            | Self::UntrustedAnalysisProvider { .. } => None,
        }
    }
}

impl From<io::Error> for ClientError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<crate::protocol::ProtocolError> for ClientError {
    fn from(value: crate::protocol::ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

pub struct StdioSidecarClient {
    child: Child,
    stdin: ChildStdin,
    stdout_rx: Receiver<Result<String, io::Error>>,
    next_request_id: u64,
    timeout_policy: SidecarTimeoutPolicy,
    protocol_compatible: bool,
}

impl StdioSidecarClient {
    pub fn spawn_python(script_path: impl AsRef<Path>) -> Result<Self, ClientError> {
        let script_path = script_path.as_ref();
        let metadata = script_path
            .metadata()
            .map_err(|source| ClientError::ScriptUnavailable {
                path: script_path.to_path_buf(),
                source,
            })?;
        if !metadata.is_file() {
            return Err(ClientError::ScriptUnavailable {
                path: script_path.to_path_buf(),
                source: io::Error::new(io::ErrorKind::InvalidInput, "path is not a file"),
            });
        }
        let mut child = Command::new("python3")
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(ClientError::Spawn)?;

        let stdin = child.stdin.take().ok_or(ClientError::MissingStdin)?;
        let stdout = child.stdout.take().ok_or(ClientError::MissingStdout)?;

        Ok(Self {
            child,
            stdin,
            stdout_rx: spawn_stdout_reader(stdout),
            next_request_id: 1,
            timeout_policy: SidecarTimeoutPolicy::default(),
            protocol_compatible: false,
        })
    }

    #[must_use]
    pub fn with_timeout_policy(mut self, timeout_policy: SidecarTimeoutPolicy) -> Self {
        self.timeout_policy = timeout_policy;
        self
    }

    /// Compatibility helper for tests and callers that need one deadline for every operation.
    #[must_use]
    pub fn with_response_timeout(mut self, response_timeout: Duration) -> Self {
        self.timeout_policy = SidecarTimeoutPolicy {
            control: response_timeout,
            analysis: response_timeout,
        };
        self
    }

    pub fn ping(&mut self) -> Result<PongPayload, ClientError> {
        self.protocol_compatible = false;
        let request_id = self.next_request_id();
        let request = SidecarRequest::Ping(PingPayload {
            request_id: request_id.clone(),
        });

        self.write_request(&request)?;

        match self.read_response(SidecarOperation::Control)? {
            SidecarResponse::Pong(pong) => {
                validate_request_id(&request_id, Some(&pong.request_id))?;
                if pong.protocol_version != PROTOCOL_VERSION {
                    return Err(ClientError::ProtocolVersionMismatch {
                        expected: PROTOCOL_VERSION.to_string(),
                        received: pong.protocol_version,
                    });
                }
                self.protocol_compatible = true;
                Ok(pong)
            }
            SidecarResponse::Error(error) => Err(classify_sidecar_error(&request_id, error)),
            SidecarResponse::SourceGraphBuilt(_) => {
                Err(ClientError::UnexpectedResponse("source_graph_built"))
            }
        }
    }

    pub fn build_source_graph_stub(
        &mut self,
        source: SourceDescriptor,
        analysis_seed: u64,
    ) -> Result<SourceGraph, ClientError> {
        self.ensure_protocol_compatible()?;
        let request_id = self.next_request_id();
        let request = SidecarRequest::BuildSourceGraphStub(BuildSourceGraphStubPayload {
            request_id: request_id.clone(),
            source,
            analysis_seed,
        });

        self.write_request(&request)?;

        match self.read_response(SidecarOperation::Analysis)? {
            SidecarResponse::SourceGraphBuilt(payload) => {
                validate_request_id(&request_id, Some(&payload.request_id))?;
                Ok(payload.graph)
            }
            SidecarResponse::Error(error) => Err(classify_sidecar_error(&request_id, error)),
            SidecarResponse::Pong(_) => Err(ClientError::UnexpectedResponse("pong")),
        }
    }

    pub fn analyze_source_file(
        &mut self,
        source_path: impl AsRef<Path>,
        analysis_seed: u64,
    ) -> Result<SourceGraph, ClientError> {
        self.ensure_protocol_compatible()?;
        let request_id = self.next_request_id();
        let request = SidecarRequest::AnalyzeSourceFile(AnalyzeSourceFilePayload {
            request_id: request_id.clone(),
            source_path: source_path.as_ref().to_string_lossy().into_owned(),
            analysis_seed,
        });

        self.write_request(&request)?;

        match self.read_response(SidecarOperation::Analysis)? {
            SidecarResponse::SourceGraphBuilt(payload) => {
                validate_request_id(&request_id, Some(&payload.request_id))?;
                validate_source_analysis_provider(&payload.graph)?;
                Ok(payload.graph)
            }
            SidecarResponse::Error(error) => Err(classify_sidecar_error(&request_id, error)),
            SidecarResponse::Pong(_) => Err(ClientError::UnexpectedResponse("pong")),
        }
    }

    fn next_request_id(&mut self) -> String {
        let request_id = format!("req-{}", self.next_request_id);
        self.next_request_id += 1;
        request_id
    }

    fn ensure_protocol_compatible(&mut self) -> Result<(), ClientError> {
        if !self.protocol_compatible {
            self.ping()?;
        }
        Ok(())
    }

    fn write_request(&mut self, request: &SidecarRequest) -> Result<(), ClientError> {
        let line = encode_json_line(request)?;
        self.stdin.write_all(line.as_bytes())?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(
        &mut self,
        operation: SidecarOperation,
    ) -> Result<SidecarResponse, ClientError> {
        let timeout = match operation {
            SidecarOperation::Control => self.timeout_policy.control,
            SidecarOperation::Analysis => self.timeout_policy.analysis,
        };
        let line = match self.stdout_rx.recv_timeout(timeout) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => return Err(ClientError::Io(error)),
            Err(RecvTimeoutError::Timeout) => {
                return Err(ClientError::ResponseTimeout { operation, timeout });
            }
            Err(RecvTimeoutError::Disconnected) => return Err(ClientError::UnexpectedEof),
        };

        Ok(decode_json_line(&line)?)
    }
}

fn classify_sidecar_error(expected_request_id: &str, error: SidecarErrorPayload) -> ClientError {
    if let Some(received_request_id) = error.request_id.as_deref()
        && received_request_id != expected_request_id
    {
        return ClientError::RequestIdMismatch {
            expected: expected_request_id.to_string(),
            received: Some(received_request_id.to_string()),
        };
    }
    ClientError::Sidecar(error)
}

fn validate_source_analysis_provider(graph: &SourceGraph) -> Result<(), ClientError> {
    let providers = &graph.provenance.provider_set;
    if providers.is_empty()
        || providers
            .iter()
            .any(|provider| provider.starts_with("stub."))
    {
        return Err(ClientError::UntrustedAnalysisProvider {
            providers: providers.clone(),
        });
    }
    Ok(())
}

fn spawn_stdout_reader(stdout: ChildStdout) -> Receiver<Result<String, io::Error>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut stdout = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match stdout.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if tx.send(Ok(line)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.send(Err(error));
                    break;
                }
            }
        }
    });
    rx
}

fn validate_request_id(expected: &str, received: Option<&str>) -> Result<(), ClientError> {
    if received == Some(expected) {
        Ok(())
    } else {
        Err(ClientError::RequestIdMismatch {
            expected: expected.to_string(),
            received: received.map(ToOwned::to_owned),
        })
    }
}

impl Drop for StdioSidecarClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use std::{f32::consts::PI, fs, path::Path};

    use riotbox_core::{
        ids::SourceId,
        source_graph::{DecodeProfile, SourceDescriptor},
    };

    use super::*;
    use crate::path::bundled_sidecar_script_path;

    fn sample_source() -> SourceDescriptor {
        SourceDescriptor {
            source_id: SourceId::from("src-transport-1"),
            path: "fixtures/break.wav".into(),
            content_hash: "sha256:abc123".into(),
            duration_seconds: 92.5,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        }
    }

    fn protocol_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn write_pcm16_wave(
        path: impl AsRef<Path>,
        sample_rate: u32,
        channel_count: u16,
        duration_seconds: f32,
    ) {
        let path = path.as_ref();
        let frame_count = (sample_rate as f32 * duration_seconds) as u32;
        let bits_per_sample = 16_u16;
        let bytes_per_sample = (bits_per_sample / 8) as u32;
        let byte_rate = sample_rate * channel_count as u32 * bytes_per_sample;
        let block_align = channel_count * (bits_per_sample / 8);
        let data_len = frame_count * channel_count as u32 * bytes_per_sample;

        let mut bytes = Vec::with_capacity((44 + data_len) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&channel_count.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());

        for frame_index in 0..frame_count {
            let phase = (frame_index as f32 / sample_rate as f32) * 220.0 * 2.0 * PI;
            let sample = (phase.sin() * i16::MAX as f32 * 0.25) as i16;
            for _ in 0..channel_count {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
        }

        fs::write(path, bytes).expect("write PCM wave fixture");
    }

    #[test]
    fn stdio_sidecar_ping_and_graph_build_work() {
        let mut client = StdioSidecarClient::spawn_python(bundled_sidecar_script_path())
            .expect("spawn python sidecar");

        let pong = client.ping().expect("receive pong");
        assert_eq!(pong.protocol_version, "0.1");
        assert_eq!(pong.sidecar_version, "0.1.0");

        let graph = client
            .build_source_graph_stub(sample_source(), 17)
            .expect("build source graph stub");

        assert_eq!(graph.source.source_id.as_str(), "src-transport-1");
        assert_eq!(graph.provenance.analysis_seed, 17);
        assert_eq!(graph.loop_candidate_count(), 1);
        assert_eq!(graph.provenance.provider_set, vec!["stub.transport"]);
        assert!(graph.provenance.generated_at.ends_with('Z'));
        assert_ne!(graph.provenance.generated_at, "2026-04-12T19:30:00Z");
        assert_eq!(graph.warnings()[0], "transport spike returned a stub graph");
        match validate_source_analysis_provider(&graph)
            .expect_err("transport stub must not qualify as source analysis")
        {
            ClientError::UntrustedAnalysisProvider { providers } => {
                assert_eq!(providers, ["stub.transport"]);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn stdio_sidecar_can_analyze_a_real_source_file_path() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let source_path = temp_dir.path().join("input.wav");
        write_pcm16_wave(&source_path, 44_100, 2, 2.0);

        let mut client = StdioSidecarClient::spawn_python(bundled_sidecar_script_path())
            .expect("spawn python sidecar");

        let graph = client
            .analyze_source_file(&source_path, 23)
            .expect("analyze source file");

        assert_eq!(graph.source.path, source_path.to_string_lossy());
        assert_eq!(graph.source.sample_rate, 44_100);
        assert_eq!(graph.source.channel_count, 2);
        assert!(graph.source.duration_seconds >= 1.9);
        assert_eq!(graph.provenance.analysis_seed, 23);
        assert_eq!(graph.provenance.provider_set, vec!["decoded.wav_baseline"]);
        assert!(graph.loop_candidate_count() >= 1);
        assert!(graph.timing.bpm_estimate.is_some());
        assert!(!graph.phrase_audio_features.is_empty());
        assert!(
            graph.phrase_audio_features[0]
                .provenance_refs
                .contains(&"mc202.phrase-audio-features.v0".into())
        );
        assert!(graph.phrase_audio_features[0].has_measured_evidence());
        assert_eq!(graph.source_map.buckets.len(), 32);
        assert_eq!(graph.source_map.buckets[0].start_seconds, 0.0);
        assert!(graph.source_map.buckets[0].end_seconds > 0.0);
        assert!(
            graph
                .source_map
                .buckets
                .iter()
                .any(|bucket| bucket.energy_class != riotbox_core::source_graph::EnergyClass::Low)
        );
        assert!(
            graph.source_map.buckets.iter().all(
                |bucket| bucket.provenance_refs.as_slice() == ["provider:decoded.wav_baseline"]
            )
        );
    }

    #[test]
    fn stdio_sidecar_rejects_unsupported_source_files() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let source_path = temp_dir.path().join("input.txt");
        fs::write(&source_path, b"not a wav file").expect("write unsupported fixture");

        let mut client = StdioSidecarClient::spawn_python(bundled_sidecar_script_path())
            .expect("spawn python sidecar");

        let error = client
            .analyze_source_file(&source_path, 23)
            .expect_err("unsupported source should fail");

        match error {
            ClientError::Sidecar(payload) => assert_eq!(payload.code, "source_unsupported"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn stdio_sidecar_times_out_when_child_stops_replying() {
        let mut client = StdioSidecarClient::spawn_python(protocol_fixture_path("hung_control.py"))
            .expect("spawn hung python sidecar")
            .with_response_timeout(Duration::from_millis(50));

        let error = client.ping().expect_err("hung sidecar should time out");

        match error {
            ClientError::ResponseTimeout { operation, timeout } => {
                assert_eq!(operation, SidecarOperation::Control);
                assert_eq!(timeout, Duration::from_millis(50));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn graph_request_rejects_incompatible_protocol_before_graph_data() {
        let mut client =
            StdioSidecarClient::spawn_python(protocol_fixture_path("protocol_version_mismatch.py"))
                .expect("spawn mismatch sidecar");
        let error = client
            .build_source_graph_stub(sample_source(), 17)
            .expect_err("protocol mismatch must fail before graph request");

        match error {
            ClientError::ProtocolVersionMismatch { expected, received } => {
                assert_eq!(expected, PROTOCOL_VERSION);
                assert_eq!(received, "99.0");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn requestless_sidecar_error_preserves_code_and_message() {
        let mut client =
            StdioSidecarClient::spawn_python(protocol_fixture_path("requestless_error.py"))
                .expect("spawn error sidecar");
        let error = client
            .build_source_graph_stub(sample_source(), 17)
            .expect_err("request-less error must fail");

        match error {
            ClientError::Sidecar(payload) => {
                assert_eq!(payload.request_id, None);
                assert_eq!(payload.code, "invalid_json");
                assert_eq!(payload.message, "malformed provider response");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn analysis_uses_separate_configurable_bounded_timeout() {
        let timeout_policy = SidecarTimeoutPolicy {
            control: Duration::from_secs(1),
            analysis: Duration::from_millis(50),
        };
        let mut client =
            StdioSidecarClient::spawn_python(protocol_fixture_path("hung_analysis.py"))
                .expect("spawn hung analysis sidecar")
                .with_timeout_policy(timeout_policy);
        let error = client
            .build_source_graph_stub(sample_source(), 17)
            .expect_err("hung analysis should time out");

        match error {
            ClientError::ResponseTimeout { operation, timeout } => {
                assert_eq!(operation, SidecarOperation::Analysis);
                assert_eq!(timeout, Duration::from_millis(50));
            }
            other => panic!("unexpected error: {other}"),
        }
        assert!(SidecarTimeoutPolicy::default().analysis > Duration::from_secs(10));
        assert!(SidecarTimeoutPolicy::default().analysis < Duration::from_secs(300));
    }

    #[test]
    fn analysis_can_outlive_control_budget_without_timing_out() {
        let timeout_policy = SidecarTimeoutPolicy {
            control: Duration::from_millis(50),
            analysis: Duration::from_millis(250),
        };
        let mut client =
            StdioSidecarClient::spawn_python(protocol_fixture_path("slow_analysis_error.py"))
                .expect("spawn slow analysis sidecar")
                .with_timeout_policy(timeout_policy);
        let error = client
            .build_source_graph_stub(sample_source(), 17)
            .expect_err("fixture returns an error after its slow response");

        match error {
            ClientError::Sidecar(payload) => assert_eq!(payload.code, "fixture_complete"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn missing_configured_script_reports_exact_path() {
        let missing_path = PathBuf::from("/tmp/riotbox-sidecar-does-not-exist.py");
        let error = match StdioSidecarClient::spawn_python(&missing_path) {
            Ok(_) => panic!("missing configured sidecar must fail before spawn"),
            Err(error) => error,
        };

        match error {
            ClientError::ScriptUnavailable { path, .. } => assert_eq!(path, missing_path),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn rejects_response_with_mismatched_request_id() {
        let error = validate_request_id("req-1", Some("req-2"))
            .expect_err("mismatched request id should fail");

        match error {
            ClientError::RequestIdMismatch { expected, received } => {
                assert_eq!(expected, "req-1");
                assert_eq!(received.as_deref(), Some("req-2"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
