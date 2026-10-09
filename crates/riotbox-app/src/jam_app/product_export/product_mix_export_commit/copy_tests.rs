use std::io::{self, Read};

use tempfile::tempdir;

use super::copy_file_new_verified;

struct PartialReadFailure {
    prefix_read: bool,
}

impl Read for PartialReadFailure {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.prefix_read {
            return Err(io::Error::other(
                "injected read failure after copied prefix",
            ));
        }
        self.prefix_read = true;
        buffer[..3].copy_from_slice(b"abc");
        Ok(3)
    }
}

#[test]
fn partial_copy_failure_removes_only_the_newly_created_output() {
    let temp = tempdir().expect("tempdir");
    let target = temp.path().join("artifact.wav");
    let sentinel = temp.path().join("unrelated.txt");
    std::fs::write(&sentinel, b"preserve").expect("sentinel");
    let error = copy_file_new_verified(
        &mut PartialReadFailure { prefix_read: false },
        &target,
        "unused-on-I/O-failure",
    )
    .expect_err("partial copy fails");
    assert!(error.to_string().contains("injected read failure"));
    assert!(!target.exists());
    assert_eq!(
        std::fs::read(sentinel).expect("sentinel retained"),
        b"preserve"
    );
}

#[test]
fn copied_byte_hash_mismatch_removes_output_but_failed_create_never_does() {
    let temp = tempdir().expect("tempdir");
    let target = temp.path().join("artifact.wav");
    copy_file_new_verified(
        &mut b"different copied bytes".as_slice(),
        &target,
        "expected",
    )
    .expect_err("copy hash mismatch fails");
    assert!(!target.exists());

    std::fs::write(&target, b"pre-existing output").expect("existing output");
    copy_file_new_verified(&mut b"new bytes".as_slice(), &target, "expected")
        .expect_err("exclusive create collision fails");
    assert_eq!(
        std::fs::read(target).expect("unowned target remains"),
        b"pre-existing output"
    );
}
