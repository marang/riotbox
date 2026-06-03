use ratatui::text::Line;
use riotbox_core::{
    export_readiness::ProductExportBoundary,
    session::{
        ExportArtifactRole, ExportReceiptQaGateStatus, ExportReceiptState,
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID, STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID,
        STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID, STEM_PACKAGE_LINEAGE_QA_GATE_ID,
        STEM_PACKAGE_NON_SILENCE_QA_GATE_ID, StemPackageReceiptReadinessBlocker,
        StemPackageReceiptReadinessStatus,
    },
};

pub(super) fn stem_package_export_receipt_lines(
    receipt: &ExportReceiptState,
) -> Vec<Line<'static>> {
    let boundary = export_boundary_short_label(receipt.export_boundary);
    let readiness = receipt.stem_package_readiness_report();
    let artifact_set = receipt.artifact_set_or_legacy();
    let roles = compact_stem_artifact_roles(&artifact_set);

    vec![
        Line::from(format!("export stem_package | {boundary}")),
        Line::from(format!(
            "{} {} | {} | art{}",
            compact_export_receipt_id(receipt),
            stem_package_readiness_status_label(readiness.status),
            roles,
            artifact_set.len()
        )),
        Line::from(format!(
            "gates {}",
            compact_stem_package_gate_summary(receipt)
        )),
        Line::from(format!(
            "blockers {}",
            compact_stem_package_blocker_summary(&readiness.blockers)
        )),
    ]
}

fn compact_stem_artifact_roles(
    artifacts: &[riotbox_core::session::ExportArtifactSetEntry],
) -> String {
    let mut roles = artifacts
        .iter()
        .filter_map(|artifact| stem_artifact_role_short_label(artifact.role))
        .collect::<Vec<_>>();
    roles.sort_unstable();
    roles.dedup();
    if roles.is_empty() {
        "none".into()
    } else {
        roles.join("/")
    }
}

fn compact_stem_package_gate_summary(receipt: &ExportReceiptState) -> String {
    let gates = [
        ("artifact", STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID),
        ("hash", STEM_PACKAGE_HASH_STABILITY_QA_GATE_ID),
        ("audio", STEM_PACKAGE_NON_SILENCE_QA_GATE_ID),
        ("lineage", STEM_PACKAGE_LINEAGE_QA_GATE_ID),
        ("compare", STEM_PACKAGE_FALLBACK_COMPARISON_QA_GATE_ID),
    ]
    .into_iter()
    .map(|(label, gate_id)| {
        (
            stem_package_gate_group_label(label),
            receipt.qa_gates.iter().find(|gate| gate.gate_id == gate_id),
        )
    })
    .collect::<Vec<_>>();

    let mut passed = Vec::new();
    let mut missing = Vec::new();
    let mut deferred = Vec::new();
    let mut failed = Vec::new();
    for (label, gate) in gates {
        match gate.map(|gate| gate.status) {
            Some(ExportReceiptQaGateStatus::Passed) => passed.push(label),
            Some(ExportReceiptQaGateStatus::Deferred) => deferred.push(label),
            Some(ExportReceiptQaGateStatus::Failed) => failed.push(label),
            None => missing.push(label),
        }
    }

    if missing.is_empty() && deferred.is_empty() && failed.is_empty() {
        return format!("{} pass", passed.join("/"));
    }

    let mut parts = Vec::new();
    if !passed.is_empty() {
        parts.push(format!("pass {}", passed.join("/")));
    }
    if !missing.is_empty() {
        parts.push(format!("miss {}", missing.join("/")));
    }
    if !deferred.is_empty() {
        parts.push(format!("defer {}", deferred.join("/")));
    }
    if !failed.is_empty() {
        parts.push(format!("fail {}", failed.join("/")));
    }
    parts.join(" | ")
}

fn stem_package_gate_group_label(label: &str) -> &'static str {
    match label {
        "artifact" => "art",
        "hash" => "hsh",
        "audio" => "aud",
        "lineage" => "lin",
        "compare" => "cmp",
        _ => "unk",
    }
}

