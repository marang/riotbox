use std::{fs, io, io::Write};

use super::write_and_publish;

#[test]
fn temporary_collision_is_not_removed_or_truncated() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("destination");
    let temporary = dir.path().join("temporary");
    fs::write(&temporary, b"another invocation").unwrap();
    let error = write_and_publish(
        &path,
        &temporary,
        b"new",
        |_, _| panic!("collision"),
        |_, _| panic!("collision"),
    )
    .unwrap_err();
    assert_eq!(error.into_io().kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&temporary).unwrap(), b"another invocation");
    assert!(!path.exists());
}

#[test]
fn partial_temporary_write_failure_never_publishes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("destination");
    let temporary = dir.path().join("temporary");
    let error = write_and_publish(
        &path,
        &temporary,
        b"complete",
        |file, bytes| {
            file.write_all(&bytes[..2])?;
            Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "injected disk failure",
            ))
        },
        |_, _| panic!("partial write must not publish"),
    )
    .unwrap_err();
    assert_eq!(error.into_io().kind(), io::ErrorKind::WriteZero);
    assert!(!path.exists());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn raced_destination_is_never_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("destination");
    let temporary = dir.path().join("temporary");
    let error = write_and_publish(
        &path,
        &temporary,
        b"complete",
        |file, bytes| file.write_all(bytes),
        |temp, dest| {
            fs::write(dest, b"race winner")?;
            fs::hard_link(temp, dest)
        },
    )
    .unwrap_err();
    assert_eq!(error.into_io().kind(), io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&path).unwrap(), b"race winner");
    assert!(!temporary.exists());
}

#[test]
fn completed_publication_stays_success_when_temporary_cleanup_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("destination");
    let temporary = dir.path().join("temporary");
    write_and_publish(
        &path,
        &temporary,
        b"complete",
        |file, bytes| file.write_all(bytes),
        |temp, dest| {
            fs::hard_link(temp, dest)?;
            // Deterministic cleanup-failure injection, not a production race claim.
            fs::remove_file(temp)?;
            fs::create_dir(temp)
        },
    )
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"complete");
    assert!(temporary.is_dir());
}

#[test]
fn unsupported_write_and_link_failures_retain_distinct_stages() {
    use super::ExclusiveFileError;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("destination");
    let temporary = dir.path().join("temporary");
    let write_error = write_and_publish(
        &path,
        &temporary,
        b"complete",
        |_, _| {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "write unsupported",
            ))
        },
        |_, _| panic!("failed write must not link"),
    )
    .unwrap_err();
    assert!(
        matches!(write_error, ExclusiveFileError::Preparation(source) if source.kind() == io::ErrorKind::Unsupported)
    );
    let link_error = write_and_publish(
        &path,
        &temporary,
        b"complete",
        |file, bytes| file.write_all(bytes),
        |_, _| {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "link unsupported",
            ))
        },
    )
    .unwrap_err();
    assert!(
        matches!(link_error, ExclusiveFileError::Link(source) if source.kind() == io::ErrorKind::Unsupported)
    );
    assert!(!path.exists());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}
