//! The Session rename commits a graph generation. The user-facing graph path
//! remains a compatible latest-graph alias, not the multi-file commit authority.

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use riotbox_core::{
    persistence::{
        load_source_graph_json, publish_source_graph_json_generation, save_session_json,
        save_source_graph_json,
    },
    session::{SessionFile, SourceGraphRef},
    source_graph::SourceGraph,
};

use super::{JamAppError, source_graph_hash, validate_source_graph_hash};

pub(in crate::jam_app) fn load_graph_for_ref(
    alias: &Path,
    graph_ref: &SourceGraphRef,
) -> Result<SourceGraph, JamAppError> {
    let alias_result = load_source_graph_json(alias)
        .map_err(JamAppError::from)
        .and_then(|graph| {
            validate_source_graph_hash(graph_ref, &graph)?;
            Ok(graph)
        });
    match alias_result {
        Ok(graph) => Ok(graph),
        Err(alias_error) => {
            let generation = match generation_path(alias, &graph_ref.graph_hash) {
                Ok(path) => path,
                Err(_) => return Err(alias_error),
            };
            // Only the exact Session-bound hash is eligible. Never scan, select
            // the newest file, or repair the alias as part of loading.
            match load_source_graph_json(&generation) {
                Ok(graph) => {
                    validate_source_graph_hash(graph_ref, &graph)?;
                    Ok(graph)
                }
                Err(_) => Err(alias_error),
            }
        }
    }
}

pub(in crate::jam_app) fn save_graph_and_session(
    session_path: &Path,
    session: &SessionFile,
    graph: Option<&SourceGraph>,
    alias: Option<&Path>,
) -> Result<(), JamAppError> {
    save_with_checkpoint(session_path, session, graph, alias, |_| Ok(()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::jam_app) enum SaveCheckpoint {
    PreviousGenerationReady,
    CurrentGenerationReady,
    AliasPublished,
}

pub(in crate::jam_app) fn save_with_checkpoint(
    session_path: &Path,
    session: &SessionFile,
    graph: Option<&SourceGraph>,
    alias: Option<&Path>,
    mut checkpoint: impl FnMut(SaveCheckpoint) -> Result<(), JamAppError>,
) -> Result<(), JamAppError> {
    // Serialize before any publication, including graph-side effects.
    serde_json::to_vec(session)?;
    validate_mutable_destination(session_path)?;
    if let (Some(graph), Some(alias)) = (graph, alias) {
        validate_mutable_destination(alias)?;
        let current_path = generation_path(alias, &source_graph_hash(graph)?)?;
        validate_distinct_paths(session_path, alias, &current_path)?;
        // A legacy save has no generation yet: preserve the graph its Session
        // currently names before replacing that graph's mutable alias.
        match load_source_graph_json(alias) {
            Ok(previous) => {
                let previous_path = generation_path(alias, &source_graph_hash(&previous)?)?;
                validate_distinct_paths(session_path, alias, &previous_path)?;
                publish_generation(&previous_path, &previous)?;
            }
            Err(riotbox_core::persistence::PersistenceError::Io(error))
                if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        checkpoint(SaveCheckpoint::PreviousGenerationReady)?;
        publish_generation(&current_path, graph)?;
        checkpoint(SaveCheckpoint::CurrentGenerationReady)?;
        save_source_graph_json(alias, graph)?;
        checkpoint(SaveCheckpoint::AliasPublished)?;
    }
    save_session_json(session_path, session)?;
    Ok(())
}

pub(in crate::jam_app) fn generation_path(
    alias: &Path,
    hash: &str,
) -> Result<PathBuf, JamAppError> {
    let digest = hash
        .strip_prefix("sha256:")
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| {
            JamAppError::InvalidSession("invalid canonical Source Graph sha256 identity".into())
        })?;
    let mut directory_name = alias
        .file_name()
        .ok_or_else(|| JamAppError::InvalidSession("Source Graph alias must name a file".into()))?
        .to_os_string();
    directory_name.push(".riotbox-graphs");
    Ok(alias
        .with_file_name(directory_name)
        .join(format!("{digest}.json")))
}

fn publish_generation(path: &Path, graph: &SourceGraph) -> Result<(), JamAppError> {
    match publish_source_graph_json_generation(path, graph) {
        Ok(()) => Ok(()),
        Err(riotbox_core::persistence::PersistenceError::Io(error))
            if error.kind() == io::ErrorKind::AlreadyExists =>
        {
            let existing = load_source_graph_json(path)?;
            if source_graph_hash(&existing)? != source_graph_hash(graph)? {
                return Err(JamAppError::InvalidSession(format!(
                    "immutable Source Graph generation has a hash mismatch: {}",
                    path.display()
                )));
            }
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn validate_distinct_paths(
    session: &Path,
    alias: &Path,
    generation: &Path,
) -> Result<(), JamAppError> {
    let session = resolved_destination(session)?;
    let alias = resolved_destination(alias)?;
    let resolved_generation = resolved_destination(generation)?;
    if session == alias || session == resolved_generation || alias == resolved_generation {
        return Err(JamAppError::InvalidSession(
            "Session, Source Graph alias, and immutable generation paths must be distinct".into(),
        ));
    }
    // Generations are owned immutable files. Refuse pre-existing symlink
    // redirects rather than treating another path's bytes as owned storage.
    for path in [
        generation,
        generation.parent().expect("generation has a parent"),
    ] {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(JamAppError::InvalidSession(
                    "immutable Source Graph generation storage must not be a symlink".into(),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

pub(in crate::jam_app) fn validate_mutable_destination(path: &Path) -> Result<(), JamAppError> {
    let resolved = resolved_destination(path)?;
    if [path, resolved.as_path()].into_iter().any(|path| {
        path.components().any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .ends_with(".riotbox-graphs")
        })
    }) {
        return Err(JamAppError::InvalidSession(
            "Session and Source Graph aliases must not overwrite immutable generation storage"
                .into(),
        ));
    }
    Ok(())
}

// Canonicalize existing prefixes too, so missing destination files and symlinked
// parents cannot evade collision checks. Multiwriter races are outside this seam.
fn resolved_destination(path: &Path) -> Result<PathBuf, io::Error> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut resolved = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            component => {
                resolved.push(component.as_os_str());
                match fs::canonicalize(&resolved) {
                    Ok(canonical) => resolved = canonical,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(resolved)
}
