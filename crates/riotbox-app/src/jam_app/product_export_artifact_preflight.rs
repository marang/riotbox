use std::{
    io,
    path::{Path, PathBuf},
};

use riotbox_core::{
    export_readiness::ExportScope,
    session::{
        ArrangementExportPlacementReadinessBlocker, DawTempoMapReadinessBlocker,
        ExportArtifactLocation, ExportArtifactRole, ExportArtifactSetEntry, ExportReceiptState,
        STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::jam_app) enum ExportReceiptArtifactPreflightError {
    MissingArtifactPath {
        receipt_id: riotbox_core::ids::ExportReceiptId,
    },
    MissingProofPath {
        receipt_id: riotbox_core::ids::ExportReceiptId,
    },
    MissingArtifactSetPath {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
    },
    MissingSessionFileSet {
        receipt_id: riotbox_core::ids::ExportReceiptId,
    },
    MissingExportArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
    },
    MissingProofArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
    },
    MissingArtifactSetArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
        path: PathBuf,
    },
    NotFile {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
    },
    ArtifactSetNotFile {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
        path: PathBuf,
    },
    UnreadableArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        path: PathBuf,
        reason: String,
    },
    UnreadableArtifactSetArtifact {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        role: ExportArtifactRole,
        path: PathBuf,
        reason: String,
    },
    ArrangementPlacementBlocked {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        blockers: Vec<ArrangementExportPlacementReadinessBlocker>,
    },
    DawTempoMapBlocked {
        receipt_id: riotbox_core::ids::ExportReceiptId,
        blockers: Vec<DawTempoMapReadinessBlocker>,
    },
}

pub(in crate::jam_app) fn preflight_export_receipt_artifacts(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
) -> Result<(PathBuf, PathBuf), ExportReceiptArtifactPreflightError> {
    preflight_arrangement_export_placement_contract(receipt)?;
    preflight_daw_tempo_map_contract(receipt)?;
    let artifact_path = resolve_receipt_path(
        receipt,
        &receipt.artifact_path,
        base_dir,
        ReceiptPathKind::Artifact,
    )?;
    let proof_path = resolve_receipt_path(
        receipt,
        &receipt.proof_path,
        base_dir,
        ReceiptPathKind::Proof,
    )?;

    let artifact = require_receipt_file(receipt, artifact_path, ReceiptPathKind::Artifact)?;
    let proof = require_receipt_file(receipt, proof_path, ReceiptPathKind::Proof)?;
    preflight_artifact_set_entries(receipt, base_dir)?;
    preflight_stem_package_artifact_contract(receipt, base_dir)?;

    Ok((artifact, proof))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ReceiptPathKind {
    Artifact,
    Proof,
    ArtifactSet(ExportArtifactRole),
}

fn preflight_artifact_set_entries(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
) -> Result<(), ExportReceiptArtifactPreflightError> {
    for artifact in receipt.artifact_set_or_legacy() {
        preflight_artifact_set_entry(receipt, &artifact, base_dir)?;
    }

    Ok(())
}

fn preflight_arrangement_export_placement_contract(
    receipt: &ExportReceiptState,
) -> Result<(), ExportReceiptArtifactPreflightError> {
    if receipt.export_scope != ExportScope::DawSession {
        return Ok(());
    }

    let report = receipt.arrangement_export_placement_report();
    if report.ready() {
        return Ok(());
    }

    Err(
        ExportReceiptArtifactPreflightError::ArrangementPlacementBlocked {
            receipt_id: receipt.receipt_id.clone(),
            blockers: report.blockers,
        },
    )
}

fn preflight_daw_tempo_map_contract(
    receipt: &ExportReceiptState,
) -> Result<(), ExportReceiptArtifactPreflightError> {
    if receipt.export_scope != ExportScope::DawSession {
        return Ok(());
    }

    let report = receipt.daw_tempo_map_report();
    if report.ready() {
        return Ok(());
    }

    Err(ExportReceiptArtifactPreflightError::DawTempoMapBlocked {
        receipt_id: receipt.receipt_id.clone(),
        blockers: report.blockers,
    })
}

fn preflight_stem_package_artifact_contract(
    receipt: &ExportReceiptState,
    base_dir: Option<&Path>,
) -> Result<(), ExportReceiptArtifactPreflightError> {
    if receipt.export_scope != ExportScope::StemPackage {
        return Ok(());
    }

    let claimed_roles = stem_package_claimed_roles(receipt)?;
    for role in claimed_roles.into_iter().chain([
        ExportArtifactRole::ExportManifest,
        ExportArtifactRole::ProductExportProof,
    ]) {
        let artifact = required_stem_package_artifact_set_entry(receipt, role)?;
        preflight_artifact_set_entry(receipt, artifact, base_dir)?;
    }

    Ok(())
}

fn stem_package_claimed_roles(
    receipt: &ExportReceiptState,
) -> Result<Vec<ExportArtifactRole>, ExportReceiptArtifactPreflightError> {
    let Some(gate) = receipt
        .qa_gates
        .iter()
        .find(|gate| gate.gate_id == STEM_PACKAGE_ARTIFACT_SET_QA_GATE_ID)
    else {
        return Err(
            ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                receipt_id: receipt.receipt_id.clone(),
                role: ExportArtifactRole::ExportManifest,
            },
        );
    };

    if gate.artifact_roles.is_empty() {
        return Err(
            ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                receipt_id: receipt.receipt_id.clone(),
                role: ExportArtifactRole::ExportManifest,
            },
        );
    }

    for role in &gate.artifact_roles {
        if !role.is_stem_role() {
            return Err(
                ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                    receipt_id: receipt.receipt_id.clone(),
                    role: *role,
                },
            );
        }
    }

    Ok(gate.artifact_roles.clone())
}

