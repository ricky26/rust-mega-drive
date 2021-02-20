use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Metadata {
    pub linker_script: Option<PathBuf>,
    pub entry_assembly: Option<PathBuf>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            linker_script: None,
            entry_assembly: None,
        }
    }
}
