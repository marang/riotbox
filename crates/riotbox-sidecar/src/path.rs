use std::path::PathBuf;

const BUNDLED_SIDECAR_RELATIVE_PATH: &str = "../../python/sidecar/json_stdio_sidecar.py";

/// Returns the repository-bundled Python sidecar without consulting the process CWD.
#[must_use]
pub fn bundled_sidecar_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BUNDLED_SIDECAR_RELATIVE_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_sidecar_path_is_absolute_and_points_to_a_file() {
        let path = bundled_sidecar_script_path();

        assert!(path.is_absolute());
        assert!(
            path.is_file(),
            "missing bundled sidecar at {}",
            path.display()
        );
    }
}
