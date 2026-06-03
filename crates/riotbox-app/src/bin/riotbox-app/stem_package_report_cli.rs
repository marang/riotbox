use riotbox_core::{
    export_readiness::ExportScope,
    persistence::load_session_json,
    session::{
        ExportArtifactLocation as ReportExportArtifactLocation,
        ExportArtifactRole as ReportExportArtifactRole,
        ExportArtifactSetEntry as ReportExportArtifactSetEntry,
        ExportReceiptQaGateResult as ReportExportReceiptQaGateResult,
        ExportReceiptState as ReportExportReceiptState,
        StemPackageReceiptReadinessBlocker as ReportStemPackageReceiptReadinessBlocker,
    },
};

fn run_stem_package_local_ci_report(
    launch: &AppLaunch,
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = stem_package_local_ci_report_summary(launch)?;
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();
    Ok(())
}

fn stem_package_local_ci_report_summary(
    launch: &AppLaunch,
) -> Result<Value, Box<dyn std::error::Error>> {
    let LaunchMode::StemPackageLocalCiReport { session_path } = &launch.mode else {
        return Err("not a stem package local CI report launch".into());
    };
    let session = load_session_json(session_path)?;
    let product_mix_receipt_count = session
        .export_receipts
        .iter()
        .filter(|receipt| receipt.export_scope == ExportScope::ProductMix)
        .count();
    let Some(receipt) = session
        .export_receipts
        .iter()
        .rev()
        .find(|receipt| receipt.export_scope == ExportScope::StemPackage)
    else {
        return Ok(json!({
            "mode": "stem_package_local_ci_report",
            "status": "blocked",
            "ready": false,
            "writes_files": false,
            "session_path": session_path,
            "developer_proof_status": "no_stem_package_receipt",
            "musician_export_readiness": "not_final_daw_export_workflow",
            "readiness_blockers": ["no_stem_package_receipt"],
            "product_mix_receipt_count": product_mix_receipt_count,
            "receipt": null,
            "package_dir": null,
            "stem_roles": [],
            "manifest": null,
            "proof": null,
            "qa_gates": [],
            "local_artifacts": [],
            "missing_local_files": [],
        }));
    };

    let base_dir = session_path.parent();
    let readiness = receipt.stem_package_readiness_report();
    let local_artifacts = receipt
        .artifact_set
        .iter()
        .map(|artifact| report_artifact_availability(artifact, base_dir))
        .collect::<Vec<_>>();
    let missing_local_files = local_artifacts
        .iter()
        .filter(|artifact| artifact["available"] == false)
        .cloned()
        .collect::<Vec<_>>();
    let stem_roles = receipt
        .artifact_set
        .iter()
        .filter(|artifact| artifact.role.is_stem_role())
        .map(|artifact| export_artifact_role_label(artifact.role))
        .collect::<Vec<_>>();
    let mut readiness_blockers = readiness
        .blockers
        .iter()
        .copied()
        .map(stem_package_readiness_blocker_label)
        .collect::<Vec<_>>();
    if !missing_local_files.is_empty() {
        readiness_blockers.push("missing_local_files");
    }
    let ready = readiness.ready() && missing_local_files.is_empty();
    let status = if ready { "ready" } else { "blocked" };
    let developer_proof_status = if ready {
        "local_ci_package_ready"
    } else {
        "local_ci_package_blocked"
    };

    Ok(json!({
        "mode": "stem_package_local_ci_report",
        "status": status,
        "ready": ready,
        "writes_files": false,
        "session_path": session_path,
        "developer_proof_status": developer_proof_status,
        "musician_export_readiness": "not_final_daw_export_workflow",
        "readiness_blockers": readiness_blockers,
        "readiness_report": readiness,
        "product_mix_receipt_count": product_mix_receipt_count,
        "receipt": stem_package_receipt_summary(receipt),
        "package_dir": stem_package_package_dir(receipt, base_dir),
        "stem_roles": stem_roles,
        "manifest": artifact_summary_for_role(receipt, ReportExportArtifactRole::ExportManifest),
        "proof": artifact_summary_for_role(receipt, ReportExportArtifactRole::ProductExportProof),
        "qa_gates": receipt
            .qa_gates
            .iter()
            .map(export_receipt_qa_gate_summary)
            .collect::<Vec<_>>(),
        "local_artifacts": local_artifacts,
        "missing_local_files": missing_local_files,
    }))
}

fn artifact_summary_for_role(
    receipt: &ReportExportReceiptState,
    role: ReportExportArtifactRole,
) -> Value {
    receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == role)
        .map(export_receipt_artifact_summary)
        .unwrap_or(Value::Null)
}

