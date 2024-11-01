use std::{fs, io, path::Path};

use serde::Deserialize;
use snafu::prelude::*;

use crate::source_file::{SourceFile, SourceFileLoadError};

pub struct Project {
    pub manifest: ProjectManifest,
    pub files: Vec<SourceFile>,
}

#[derive(Debug, Snafu)]
pub enum ProjectLoadError {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(transparent)]
    Walkdir { source: walkdir::Error },
    #[snafu(transparent)]
    De { source: toml::de::Error },
    #[snafu(transparent)]
    SourceFile { source: SourceFileLoadError },
}

impl Project {
    pub fn load(path: impl AsRef<Path>) -> Result<Project, ProjectLoadError> {
        let path = path.as_ref();
        let manifest: ProjectManifest =
            toml::from_str(&fs::read_to_string(path.join("project.toml"))?)?;

        let files = walkdir::WalkDir::new(path.join("src"))
            .follow_links(true)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|entry| {
                (entry.file_type().is_file()
                    && entry.file_name().as_encoded_bytes().ends_with(b".yaff"))
                .then(|| SourceFile::load(entry.path()))
            })
            .collect::<Result<_, _>>()?;

        Ok(Project { manifest, files })
    }
}

#[derive(Deserialize)]
pub struct ProjectManifest {}
