use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Component, Path},
};

use dawproject::{Dawproject, DawprojectReader, DawprojectWriter, MetaData, Project};
use sha2::{Digest, Sha256};

use super::JamAppError;

pub(in crate::jam_app) const DAWPROJECT_PROOF_PATH: &str = "riotbox-proof.json";
const PROJECT_XML_PATH: &str = "project.xml";
const METADATA_XML_PATH: &str = "metadata.xml";

pub(in crate::jam_app) struct DawprojectArchiveAudio<'a> {
    pub(in crate::jam_app) path: &'a str,
    pub(in crate::jam_app) bytes: &'a [u8],
    pub(in crate::jam_app) sha256: &'a str,
}

pub(in crate::jam_app) struct DawprojectArchivePayload<'a> {
    pub(in crate::jam_app) metadata: &'a MetaData,
    pub(in crate::jam_app) project: &'a Project,
    pub(in crate::jam_app) audio: DawprojectArchiveAudio<'a>,
    pub(in crate::jam_app) proof_json: &'a [u8],
}

pub(in crate::jam_app) struct PublishedDawprojectArchive {
    pub(in crate::jam_app) archive_sha256: String,
    pub(in crate::jam_app) project_xml_sha256: String,
    pub(in crate::jam_app) proof_sha256: String,
}

struct ValidatedArchive {
    bytes: Vec<u8>,
    published: PublishedDawprojectArchive,
}

/// Encodes, validates, and publishes the fixed Riotbox DAWproject archive.
///
/// The archive owns its member set, typed DAWproject readback, exact payload
/// verification, no-clobber publication, published-byte readback, and cleanup
/// of only the bytes it wrote.
pub(in crate::jam_app) fn write_dawproject_archive(
    destination: &Path,
    payload: DawprojectArchivePayload<'_>,
) -> Result<PublishedDawprojectArchive, JamAppError> {
    validate_destination(destination)?;
    let archive = build_and_validate_archive(&payload)?;
    publish_archive(destination, &archive)?;
    if let Err(error) = validate_published_archive(destination, &archive, &payload) {
        remove_owned_destination(destination, &archive.published.archive_sha256);
        return Err(error);
    }
    Ok(archive.published)
}

pub(in crate::jam_app) fn remove_owned_destination(destination: &Path, expected_sha256: &str) {
    if sha256_file(destination).is_ok_and(|sha256| sha256 == expected_sha256) {
        let _ = fs::remove_file(destination);
    }
}

pub(in crate::jam_app) fn validate_destination(destination: &Path) -> Result<(), JamAppError> {
    if destination
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("dawproject")
    {
        return Err(JamAppError::InvalidSession(
            "DAWproject destination must end in .dawproject".into(),
        ));
    }
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| {
            JamAppError::InvalidSession(
                "DAWproject destination requires an explicit parent directory".into(),
            )
        })?;
    if !parent.is_dir() {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject destination parent does not exist: {}",
            parent.display()
        )));
    }
    if destination.exists() || fs::symlink_metadata(destination).is_ok() {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject destination already exists: {}",
            destination.display()
        )));
    }
    Ok(())
}

fn build_and_validate_archive(
    payload: &DawprojectArchivePayload<'_>,
) -> Result<ValidatedArchive, JamAppError> {
    validate_audio_member_path(payload.audio.path)?;
    if sha256_bytes(payload.audio.bytes) != payload.audio.sha256 {
        return Err(JamAppError::InvalidSession(
            "DAWproject audio bytes do not match their declared SHA-256".into(),
        ));
    }
    serde_json::from_slice::<serde_json::Value>(payload.proof_json).map_err(|error| {
        JamAppError::InvalidSession(format!("DAWproject proof is not valid JSON: {error}"))
    })?;

    let cursor = Cursor::new(Vec::new());
    let mut writer = DawprojectWriter::new(cursor)
        .map_err(|error| invalid_dawproject("could not create archive writer", error))?;
    writer
        .write_dawproject(&Dawproject::new(
            payload.metadata.clone(),
            payload.project.clone(),
        ))
        .map_err(|error| invalid_dawproject("could not serialize DAWproject model", error))?;
    writer
        .write_file(payload.audio.path, payload.audio.bytes)
        .map_err(|error| invalid_dawproject("could not embed archive audio", error))?;
    writer
        .write_file(DAWPROJECT_PROOF_PATH, payload.proof_json)
        .map_err(|error| invalid_dawproject("could not embed proof", error))?;
    let bytes = writer
        .finish()
        .map_err(|error| invalid_dawproject("could not finish DAWproject archive", error))?
        .into_inner();
    let (project_xml_sha256, proof_sha256) = validate_archive_bytes(&bytes, payload)?;
    Ok(ValidatedArchive {
        published: PublishedDawprojectArchive {
            archive_sha256: sha256_bytes(&bytes),
            project_xml_sha256,
            proof_sha256,
        },
        bytes,
    })
}