fn export_receipt_qa_gate_summary(gate: &ReportExportReceiptQaGateResult) -> Value {
    json!({
        "gate_id": gate.gate_id,
        "status": gate.status,
        "artifact_roles": gate.artifact_roles
            .iter()
            .copied()
            .map(export_artifact_role_label)
            .collect::<Vec<_>>(),
        "summary": gate.summary,
    })
}

fn report_artifact_availability(
    artifact: &ReportExportArtifactSetEntry,
    base_dir: Option<&std::path::Path>,
) -> Value {
    let mut summary = export_receipt_artifact_summary(artifact);
    match &artifact.location {
        ReportExportArtifactLocation::LocalPath { path } => {
            let resolved_path = resolve_report_local_path(path, base_dir);
            let (available, reason) = match resolved_path.as_ref() {
                Some(path) if path.is_file() => (true, Value::Null),
                Some(path) if path.exists() => (false, json!("not_file")),
                Some(_) => (false, json!("missing_file")),
                None => (false, json!("missing_session_base_dir")),
            };
            summary["available"] = json!(available);
            summary["availability_reason"] = reason;
            summary["resolved_path"] = resolved_path
                .as_ref()
                .map(|path| json!(path))
                .unwrap_or(Value::Null);
        }
        ReportExportArtifactLocation::Uri { .. } => {
            summary["available"] = Value::Null;
            summary["availability_reason"] = json!("uri_not_checked");
            summary["resolved_path"] = Value::Null;
        }
    }
    summary
}

fn resolve_report_local_path(
    path: &str,
    base_dir: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    let path = std::path::Path::new(path);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }
    base_dir.map(|base_dir| base_dir.join(path))
}

fn stem_package_package_dir(
    receipt: &ReportExportReceiptState,
    base_dir: Option<&std::path::Path>,
) -> Value {
    for role in [
        ReportExportArtifactRole::ExportManifest,
        ReportExportArtifactRole::ProductExportProof,
    ] {
        if let Some(path) = local_path_for_role(receipt, role, base_dir)
            && let Some(package_dir) = path.parent()
        {
            return json!(package_dir);
        }
    }
    if let Some(path) = receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role.is_stem_role())
        .and_then(|artifact| resolve_artifact_local_path(artifact, base_dir))
        && let Some(stems_dir) = path.parent()
        && let Some(package_dir) = stems_dir.parent()
    {
        return json!(package_dir);
    }
    Value::Null
}

fn local_path_for_role(
    receipt: &ReportExportReceiptState,
    role: ReportExportArtifactRole,
    base_dir: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == role)
        .and_then(|artifact| resolve_artifact_local_path(artifact, base_dir))
}

fn resolve_artifact_local_path(
    artifact: &ReportExportArtifactSetEntry,
    base_dir: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    match &artifact.location {
        ReportExportArtifactLocation::LocalPath { path } => {
            resolve_report_local_path(path, base_dir)
        }
        ReportExportArtifactLocation::Uri { .. } => None,
    }
}

fn stem_package_readiness_blocker_label(
    blocker: ReportStemPackageReceiptReadinessBlocker,
) -> &'static str {
    match blocker {
        ReportStemPackageReceiptReadinessBlocker::NotStemPackageScope => "not_stem_package_scope",
        ReportStemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent => {
            "unsupported_scope_flag_present"
        }
        ReportStemPackageReceiptReadinessBlocker::MissingArtifactSetQaGate => {
            "missing_artifact_set_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::DeferredArtifactSetQaGate => {
            "deferred_artifact_set_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::FailedArtifactSetQaGate => {
            "failed_artifact_set_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::MissingHashStabilityQaGate => {
            "missing_hash_stability_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::DeferredHashStabilityQaGate => {
            "deferred_hash_stability_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::FailedHashStabilityQaGate => {
            "failed_hash_stability_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::MissingNonSilenceQaGate => {
            "missing_non_silence_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::DeferredNonSilenceQaGate => {
            "deferred_non_silence_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::FailedNonSilenceQaGate => {
            "failed_non_silence_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::MissingLineageQaGate => {
            "missing_lineage_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::DeferredLineageQaGate => {
            "deferred_lineage_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::FailedLineageQaGate => "failed_lineage_qa_gate",
        ReportStemPackageReceiptReadinessBlocker::MissingFallbackComparisonQaGate => {
            "missing_fallback_comparison_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::DeferredFallbackComparisonQaGate => {
            "deferred_fallback_comparison_qa_gate"
        }
        ReportStemPackageReceiptReadinessBlocker::FailedFallbackComparisonQaGate => {
            "failed_fallback_comparison_qa_gate"
        }
    }
}
