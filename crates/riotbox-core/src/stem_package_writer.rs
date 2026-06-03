use crate::{
    export_readiness::ProductExportDestinationKind,
    ids::ActionId,
    session::{ExportArtifactLocation, ExportArtifactMediaType, ExportArtifactRole},
};

pub const STEM_PACKAGE_LOCAL_CI_PACKAGE_BOUNDARY_ID: &str = "stem_package.local_ci_package_v1";
pub const STEM_PACKAGE_PACKAGE_DIR: &str = "stem_package";
pub const STEM_PACKAGE_STEMS_DIR: &str = "stems";
pub const STEM_PACKAGE_MANIFEST_FILE: &str = "stem_package_manifest.json";
pub const STEM_PACKAGE_PROOF_FILE: &str = "stem_package_proof.json";

pub const SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES: &[ExportArtifactRole] =
    &[ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StemPackageLocalWriterBoundary {
    LocalCiPackageV1,
}

impl StemPackageLocalWriterBoundary {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalCiPackageV1 => STEM_PACKAGE_LOCAL_CI_PACKAGE_BOUNDARY_ID,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StemPackageLocalWriterRequest {
    pub created_by_action: ActionId,
    pub boundary: StemPackageLocalWriterBoundary,
    pub destination_kind: ProductExportDestinationKind,
    pub destination_root: String,
    pub claimed_stem_roles: Vec<ExportArtifactRole>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StemPackageLocalWriterPlan {
    pub boundary: StemPackageLocalWriterBoundary,
    pub boundary_id: &'static str,
    pub created_by_action: ActionId,
    pub destination_root: String,
    pub claimed_stem_roles: Vec<ExportArtifactRole>,
    pub artifacts: Vec<StemPackagePlannedArtifactIdentity>,
}

impl StemPackageLocalWriterPlan {
    pub fn stem_artifacts(&self) -> impl Iterator<Item = &StemPackagePlannedArtifactIdentity> {
        self.artifacts
            .iter()
            .filter(|artifact| artifact.role.is_stem_role())
    }

    #[must_use]
    pub fn manifest_artifact(&self) -> Option<&StemPackagePlannedArtifactIdentity> {
        self.artifacts
            .iter()
            .find(|artifact| artifact.role == ExportArtifactRole::ExportManifest)
    }

    #[must_use]
    pub fn proof_artifact(&self) -> Option<&StemPackagePlannedArtifactIdentity> {
        self.artifacts
            .iter()
            .find(|artifact| artifact.role == ExportArtifactRole::ProductExportProof)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StemPackagePlannedArtifactIdentity {
    pub role: ExportArtifactRole,
    pub location: ExportArtifactLocation,
    pub media_type: ExportArtifactMediaType,
}

impl StemPackagePlannedArtifactIdentity {
    #[must_use]
    pub fn location_identity(&self) -> &str {
        match &self.location {
            ExportArtifactLocation::LocalPath { path }
            | ExportArtifactLocation::Uri { uri: path } => path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StemPackageLocalWriterPlanError {
    UnsupportedDestinationKind {
        destination_kind: ProductExportDestinationKind,
    },
    BlankDestinationRoot,
    NoClaimedStemRoles,
    NonStemRoleClaimed {
        role: ExportArtifactRole,
    },
    DuplicateStemRole {
        role: ExportArtifactRole,
    },
    UnsupportedStemRole {
        role: ExportArtifactRole,
    },
}

pub fn plan_stem_package_local_ci_package(
    request: StemPackageLocalWriterRequest,
) -> Result<StemPackageLocalWriterPlan, StemPackageLocalWriterPlanError> {
    validate_request(&request)?;

    let destination_root = normalize_destination_root(&request.destination_root);
    let mut artifacts = request
        .claimed_stem_roles
        .iter()
        .copied()
        .map(|role| planned_local_wav(role, stem_path(&destination_root, role)))
        .collect::<Vec<_>>();
    artifacts.push(planned_local_json(
        ExportArtifactRole::ExportManifest,
        package_path(&destination_root, STEM_PACKAGE_MANIFEST_FILE),
    ));
    artifacts.push(planned_local_json(
        ExportArtifactRole::ProductExportProof,
        package_path(&destination_root, STEM_PACKAGE_PROOF_FILE),
    ));

    Ok(StemPackageLocalWriterPlan {
        boundary: request.boundary,
        boundary_id: request.boundary.as_str(),
        created_by_action: request.created_by_action,
        destination_root,
        claimed_stem_roles: request.claimed_stem_roles,
        artifacts,
    })
}

fn validate_request(
    request: &StemPackageLocalWriterRequest,
) -> Result<(), StemPackageLocalWriterPlanError> {
    if request.destination_kind != ProductExportDestinationKind::LocalArtifactDirectory {
        return Err(
            StemPackageLocalWriterPlanError::UnsupportedDestinationKind {
                destination_kind: request.destination_kind,
            },
        );
    }
    if request.destination_root.trim().is_empty() {
        return Err(StemPackageLocalWriterPlanError::BlankDestinationRoot);
    }
    if request.claimed_stem_roles.is_empty() {
        return Err(StemPackageLocalWriterPlanError::NoClaimedStemRoles);
    }

    let mut seen = Vec::new();
    for role in &request.claimed_stem_roles {
        if !role.is_stem_role() {
            return Err(StemPackageLocalWriterPlanError::NonStemRoleClaimed { role: *role });
        }
        if seen.contains(role) {
            return Err(StemPackageLocalWriterPlanError::DuplicateStemRole { role: *role });
        }
        if !SUPPORTED_LOCAL_CI_PACKAGE_STEM_ROLES.contains(role) {
            return Err(StemPackageLocalWriterPlanError::UnsupportedStemRole { role: *role });
        }
        seen.push(*role);
    }

    Ok(())
}

fn planned_local_wav(role: ExportArtifactRole, path: String) -> StemPackagePlannedArtifactIdentity {
    StemPackagePlannedArtifactIdentity {
        role,
        location: ExportArtifactLocation::LocalPath { path },
        media_type: ExportArtifactMediaType::AudioWav,
    }
}

fn planned_local_json(
    role: ExportArtifactRole,
    path: String,
) -> StemPackagePlannedArtifactIdentity {
    StemPackagePlannedArtifactIdentity {
        role,
        location: ExportArtifactLocation::LocalPath { path },
        media_type: ExportArtifactMediaType::Json,
    }
}

fn normalize_destination_root(destination_root: &str) -> String {
    destination_root.trim().trim_end_matches('/').to_owned()
}

fn stem_path(destination_root: &str, role: ExportArtifactRole) -> String {
    package_path(
        destination_root,
        &format!(
            "{}/{}.wav",
            STEM_PACKAGE_STEMS_DIR,
            stem_role_file_stem(role)
        ),
    )
}

fn package_path(destination_root: &str, relative_path: &str) -> String {
    format!(
        "{}/{}/{}",
        destination_root, STEM_PACKAGE_PACKAGE_DIR, relative_path
    )
}

fn stem_role_file_stem(role: ExportArtifactRole) -> &'static str {
    match role {
        ExportArtifactRole::StemDrums => "stem_drums",
        ExportArtifactRole::StemBass => "stem_bass",
        ExportArtifactRole::StemMusic => "stem_music",
        ExportArtifactRole::StemVocals => "stem_vocals",
        _ => "non_stem",
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn local_ci_package_plan_builds_final_identities_without_side_effects() {
        let destination_root = format!("/tmp/riotbox-stem-package-plan-{}", std::process::id());
        assert!(!Path::new(&destination_root).exists());

        let plan = plan_stem_package_local_ci_package(StemPackageLocalWriterRequest {
            created_by_action: ActionId(42),
            boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_root: format!("{destination_root}/"),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass],
        })
        .expect("plan local ci stem package");

        assert_eq!(plan.boundary_id, STEM_PACKAGE_LOCAL_CI_PACKAGE_BOUNDARY_ID);
        assert_eq!(
            plan.boundary,
            StemPackageLocalWriterBoundary::LocalCiPackageV1
        );
        assert_eq!(plan.created_by_action, ActionId(42));
        assert_eq!(plan.destination_root, destination_root);
        assert_eq!(
            plan.claimed_stem_roles,
            vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemBass]
        );
        assert_eq!(plan.stem_artifacts().count(), 2);
        assert_eq!(
            plan.artifacts
                .iter()
                .map(|artifact| (artifact.role, artifact.location_identity().to_owned()))
                .collect::<Vec<_>>(),
            [
                (
                    ExportArtifactRole::StemDrums,
                    format!("{destination_root}/stem_package/stems/stem_drums.wav")
                ),
                (
                    ExportArtifactRole::StemBass,
                    format!("{destination_root}/stem_package/stems/stem_bass.wav")
                ),
                (
                    ExportArtifactRole::ExportManifest,
                    format!("{destination_root}/stem_package/stem_package_manifest.json")
                ),
                (
                    ExportArtifactRole::ProductExportProof,
                    format!("{destination_root}/stem_package/stem_package_proof.json")
                ),
            ]
            .into_iter()
            .collect::<Vec<_>>()
        );
        assert_eq!(
            plan.manifest_artifact().map(|artifact| artifact.media_type),
            Some(ExportArtifactMediaType::Json)
        );
        assert_eq!(
            plan.proof_artifact().map(|artifact| artifact.media_type),
            Some(ExportArtifactMediaType::Json)
        );
        assert!(!Path::new(&destination_root).exists());
    }

    #[test]
    fn local_ci_package_plan_rejects_unsupported_claims_before_side_effects() {
        let cases = [
            (
                vec![ExportArtifactRole::FullGridMix],
                StemPackageLocalWriterPlanError::NonStemRoleClaimed {
                    role: ExportArtifactRole::FullGridMix,
                },
            ),
            (
                vec![ExportArtifactRole::StemMusic],
                StemPackageLocalWriterPlanError::UnsupportedStemRole {
                    role: ExportArtifactRole::StemMusic,
                },
            ),
            (
                vec![ExportArtifactRole::StemDrums, ExportArtifactRole::StemDrums],
                StemPackageLocalWriterPlanError::DuplicateStemRole {
                    role: ExportArtifactRole::StemDrums,
                },
            ),
        ];

        for (claimed_stem_roles, expected) in cases {
            let err = plan_stem_package_local_ci_package(StemPackageLocalWriterRequest {
                created_by_action: ActionId(7),
                boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
                destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
                destination_root: "/tmp/riotbox-stem-package-plan-invalid".into(),
                claimed_stem_roles,
            })
            .expect_err("invalid claim should fail before side effects");

            assert_eq!(err, expected);
        }
    }

    #[test]
    fn local_ci_package_plan_rejects_non_directory_and_blank_destinations() {
        let err = plan_stem_package_local_ci_package(StemPackageLocalWriterRequest {
            created_by_action: ActionId(7),
            boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
            destination_kind: ProductExportDestinationKind::LocalFilePath,
            destination_root: "/tmp/riotbox-stem-package-plan-file".into(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums],
        })
        .expect_err("file path destination should be rejected");
        assert_eq!(
            err,
            StemPackageLocalWriterPlanError::UnsupportedDestinationKind {
                destination_kind: ProductExportDestinationKind::LocalFilePath
            }
        );

        let err = plan_stem_package_local_ci_package(StemPackageLocalWriterRequest {
            created_by_action: ActionId(7),
            boundary: StemPackageLocalWriterBoundary::LocalCiPackageV1,
            destination_kind: ProductExportDestinationKind::LocalArtifactDirectory,
            destination_root: " ".into(),
            claimed_stem_roles: vec![ExportArtifactRole::StemDrums],
        })
        .expect_err("blank destination should be rejected");
        assert_eq!(err, StemPackageLocalWriterPlanError::BlankDestinationRoot);
    }
}