fn validate_archive_bytes(
    bytes: &[u8],
    payload: &DawprojectArchivePayload<'_>,
) -> Result<(String, String), JamAppError> {
    let mut reader = DawprojectReader::new(Cursor::new(bytes))
        .map_err(|error| invalid_dawproject("archive is not readable", error))?;
    let paths = reader.file_names().map(str::to_owned).collect::<Vec<_>>();
    if paths != expected_archive_paths(payload.audio.path) {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject archive paths differ from the frozen set: {paths:?}"
        )));
    }
    reader
        .read_dawproject()
        .map_err(|error| invalid_dawproject("project or metadata XML did not parse", error))?;
    let parsed = reader.build_dawproject().ok_or_else(|| {
        JamAppError::InvalidSession("DAWproject reader produced no typed model".into())
    })?;
    if &parsed.metadata != payload.metadata || &parsed.project != payload.project {
        return Err(JamAppError::InvalidSession(
            "DAWproject typed read-back differs from the frozen model".into(),
        ));
    }
    let embedded_audio = read_archive_file(&mut reader, payload.audio.path)?;
    if embedded_audio != payload.audio.bytes
        || sha256_bytes(&embedded_audio) != payload.audio.sha256
    {
        return Err(JamAppError::InvalidSession(
            "DAWproject embedded audio is not byte-identical to its source payload".into(),
        ));
    }
    let proof_bytes = read_archive_file(&mut reader, DAWPROJECT_PROOF_PATH)?;
    if proof_bytes != payload.proof_json {
        return Err(JamAppError::InvalidSession(
            "DAWproject embedded proof differs from its source payload".into(),
        ));
    }
    let project_xml = read_archive_file(&mut reader, PROJECT_XML_PATH)?;
    Ok((sha256_bytes(&project_xml), sha256_bytes(&proof_bytes)))
}

fn publish_archive(destination: &Path, archive: &ValidatedArchive) -> Result<(), JamAppError> {
    let parent = destination.parent().ok_or_else(|| {
        JamAppError::InvalidSession("DAWproject destination has no parent".into())
    })?;
    let mut staging = tempfile::Builder::new()
        .prefix(".riotbox-dawproject-staging-")
        .tempfile_in(parent)?;
    staging.write_all(&archive.bytes)?;
    staging.as_file().sync_all()?;
    staging
        .persist_noclobber(destination)
        .map_err(|error| error.error)?;
    Ok(())
}

fn validate_published_archive(
    destination: &Path,
    expected_archive: &ValidatedArchive,
    payload: &DawprojectArchivePayload<'_>,
) -> Result<(), JamAppError> {
    let bytes = fs::read(destination)?;
    if bytes != expected_archive.bytes
        || sha256_bytes(&bytes) != expected_archive.published.archive_sha256
    {
        return Err(JamAppError::InvalidSession(
            "published DAWproject bytes differ from the validated staging archive".into(),
        ));
    }
    validate_archive_bytes(&bytes, payload)?;
    Ok(())
}

