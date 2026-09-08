//! CLI paths are cwd-relative inputs; stored relative references belong to the
//! Session directory. Anchor runtime destinations once, without following the
//! final alias (atomic saves replace that directory entry, not its referent).

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

pub(super) fn anchored_file_path(path: &Path) -> io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let name = absolute.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "persistence path must name a file",
        )
    })?;
    let mut parent = PathBuf::new();
    for component in absolute
        .parent()
        .expect("absolute file has parent")
        .components()
    {
        match component {
            Component::ParentDir => {
                parent.pop();
            }
            Component::CurDir => {}
            component => {
                parent.push(component.as_os_str());
                match fs::canonicalize(&parent) {
                    Ok(canonical) => parent = canonical,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(parent.join(name))
}

pub(super) fn stored_graph_path(session: &Path, graph: &Path) -> io::Result<String> {
    let session = anchored_file_path(session)?;
    let graph = anchored_file_path(graph)?;
    let base: Vec<_> = session
        .parent()
        .expect("anchored Session has parent")
        .components()
        .collect();
    let target: Vec<_> = graph.components().collect();
    let common = base.iter().zip(&target).take_while(|(a, b)| a == b).count();
    let stored = if common == 0 {
        // Different filesystem roots (for example Windows drive letters).
        graph
    } else {
        let mut relative = PathBuf::new();
        for _ in &base[common..] {
            relative.push("..");
        }
        for part in &target[common..] {
            relative.push(part.as_os_str());
        }
        relative
    };
    stored.into_os_string().into_string().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Graph reference must be valid UTF-8",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{anchored_file_path, stored_graph_path};

    #[test]
    fn references_share_the_session_directory_contract() {
        assert_eq!(
            stored_graph_path("sessions/jam.json".as_ref(), "sessions/graph.json".as_ref())
                .unwrap(),
            "graph.json"
        );
        assert_eq!(
            stored_graph_path("sessions/jam.json".as_ref(), "graphs/graph.json".as_ref()).unwrap(),
            "../graphs/graph.json"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_symlink_parents_before_dotdot_but_preserves_alias_leaf() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("actual/deep")).unwrap();
        std::os::unix::fs::symlink(dir.path().join("actual/deep"), dir.path().join("link"))
            .unwrap();
        let path = anchored_file_path(&dir.path().join("link/../graph.json")).unwrap();
        assert_eq!(path, dir.path().join("actual/graph.json"));
        std::os::unix::fs::symlink("target.json", dir.path().join("alias.json")).unwrap();
        assert_eq!(
            anchored_file_path(&dir.path().join("alias.json")).unwrap(),
            dir.path().join("alias.json")
        );
    }
}
