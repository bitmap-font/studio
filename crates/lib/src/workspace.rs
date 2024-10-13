use serde::Deserialize;
use snafu::prelude::*;
use std::{fs, io, path::Path};

use crate::{Project, ProjectLoadError};

pub struct Workspace {
    pub projects: Vec<Project>,
}

#[derive(Debug, Snafu)]
pub enum WorkspaceLoadError {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(transparent)]
    De { source: toml::de::Error },
    #[snafu(transparent)]
    Project { source: ProjectLoadError },
}

impl Workspace {
    pub fn load(path: impl AsRef<Path>) -> Result<Workspace, WorkspaceLoadError> {
        let path = path.as_ref();
        let config: WorkspaceManifest =
            toml::from_str(&fs::read_to_string(path.join("workspace.toml"))?)?;
        let projects = config
            .workspace
            .members
            .iter()
            .map(|subpath| Project::load(path.join(subpath)))
            .collect::<Result<_, _>>()?;

        Ok(Workspace { projects })
    }
}

#[derive(Deserialize)]
pub struct WorkspaceManifest {
    pub workspace: WorkspaceSection,
}

#[derive(Deserialize)]
pub struct WorkspaceSection {
    pub members: Vec<String>,
}