fn required_stem_package_artifact_set_entry(
    receipt: &ExportReceiptState,
    role: ExportArtifactRole,
) -> Result<&ExportArtifactSetEntry, ExportReceiptArtifactPreflightError> {
    receipt
        .artifact_set
        .iter()
        .find(|artifact| artifact.role == role)
        .ok_or_else(
            || ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                receipt_id: receipt.receipt_id.clone(),
                role,
            },
        )
}

fn preflight_artifact_set_entry(
    receipt: &ExportReceiptState,
    artifact: &ExportArtifactSetEntry,
    base_dir: Option<&Path>,
) -> Result<(), ExportReceiptArtifactPreflightError> {
    match &artifact.location {
        ExportArtifactLocation::LocalPath { path } => {
            let artifact_path = resolve_receipt_path(
                receipt,
                path,
                base_dir,
                ReceiptPathKind::ArtifactSet(artifact.role),
            )?;
            require_receipt_file(
                receipt,
                artifact_path,
                ReceiptPathKind::ArtifactSet(artifact.role),
            )?;
        }
        ExportArtifactLocation::Uri { uri } if uri.trim().is_empty() => {
            return Err(
                ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                    receipt_id: receipt.receipt_id.clone(),
                    role: artifact.role,
                },
            );
        }
        ExportArtifactLocation::Uri { .. } => {}
    }

    Ok(())
}

fn resolve_receipt_path(
    receipt: &ExportReceiptState,
    path: &str,
    base_dir: Option<&Path>,
    kind: ReceiptPathKind,
) -> Result<PathBuf, ExportReceiptArtifactPreflightError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(match kind {
            ReceiptPathKind::Artifact => ExportReceiptArtifactPreflightError::MissingArtifactPath {
                receipt_id: receipt.receipt_id.clone(),
            },
            ReceiptPathKind::Proof => ExportReceiptArtifactPreflightError::MissingProofPath {
                receipt_id: receipt.receipt_id.clone(),
            },
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::MissingArtifactSetPath {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                }
            }
        });
    }

    let path = Path::new(path);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let base_dir =
        base_dir.ok_or_else(
            || ExportReceiptArtifactPreflightError::MissingSessionFileSet {
                receipt_id: receipt.receipt_id.clone(),
            },
        )?;
    Ok(base_dir.join(path))
}

fn require_receipt_file(
    receipt: &ExportReceiptState,
    path: PathBuf,
    kind: ReceiptPathKind,
) -> Result<PathBuf, ExportReceiptArtifactPreflightError> {
    match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Ok(path),
        Ok(_) => Err(match kind {
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::ArtifactSetNotFile {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                    path,
                }
            }
            ReceiptPathKind::Artifact | ReceiptPathKind::Proof => {
                ExportReceiptArtifactPreflightError::NotFile {
                    receipt_id: receipt.receipt_id.clone(),
                    path,
                }
            }
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(match kind {
            ReceiptPathKind::Artifact => {
                ExportReceiptArtifactPreflightError::MissingExportArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    path,
                }
            }
            ReceiptPathKind::Proof => ExportReceiptArtifactPreflightError::MissingProofArtifact {
                receipt_id: receipt.receipt_id.clone(),
                path,
            },
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::MissingArtifactSetArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                    path,
                }
            }
        }),
        Err(error) => Err(match kind {
            ReceiptPathKind::ArtifactSet(role) => {
                ExportReceiptArtifactPreflightError::UnreadableArtifactSetArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    role,
                    path,
                    reason: error.to_string(),
                }
            }
            ReceiptPathKind::Artifact | ReceiptPathKind::Proof => {
                ExportReceiptArtifactPreflightError::UnreadableArtifact {
                    receipt_id: receipt.receipt_id.clone(),
                    path,
                    reason: error.to_string(),
                }
            }
        }),
    }
}
