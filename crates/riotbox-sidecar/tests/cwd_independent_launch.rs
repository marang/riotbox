use std::process::Command;

#[test]
fn stdio_probe_finds_bundled_sidecar_outside_repository_cwd() {
    let unrelated_cwd = tempfile::tempdir().expect("create unrelated working directory");
    let output = Command::new(env!("CARGO_BIN_EXE_stdio_probe"))
        .current_dir(unrelated_cwd.path())
        .output()
        .expect("launch stdio probe outside repository CWD");

    assert!(
        output.status.success(),
        "stdio probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("pong: protocol_version=0.1"));
}
