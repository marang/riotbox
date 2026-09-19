use std::{fs, io, path::Path};

use crate::source_graph::SourceGraph;

use super::{PersistenceError, atomic_save_temp_path};

pub(super) fn publish(path: &Path, graph: &SourceGraph) -> Result<(), PersistenceError> {
    publish_with_hard_link(path, graph, |temporary, destination| {
        fs::hard_link(temporary, destination)
    })
}

fn publish_with_hard_link<F>(
    path: &Path,
    graph: &SourceGraph,
    hard_link: F,
) -> Result<(), PersistenceError>
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    use std::io::Write;

    let json = serde_json::to_vec_pretty(graph)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = atomic_save_temp_path(path);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)?;
    let write_result = file.write_all(&json);
    drop(file);
    if let Err(source) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(source.into());
    }

    let result = hard_link(&temp_path, path);
    let _ = fs::remove_file(&temp_path);
    match result {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::Unsupported => {
            Err(PersistenceError::ImmutablePublicationUnsupported {
                path: path.to_path_buf(),
                source,
            })
        }
        Err(source) => Err(source.into()),
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs, io};

    use tempfile::tempdir;

    use crate::{
        ids::SourceId,
        source_graph::{DecodeProfile, GraphProvenance, SourceDescriptor, SourceGraph},
    };

    use super::{PersistenceError, publish_with_hard_link};

    #[test]
    fn immutable_graph_publication_never_replaces_an_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("generation.json");
        let graph = sample_graph();
        crate::persistence::publish_source_graph_json_generation(&path, &graph).unwrap();
        let original = fs::read(&path).unwrap();
        let mut changed = graph.clone();
        changed.source.duration_seconds += 1.0;
        let error =
            crate::persistence::publish_source_graph_json_generation(&path, &changed).unwrap_err();
        assert!(
            matches!(error, PersistenceError::Io(error) if error.kind() == io::ErrorKind::AlreadyExists)
        );
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn immutable_graph_publication_preserves_existing_destination_when_hard_links_are_unsupported()
    {
        let dir = tempdir().unwrap();
        let path = dir.path().join("generation.json");
        let existing = b"existing immutable generation";
        fs::write(&path, existing).unwrap();

        let error = publish_with_hard_link(&path, &sample_graph(), |_temporary, _destination| {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "hard links are unavailable on this filesystem",
            ))
        })
        .unwrap_err();

        assert!(matches!(
            &error,
            PersistenceError::ImmutablePublicationUnsupported { source, .. }
                if source.kind() == io::ErrorKind::Unsupported
        ));
        assert!(
            error
                .to_string()
                .contains("hard-link publication is unsupported")
        );
        assert!(
            error
                .source()
                .expect("preserve OS failure")
                .to_string()
                .contains("hard links are unavailable")
        );
        assert_eq!(fs::read(&path).unwrap(), existing);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn immutable_graph_publication_cleans_temporary_file_when_hard_links_are_unsupported() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("generation.json");

        let error = publish_with_hard_link(&path, &sample_graph(), |_temporary, _destination| {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "hard links are unavailable on this filesystem",
            ))
        })
        .unwrap_err();

        assert!(matches!(
            &error,
            PersistenceError::ImmutablePublicationUnsupported { source, .. }
                if source.kind() == io::ErrorKind::Unsupported
        ));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn immutable_graph_publication_does_not_mislabel_other_hard_link_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("generation.json");

        let error = publish_with_hard_link(&path, &sample_graph(), |_temporary, _destination| {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "destination directory is read-only",
            ))
        })
        .unwrap_err();

        assert!(matches!(
            &error,
            PersistenceError::Io(error) if error.kind() == io::ErrorKind::PermissionDenied
        ));
        assert!(!error.to_string().contains("unsupported"));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    fn sample_graph() -> SourceGraph {
        SourceGraph::new(
            SourceDescriptor {
                source_id: SourceId::from("src-1"),
                path: "input.wav".into(),
                content_hash: "hash-1".into(),
                duration_seconds: 120.0,
                sample_rate: 48_000,
                channel_count: 2,
                decode_profile: DecodeProfile::NormalizedStereo,
            },
            GraphProvenance {
                sidecar_version: "0.1.0".into(),
                provider_set: vec!["beat".into()],
                generated_at: "2026-04-12T18:00:00Z".into(),
                source_hash: "hash-1".into(),
                analysis_seed: 7,
                run_notes: Some("graph-publication-test".into()),
            },
        )
    }
}
