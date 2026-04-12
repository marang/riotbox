use riotbox_core::{
    ids::SourceId,
    source_graph::{DecodeProfile, SourceDescriptor},
};
use riotbox_sidecar::client::StdioSidecarClient;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../python/sidecar/json_stdio_sidecar.py");

    let mut client = StdioSidecarClient::spawn_python(script_path)?;
    let pong = client.ping()?;

    println!(
        "pong: protocol_version={}, sidecar_version={}",
        pong.protocol_version, pong.sidecar_version
    );

    let graph = client.build_source_graph_stub(
        SourceDescriptor {
            source_id: SourceId::from("src-probe-1"),
            path: "fixtures/probe.wav".into(),
            content_hash: "sha256:probe".into(),
            duration_seconds: 64.0,
            sample_rate: 48_000,
            channel_count: 2,
            decode_profile: DecodeProfile::NormalizedStereo,
        },
        23,
    )?;

    println!(
        "source_graph_built: source_id={}, loop_candidates={}, warnings={}",
        graph.source.source_id,
        graph.loop_candidate_count(),
        graph.warnings().join(", ")
    );

    Ok(())
}
