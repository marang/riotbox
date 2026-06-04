fn next_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn parse_export_artifact_role(value: &str) -> Result<ExportArtifactRole, String> {
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

fn help_text() -> String {
    format!(
        "Usage:\n  riotbox-app --source <audio.wav> [--session <session.json>] [--graph <source-graph.json>] [--sidecar <script.py>] [--seed <n>] [--observer <events.ndjson>]\n  riotbox-app --session <session.json> [--graph <source-graph.json>] [--observer <events.ndjson>]\n  riotbox-app --stem-package-local-ci-dry-run --stem-package-destination <dir> --stem-role stem_drums --stem-role stem_bass\n  riotbox-app --stem-package-local-ci-execute --session <session.json> [--graph <source-graph.json>] --stem-package-destination <dir> --stem-role stem_drums --stem-role stem_bass [--observer <events.ndjson>]\n  riotbox-app --stem-package-local-ci-report --session <session.json>\n  riotbox-app --live-recording-readiness-report --session <session.json>\n  riotbox-app --daw-export-readiness-report --session <session.json>\n  riotbox-app --daw-session-writer-plan --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-json-package-execute --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-json-package-evidence-apply --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-writer-proof-execute --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-writer-proof-apply --session <session.json> --daw-session-destination <dir>\n  riotbox-app --daw-session-writer-export-execute --session <session.json> --daw-session-destination <dir> [--observer <events.ndjson>]\n  riotbox-app --daw-session-host-import-proof-apply --session <session.json> --daw-session-host-import-proof <proof.json>\n  riotbox-app --daw-session-audible-output-proof-apply --session <session.json> --daw-session-audible-output-proof <proof.json>\n\nDefaults:\n  --session {}\n  --sidecar {}",
        DEFAULT_SESSION_PATH, DEFAULT_SIDECAR_PATH
    )
}

impl LaunchMode {
    fn shell_launch_mode(&self) -> ShellLaunchMode {
        match self {
            Self::Load { .. } => ShellLaunchMode::Load,
            Self::Ingest { .. } => ShellLaunchMode::Ingest,
            Self::StemPackageLocalCiDryRun { .. }
            | Self::StemPackageLocalCiExecute { .. }
            | Self::StemPackageLocalCiReport { .. }
            | Self::LiveRecordingReadinessReport { .. }
            | Self::DawExportReadinessReport { .. }
            | Self::DawSessionJsonPackageExecute { .. }
            | Self::DawSessionJsonPackageEvidenceApply { .. }
            | Self::DawSessionHostImportProofApply { .. }
            | Self::DawSessionAudibleOutputProofApply { .. }
            | Self::DawSessionWriterProofExecute { .. }
            | Self::DawSessionWriterProofApply { .. }
            | Self::DawSessionWriterExportExecute { .. }
            | Self::DawSessionWriterPlan { .. } => ShellLaunchMode::Load,
        }
    }
}
