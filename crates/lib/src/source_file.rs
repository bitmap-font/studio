use std::{
    fs, io,
    path::{Path, PathBuf},
};

use snafu::prelude::*;
use yaff::{parse_document, Document};

pub struct SourceFile {
    pub document: Document,
}

#[derive(Debug, Snafu)]
pub enum SourceFileLoadError {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(display("failed to parse {path}", path = path.to_string_lossy()))]
    Yaff {
        path: PathBuf,
        source: yaff::YaffParseError,
    },
}

impl SourceFile {
    pub fn load(path: impl AsRef<Path>) -> Result<SourceFile, SourceFileLoadError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let document =
            parse_document(&mut content.as_ref()).map_err(|source| SourceFileLoadError::Yaff {
                path: path.to_owned(),
                source,
            })?;

        Ok(SourceFile { document })
    }
}
