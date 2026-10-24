use crate::utils::consts::{
    METADATA_PATH, PROJECT_CONTENT_TYPE, PROJECT_FIXED_CONTENT_TYPE, PROJECT_PATH, XML_DECLARATION,
};
use crate::{Dawproject, MetaData, Project};
use std::fs::File;
use std::io::{BufWriter, Read, Seek, Write};
use std::path::Path;
use thiserror::Error;
use zip::write::SimpleFileOptions;

#[derive(Error, Debug)]
pub enum DawprojectWriteError {
    #[error("zip error")]
    ZipError(#[from] zip::result::ZipError),
    #[error("metadata.xml serialize error: {0}")]
    MetadataSerializeError(String),
    #[error("project.xml serialize error: {0}")]
    ProjectSerializeError(String),
    #[error("std io error")]
    StdIoError(#[from] std::io::Error),
}

/// Write `.dawproject` file.
pub struct DawprojectWriter<W: Write + Seek> {
    zip_writer: zip::ZipWriter<W>,
}

impl<W> DawprojectWriter<W>
where
    W: Write + Seek,
{
    pub fn new(writer: W) -> Result<Self, DawprojectWriteError> {
        let zip_writer = zip::ZipWriter::new(writer);

        Ok(DawprojectWriter { zip_writer })
    }

    pub fn write_dawproject(
        &mut self,
        dawproject: &Dawproject,
    ) -> Result<(), DawprojectWriteError> {
        self.write_metadata(&dawproject.metadata)?;
        self.write_project(&dawproject.project)?;
        Ok(())
    }

    fn write_metadata(&mut self, metadata: &MetaData) -> Result<(), DawprojectWriteError> {
        let options = SimpleFileOptions::default();
        self.zip_writer
            .start_file(METADATA_PATH, options)
            .map_err(DawprojectWriteError::ZipError)?;

        let mut xml_str = String::from(XML_DECLARATION);
        let body = quick_xml::se::to_string(metadata)
            .map_err(|e| DawprojectWriteError::MetadataSerializeError(e.to_string()))?;
        xml_str.push_str(&body);

        self.zip_writer
            .write_all(xml_str.as_bytes())
            .map_err(DawprojectWriteError::StdIoError)?;
        Ok(())
    }
    fn write_project(&mut self, project: &Project) -> Result<(), DawprojectWriteError> {
        let options = SimpleFileOptions::default();
        self.zip_writer
            .start_file(PROJECT_PATH, options)
            .map_err(DawprojectWriteError::ZipError)?;

        let xml_buf = quick_xml::se::to_string(project)
            .map_err(|e| DawprojectWriteError::ProjectSerializeError(e.to_string()))?;
        // Change into original project.xml
        let original_project_xml =
            xml_buf.replace(PROJECT_FIXED_CONTENT_TYPE, PROJECT_CONTENT_TYPE);

        self.zip_writer
            .write_all(original_project_xml.as_bytes())
            .map_err(DawprojectWriteError::StdIoError)?;
        Ok(())
    }

    pub fn raw_copy_file<R>(
        &mut self,
        file: zip::read::ZipFile<'_, R>,
    ) -> Result<(), DawprojectWriteError>
    where
        R: Read,
    {
        self.zip_writer
            .raw_copy_file(file)
            .map_err(DawprojectWriteError::ZipError)?;
        Ok(())
    }

    pub fn write_file(&mut self, name: &str, buf: &[u8]) -> Result<(), DawprojectWriteError> {
        let options = SimpleFileOptions::default();
        self.zip_writer
            .start_file(name, options)
            .map_err(DawprojectWriteError::ZipError)?;
        self.zip_writer.write_all(buf)?;
        Ok(())
    }

    pub fn finish(self) -> Result<W, DawprojectWriteError> {
        self.zip_writer
            .finish()
            .map_err(DawprojectWriteError::ZipError)
    }
}

impl DawprojectWriter<BufWriter<File>> {
    /// Create a `.dawproject` file.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, DawprojectWriteError> {
        let f = File::create(path).map_err(DawprojectWriteError::StdIoError)?;
        let writer = BufWriter::new(f);
        Self::new(writer)
    }
}
