use crate::protocol::{
    AnalyzeSourceFilePayload, BuildSourceGraphStubPayload, PingPayload, PongPayload,
    SidecarErrorPayload, SidecarRequest, SidecarResponse, decode_json_line, encode_json_line,
};
use riotbox_core::source_graph::{SourceDescriptor, SourceGraph};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

#[derive(Debug)]
pub enum ClientError {
    Spawn(io::Error),
    MissingStdin,
    MissingStdout,
    Io(io::Error),
    Protocol(crate::protocol::ProtocolError),
    UnexpectedEof,
    Sidecar(SidecarErrorPayload),
    UnexpectedResponse(&'static str),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "failed to spawn sidecar: {error}"),
            Self::MissingStdin => write!(f, "spawned sidecar without piped stdin"),
            Self::MissingStdout => write!(f, "spawned sidecar without piped stdout"),
            Self::Io(error) => write!(f, "stdio transport failed: {error}"),
            Self::Protocol(error) => write!(f, "{error}"),
            Self::UnexpectedEof => write!(f, "sidecar closed stdout before replying"),
            Self::Sidecar(error) => write!(f, "sidecar returned {}: {}", error.code, error.message),
            Self::UnexpectedResponse(kind) => write!(f, "unexpected sidecar response: {kind}"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn(error) | Self::Io(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::MissingStdin
            | Self::MissingStdout
            | Self::UnexpectedEof
            | Self::Sidecar(_)
            | Self::UnexpectedResponse(_) => None,
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
    stdout: BufReader<ChildStdout>,
    next_request_id: u64,
}

impl StdioSidecarClient {
    pub fn spawn_python(script_path: impl AsRef<Path>) -> Result<Self, ClientError> {
        let mut child = Command::new("python3")
            .arg(script_path.as_ref())
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
            stdout: BufReader::new(stdout),
            next_request_id: 1,
        })
    }

    pub fn ping(&mut self) -> Result<PongPayload, ClientError> {
        let request_id = self.next_request_id();
        let request = SidecarRequest::Ping(PingPayload { request_id });

        self.write_request(&request)?;

        match self.read_response()? {
            SidecarResponse::Pong(pong) => Ok(pong),
            SidecarResponse::Error(error) => Err(ClientError::Sidecar(error)),
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
        let request_id = self.next_request_id();
        let request = SidecarRequest::BuildSourceGraphStub(BuildSourceGraphStubPayload {
            request_id,
            source,
            analysis_seed,
        });

        self.write_request(&request)?;

        match self.read_response()? {
            SidecarResponse::SourceGraphBuilt(payload) => Ok(payload.graph),
            SidecarResponse::Error(error) => Err(ClientError::Sidecar(error)),
            SidecarResponse::Pong(_) => Err(ClientError::UnexpectedResponse("pong")),
        }
    }

    pub fn analyze_source_file(
        &mut self,
        source_path: impl AsRef<Path>,
        analysis_seed: u64,
    ) -> Result<SourceGraph, ClientError> {
        let request_id = self.next_request_id();
        let request = SidecarRequest::AnalyzeSourceFile(AnalyzeSourceFilePayload {
            request_id,
            source_path: source_path.as_ref().to_string_lossy().into_owned(),
            analysis_seed,
        });

        self.write_request(&request)?;

        match self.read_response()? {
            SidecarResponse::SourceGraphBuilt(payload) => Ok(payload.graph),
            SidecarResponse::Error(error) => Err(ClientError::Sidecar(error)),
            SidecarResponse::Pong(_) => Err(ClientError::UnexpectedResponse("pong")),
        }
    }

    fn next_request_id(&mut self) -> String {
        let request_id = format!("req-{}", self.next_request_id);
        self.next_request_id += 1;
        request_id
    }

    fn write_request(&mut self, request: &SidecarRequest) -> Result<(), ClientError> {
        let line = encode_json_line(request)?;
        self.stdin.write_all(line.as_bytes())?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<SidecarResponse, ClientError> {
        let mut line = String::new();
        let bytes_read = self.stdout.read_line(&mut line)?;

        if bytes_read == 0 {
            return Err(ClientError::UnexpectedEof);
        }

        Ok(decode_json_line(&line)?)
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
    use std::{f32::consts::PI, fs, path::Path, path::PathBuf};

    use riotbox_core::{
        ids::SourceId,
        source_graph::{DecodeProfile, SourceDescriptor},
    };

    use super::*;

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

    fn sidecar_script_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../python/sidecar/json_stdio_sidecar.py")
            .canonicalize()
            .expect("resolve sidecar script path")
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
        let mut client =
            StdioSidecarClient::spawn_python(sidecar_script_path()).expect("spawn python sidecar");

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
    }

    #[test]
    fn stdio_sidecar_can_analyze_a_real_source_file_path() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let source_path = temp_dir.path().join("input.wav");
        write_pcm16_wave(&source_path, 44_100, 2, 2.0);

        let mut client =
            StdioSidecarClient::spawn_python(sidecar_script_path()).expect("spawn python sidecar");

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
    }

    #[test]
    fn stdio_sidecar_rejects_unsupported_source_files() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let source_path = temp_dir.path().join("input.txt");
        fs::write(&source_path, b"not a wav file").expect("write unsupported fixture");

        let mut client =
            StdioSidecarClient::spawn_python(sidecar_script_path()).expect("spawn python sidecar");

        let error = client
            .analyze_source_file(&source_path, 23)
            .expect_err("unsupported source should fail");

        match error {
            ClientError::Sidecar(payload) => assert_eq!(payload.code, "source_unsupported"),
            other => panic!("unexpected error: {other}"),
        }
    }
}
