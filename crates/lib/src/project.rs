use std::{fs, io, path::Path};

use serde::Deserialize;
use snafu::prelude::*;

pub struct Project {
    files: Vec<SourceFile>,
}

#[derive(Debug, Snafu)]
pub enum ProjectLoadError {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(transparent)]
    De { source: toml::de::Error },
}

impl Project {
    pub fn load(path: impl AsRef<Path>) -> Result<Project, ProjectLoadError> {
        let path = path.as_ref();
        let manifest: ProjectManifest =
            toml::from_str(&fs::read_to_string(path.join("project.toml"))?)?;

        todo!()
    }
}

#[derive(Deserialize)]
pub struct ProjectManifest {}