fn compact_stem_package_blocker_summary(blockers: &[StemPackageReceiptReadinessBlocker]) -> String {
    if blockers.is_empty() {
        return "none".into();
    }

    let mut unsupported = false;
    let mut missing = Vec::new();
    let mut deferred = Vec::new();
    let mut failed = Vec::new();
    let mut other = Vec::new();

    for blocker in blockers {
        match blocker {
            StemPackageReceiptReadinessBlocker::UnsupportedScopeFlagPresent => unsupported = true,
            StemPackageReceiptReadinessBlocker::MissingArtifactSetQaGate => missing.push("art"),
            StemPackageReceiptReadinessBlocker::MissingHashStabilityQaGate => missing.push("hsh"),
            StemPackageReceiptReadinessBlocker::MissingNonSilenceQaGate => missing.push("aud"),
            StemPackageReceiptReadinessBlocker::MissingLineageQaGate => missing.push("lin"),
            StemPackageReceiptReadinessBlocker::MissingFallbackComparisonQaGate => {
                missing.push("cmp");
            }
            StemPackageReceiptReadinessBlocker::DeferredArtifactSetQaGate => deferred.push("art"),
            StemPackageReceiptReadinessBlocker::DeferredHashStabilityQaGate => deferred.push("hsh"),
            StemPackageReceiptReadinessBlocker::DeferredNonSilenceQaGate => deferred.push("aud"),
            StemPackageReceiptReadinessBlocker::DeferredLineageQaGate => deferred.push("lin"),
            StemPackageReceiptReadinessBlocker::DeferredFallbackComparisonQaGate => {
                deferred.push("cmp");
            }
            StemPackageReceiptReadinessBlocker::FailedArtifactSetQaGate => failed.push("art"),
            StemPackageReceiptReadinessBlocker::FailedHashStabilityQaGate => failed.push("hsh"),
            StemPackageReceiptReadinessBlocker::FailedNonSilenceQaGate => failed.push("aud"),
            StemPackageReceiptReadinessBlocker::FailedLineageQaGate => failed.push("lin"),
            StemPackageReceiptReadinessBlocker::FailedFallbackComparisonQaGate => {
                failed.push("cmp");
            }
            StemPackageReceiptReadinessBlocker::NotStemPackageScope => other.push("wrong_scope"),
        }
    }

    let mut parts = Vec::new();
    if unsupported {
        parts.push("unsup".into());
    }
    if !missing.is_empty() {
        parts.push(format!("miss {}", missing.join("/")));
    }
    if !deferred.is_empty() {
        parts.push(format!("defer {}", deferred.join("/")));
    }
    if !failed.is_empty() {
        parts.push(format!("fail {}", failed.join("/")));
    }
    parts.extend(other.into_iter().map(String::from));
    parts.join(" | ")
}

fn stem_package_readiness_status_label(status: StemPackageReceiptReadinessStatus) -> &'static str {
    match status {
        StemPackageReceiptReadinessStatus::Ready => "ready",
        StemPackageReceiptReadinessStatus::Blocked => "blocked",
    }
}

fn stem_artifact_role_short_label(role: ExportArtifactRole) -> Option<&'static str> {
    match role {
        ExportArtifactRole::StemDrums => Some("drums"),
        ExportArtifactRole::StemBass => Some("bass"),
        ExportArtifactRole::StemMusic => Some("music"),
        ExportArtifactRole::StemVocals => Some("vocals"),
        ExportArtifactRole::FullGridMix
        | ExportArtifactRole::ProductExportProof
        | ExportArtifactRole::ExportManifest
        | ExportArtifactRole::DawSessionTempoMap
        | ExportArtifactRole::DawSessionWriterProof => None,
    }
}

fn compact_export_receipt_id(receipt: &ExportReceiptState) -> &str {
    receipt
        .receipt_id
        .as_str()
        .strip_prefix("export-receipt-")
        .unwrap_or_else(|| receipt.receipt_id.as_str())
}

fn export_boundary_short_label(boundary: ProductExportBoundary) -> &'static str {
    match boundary {
        ProductExportBoundary::FeralGridGeneratedSupport => "feral-grid",
        ProductExportBoundary::StemPackageLocalCiPackageV1 => "stem-pkg",
        ProductExportBoundary::ArrangementDawPlacementContractV1 => "arrange-daw",
    }
}
