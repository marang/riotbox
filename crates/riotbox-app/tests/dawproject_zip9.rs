//! Synthetic integration proof for the temporary upstream ZIP 9 adaptation.
use std::io::{Cursor, Write};

use dawproject::{DawprojectReadError, DawprojectReader, zip};

fn archive_with_one_file() -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file("a", zip::write::SimpleFileOptions::default())
        .expect("start synthetic member");
    writer.write_all(b"synthetic").expect("write member");
    writer.finish().expect("finish archive").into_inner()
}

#[test]
fn zip9_reader_keeps_valid_names_and_member_bytes() {
    let bytes = archive_with_one_file();
    let mut reader = DawprojectReader::new(Cursor::new(bytes)).expect("valid archive");
    assert_eq!(reader.file_names().collect::<Vec<_>>(), vec!["a"]);
    let mut content = Vec::new();
    std::io::Read::read_to_end(&mut reader.by_name("a").expect("member"), &mut content)
        .expect("read member");
    assert_eq!(content, b"synthetic");
}

#[test]
fn zip9_reader_preserves_legacy_cp437_names() {
    let mut bytes = archive_with_one_file();
    let central = bytes
        .windows(4)
        .position(|part| part == b"PK\x01\x02")
        .unwrap();
    bytes[30] = 0xff;
    bytes[central + 46] = 0xff;
    let reader = DawprojectReader::new(Cursor::new(bytes)).expect("legacy filename");
    assert_eq!(reader.file_names().collect::<Vec<_>>(), vec!["\u{a0}"]);
}

#[test]
fn zip9_reader_propagates_invalid_archive_errors() {
    let mut bytes = archive_with_one_file();
    bytes.truncate(10);
    assert!(matches!(
        DawprojectReader::new(Cursor::new(bytes)),
        Err(DawprojectReadError::ZipError(_))
    ));
}
