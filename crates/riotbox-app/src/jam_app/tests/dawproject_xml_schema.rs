use std::{
    io::{Read, Seek, Write as _},
    path::Path,
    process::{Command, Stdio},
};

use dawproject::DawprojectReader;

const PROJECT_SCHEMA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/dawproject-schema/Project.xsd"
);
const METADATA_SCHEMA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/dawproject-schema/MetaData.xsd"
);

pub(super) fn assert_dawproject_xml_documents_conform(archive: &Path) {
    let mut reader = DawprojectReader::open(archive).expect("open synthetic DAWproject archive");
    let project = archive_member(&mut reader, "project.xml");
    let metadata = archive_member(&mut reader, "metadata.xml");
    validate_xml_against_schema(PROJECT_SCHEMA, &project)
        .expect("project.xml conforms to pinned Project.xsd");
    validate_xml_against_schema(METADATA_SCHEMA, &metadata)
        .expect("metadata.xml conforms to pinned MetaData.xsd");
}

fn archive_member<R: Read + Seek>(reader: &mut DawprojectReader<R>, name: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    reader
        .by_name(name)
        .unwrap_or_else(|error| panic!("read {name} from synthetic DAWproject: {error}"))
        .read_to_end(&mut bytes)
        .unwrap_or_else(|error| panic!("read bytes for {name}: {error}"));
    bytes
}

fn validate_xml_against_schema(schema: &str, xml: &[u8]) -> Result<(), String> {
    let mut child = Command::new("xmllint")
        .args(["--nonet", "--noout", "--schema", schema, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("xmllint is required for pinned DAWproject schema validation");
    let write_result = child
        .stdin
        .take()
        .expect("xmllint stdin")
        .write_all(xml)
        .map_err(|error| format!("write XML to xmllint: {error}"));
    let output = child
        .wait_with_output()
        .map_err(|error| format!("wait for xmllint: {error}"))?;
    write_result?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[test]
fn pinned_project_schema_rejects_the_generated_rust_type_root() {
    let error =
        validate_xml_against_schema(PROJECT_SCHEMA, b"<?xml version=\"1.0\"?><ProjectType/>")
            .expect_err("ProjectType is not the official DAWproject root");
    assert!(error.contains("ProjectType") || error.contains("Project"));
}

#[test]
fn pinned_metadata_schema_rejects_the_generated_rust_type_root() {
    let error = validate_xml_against_schema(METADATA_SCHEMA, b"<MetaDataType/>")
        .expect_err("MetaDataType is not the official metadata root");
    assert!(error.contains("MetaDataType") || error.contains("MetaData"));
}
