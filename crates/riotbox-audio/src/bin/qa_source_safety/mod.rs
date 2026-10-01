//! Read-only physical alias preflight shared by offline QA binaries.
//! No concurrent namespace locking or atomic multi-file transaction is implied.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(super) fn reject_source_aliases(
    source: &Path,
    artifacts: impl IntoIterator<Item = PathBuf>,
) -> io::Result<()> {
    let source_handle = same_file::Handle::from_path(source)?;
    for path in artifacts {
        // Only a genuinely absent directory entry is safe to skip. An
        // existing dangling symlink or any metadata/open failure rejects.
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            result => {
                result?;
            }
        }
        if !fs::metadata(&path)?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("output is not a regular file: {}", path.display()),
            ));
        }
        let output_handle = same_file::Handle::from_path(&path)?;
        if source_handle == output_handle {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("output aliases input source: {}", path.display()),
            ));
        }
    }
    Ok(())
}
