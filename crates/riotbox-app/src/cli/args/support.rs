use crate::cli::args::DEFAULT_SESSION_PATH;
use crate::cli::args::DEFAULT_SIDECAR_PATH;
use riotbox_core::session::ExportArtifactRole;
use std::path::PathBuf;

pub(in crate::cli) fn next_path(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value for {flag}"))
}

pub(in crate::cli) fn parse_export_artifact_role(
    value: &str,
) -> Result<ExportArtifactRole, String> {
    match value {
        "stem_drums" | "drums" => Ok(ExportArtifactRole::StemDrums),
        "stem_bass" | "bass" => Ok(ExportArtifactRole::StemBass),
        "stem_music" | "music" => Ok(ExportArtifactRole::StemMusic),
        "stem_vocals" | "vocals" => Ok(ExportArtifactRole::StemVocals),
        "full_grid_mix" => Ok(ExportArtifactRole::FullGridMix),
        "product_export_proof" => Ok(ExportArtifactRole::ProductExportProof),
        "export_manifest" => Ok(ExportArtifactRole::ExportManifest),
        "daw_session_tempo_map" => Ok(ExportArtifactRole::DawSessionTempoMap),
        other => Err(format!("unknown stem role: {other}")),
    }
}

pub(in crate::cli) fn help_text() -> String {
    let base = format!(
        "Usage:\n  riotbox-app --source <audio.wav> [--source-bpm <bpm> [--source-downbeat-seconds <seconds>]] [--session <session.json>] [--graph <source-graph.json>] [--sidecar <script.py>] [--seed <n>] [--observer <events.ndjson>] [--product-export-proof <proof.json> --product-export-destination <dir>]\n  riotbox-app --session <session.json> [--graph <source-graph.json>] [--observer <events.ndjson>] [--product-export-proof <proof.json> --product-export-destination <dir>]\n  riotbox-app --stem-package-local-ci-dry-run --stem-package-destination <dir> --stem-role stem_drums --stem-role stem_bass\n  riotbox-app --stem-package-local-ci-execute --session <session.json> [--graph <source-graph.json>] --stem-package-destination <dir> --stem-role stem_drums --stem-role stem_bass [--observer <events.ndjson>]\n  riotbox-app --stem-package-source-matched-execute --session <session.json> [--graph <source-graph.json>] --product-stem-handoff-proof <proof.json> --stem-package-destination <dir> [--observer <events.ndjson>]\n  riotbox-app --stem-package-w30-hook-execute --session <session.json> [--graph <source-graph.json>] --stem-package-destination <dir> [--observer <events.ndjson>]\n  riotbox-app --stem-package-local-ci-report --session <session.json>\n  riotbox-app --live-recording-readiness-report --session <session.json>\n  riotbox-app --daw-export-readiness-report --session <session.json>\n  riotbox-app --daw-session-writer-plan --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-json-package-execute --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-json-package-evidence-apply --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-writer-proof-execute --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-writer-proof-apply --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-writer-export-execute --session <session.json> --daw-session-destination <dir> [--observer <events.ndjson>]\n  riotbox-app --w30-hook-dawproject-execute --session <session.json> --daw-session-destination <file.dawproject> [--observer <events.ndjson>]\n  riotbox-app --daw-session-host-import-proof-apply --session <session.json> --daw-session-host-import-proof <proof.json>\n  riotbox-app --daw-session-host-import-proof-export-execute --session <session.json> --daw-session-host-import-proof <proof.json> [--observer <events.ndjson>]\n  riotbox-app --daw-session-audible-output-proof-apply --session <session.json> --daw-session-audible-output-proof <proof.json>\n\nDefaults:\n  --session {}\n  --sidecar {}",
        DEFAULT_SESSION_PATH, DEFAULT_SIDECAR_PATH
    );
    format!(
        "{base}\n\nLive master recording:\n  riotbox-app --live-master-recording-execute --session <session.json> [--graph <source-graph.json>] --live-recording-destination <file.wav> [--observer <events.ndjson>] [--live-recording-bars <2|8|16>]\n  --live-recording-bars defaults to 2; the non-interactive capture keeps the current scene unchanged."
    )
}