fn read_archive_file<R: Read + std::io::Seek>(
    reader: &mut DawprojectReader<R>,
    path: &str,
) -> Result<Vec<u8>, JamAppError> {
    let mut file = reader
        .by_name(path)
        .map_err(|error| invalid_dawproject("required archive member is missing", error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn sha256_file(path: &Path) -> Result<String, JamAppError> {
    Ok(sha256_bytes(&fs::read(path)?))
}

fn validate_audio_member_path(path: &str) -> Result<(), JamAppError> {
    let member = Path::new(path);
    if path.is_empty()
        || !member.is_relative()
        || member
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || [METADATA_XML_PATH, PROJECT_XML_PATH, DAWPROJECT_PROOF_PATH].contains(&path)
    {
        return Err(JamAppError::InvalidSession(format!(
            "DAWproject embedded audio path is unsafe or reserved: {path}"
        )));
    }
    Ok(())
}

fn expected_archive_paths(audio_path: &str) -> Vec<String> {
    let mut paths = [
        audio_path.to_owned(),
        METADATA_XML_PATH.to_owned(),
        PROJECT_XML_PATH.to_owned(),
        DAWPROJECT_PROOF_PATH.to_owned(),
    ];
    paths.sort_unstable();
    paths.into()
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn invalid_dawproject(context: &str, error: impl std::fmt::Display) -> JamAppError {
    JamAppError::InvalidSession(format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use dawproject::{
        Dawproject, DawprojectWriter, MetaData, Project, prelude::project::ApplicationType,
    };

    use super::*;

    fn synthetic_metadata() -> MetaData {
        MetaData {
            title: Some("Synthetic Riotbox archive".into()),
            artist: None,
            album: None,
            original_artist: None,
            composer: None,
            songwriter: None,
            producer: None,
            arranger: None,
            year: None,
            genre: None,
            copyright: None,
            website: None,
            comment: None,
        }
    }

    fn synthetic_project() -> Project {
        Project {
            version: "1.0".into(),
            application: ApplicationType {
                name: "Riotbox synthetic test".into(),
                version: "1".into(),
            },
            transport: None,
            structure: None,
            arrangement: None,
            scenes: None,
        }
    }

    fn legacy_writer_bytes(payload: &DawprojectArchivePayload<'_>) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = DawprojectWriter::new(cursor).expect("legacy writer");
        writer
            .write_dawproject(&Dawproject::new(
                payload.metadata.clone(),
                payload.project.clone(),
            ))
            .expect("legacy model");
        writer
            .write_file(payload.audio.path, payload.audio.bytes)
            .expect("legacy audio");
        writer
            .write_file(DAWPROJECT_PROOF_PATH, payload.proof_json)
            .expect("legacy proof");
        writer.finish().expect("legacy finish").into_inner()
    }

    #[test]
    fn archive_bytes_match_the_legacy_writer_reference() {
        let metadata = synthetic_metadata();
        let project = synthetic_project();
        let audio = [0_u8, 1, 2, 3, 4, 5];
        let audio_sha256 = sha256_bytes(&audio);
        let proof = br#"{"schema":"synthetic.v1"}"#;
        let payload = DawprojectArchivePayload {
            metadata: &metadata,
            project: &project,
            audio: DawprojectArchiveAudio {
                path: "audio/synthetic.wav",
                bytes: &audio,
                sha256: &audio_sha256,
            },
            proof_json: proof,
        };

        let archive = build_and_validate_archive(&payload).expect("validated archive");
        assert_eq!(archive.bytes, legacy_writer_bytes(&payload));
    }

    #[test]
    fn archive_rejects_mutated_bytes_and_never_clobbers_a_destination() {
        let metadata = synthetic_metadata();
        let project = synthetic_project();
        let audio = [0_u8, 1, 2, 3, 4, 5];
        let audio_sha256 = sha256_bytes(&audio);
        let payload = DawprojectArchivePayload {
            metadata: &metadata,
            project: &project,
            audio: DawprojectArchiveAudio {
                path: "audio/synthetic.wav",
                bytes: &audio,
                sha256: &audio_sha256,
            },
            proof_json: br#"{"schema":"synthetic.v1"}"#,
        };
        let mut mutated_audio = audio;
        mutated_audio[0] ^= 1;
        let mutated_audio_sha256 = sha256_bytes(&mutated_audio);
        let mutated_payload = DawprojectArchivePayload {
            metadata: &metadata,
            project: &project,
            audio: DawprojectArchiveAudio {
                path: "audio/synthetic.wav",
                bytes: &mutated_audio,
                sha256: &mutated_audio_sha256,
            },
            proof_json: br#"{"schema":"synthetic.v1"}"#,
        };
        let mutated_archive = legacy_writer_bytes(&mutated_payload);
        assert!(validate_archive_bytes(&mutated_archive, &payload).is_err());

        let temp = tempfile::tempdir().expect("tempdir");
        let destination = temp.path().join("existing.dawproject");
        let existing = b"must-not-be-replaced";
        fs::write(&destination, existing).expect("synthetic existing archive");
        assert!(write_dawproject_archive(&destination, payload).is_err());
        assert_eq!(fs::read(&destination).expect("existing bytes"), existing);
    }

    #[test]
    fn frozen_archive_paths_are_sorted_for_exact_reader_comparison() {
        assert_eq!(
            expected_archive_paths("audio/w30_hook_loop.wav"),
            [
                "audio/w30_hook_loop.wav",
                "metadata.xml",
                "project.xml",
                "riotbox-proof.json"
            ]
        );
    }
}
