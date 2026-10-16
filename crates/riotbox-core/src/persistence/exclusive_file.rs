//! Complete-content, no-replace publication. Callers own path and error policy.
use std::{fs, io, io::Write, path::Path};

use super::atomic_save_temp_path;

#[derive(Debug)]
pub(super) enum ExclusiveFileError {
    Preparation(io::Error),
    Link(io::Error),
}

impl ExclusiveFileError {
    pub(super) fn into_io(self) -> io::Error {
        match self {
            Self::Preparation(source) | Self::Link(source) => source,
        }
    }
}

pub(super) fn publish(path: &Path, bytes: &[u8]) -> Result<(), ExclusiveFileError> {
    publish_with_hard_link(path, bytes, |temporary, destination| {
        fs::hard_link(temporary, destination)
    })
}

pub(super) fn publish_with_hard_link<F>(
    path: &Path,
    bytes: &[u8],
    link: F,
) -> Result<(), ExclusiveFileError>
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    write_and_publish(
        path,
        &atomic_save_temp_path(path),
        bytes,
        |file, bytes| file.write_all(bytes),
        link,
    )
}

fn write_and_publish<W, L>(
    path: &Path,
    temporary: &Path,
    bytes: &[u8],
    write: W,
    link: L,
) -> Result<(), ExclusiveFileError>
where
    W: FnOnce(&mut fs::File, &[u8]) -> io::Result<()>,
    L: FnOnce(&Path, &Path) -> io::Result<()>,
{
    // Cleanup ownership begins only after exclusive creation. A collision is
    // somebody else's entry and must never be removed or truncated.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(ExclusiveFileError::Preparation)?;
    let result = write(&mut file, bytes).map_err(ExclusiveFileError::Preparation);
    drop(file);
    let result = result.and_then(|()| link(temporary, path).map_err(ExclusiveFileError::Link));
    let _ = fs::remove_file(temporary);
    // Publication is the commit point, not best-effort temporary cleanup.
    result
}

#[cfg(test)]
mod tests;
